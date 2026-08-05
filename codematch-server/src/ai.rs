//! AI proxy — minimal OpenAI-compatible chat completion client.
//!
//! For the W2 prototype we want *one* AI to actually answer the room,
//! not just stub it. We standardise on the OpenAI Chat Completions
//! schema because Anthropic, Google (Vertex), DeepSeek, and others all
//! speak it (or a close enough variant). Per-user BYOK is on the W3
//! list; for now we read a single server-side key from the env so the
//! end-to-end loop can be exercised.
//!
//! Set `OPENAI_API_KEY` (and optionally `OPENAI_BASE_URL` and
//! `OPENAI_MODEL`) in the environment to enable. With no key set the
//! proxy returns a clear "AI disabled" event so the room still works
//! without one.

use crate::error::{AppError, AppResult};
use crate::models::User;
use crate::room::{self, RoomEvent};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Server-side AI config. Read from env at startup; immutable after.
#[derive(Clone, Debug, Default)]
pub struct AiConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
}

impl AiConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").ok().filter(|s| !s.is_empty()),
            base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into()),
        }
    }

    pub fn enabled(&self) -> bool {
        self.api_key.is_some()
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

/// Build the prompt we send to the AI. Concise and explicit about
/// the role: observer, not participant. We give it the recent chat
/// context (last ~12 messages) so it can actually comment meaningfully.
fn build_messages(
    system_prompt: &str,
    history: &[RoomEvent],
    user: &User,
) -> Vec<ChatMessage> {
    let mut msgs = vec![ChatMessage {
        role: "system".into(),
        content: system_prompt.to_string(),
    }];
    for ev in history.iter().rev().take(12).collect::<Vec<_>>().into_iter().rev() {
        let role = match ev.kind.as_str() {
            "chat" => "user",
            "ai.done" => "assistant",
            _ => continue,
        };
        // Coerce payload to a single string regardless of structure.
        let text = ev
            .payload
            .get("text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| ev.payload.to_string());
        let prefix = if role == "user" {
            // Add the author so the model has speaker context.
            format!("[user {}] {}", ev.user_id.unwrap_or(0), text)
        } else {
            text
        };
        msgs.push(ChatMessage { role: role.into(), content: prefix });
    }
    msgs.push(ChatMessage {
        role: "user".into(),
        content: format!(
            "[system] {} ({}'s AI) — please observe and contribute one short idea.",
            user.display_name_or(),
            user.username
        ),
    });
    msgs
}

/// Call the OpenAI-compatible API non-streamingly. We keep it simple
/// rather than streaming because:
///   - SSE parsing in the WebSocket fan-out is its own project
///   - the room already feels responsive when the model takes ~1.5s
///   - we can swap in streaming later without changing the public API
pub async fn ask_for_observation(
    http: &reqwest::Client,
    cfg: &AiConfig,
    user: &User,
    history: &[RoomEvent],
) -> AppResult<String> {
    let Some(key) = &cfg.api_key else {
        return Err(AppError::BadRequest("AI not configured on server".into()));
    };

    let system = "You are an observer in a 4-person brainstorming room. \
        The participants are collaborating on a topic and you're the user's \
        AI sidekick. Read the recent context and offer ONE short observation, \
        question, or idea. Be specific, not generic. Two sentences max. \
        Do not greet. Do not introduce yourself.";

    let req = ChatRequest {
        model: &cfg.model,
        messages: build_messages(system, history, user),
        stream: false,
    };

    let res = http
        .post(format!("{}/chat/completions", cfg.base_url))
        .bearer_auth(key)
        .json(&req)
        .send()
        .await
        .map_err(|e| AppError::Upstream(format!("openai: {e}")))?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(AppError::Upstream(format!("openai: {body}")));
    }

    let body: ChatResponse = res
        .json()
        .await
        .map_err(|e| AppError::Upstream(format!("openai decode: {e}")))?;

    let content = body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    Ok(content)
}

/// High-level helper: ask the AI, publish `ai.thinking` and `ai.done`
/// events. The WebSocket fan-out happens via `room::publish`.
pub async fn ask_and_publish(
    pool: &sqlx::SqlitePool,
    bus: &Arc<crate::room::RoomBus>,
    http: &reqwest::Client,
    cfg: &AiConfig,
    user: &User,
    room_id: &str,
) -> AppResult<RoomEvent> {
    // 1) "thinking" indicator
    let thinking_payload = serde_json::json!({
        "ai_name": cfg.model,
        "user_id": user.id,
    });
    room::publish(pool, bus, room_id, Some(user.id), "ai.thinking", &thinking_payload).await?;

    // 2) pull a small backlog for context
    let history = room::backlog(pool, room_id).await?;

    // 3) ask the model
    let text = ask_for_observation(http, cfg, user, &history).await?;

    // 4) publish the answer
    let done_payload = serde_json::json!({
        "ai_name": cfg.model,
        "user_id": user.id,
        "text": text,
    });
    room::publish(pool, bus, room_id, Some(user.id), "ai.done", &done_payload).await
}
