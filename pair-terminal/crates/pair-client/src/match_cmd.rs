use crate::connect::{connect, serialize_message};
use crate::raw_guard::RawModeGuard;
use crate::tui::StatusBar;
use pair_common::protocol::*;
use pair_common::types::{PairMode, SkillLevel, UserId};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn run(
    server_url: &str,
    languages: &[String],
    skill_str: &str,
    mode_str: &str,
) -> anyhow::Result<()> {
    let skill = match skill_str {
        "beginner" => SkillLevel::Beginner,
        "expert" => SkillLevel::Expert,
        _ => SkillLevel::Intermediate,
    };

    let mode = match mode_str {
        "driver" => PairMode::Driver,
        "navigator" => PairMode::Navigator,
        _ => PairMode::Collaborative,
    };

    println!("pair: Looking for a coding partner...");
    println!("pair: Languages: {:?}", languages);
    println!("pair: Skill level: {:?}", skill);
    println!("pair: Mode: {:?}", mode);

    let _raw_guard = RawModeGuard::new()?;
    let mut ws = connect(server_url).await?;
    let user_id = UserId::anonymous();

    let register = ClientMessage::MatchRegister(MatchRegisterPayload {
        user_id: user_id.0.clone(),
        preferences: pair_common::types::MatchPreferences {
            languages: languages.to_vec(),
            skill_level: skill,
            mode,
        },
    });

    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        serialize_message(&register)?,
    ))
    .await?;

    let (mut ws_tx, mut ws_rx) = ws.split();
    let running = Arc::new(AtomicBool::new(true));

    println!("pair: Waiting for match... (Ctrl+C to cancel)");

    while running.load(Ordering::Relaxed) {
        tokio::select! {
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(ServerMessage::MatchStatus(status)) => {
                                println!(
                                    "\rpair: Position in queue: {} | ETA: ~{}s  ",
                                    status.position, status.eta_seconds
                                );
                            }
                            Ok(ServerMessage::Matched(matched)) => {
                                println!("\npair: Matched with @{}!", matched.peer.username);
                                println!("pair: Session ID: {}", matched.session_id);
                                println!("pair: Connecting...");

                                let _ = ws_tx.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;

                                // Switch to join mode with the matched session
                                // In a real implementation, we'd get the encrypted join URL from the server
                                // For now, print instructions
                                println!("pair: Connection established! Starting pair session...");
                                running.store(false, Ordering::Relaxed);
                            }
                            Ok(ServerMessage::MatchTimeout) => {
                                println!("\npair: No match found. Try again with broader preferences.");
                                running.store(false, Ordering::Relaxed);
                            }
                            Ok(ServerMessage::FatalError(e)) => {
                                println!("\npair: Error: {}", e);
                                running.store(false, Ordering::Relaxed);
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) | None | Err(_) => {
                        println!("\npair: Disconnected from server.");
                        break;
                    }
                    _ => {}
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\npair: Cancelled match search.");
                let _ = ws_tx.send(tokio_tungstenite::tungstenite::Message::Text(
                    serialize_message(&ClientMessage::MatchCancel)?,
                )).await;
                let _ = ws_tx.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
                running.store(false, Ordering::Relaxed);
            }
        }
    }

    Ok(())
}
