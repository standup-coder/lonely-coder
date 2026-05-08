use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use pair_common::types::{SkillLevel, MatchPreferences};
use axum::{extract::State, Json};

#[derive(Debug, Clone)]
pub struct MatchRequest {
    pub user_id: String,
    pub username: String,
    pub preferences: MatchPreferences,
    pub enqueued_at: Instant,
}

#[derive(Debug, Clone)]
pub struct MatchedPair {
    pub user_a: String,
    pub user_b: String,
    pub session_id: String,
    pub matched_at: Instant,
}

pub struct MatchQueue {
    queue: RwLock<Vec<MatchRequest>>,
    match_threshold: f64,
}

impl MatchQueue {
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(Vec::new()),
            match_threshold: 0.3,
        }
    }

    pub async fn enqueue(&self, request: MatchRequest) {
        let mut queue = self.queue.write().await;
        queue.push(request);
    }

    pub async fn dequeue(&self, user_id: &str) -> Option<MatchRequest> {
        let mut queue = self.queue.write().await;
        if let Some(pos) = queue.iter().position(|r| r.user_id == user_id) {
            Some(queue.remove(pos))
        } else {
            None
        }
    }

    pub async fn position(&self, user_id: &str) -> Option<usize> {
        let queue = self.queue.read().await;
        queue.iter().position(|r| r.user_id == user_id)
    }

    pub async fn try_match(&self) -> Option<MatchedPair> {
        let mut queue = self.queue.write().await;

        if queue.len() < 2 {
            return None;
        }

        let mut best_score = 0.0f64;
        let mut best_i = 0;
        let mut best_j = 0;

        for i in 0..queue.len() {
            for j in (i + 1)..queue.len() {
                let score = calculate_match_score(&queue[i], &queue[j]);

                let wait_bonus_i = queue[i].enqueued_at.elapsed().as_secs() as f64 * 0.01;
                let wait_bonus_j = queue[j].enqueued_at.elapsed().as_secs() as f64 * 0.01;
                let total_score = score + wait_bonus_i + wait_bonus_j;

                if total_score > best_score {
                    best_score = total_score;
                    best_i = i;
                    best_j = j;
                }
            }
        }

        if best_score >= self.match_threshold {
            let a = queue.remove(best_j);
            let b = queue.remove(best_i);

            let session_id = uuid::Uuid::new_v4().to_string().replace('-', "")[..24].to_string();

            Some(MatchedPair {
                user_a: a.user_id,
                user_b: b.user_id,
                session_id,
                matched_at: Instant::now(),
            })
        } else {
            None
        }
    }

    pub async fn size(&self) -> usize {
        self.queue.read().await.len()
    }
}

fn calculate_match_score(a: &MatchRequest, b: &MatchRequest) -> f64 {
    let lang_a: std::collections::HashSet<_> = a.preferences.languages.iter().cloned().collect();
    let lang_b: std::collections::HashSet<_> = b.preferences.languages.iter().cloned().collect();

    let intersection = lang_a.intersection(&lang_b).count();
    let union = lang_a.union(&lang_b).count();

    let lang_score = if union == 0 { 0.0 } else { intersection as f64 / union as f64 };

    let skill_a = skill_value(&a.preferences.skill_level);
    let skill_b = skill_value(&b.preferences.skill_level);
    let skill_diff = (skill_a - skill_b).abs();
    let skill_score = 1.0 - skill_diff / 2.0;

    let mode_match = if a.preferences.mode == b.preferences.mode { 0.2 } else { 0.0 };

    lang_score + skill_score + mode_match
}

fn skill_value(level: &SkillLevel) -> f64 {
    match level {
        SkillLevel::Beginner => 0.0,
        SkillLevel::Intermediate => 1.0,
        SkillLevel::Expert => 2.0,
    }
}

pub fn start_matching_task(queue: Arc<MatchQueue>, on_match: impl Fn(MatchedPair) + Send + Sync + 'static) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            if let Some(pair) = queue.try_match().await {
                on_match(pair);
            }
        }
    });
}

// HTTP endpoint handler

use pair_common::protocol::MatchRegisterPayload;

#[derive(serde::Serialize)]
pub struct MatchResponse {
    pub status: String,
    pub message: String,
}

pub async fn register_match(
    State(queue): State<Arc<MatchQueue>>,
    Json(payload): Json<MatchRegisterPayload>,
) -> Json<MatchResponse> {
    let request = MatchRequest {
        user_id: payload.user_id.clone(),
        username: payload.user_id.clone(),
        preferences: payload.preferences.clone(),
        enqueued_at: std::time::Instant::now(),
    };

    queue.enqueue(request).await;

    let position = queue.position(&payload.user_id).await.unwrap_or(0) + 1;

    Json(MatchResponse {
        status: "queued".to_string(),
        message: format!("Position in queue: {}", position),
    })
}
