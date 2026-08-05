//! Matching engine + lobby state machine.
//!
//! ## Flow
//!
//! ```text
//! user ──POST /api/match/queue──▶ match_queue
//!                                      │
//!                                      ▼
//!                          background task (every 2s)
//!                                      │
//!                                      ▼
//!                          form 4-person lobby
//!                          (lobby_seats × 4)
//!                                      │
//!                                      ▼
//!                          lobby status = "negotiating"
//!                          60s voting window
//!                                      │
//!                          all 4 vote 'accept'?
//!                          ├── yes ─▶ status = "matched" → create room
//!                          └── no  ─▶ close lobby, requeue the user
//! ```
//!
//! Scoring is intentionally simple: Jaccard language overlap + skill
//! complementarity. We don't try to be clever — the product decision
//! (4 people, mutual yes, BYOK) is the moat, not the algorithm.

use crate::error::{AppError, AppResult};
use crate::models::MatchPreferences;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::{Duration, Instant};

const SQUAD_SIZE: usize = 4;
const VOTE_WINDOW_SECONDS: i64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    pub user_id: i64,
    pub username: String,
    pub preferences: MatchPreferences,
    pub enqueued_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchStatus {
    pub in_queue: bool,
    pub queue_size: usize,
    /// How long the user has been waiting, in seconds.
    pub waited_seconds: i64,
    /// If a lobby was just created for this user, this is its id.
    pub pending_lobby_id: Option<String>,
}

// =====================================================================
// Queue CRUD
// =====================================================================

pub async fn enqueue(
    pool: &SqlitePool,
    user_id: i64,
    prefs: &MatchPreferences,
) -> AppResult<()> {
    let payload = serde_json::to_string(prefs).unwrap_or_else(|_| "{}".into());
    sqlx::query(
        "INSERT INTO match_queue (user_id, preferences) VALUES (?, ?)
         ON CONFLICT(user_id) DO UPDATE SET preferences = excluded.preferences,
                                           enqueued_at = datetime('now')",
    )
    .bind(user_id)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn dequeue(pool: &SqlitePool, user_id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM match_queue WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn status_for(
    pool: &SqlitePool,
    user_id: i64,
) -> AppResult<MatchStatus> {
    let row: Option<(String, i64)> = sqlx::query_as(
        "SELECT enqueued_at, CAST((julianday('now') - julianday(enqueued_at)) * 86400 AS INTEGER)
         FROM match_queue WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM match_queue")
        .fetch_one(pool)
        .await?;

    // Check if this user is a seat in any open (waiting/negotiating/matched)
    // lobby. We use `lobby_seats` rather than `created_by` because the
    // matching engine shuffles the group before forming a lobby, so the
    // "host" is whoever happened to land in seat 0 — not necessarily the
    // first user to enqueue. Any user with a seat should see the lobby.
    let pending_lobby: Option<String> = sqlx::query_scalar(
        "SELECT l.id FROM lobbies l
         JOIN lobby_seats s ON s.lobby_id = l.id
         WHERE s.user_id = ? AND l.status IN ('waiting', 'negotiating', 'matched')
         ORDER BY l.created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((_enq_at, waited)) => Ok(MatchStatus {
            in_queue: true,
            queue_size: total as usize,
            waited_seconds: waited,
            pending_lobby_id: pending_lobby,
        }),
        None => Ok(MatchStatus {
            in_queue: false,
            queue_size: total as usize,
            waited_seconds: 0,
            pending_lobby_id: pending_lobby,
        }),
    }
}

pub async fn queue_snapshot(pool: &SqlitePool) -> AppResult<Vec<QueueEntry>> {
    // Join with users so the engine can score against display_name + skills
    // without an N+1 follow-up.
    let rows: Vec<(i64, String, String, String, String)> = sqlx::query_as(
        r#"SELECT q.user_id, u.username, u.skills, u.primary_ai, q.preferences
           FROM match_queue q JOIN users u ON u.id = q.user_id
           ORDER BY q.enqueued_at ASC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(user_id, username, skills, primary_ai, prefs_json)| {
            let prefs: MatchPreferences =
                serde_json::from_str(&prefs_json).unwrap_or_default();
            // skills is the user's tags; we don't strictly need primary_ai
            // for scoring but it's useful for "their AI is X" hints.
            let _ = (skills, primary_ai);
            QueueEntry {
                user_id,
                username,
                preferences: prefs,
                enqueued_at: chrono::Utc::now(),
            }
        })
        .collect())
}

// =====================================================================
// Matching engine
// =====================================================================

/// Score a candidate group. Higher is better. We want:
///   - languages to overlap (people who can talk)
///   - skills to be complementary (don't put 4 staff Rust devs in a row)
///   - not all 4 of them in the same timezone (we like global squads)
fn score_group(candidates: &[QueueEntry]) -> f64 {
    // Jaccard overlap across all pairs of the group's language sets.
    let langs: Vec<Vec<String>> = candidates
        .iter()
        .map(|c| c.preferences.languages.clone())
        .collect();
    let mut lang_score = 0.0;
    let mut lang_pairs = 0;
    for i in 0..langs.len() {
        for j in (i + 1)..langs.len() {
            let a: std::collections::HashSet<_> = langs[i].iter().collect();
            let b: std::collections::HashSet<_> = langs[j].iter().collect();
            let inter = a.intersection(&b).count() as f64;
            let union = a.union(&b).count() as f64;
            if union > 0.0 {
                lang_score += inter / union;
            }
            lang_pairs += 1;
        }
    }
    let lang = if lang_pairs > 0 {
        lang_score / lang_pairs as f64
    } else {
        0.0
    };

    // Skill complementarity: 1.0 if everyone is at a *different* level
    // (we don't currently have explicit levels here, so we use a small
    // constant as a placeholder).
    let complement = 0.5;

    // Timezone spread: hash to a numeric bucket; we prefer a mix of
    // buckets. Real implementation would use the user's tz field.
    let tz_spread = 0.4;

    0.5 * lang + 0.3 * complement + 0.2 * tz_spread
}

/// Pull up to N candidates off the queue, score, and form a lobby.
/// Called by the background task; idempotent enough that running it
/// twice in a row is safe (the second call will see an empty queue
/// after the first created lobbies).
pub async fn try_form_lobby(pool: &SqlitePool) -> AppResult<Option<String>> {
    let queue = queue_snapshot(pool).await?;
    if queue.len() < SQUAD_SIZE {
        return Ok(None);
    }

    // Naive selection: take the oldest SQUAD_SIZE entries. The scoring
    // would be a future refinement; for the prototype we just want any
    // group of 4 to form. A weighted sample by `score_group` is on the
    // W3 list.
    let mut group: Vec<QueueEntry> = queue.into_iter().take(SQUAD_SIZE).collect();
    group.shuffle(&mut rand::thread_rng()); // randomise seat order
    let _ = score_group(&group); // exercise the path; we don't filter on it yet

    let lobby_id = format!("lobby-{}", generate_short_id());
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO lobbies (id, topic, status, created_by) VALUES (?, ?, 'negotiating', ?)",
    )
    .bind(&lobby_id)
    .bind(group[0].preferences.topic.as_deref().unwrap_or("(no topic)"))
    .bind(group[0].user_id)
    .execute(&mut *tx)
    .await?;

    let expires_at = (chrono::Utc::now() + chrono::Duration::seconds(VOTE_WINDOW_SECONDS))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    sqlx::query("UPDATE lobbies SET expires_at = ? WHERE id = ?")
        .bind(&expires_at)
        .bind(&lobby_id)
        .execute(&mut *tx)
        .await?;

    for (i, member) in group.iter().enumerate() {
        let role = if i == 0 { "host" } else { "guest" };
        sqlx::query(
            "INSERT INTO lobby_seats (lobby_id, user_id, seat_role) VALUES (?, ?, ?)",
        )
        .bind(&lobby_id)
        .bind(member.user_id)
        .bind(role)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM match_queue WHERE user_id = ?")
            .bind(member.user_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    tracing::info!(lobby_id = %lobby_id, size = group.len(), "formed lobby");
    Ok(Some(lobby_id))
}

/// Check whether a negotiating lobby has all-accept votes; if so,
/// promote it to "matched" and create the corresponding room.
pub async fn maybe_finalise_lobby(pool: &SqlitePool, lobby_id: &str) -> AppResult<Option<String>> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM lobby_seats WHERE lobby_id = ?",
    )
    .bind(lobby_id)
    .fetch_one(pool)
    .await?;
    let accepted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM lobby_seats WHERE lobby_id = ? AND vote = 'accept'",
    )
    .bind(lobby_id)
    .fetch_one(pool)
    .await?;
    let rejected: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM lobby_seats WHERE lobby_id = ? AND vote = 'skip'",
    )
    .bind(lobby_id)
    .fetch_one(pool)
    .await?;

    if total < SQUAD_SIZE as i64 {
        return Ok(None);
    }
    if accepted as i64 == total {
        // All accepted — promote + create room
        sqlx::query("UPDATE lobbies SET status = 'matched', matched_at = datetime('now') WHERE id = ?")
            .bind(lobby_id)
            .execute(pool)
            .await?;
        let room_id = format!("room-{}", generate_short_id());
        sqlx::query("INSERT INTO rooms (id, lobby_id) VALUES (?, ?)")
            .bind(&room_id)
            .bind(lobby_id)
            .execute(pool)
            .await?;
        tracing::info!(lobby_id = %lobby_id, room_id = %room_id, "lobby matched → room created");
        return Ok(Some(room_id));
    }
    if rejected > 0 {
        // Anyone skipped — close, requeue the survivors
        sqlx::query("UPDATE lobbies SET status = 'closed' WHERE id = ?")
            .bind(lobby_id)
            .execute(pool)
            .await?;
        // Re-add surviving members to the queue
        let survivors: Vec<(i64, String)> = sqlx::query_as(
            "SELECT user_id, '' FROM lobby_seats WHERE lobby_id = ? AND vote != 'skip'",
        )
        .bind(lobby_id)
        .fetch_all(pool)
        .await?;
        for (uid, _) in survivors {
            sqlx::query(
                "INSERT INTO match_queue (user_id, preferences) VALUES (?, '{}')
                 ON CONFLICT(user_id) DO NOTHING",
            )
            .bind(uid)
            .execute(pool)
            .await?;
        }
        tracing::info!(lobby_id = %lobby_id, "lobby closed due to skip votes");
    }
    Ok(None)
}

/// Sweep all negotiating lobbies and try to finalise each. Returns the
/// list of newly-created room ids.
pub async fn finalise_due_lobbies(pool: &SqlitePool) -> AppResult<Vec<String>> {
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM lobbies WHERE status = 'negotiating'",
    )
    .fetch_all(pool)
    .await?;
    let mut created = Vec::new();
    for id in ids {
        if let Some(room) = maybe_finalise_lobby(pool, &id).await? {
            created.push(room);
        }
    }
    Ok(created)
}

// =====================================================================
// Lobby view (used by API + tests)
// =====================================================================

#[derive(Debug, Clone, Serialize)]
pub struct LobbyView {
    pub id: String,
    pub topic: String,
    pub status: String,
    pub created_by: i64,
    pub seats: Vec<LobbySeatView>,
    pub created_at: String,
    pub matched_at: Option<String>,
    pub expires_at: Option<String>,
    pub room_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LobbySeatView {
    pub user_id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub primary_ai: Option<String>,
    pub seat_role: String,
    pub vote: Option<String>,
}

pub async fn lobby_view(pool: &SqlitePool, lobby_id: &str) -> AppResult<Option<LobbyView>> {
    let header: Option<(String, String, i64, String, Option<String>, Option<String>)> =
        sqlx::query_as(
            r#"SELECT id, topic, created_by, created_at, matched_at, expires_at
               FROM lobbies WHERE id = ?"#,
        )
        .bind(lobby_id)
        .fetch_optional(pool)
        .await?;

    let Some((id, topic, created_by, created_at, matched_at, expires_at)) = header else {
        return Ok(None);
    };

    let seats_raw: Vec<(
        i64, String, Option<String>, Option<String>, Option<String>, String, Option<String>,
    )> = sqlx::query_as(
        r#"SELECT s.user_id, u.username, u.display_name, u.avatar_url, u.primary_ai,
                  s.seat_role, s.vote
           FROM lobby_seats s JOIN users u ON u.id = s.user_id
           WHERE s.lobby_id = ?
           ORDER BY s.joined_at ASC"#,
    )
    .bind(lobby_id)
    .fetch_all(pool)
    .await?;

    let seats: Vec<LobbySeatView> = seats_raw
        .into_iter()
        .map(|(user_id, username, display_name, avatar_url, primary_ai, seat_role, vote)| {
            LobbySeatView { user_id, username, display_name, avatar_url, primary_ai, seat_role, vote }
        })
        .collect();

    // Look up the room id if matched/closed
    let room_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM rooms WHERE lobby_id = ?",
    )
    .bind(lobby_id)
    .fetch_optional(pool)
    .await?;

    let status: String = sqlx::query_scalar("SELECT status FROM lobbies WHERE id = ?")
        .bind(lobby_id)
        .fetch_one(pool)
        .await?;

    Ok(Some(LobbyView {
        id, topic, status, created_by, seats, created_at, matched_at, expires_at, room_id,
    }))
}

pub async fn cast_vote(
    pool: &SqlitePool,
    lobby_id: &str,
    user_id: i64,
    vote: &str,
) -> AppResult<LobbyView> {
    if !["accept", "skip"].contains(&vote) {
        return Err(AppError::BadRequest("vote must be 'accept' or 'skip'".into()));
    }
    let affected = sqlx::query(
        "UPDATE lobby_seats SET vote = ?, voted_at = datetime('now')
         WHERE lobby_id = ? AND user_id = ?",
    )
    .bind(vote)
    .bind(lobby_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    // After every vote, try to finalise
    maybe_finalise_lobby(pool, lobby_id).await?;
    lobby_view(pool, lobby_id)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn leave_lobby(
    pool: &SqlitePool,
    lobby_id: &str,
    user_id: i64,
) -> AppResult<()> {
    sqlx::query("DELETE FROM lobby_seats WHERE lobby_id = ? AND user_id = ?")
        .bind(lobby_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    // If the lobby is empty, close it.
    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM lobby_seats WHERE lobby_id = ?",
    )
    .bind(lobby_id)
    .fetch_one(pool)
    .await?;
    if remaining == 0 {
        sqlx::query("UPDATE lobbies SET status = 'closed' WHERE id = ?")
            .bind(lobby_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

// =====================================================================
// Background task: poll the queue + finalise lobbies
// =====================================================================

pub fn spawn_background_tasks(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(2));
        // The first tick fires immediately; skip it to give the server a
        // beat to start serving.
        tick.tick().await;
        loop {
            tick.tick().await;
            let started = Instant::now();
            if let Err(e) = sweep_once(&pool).await {
                tracing::warn!(error = %e, "matching sweep failed");
            } else {
                tracing::debug!(elapsed_ms = started.elapsed().as_millis() as u64, "sweep ok");
            }
        }
    });
}

async fn sweep_once(pool: &SqlitePool) -> AppResult<()> {
    // Form a lobby if we have enough
    if let Some(lobby_id) = try_form_lobby(pool).await? {
        tracing::info!(lobby_id = %lobby_id, "auto-formed lobby");
    }
    // Finalise any negotiating lobbies
    let new_rooms = finalise_due_lobbies(pool).await?;
    if !new_rooms.is_empty() {
        tracing::info!(?new_rooms, "auto-finalised lobbies");
    }
    Ok(())
}

/// Test-only: run one matching sweep synchronously. The background
/// task only runs in `main`; tests need to drive the engine
/// deterministically to avoid timing flakes.
pub async fn run_sweep_for_test(pool: &SqlitePool) -> AppResult<()> {
    sweep_once(pool).await
}

fn generate_short_id() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..8).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect()
}

// Tests can call score_group directly because the integration_test.rs
// file is in tests/, which compiles against the lib crate, so any
// `pub` function is reachable. The re-export was a misunderstanding.
