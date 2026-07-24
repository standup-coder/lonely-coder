use crate::session::{ServerForwardMsg, SessionManager};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use pair_common::protocol::*;
use std::sync::Arc;

type SharedState = Arc<crate::AppState>;

pub async fn handle_ws(ws: WebSocketUpgrade, State(state): State<SharedState>) -> Response {
    if !state.try_connect() {
        tracing::warn!("connection rejected: server at capacity");
        // Return a simple 429-like response by upgrading and immediately closing
    }
    let session_mgr = state.session_mgr.clone();
    let app_state = state.clone();
    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, session_mgr).await;
        app_state.disconnect();
    })
}

async fn handle_socket(socket: WebSocket, session_mgr: Arc<SessionManager>) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Channel for spawned tasks to send messages back through ws_tx
    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<String>(256);

    let mut terminal_id: Option<String> = None;
    let mut role: Option<Role> = None;
    let mut user_id: Option<String> = None;

    // Spawn a task to forward outgoing messages to the WebSocket
    let send_handle = tokio::spawn(async move {
        while let Some(data) = out_rx.recv().await {
            if ws_tx.send(Message::Text(data.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_rx.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text.to_string(),
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Invalid message: {} - {}", e, msg);
                let err_msg = serde_json::to_string(&ServerMessage::FatalError(format!(
                    "invalid message: {}",
                    e
                )))
                .unwrap_or_default();
                let _ = out_tx.send(err_msg).await;
                continue;
            }
        };

        match client_msg {
            ClientMessage::Handshake(h) => {
                role = Some(h.role);
                user_id = Some(h.user_id.clone());
                terminal_id = h.terminal_id.clone();

                if h.cols > 500 || h.rows > 500 {
                    let err_msg = serde_json::to_string(&ServerMessage::FatalError(
                        "terminal dimensions too large".to_string(),
                    ))
                    .unwrap_or_default();
                    let _ = out_tx.send(err_msg).await;
                    continue;
                }

                match h.role {
                    Role::Host => {
                        let tid = h
                            .terminal_id
                            .unwrap_or_else(|| pair_common::types::TerminalId::generate().0);

                        let (host_tx, mut host_rx) = tokio::sync::mpsc::channel(256);

                        match session_mgr
                            .register_host(tid.clone(), h.user_id, host_tx)
                            .await
                        {
                            Ok((_output_rx, _close_rx)) => {
                                terminal_id = Some(tid.clone());

                                let ok_msg = serde_json::to_string(&ServerMessage::HandshakeOk(
                                    HandshakeOkPayload {
                                        session_id: tid.clone(),
                                        role: Role::Host,
                                        terminal_id: tid.clone(),
                                    },
                                ))
                                .unwrap_or_default();
                                let _ = out_tx.send(ok_msg).await;

                                let tid_clone = tid.clone();
                                let _sm = session_mgr.clone();
                                let host_out_tx = out_tx.clone();

                                tokio::spawn(async move {
                                    while let Some(forward_msg) = host_rx.recv().await {
                                        match forward_msg {
                                            ServerForwardMsg::KeyInput(data) => {
                                                // Guest keystrokes must be delivered
                                                // to the HOST's WebSocket, not
                                                // broadcast back to the guests.
                                                // (Previously the message went via
                                                // `broadcast_output`, which means
                                                // the same guest who typed the
                                                // input would echo it back to
                                                // themselves.)
                                                let msg = serde_json::to_string(
                                                    &ServerMessage::KeyInput(KeyInputPayload {
                                                        data,
                                                        encrypted: true,
                                                    }),
                                                )
                                                .unwrap_or_default();
                                                let _ = host_out_tx.send(msg).await;
                                            }
                                            ServerForwardMsg::Resize { cols, rows } => {
                                                let msg =
                                                    serde_json::to_string(&ServerMessage::Resize(
                                                        ResizePayload { cols, rows },
                                                    ))
                                                    .unwrap_or_default();
                                                let _ = host_out_tx.send(msg).await;
                                            }
                                            ServerForwardMsg::Chat(text) => {
                                                let msg = serde_json::to_string(
                                                    &ServerMessage::Chat(ChatPayload {
                                                        from: "peer".to_string(),
                                                        text,
                                                        timestamp: chrono::Utc::now().timestamp(),
                                                    }),
                                                )
                                                .unwrap_or_default();
                                                let _ = host_out_tx.send(msg).await;
                                            }
                                            ServerForwardMsg::GuestConnected => {
                                                let msg = serde_json::to_string(
                                                    &ServerMessage::NewPeerConnected,
                                                )
                                                .unwrap_or_default();
                                                let _ = host_out_tx.send(msg).await;
                                            }
                                            ServerForwardMsg::NumClients(n) => {
                                                let msg = serde_json::to_string(
                                                    &ServerMessage::NumClients(n),
                                                )
                                                .unwrap_or_default();
                                                let _ = host_out_tx.send(msg).await;
                                            }
                                            ServerForwardMsg::SnapshotRequest => {}
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                let err_msg = serde_json::to_string(&ServerMessage::FatalError(
                                    format!("failed to register: {}", e),
                                ))
                                .unwrap_or_default();
                                let _ = out_tx.send(err_msg).await;
                            }
                        }
                    }
                    Role::Guest => {
                        let tid = match h.terminal_id.as_ref() {
                            Some(t) => t,
                            None => {
                                let err_msg = serde_json::to_string(&ServerMessage::FatalError(
                                    "guest must specify terminal_id".to_string(),
                                ))
                                .unwrap_or_default();
                                let _ = out_tx.send(err_msg).await;
                                continue;
                            }
                        };

                        let guest_id = uuid::Uuid::new_v4().to_string();

                        match session_mgr
                            .register_guest(tid, guest_id.clone(), h.user_id)
                            .await
                        {
                            Ok(mut output_rx) => {
                                let tid_for_host = tid.to_string();
                                let sm_for_notify = session_mgr.clone();

                                let _ = sm_for_notify
                                    .forward_to_host(
                                        &tid_for_host,
                                        ServerForwardMsg::SnapshotRequest,
                                    )
                                    .await;

                                let ok_msg = serde_json::to_string(&ServerMessage::HandshakeOk(
                                    HandshakeOkPayload {
                                        session_id: tid.to_string(),
                                        role: Role::Guest,
                                        terminal_id: tid.to_string(),
                                    },
                                ))
                                .unwrap_or_default();
                                let _ = out_tx.send(ok_msg).await;

                                // Notify the host that a new guest has joined so
                                // it can rotate E2E keys. (Previously these two
                                // messages were sent on the *guest's* out_tx,
                                // meaning the host never saw them and the host's
                                // `share.rs` loop would never trigger rotation.)
                                let count = session_mgr.guest_count(tid).await;
                                let _ = sm_for_notify
                                    .forward_to_host(
                                        &tid_for_host,
                                        ServerForwardMsg::GuestConnected,
                                    )
                                    .await;
                                let _ = sm_for_notify
                                    .forward_to_host(
                                        &tid_for_host,
                                        ServerForwardMsg::NumClients(count),
                                    )
                                    .await;

                                // Forward PTY output to guest
                                let guest_out_tx = out_tx.clone();
                                tokio::spawn(async move {
                                    while let Ok(data) = output_rx.recv().await {
                                        if guest_out_tx.send(data).await.is_err() {
                                            break;
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                let err_msg = serde_json::to_string(&ServerMessage::FatalError(
                                    format!("failed to join: {}", e),
                                ))
                                .unwrap_or_default();
                                let _ = out_tx.send(err_msg).await;
                            }
                        }
                    }
                }
            }

            ClientMessage::PtyOutput(payload) => {
                if role != Some(Role::Host) {
                    continue;
                }
                if let Some(ref tid) = terminal_id {
                    let msg = serde_json::to_string(&ServerMessage::PtyOutput(payload))
                        .unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::KeyInput(payload) => {
                if role != Some(Role::Guest) {
                    continue;
                }
                if let Some(ref tid) = terminal_id {
                    let _ = session_mgr
                        .forward_to_host(tid, ServerForwardMsg::KeyInput(payload.data))
                        .await;
                }
            }

            ClientMessage::Resize(r) => {
                if role != Some(Role::Host) {
                    continue;
                }
                if let Some(ref tid) = terminal_id {
                    let _ = session_mgr
                        .forward_to_host(
                            tid,
                            ServerForwardMsg::Resize {
                                cols: r.cols,
                                rows: r.rows,
                            },
                        )
                        .await;
                }
            }

            ClientMessage::ModeChange(mc) => {
                if let Some(ref tid) = terminal_id {
                    let msg =
                        serde_json::to_string(&ServerMessage::ModeChange(mc)).unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::AesKeys(ak) => {
                if let Some(ref tid) = terminal_id {
                    let msg =
                        serde_json::to_string(&ServerMessage::AesKeys(ak)).unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::Chat(chat) => {
                if let Some(ref tid) = terminal_id {
                    let msg = serde_json::to_string(&ServerMessage::Chat(chat)).unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::Ping => {
                let pong = serde_json::to_string(&ServerMessage::Pong).unwrap_or_default();
                let _ = out_tx.send(pong).await;
            }

            ClientMessage::Pong => {
                // Client responding to our ping, no action needed
            }

            ClientMessage::SnapshotRequest => {
                if role != Some(Role::Guest) {
                    continue;
                }
                if let Some(ref tid) = terminal_id {
                    let _ = session_mgr
                        .forward_to_host(tid, ServerForwardMsg::SnapshotRequest)
                        .await;
                }
            }

            ClientMessage::MatchRegister(_) | ClientMessage::MatchCancel => {
                // handled by separate HTTP endpoint
            }
        }
    }

    // Cleanup
    if let (Some(Role::Host), Some(ref tid)) = (role, &terminal_id) {
        if let Some(ref uid) = user_id {
            let end_msg = serde_json::to_string(&ServerMessage::SessionEnd(SessionEndPayload {
                reason: format!("host {} disconnected", uid),
            }))
            .unwrap_or_default();
            let _ = session_mgr.broadcast_output(tid, end_msg).await;
        }
        session_mgr.remove_host(tid).await;
    }

    // Wait for the send task to finish
    drop(out_tx);
    let _ = send_handle.await;
}
