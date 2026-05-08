use crate::connect::{connect, serialize_message};
use crate::pty_guest::PtyGuest;
use crate::raw_guard::{terminal_size, RawModeGuard};
use crate::tui::{StatusBar, TuiEvent};
use pair_common::crypto::SessionKeys;
use pair_common::protocol::*;
use pair_common::types::{PairMode, UserId};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn run(server_url: &str, url: &str) -> anyhow::Result<()> {
    let (session_id, bootstrap_key_str) = parse_pair_url(url)?;
    let bootstrap_key = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &bootstrap_key_str,
    )?;

    let (cols, rows) = terminal_size();
    let _raw_guard = RawModeGuard::new()?;

    let mut ws = connect(server_url).await?;
    let user_id = UserId::anonymous();

    let handshake = ClientMessage::Handshake(HandshakePayload {
        user_id: user_id.0.clone(),
        role: Role::Guest,
        cols,
        rows,
        terminal_id: None,
        mode: PairMode::Collaborative,
        allow_guest_control: true,
    });

    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        serialize_message(&handshake)?,
    ))
    .await?;

    let (mut ws_tx, mut ws_rx) = ws.split();
    let (to_ws_tx, mut to_ws_rx) = mpsc::channel::<ClientMessage>(256);

    let running = Arc::new(AtomicBool::new(true));
    let keys = Arc::new(std::sync::Mutex::new(None::<SessionKeys>));

    println!("\x1b[2J\x1b[Hpair: Connecting to session {}...", session_id);
    println!("pair: Waiting for host to accept...\n");

    // Task 1: Receive from WebSocket, render to terminal
    let recv_running = running.clone();
    let recv_keys = keys.clone();

    let recv_handle = tokio::spawn(async move {
        while let Some(msg) = ws_rx.next().await {
            if !recv_running.load(Ordering::Relaxed) {
                break;
            }
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(ServerMessage::HandshakeOk(h)) => {
                            tracing::info!("Connected to session {}", h.session_id);
                        }
                        Ok(ServerMessage::PtyOutput(payload)) => {
                            let bytes = match base64::Engine::decode(
                                &base64::engine::general_purpose::STANDARD,
                                &payload.data,
                            ) {
                                Ok(b) => b,
                                Err(_) => continue,
                            };

                            let data = if payload.encrypted {
                                let guard = recv_keys.lock().await;
                                match guard.as_ref() {
                                    Some(k) => k.decrypt_output(&bytes).unwrap_or_default(),
                                    None => bytes,
                                }
                            } else {
                                bytes
                            };

                            PtyGuest::render_output(&data).ok();
                        }
                        Ok(ServerMessage::Resize(r)) => {
                            // informational: host resized their terminal
                            tracing::debug!("Host resized to {}x{}", r.cols, r.rows);
                        }
                        Ok(ServerMessage::AesKeys(ak)) => {
                            let mut bootstrap = [0u8; 16];
                            bootstrap.copy_from_slice(&bootstrap_key);
                            match SessionKeys::extract_keys(&bootstrap, &ak) {
                                Ok(session_keys) => {
                                    let mut guard = recv_keys.lock().await;
                                    *guard = Some(session_keys);
                                    tracing::info!("E2E encryption established");
                                }
                                Err(e) => tracing::error!("Failed to extract keys: {}", e),
                            }
                        }
                        Ok(ServerMessage::AesKeyRotation(ak)) => {
                            let mut bootstrap = [0u8; 16];
                            bootstrap.copy_from_slice(&bootstrap_key);
                            match SessionKeys::extract_keys(&bootstrap, &ak) {
                                Ok(session_keys) => {
                                    let mut guard = recv_keys.lock().await;
                                    *guard = Some(session_keys);
                                }
                                Err(e) => tracing::error!("Key rotation failed: {}", e),
                            }
                        }
                        Ok(ServerMessage::ModeChange(mc)) => {
                            println!(
                                "\r\x1b[Kpair: Mode changed to {} by {}",
                                match mc.mode {
                                    PairMode::Driver => "DRIVER (host only)",
                                    PairMode::Navigator => "NAVIGATOR (guest only)",
                                    PairMode::Collaborative => "COLLABORATIVE (both)",
                                },
                                mc.changed_by,
                            );
                        }
                        Ok(ServerMessage::Chat(chat)) => {
                            println!(
                                "\r\x1b[K[pair chat] {}: {}",
                                chat.from, chat.text
                            );
                        }
                        Ok(ServerMessage::SessionEnd(reason)) => {
                            tracing::info!("Session ended: {}", reason.reason);
                            recv_running.store(false, Ordering::Relaxed);
                            break;
                        }
                        Ok(ServerMessage::Ping) => {
                            let _ = to_ws_tx.send(ClientMessage::Pong).await;
                        }
                        Ok(ServerMessage::FatalError(e)) => {
                            tracing::error!("Server error: {}", e);
                            recv_running.store(false, Ordering::Relaxed);
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(tokio_tungstenite::tungstenite::Message::Close(_)) | Err(_) => {
                    tracing::info!("Server disconnected");
                    break;
                }
                _ => {}
            }
        }
    });

    // Task 2: Read local input, send to WebSocket
    let input_running = running.clone();
    let input_keys = keys.clone();

    let input_handle = tokio::spawn(async move {
        let mut buf = [0u8; 32];
        let mut tui = StatusBar::new("guest", "host");

        loop {
            if !input_running.load(Ordering::Relaxed) {
                break;
            }

            match PtyGuest::read_local_input(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = buf[..n].to_vec();

                    if n == 1 && data[0] == 0x14 {
                        tui.toggle_help();
                        continue;
                    }

                    if tui.show_help {
                        if let Ok(TuiEvent::Quit) = tui.poll_event().await {
                            input_running.store(false, Ordering::Relaxed);
                            break;
                        }
                        continue;
                    }

                    let guard = input_keys.lock().await;
                    if let Some(session_keys) = guard.as_ref() {
                        if let Ok(encrypted) = session_keys.encrypt_input(&data) {
                            let b64 = base64::Engine::encode(
                                &base64::engine::general_purpose::STANDARD,
                                &encrypted,
                            );
                            let _ = to_ws_tx.send(ClientMessage::KeyInput(KeyInputPayload {
                                data: b64,
                                encrypted: true,
                            })).await;
                        }
                    } else {
                        // fallback: send unencrypted before keys are established
                        let b64 = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &data,
                        );
                        let _ = to_ws_tx.send(ClientMessage::KeyInput(KeyInputPayload {
                            data: b64,
                            encrypted: false,
                        })).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Input read error: {}", e);
                    break;
                }
            }
        }
    });

    // Task 3: Forward messages to WebSocket
    let ws_send_running = running.clone();

    let ws_send_handle = tokio::spawn(async move {
        while let Some(msg) = to_ws_rx.recv().await {
            if !ws_send_running.load(Ordering::Relaxed) {
                break;
            }
            let text = match serialize_message(&msg) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if ws_tx.send(tokio_tungstenite::tungstenite::Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    let _ = tokio::try_join!(recv_handle, input_handle, ws_send_handle);

    running.store(false, Ordering::Relaxed);

    drop(_raw_guard);
    println!("\n\x1b[2J\x1b[Hpair: Session ended.");

    Ok(())
}

fn parse_pair_url(url: &str) -> anyhow::Result<(String, String)> {
    let url = url.strip_prefix("pair://").unwrap_or(url);

    let (session_part, key_part) = if let Some(idx) = url.find('#') {
        (&url[..idx], &url[idx + 1..])
    } else {
        anyhow::bail!("URL must contain a #key fragment for encryption");
    };

    let session_id = session_part.trim_matches('/').to_string();
    let bootstrap_key = key_part.trim().to_string();

    if session_id.is_empty() || bootstrap_key.is_empty() {
        anyhow::bail!("Invalid pair URL: missing session ID or key");
    }

    Ok((session_id, bootstrap_key))
}
