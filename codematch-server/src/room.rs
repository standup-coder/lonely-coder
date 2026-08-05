//! Room state: append-only event log + WebSocket fan-out.
//!
//! A room is created from a matched lobby. Users join the WebSocket
//! (`/api/rooms/:id/ws`) and receive:
//!   1. A backlog of recent events (so they catch up on canvas + chat).
//!   2. Live fan-out of new events from any other peer.
//!
//! Events are typed: `canvas.put`, `chat`, `ai.thinking`, `ai.delta`,
//! `ai.done`, `system.peer_joined`, `system.peer_left`. The payload is
//! JSON-encoded `serde_json::Value` for forward-compatibility.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

const BACKLOG_LIMIT: i64 = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomEvent {
    pub id: i64,
    pub room_id: String,
    pub user_id: Option<i64>,
    pub kind: String,
    pub payload: serde_json::Value,
    pub created_at: String,
}

/// Append a new event to the room log. Returns the inserted row id.
pub async fn append_event(
    pool: &SqlitePool,
    room_id: &str,
    user_id: Option<i64>,
    kind: &str,
    payload: &serde_json::Value,
) -> AppResult<i64> {
    let payload_str = serde_json::to_string(payload).unwrap_or_else(|_| "null".into());
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO room_events (room_id, user_id, kind, payload) VALUES (?, ?, ?, ?)
         RETURNING id",
    )
    .bind(room_id)
    .bind(user_id)
    .bind(kind)
    .bind(payload_str)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Pull the most recent events for a room. Used when a peer joins so
/// they can render the current state.
pub async fn backlog(
    pool: &SqlitePool,
    room_id: &str,
) -> AppResult<Vec<RoomEvent>> {
    let rows: Vec<(i64, String, Option<i64>, String, String, String)> = sqlx::query_as(
        r#"SELECT id, room_id, user_id, kind, payload, created_at
           FROM room_events WHERE room_id = ?
           ORDER BY id DESC LIMIT ?"#,
    )
    .bind(room_id)
    .bind(BACKLOG_LIMIT)
    .fetch_all(pool)
    .await?;

    let mut out: Vec<RoomEvent> = rows
        .into_iter()
        .map(|(id, room_id, user_id, kind, payload, created_at)| {
            let payload: serde_json::Value = serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null);
            RoomEvent { id, room_id, user_id, kind, payload, created_at }
        })
        .collect();
    out.reverse();
    Ok(out)
}

pub async fn room_exists(pool: &SqlitePool, room_id: &str) -> AppResult<bool> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rooms WHERE id = ?")
        .bind(room_id)
        .fetch_one(pool)
        .await?;
    Ok(n > 0)
}

// =====================================================================
// In-process broadcast bus
// =====================================================================
//
// One broadcast channel per active room. We hold them in a process-wide
// map. For multi-process deployment this would be Redis pubsub or
// similar; for the prototype, single-process broadcast is enough.

#[derive(Default)]
pub struct RoomBus {
    rooms: RwLock<HashMap<String, broadcast::Sender<RoomEvent>>>,
}

impl RoomBus {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Get-or-create the broadcast channel for `room_id`. Capacity is
    /// intentionally large so a slow consumer doesn't drop a fast
    /// producer; in practice the WebSocket handler falls behind on a
    /// real network and we should switch to a bounded queue + missed-
    /// event catch-up via the backlog table.
    pub async fn channel(&self, room_id: &str) -> broadcast::Sender<RoomEvent> {
        if let Some(tx) = self.rooms.read().await.get(room_id) {
            return tx.clone();
        }
        let mut w = self.rooms.write().await;
        w.entry(room_id.to_string())
            .or_insert_with(|| broadcast::channel(1024).0)
            .clone()
    }

    pub async fn drop_room(&self, room_id: &str) {
        self.rooms.write().await.remove(room_id);
    }
}

/// Append + broadcast in one call. Used by every event-emitting path.
pub async fn publish(
    pool: &SqlitePool,
    bus: &RoomBus,
    room_id: &str,
    user_id: Option<i64>,
    kind: &str,
    payload: &serde_json::Value,
) -> AppResult<RoomEvent> {
    let id = append_event(pool, room_id, user_id, kind, payload).await?;
    let event = RoomEvent {
        id,
        room_id: room_id.to_string(),
        user_id,
        kind: kind.to_string(),
        payload: payload.clone(),
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    // ignore send errors — slow consumers are OK to drop
    let _ = bus.channel(room_id).await.send(event.clone());
    Ok(event)
}

pub fn err_bad(msg: impl Into<String>) -> AppError {
    AppError::BadRequest(msg.into())
}
