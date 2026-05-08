use crate::connect::{connect, serialize_message};
use crate::pty_host::PtyHost;
use crate::raw_guard::{terminal_size, RawModeGuard};
use crate::tui::{StatusBar, TuiEvent};
use pair_common::crypto::SessionKeys;
use pair_common::protocol::*;
use pair_common::recording::{AsciiCastWriter, generate_share_path};
use pair_common::types::{PairMode, TerminalId, UserId};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn run(
    server_url: &str,
    _signaling_url: &str,
    command: &[String],
    mode_str: &str,
    record: bool,
    _p2p: bool,
) -> anyhow::Result<()> {
    let mode = match mode_str {
        "driver" => PairMode::Driver,
        "navigator" => PairMode::Navigator,
        _ => PairMode::Collaborative,
    };

    let (cols, rows) = terminal_size();
    let mut pty = PtyHost::spawn(command, cols, rows)?;

    let keys = Arc::new(SessionKeys::generate());
    let terminal_id = TerminalId::generate();

    println!("\x1b[2J\x1b[H");
    println!("pair: Starting session...");

    let mut ws = connect(server_url).await?;
    let user_id = UserId::anonymous();

    let handshake = ClientMessage::Handshake(HandshakePayload {
        user_id: user_id.0.clone(),
        role: Role::Host,
        cols,
        rows,
        terminal_id: Some(terminal_id.0.clone()),
        mode,
        allow_guest_control: mode != PairMode::Driver,
    });

    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        serialize_message(&handshake)?,
    ))
    .await?;

    let _raw_guard = RawModeGuard::new()?;

    let (mut ws_tx, mut ws_rx) = ws.split();
    let (pty_output_tx, mut pty_output_rx) = mpsc::channel::<Vec<u8>>(256);
    let (to_pty_tx, mut to_pty_rx) = mpsc::channel::<Vec<u8>>(256);
    let (to_ws_tx, mut to_ws_rx) = mpsc::channel::<ClientMessage>(256);

    let running = Arc::new(AtomicBool::new(true));
    let has_guest = Arc::new(AtomicBool::new(false));

    let mut recorder = if record {
        let path = generate_share_path();
        Some(AsciiCastWriter::new(&path, cols, rows)?)
    } else {
        None
    };

    let host = format!("{}",
        server_url
            .replace("wss://", "")
            .replace("ws://", "")
            .replace("/ws", "")
    );
    let key_fragment = keys.bootstrap_key_b64();
    let share_url = format!("pair://{}/{}\n#{}", host, terminal_id.0, key_fragment);

    println!("\x1b[s\x1b[Kpair: Share this URL with your partner:");
    println!("  {}\x1b[u", share_url);
    println!("\x1b[s\x1b[Kpair: Waiting for partner to connect...\x1b[u");

    // Task 1: Read from PTY, send to WebSocket (and optionally record)
    let pty_read_running = running.clone();
    let pty_read_has_guest = has_guest.clone();
    let pty_read_keys = keys.clone();

    let pty_read_handle = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        while pty_read_running.load(Ordering::Relaxed) {
            match pty.read_output(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = buf[..n].to_vec();

                    if let Some(ref mut rec) = recorder {
                        let _ = rec.write_output(&data);
                    }

                    if pty_read_has_guest.load(Ordering::Relaxed) {
                        if let Ok(encrypted) = pty_read_keys.encrypt_output(&data) {
                            let b64 = base64::Engine::encode(
                                &base64::engine::general_purpose::STANDARD,
                                &encrypted,
                            );
                            let _ = to_ws_tx.send(ClientMessage::PtyOutput(PtyOutputPayload {
                                data: b64,
                                encrypted: true,
                            })).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("PTY read error: {}", e);
                    break;
                }
            }
        }
    });

    // Task 2: Forward messages to WebSocket
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

    // Task 3: Receive from WebSocket, forward to PTY
    let ws_recv_running = running.clone();
    let ws_recv_has_guest = has_guest.clone();
    let ws_recv_keys = keys.clone();

    let ws_recv_handle = tokio::spawn(async move {
        while let Some(msg) = ws_rx.next().await {
            if !ws_recv_running.load(Ordering::Relaxed) {
                break;
            }
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(ServerMessage::HandshakeOk(_)) => {
                            tracing::info!("Handshake successful");
                        }
                        Ok(ServerMessage::NewPeerConnected) => {
                            ws_recv_has_guest.store(true, Ordering::Relaxed);
                            tracing::info!("Guest connected!");

                            match ws_recv_keys.rotate() {
                                Ok(encrypted_keys) => {
                                    let _ = to_ws_tx.send(ClientMessage::AesKeys(AesKeysPayload {
                                        b64_output_key: encrypted_keys.b64_output_key,
                                        b64_input_key: encrypted_keys.b64_input_key,
                                        iv_count: encrypted_keys.iv_count,
                                        max_iv_count: encrypted_keys.max_iv_count,
                                    })).await;
                                }
                                Err(e) => tracing::error!("Key rotation failed: {}", e),
                            }
                        }
                        Ok(ServerMessage::KeyInput(payload)) => {
                            let bytes = match base64::Engine::decode(
                                &base64::engine::general_purpose::STANDARD,
                                &payload.data,
                            ) {
                                Ok(b) => b,
                                Err(_) => continue,
                            };
                            let data = if payload.encrypted {
                                ws_recv_keys.decrypt_input(&bytes).unwrap_or_default()
                            } else {
                                bytes
                            };
                            let _ = to_pty_tx.send(data).await;
                        }
                        Ok(ServerMessage::Chat(chat)) => {
                            let msg = format!("\r\x1b[K[pair chat] {}: {}\r\n> ", chat.from, chat.text);
                            let _ = to_pty_tx.send(msg.into_bytes()).await;
                        }
                        Ok(ServerMessage::SessionEnd(reason)) => {
                            tracing::info!("Session ended: {}", reason.reason);
                            ws_recv_running.store(false, Ordering::Relaxed);
                            break;
                        }
                        Ok(ServerMessage::Ping) => {
                            let _ = to_ws_tx.send(ClientMessage::Pong).await;
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

    // Task 4: Read local input, write to PTY + send to WebSocket
    let input_running = running.clone();
    let input_has_guest = has_guest.clone();
    let input_keys = keys.clone();
    let input_to_ws = to_ws_tx.clone();

    let input_handle = tokio::spawn(async move {
        let mut tui = StatusBar::new(mode_str, "waiting...");
        let mut local_buf = [0u8; 32];

        loop {
            if !input_running.load(Ordering::Relaxed) {
                break;
            }

            match crate::pty_guest::PtyGuest::read_local_input(&mut local_buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = local_buf[..n].to_vec();

                    // Ctrl+T triggers TUI
                    if n == 1 && data[0] == 0x14 {
                        tui.toggle_help();
                        continue;
                    }

                    if tui.show_help {
                        // non-blocking check for TUI events
                        if let Ok(TuiEvent::Quit) = tui.poll_event().await {
                            input_running.store(false, Ordering::Relaxed);
                            break;
                        }
                        continue;
                    }

                    // Write to local PTY
                    let _ = pty_output_tx.send(data.clone()).await;

                    // Send to guest if connected
                    if input_has_guest.load(Ordering::Relaxed) {
                        if let Ok(encrypted) = input_keys.encrypt_input(&data) {
                            let b64 = base64::Engine::encode(
                                &base64::engine::general_purpose::STANDARD,
                                &encrypted,
                            );
                            let _ = input_to_ws.send(ClientMessage::KeyInput(KeyInputPayload {
                                data: b64,
                                encrypted: true,
                            })).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Input read error: {}", e);
                    break;
                }
            }
        }
    });

    // Task 5: Write received input to PTY
    let pty_write_running = running.clone();

    let pty_write_handle = tokio::spawn(async move {
        while let Some(data) = to_pty_rx.recv().await {
            if !pty_write_running.load(Ordering::Relaxed) {
                break;
            }
            if let Err(e) = pty.write_input(&data) {
                tracing::error!("PTY write error: {}", e);
                break;
            }
        }
    });

    let _ = tokio::try_join!(
        pty_read_handle,
        ws_send_handle,
        ws_recv_handle,
        input_handle,
        pty_write_handle,
    );

    running.store(false, Ordering::Relaxed);

    drop(_raw_guard);
    println!("\n\x1b[2J\x1b[Hpair: Session ended.");

    Ok(())
}
