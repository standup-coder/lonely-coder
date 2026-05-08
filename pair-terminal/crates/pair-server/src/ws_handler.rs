use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use pair_common::protocol::*;
use crate::session::{SessionManager, ServerForwardMsg};
use std::sync::Arc;

type SharedState = Arc<(
    crate::AppState,
    Arc<SessionManager>,
)>;

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    State((app_state, session_mgr)): State<SharedState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state, session_mgr))
}

async fn handle_socket(
    socket: WebSocket,
    _app_state: crate::AppState,
    session_mgr: Arc<SessionManager>,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    let mut terminal_id: Option<String> = None;
    let mut role: Option<Role> = None;
    let mut user_id: Option<String> = None;

    while let Some(msg) = ws_rx.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Invalid message: {} - {}", e, msg);
                let _ = ws_tx.send(Message::Text(
                    serde_json::to_string(&ServerMessage::FatalError(
                        format!("invalid message: {}", e),
                    ))
                    .unwrap_or_default(),
                )).await;
                continue;
            }
        };

        match client_msg {
            ClientMessage::Handshake(h) => {
                role = Some(h.role);
                user_id = Some(h.user_id.clone());
                terminal_id = h.terminal_id.clone();

                if h.cols > 500 || h.rows > 500 {
                    let _ = ws_tx.send(Message::Text(
                        serde_json::to_string(&ServerMessage::FatalError(
                            "terminal dimensions too large".to_string(),
                        ))
                        .unwrap_or_default(),
                    )).await;
                    continue;
                }

                match h.role {
                    Role::Host => {
                        let tid = h.terminal_id.unwrap_or_else(|| {
                            pair_common::types::TerminalId::generate().0
                        });

                        let (host_tx, mut host_rx) = tokio::sync::mpsc::channel(256);

                        match session_mgr.register_host(
                            tid.clone(),
                            h.user_id,
                            host_tx,
                        ).await {
                            Ok((_output_rx, _close_rx)) => {
                                terminal_id = Some(tid.clone());

                                let ok_msg = serde_json::to_string(&ServerMessage::HandshakeOk(
                                    HandshakeOkPayload {
                                        session_id: tid.clone(),
                                        role: Role::Host,
                                        terminal_id: tid.clone(),
                                    },
                                )).unwrap_or_default();
                                let _ = ws_tx.send(Message::Text(ok_msg)).await;

                                let tid_clone = tid.clone();
                                let sm = session_mgr.clone();

                                tokio::spawn(async move {
                                    while let Some(forward_msg) = host_rx.recv().await {
                                        match forward_msg {
                                            ServerForwardMsg::KeyInput(data) => {
                                                let msg = serde_json::to_string(
                                                    &ServerMessage::KeyInput(KeyInputPayload {
                                                        data,
                                                        encrypted: false,
                                                    }),
                                                ).unwrap_or_default();
                                                let _ = sm.broadcast_output(&tid_clone, msg).await;
                                            }
                                            ServerForwardMsg::Resize { cols, rows } => {
                                                let msg = serde_json::to_string(
                                                    &ServerMessage::Resize(ResizePayload { cols, rows }),
                                                ).unwrap_or_default();
                                                let _ = sm.broadcast_output(&tid_clone, msg).await;
                                            }
                                            ServerForwardMsg::Chat(text) => {
                                                let msg = serde_json::to_string(
                                                    &ServerMessage::Chat(ChatPayload {
                                                        from: "peer".to_string(),
                                                        text,
                                                        timestamp: chrono::Utc::now().timestamp(),
                                                    }),
                                                ).unwrap_or_default();
                                                let _ = sm.broadcast_output(&tid_clone, msg).await;
                                            }
                                            _ => {}
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                let _ = ws_tx.send(Message::Text(
                                    serde_json::to_string(&ServerMessage::FatalError(
                                        format!("failed to register: {}", e),
                                    ))
                                    .unwrap_or_default(),
                                )).await;
                            }
                        }
                    }
                    Role::Guest => {
                        let tid = match h.terminal_id.as_ref() {
                            Some(t) => t,
                            None => {
                                let _ = ws_tx.send(Message::Text(
                                    serde_json::to_string(&ServerMessage::FatalError(
                                        "guest must specify terminal_id".to_string(),
                                    ))
                                    .unwrap_or_default(),
                                )).await;
                                continue;
                            }
                        };

                        let guest_id = uuid::Uuid::new_v4().to_string();

                        match session_mgr.register_guest(
                            tid,
                            guest_id.clone(),
                            h.user_id,
                        ).await {
                            Ok(mut output_rx) => {
                                let tid_for_host = tid.to_string();
                                let sm_for_notify = session_mgr.clone();

                                let _ = sm_for_notify.forward_to_host(
                                    &tid_for_host,
                                    ServerForwardMsg::SnapshotRequest,
                                ).await;

                                let ok_msg = serde_json::to_string(&ServerMessage::HandshakeOk(
                                    HandshakeOkPayload {
                                        session_id: tid.to_string(),
                                        role: Role::Guest,
                                        terminal_id: tid.to_string(),
                                    },
                                )).unwrap_or_default();
                                let _ = ws_tx.send(Message::Text(ok_msg)).await;

                                let notify_msg = serde_json::to_string(
                                    &ServerMessage::NewPeerConnected,
                                ).unwrap_or_default();
                                let _ = ws_tx.send(Message::Text(notify_msg)).await;

                                let count = session_mgr.guest_count(tid).await;
                                let count_msg = serde_json::to_string(
                                    &ServerMessage::NumClients(count),
                                ).unwrap_or_default();
                                let _ = ws_tx.send(Message::Text(count_msg)).await;

                                // Forward PTY output to guest
                                tokio::spawn(async move {
                                    while let Ok(data) = output_rx.recv().await {
                                        if ws_tx.send(Message::Text(data)).await.is_err() {
                                            break;
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                let _ = ws_tx.send(Message::Text(
                                    serde_json::to_string(&ServerMessage::FatalError(
                                        format!("failed to join: {}", e),
                                    ))
                                    .unwrap_or_default(),
                                )).await;
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
                    let msg = serde_json::to_string(
                        &ServerMessage::PtyOutput(payload),
                    ).unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::KeyInput(payload) => {
                if role != Some(Role::Guest) {
                    continue;
                }
                if let Some(ref tid) = terminal_id {
                    let _ = session_mgr.forward_to_host(
                        tid,
                        ServerForwardMsg::KeyInput(payload.data),
                    ).await;
                }
            }

            ClientMessage::Resize(r) => {
                if role != Some(Role::Host) {
                    continue;
                }
                if let Some(ref tid) = terminal_id {
                    let _ = session_mgr.forward_to_host(
                        tid,
                        ServerForwardMsg::Resize { cols: r.cols, rows: r.rows },
                    ).await;
                }
            }

            ClientMessage::ModeChange(mc) => {
                if let Some(ref tid) = terminal_id {
                    let msg = serde_json::to_string(
                        &ServerMessage::ModeChange(mc),
                    ).unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::AesKeys(ak) => {
                if let Some(ref tid) = terminal_id {
                    let msg = serde_json::to_string(
                        &ServerMessage::AesKeys(ak),
                    ).unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::Chat(chat) => {
                if let Some(ref tid) = terminal_id {
                    let msg = serde_json::to_string(
                        &ServerMessage::Chat(chat),
                    ).unwrap_or_default();
                    let _ = session_mgr.broadcast_output(tid, msg).await;
                }
            }

            ClientMessage::Ping => {
                let _ = ws_tx.send(Message::Text(
                    serde_json::to_string(&ServerMessage::Pong).unwrap_or_default(),
                )).await;
            }

            ClientMessage::SnapshotRequest => {
                if role != Some(Role::Guest) {
                    continue;
                }
                if let Some(ref tid) = terminal_id {
                    let _ = session_mgr.forward_to_host(
                        tid,
                        ServerForwardMsg::SnapshotRequest,
                    ).await;
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
            let end_msg = serde_json::to_string(
                &ServerMessage::SessionEnd(SessionEndPayload {
                    reason: format!("host {} disconnected", uid),
                }),
            ).unwrap_or_default();
            let _ = session_mgr.broadcast_output(tid, end_msg).await;
        }
        session_mgr.remove_host(tid).await;
    }
}
