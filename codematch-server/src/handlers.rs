//! HTTP handlers. Every handler returns `AppResult<T>` so the
//! `IntoResponse` impl in `error.rs` does the heavy lifting.

use crate::auth::{
    build_session_cookie, clear_session_cookie, create_session, delete_session,
    exchange_code_for_token, fetch_github_user, generate_oauth_state, AuthUser, AppState,
};
use crate::ai::{self, AiConfig};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::matching;
use crate::models::{MatchPreferences, UpdateProfileRequest, UserPublic};
use crate::room::{self, RoomBus, RoomEvent};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Extension, Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// =====================================================================
// Health
// =====================================================================

pub async fn health() -> &'static str {
    "ok"
}

// =====================================================================
// Auth — GitHub OAuth (real flow)
// =====================================================================

#[derive(Serialize)]
pub struct AuthStatus {
    /// "real" when GitHub creds are configured, "dev" when DEV_MODE bypasses
    /// GitHub. The client uses this to choose what login button to show.
    mode: &'static str,
    authenticated: bool,
    user: Option<UserPublic>,
}

pub async fn auth_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<AuthStatus>> {
    let mode = if state.config.dev_mode { "dev" } else { "real" };
    let user = match crate::auth::read_cookie(&headers, &state.config.session_cookie_name) {
        Some(tok) => crate::auth::user_for_session_token(&state.pool, tok).await?,
        None => None,
    };
    Ok(Json(AuthStatus {
        mode,
        authenticated: user.is_some(),
        user: user.map(UserPublic::from),
    }))
}

pub async fn auth_github_start(
    State(state): State<AppState>,
) -> AppResult<Response> {
    let gh = state
        .config
        .github
        .as_ref()
        .ok_or_else(|| AppError::OAuth("GitHub OAuth not configured".into()))?;

    let nonce = generate_oauth_state();
    let redirect_uri = format!("{}/auth/github/callback", state.config.public_url);

    let mut url = url::Url::parse("https://github.com/login/oauth/authorize")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("client_id", &gh.client_id);
        q.append_pair("redirect_uri", &redirect_uri);
        q.append_pair("scope", "read:user user:email");
        q.append_pair("state", &nonce);
    }

    // Stash the nonce in a short-lived cookie so the callback can compare
    // against what we sent. HTTP-only to keep JS out of it.
    let cookie = format!(
        "{name}={nonce}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600",
        name = "cm_oauth_state"
    );

    let mut resp = Redirect::to(url.as_str()).into_response();
    resp.headers_mut()
        .insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(resp)
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub async fn auth_github_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<CallbackQuery>,
) -> AppResult<Response> {
    let gh = state
        .config
        .github
        .as_ref()
        .ok_or_else(|| AppError::OAuth("GitHub OAuth not configured".into()))?;

    if let Some(err) = q.error {
        return Err(AppError::OAuth(format!("github returned: {err}")));
    }
    let code = q.code.ok_or_else(|| AppError::OAuth("missing code".into()))?;

    // CSRF: verify the state cookie matches the state param.
    let expected = crate::auth::read_cookie(&headers, "cm_oauth_state")
        .ok_or_else(|| AppError::OAuth("missing oauth state cookie".into()))?;
    let got = q.state.as_deref().unwrap_or("");
    if expected != got {
        return Err(AppError::OAuth("oauth state mismatch".into()));
    }

    // 1) code → access_token
    let access_token = exchange_code_for_token(&state.http, gh, &code).await?;
    // 2) access_token → user info
    let gh_user = fetch_github_user(&state.http, &access_token).await?;
    // 3) upsert user
    let user = db::upsert_github_user(&state.pool, &gh_user).await?;
    // 4) create session
    let token = create_session(&state.pool, user.id, state.config.session_ttl_hours).await?;
    let cookie = build_session_cookie(
        &state.config.session_cookie_name,
        &token,
        state.config.session_ttl_hours,
        state.config.public_url.starts_with("https://"),
    );
    // Clear the state cookie and redirect home.
    let mut resp = Redirect::to("/").into_response();
    let cookies = format!(
        "{clear}; {cookie}",
        clear = clear_session_cookie("cm_oauth_state"),
        cookie = cookie,
    );
    resp.headers_mut()
        .insert(SET_COOKIE, HeaderValue::from_str(&cookies).unwrap());
    Ok(resp)
}

pub async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Response> {
    if let Some(tok) = crate::auth::read_cookie(&headers, &state.config.session_cookie_name) {
        delete_session(&state.pool, tok).await?;
    }
    let mut resp = (StatusCode::NO_CONTENT).into_response();
    let cookie = clear_session_cookie(&state.config.session_cookie_name);
    resp.headers_mut()
        .insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(resp)
}

// =====================================================================
// Auth — DEV_MODE bypass
// =====================================================================
//
// Only available when DEV_MODE=1. Looks up a username from the seeded
// users; if not found, creates a brand-new account on the fly. Either
// way the response is identical to a real GitHub login: a session cookie.

#[derive(Deserialize)]
pub struct DevLoginQuery {
    // `as` is a Rust keyword; serde's default field name is the Rust name
    // (with the trailing underscore). We rename on the wire so the URL
    // stays `?as=foo` — matching what the prototype and curl tests do.
    #[serde(rename = "as")]
    pub as_: Option<String>,
}

pub async fn auth_dev_login(
    State(state): State<AppState>,
    Query(q): Query<DevLoginQuery>,
) -> AppResult<Response> {
    if !state.config.dev_mode {
        return Err(AppError::BadRequest("dev login is disabled".into()));
    }
    // Refuse to log in as "you" implicitly — that handle reads as a
    // placeholder. If you want a throwaway user, pass `?as=guest` or
    // similar; otherwise we require the caller to be explicit.
    let handle = match q.as_ {
        Some(h) if !h.is_empty() => h.to_ascii_lowercase(),
        _ => return Err(AppError::BadRequest("missing ?as=HANDLE".into())),
    };

    // Find the seed user, or invent a brand-new one.
    let gh = if let Some(user) = db::fetch_by_username(&state.pool, &handle).await? {
        GitHubUserLite {
            id: user.github_id,
            login: user.username.clone(),
        }
    } else {
        let id = 9_000_000 + rand::random::<i64>().rem_euclid(1_000_000);
        sqlx::query(
            r#"INSERT INTO users (github_id, username, display_name, last_active_at)
               VALUES (?, ?, ?, datetime('now'))"#,
        )
        .bind(id)
        .bind(&handle)
        .bind(Some(&handle))
        .execute(&state.pool)
        .await?;
        GitHubUserLite { id, login: handle.clone() }
    };

    let _ = gh.login; // silence unused-field warning until the lite struct is consumed elsewhere
    let user = db::fetch_by_github_id(&state.pool, gh.id)
        .await?
        .ok_or(AppError::Internal("dev login: user missing after upsert".into()))?;

    let token = create_session(&state.pool, user.id, state.config.session_ttl_hours).await?;
    let cookie = build_session_cookie(
        &state.config.session_cookie_name,
        &token,
        state.config.session_ttl_hours,
        false,
    );
    let mut resp = (StatusCode::NO_CONTENT).into_response();
    resp.headers_mut()
        .insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(resp)
}

struct GitHubUserLite {
    id: i64,
    login: String,
}

// =====================================================================
// API — me
// =====================================================================

pub async fn api_me(AuthUser(user): AuthUser) -> AppResult<Json<UserPublic>> {
    Ok(Json(UserPublic::from(user)))
}

pub async fn api_me_update(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserPublic>> {
    // Validate skills length; cap so the deck card stays readable.
    if let Some(skills) = &req.skills {
        if skills.len() > 6 {
            return Err(AppError::BadRequest(
                "skills: at most 6 entries".into(),
            ));
        }
    }
    // Cap topic length too — it's shown on the card.
    if let Some(topic) = &req.topic {
        if topic.chars().count() > 140 {
            return Err(AppError::BadRequest(
                "topic: at most 140 characters".into(),
            ));
        }
    }
    let updated = db::update_profile(&state.pool, user.id, &req).await?;
    Ok(Json(UserPublic::from(updated)))
}

// =====================================================================
// API — deck
// =====================================================================

pub async fn api_deck(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<Vec<UserPublic>>> {
    let users = db::deck_for(&state.pool, user.id).await?;
    Ok(Json(users.into_iter().map(UserPublic::from).collect()))
}

// =====================================================================
// W2a — Match queue + lobby
// =====================================================================

#[derive(Deserialize)]
pub struct QueueRequest {
    #[serde(default)]
    pub preferences: Option<MatchPreferences>,
}

pub async fn api_match_queue(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<QueueRequest>,
) -> AppResult<Json<matching::MatchStatus>> {
    let prefs = req.preferences.unwrap_or_default();
    matching::enqueue(&state.pool, user.id, &prefs).await?;
    matching::status_for(&state.pool, user.id)
        .await
        .map(Json)
}

pub async fn api_match_dequeue(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<StatusCode> {
    matching::dequeue(&state.pool, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn api_match_status(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<matching::MatchStatus>> {
    matching::status_for(&state.pool, user.id).await.map(Json)
}

pub async fn api_lobby_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<matching::LobbyView>> {
    matching::lobby_view(&state.pool, &id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

/// Join the lobby by id. Used by the user-flow when the queue engine
/// has already created a lobby for them and the client navigates
/// directly to /lobby/:id. (Currently the match engine's auto-form
/// also writes the user as a seat, so this is mostly a no-op — kept
/// for explicit "share a lobby link" UX.)
pub async fn api_lobby_join(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<matching::LobbyView>> {
    sqlx::query(
        "INSERT OR IGNORE INTO lobby_seats (lobby_id, user_id, seat_role)
         VALUES (?, ?, 'guest')",
    )
    .bind(&id)
    .bind(user.id)
    .execute(&state.pool)
    .await?;
    matching::lobby_view(&state.pool, &id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn api_lobby_leave(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AuthUser(user): AuthUser,
) -> AppResult<StatusCode> {
    matching::leave_lobby(&state.pool, &id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct VoteRequest {
    pub vote: String,
}

pub async fn api_lobby_vote(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AuthUser(user): AuthUser,
    Json(req): Json<VoteRequest>,
) -> AppResult<Json<matching::LobbyView>> {
    matching::cast_vote(&state.pool, &id, user.id, &req.vote).await.map(Json)
}

// =====================================================================
// W2b — Room: backlog + WebSocket + AI proxy
// =====================================================================

pub async fn api_room_backlog(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AuthUser(_user): AuthUser,
) -> AppResult<Json<Vec<RoomEvent>>> {
    if !room::room_exists(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let events = room::backlog(&state.pool, &id).await?;
    Ok(Json(events))
}

pub async fn api_room_ws(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AuthUser(user): AuthUser,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    if !room::room_exists(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let bus = state.room_bus.clone();
    let pool = state.pool.clone();
    let user_id = user.id;
    let user_name = user.username.clone();
    let room_id = id;
    Ok(ws.on_upgrade(move |socket| async move {
        handle_room_socket(socket, pool, bus, room_id, user_id, user_name).await;
    }))
}

async fn handle_room_socket(
    socket: WebSocket,
    pool: sqlx::SqlitePool,
    bus: std::sync::Arc<RoomBus>,
    room_id: String,
    user_id: i64,
    user_name: String,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // 1) announce the join
    let join_payload = serde_json::json!({
        "user_id": user_id,
        "username": user_name,
    });
    let _ = room::publish(
        &pool, &bus, &room_id, Some(user_id), "system.peer_joined", &join_payload,
    )
    .await;

    // 2) send the backlog
    if let Ok(events) = room::backlog(&pool, &room_id).await {
        for ev in events {
            let payload = serde_json::to_string(&ev).unwrap_or_default();
            if ws_tx.send(Message::Text(payload.into())).await.is_err() {
                return;
            }
        }
    }

    // 3) subscribe to live events for this room
    let mut rx = bus.channel(&room_id).await.subscribe();

    // 4) write loop: forward bus events to the WebSocket
    let bus_task = tokio::spawn(async move {
        while let Ok(ev) = rx.recv().await {
            let payload = serde_json::to_string(&ev).unwrap_or_default();
            if ws_tx.send(Message::Text(payload.into())).await.is_err() {
                break;
            }
        }
    });

    // 5) read loop: accept chat events from the client
    while let Some(msg) = ws_rx.next().await {
        let Ok(msg) = msg else { break; };
        match msg {
            Message::Text(text) => {
                let text = text.to_string();
                if let Ok(req) = serde_json::from_str::<ClientRoomMessage>(&text) {
                    match req.kind.as_str() {
                        "chat" => {
                            let text = req
                                .payload
                                .get("text")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            if text.is_empty() {
                                continue;
                            }
                            let payload = serde_json::json!({
                                "user_id": user_id,
                                "username": user_name,
                                "text": text,
                            });
                            let _ = room::publish(
                                &pool, &bus, &room_id, Some(user_id), "chat", &payload,
                            )
                            .await;
                        }
                        "canvas.put" => {
                            // Persist as-is; the client is the source of
                            // truth for canvas structure.
                            let _ = room::publish(
                                &pool, &bus, &room_id, Some(user_id), "canvas.put",
                                &req.payload,
                            )
                            .await;
                        }
                        _ => {}
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    bus_task.abort();

    // 6) announce the leave
    let leave_payload = serde_json::json!({ "user_id": user_id });
    let _ = room::publish(
        &pool, &bus, &room_id, Some(user_id), "system.peer_left", &leave_payload,
    )
    .await;
}

#[derive(Deserialize)]
struct ClientRoomMessage {
    kind: String,
    #[serde(default)]
    payload: serde_json::Value,
}

pub async fn api_room_ai(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(bus): Extension<Arc<RoomBus>>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<RoomEvent>> {
    if !room::room_exists(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let cfg = AiConfig::from_env();
    if !cfg.enabled() {
        return Err(AppError::BadRequest(
            "AI is not configured on the server (set OPENAI_API_KEY)".into(),
        ));
    }
    let event = ai::ask_and_publish(
        &state.pool, &bus, &state.http, &cfg, &user, &id,
    )
    .await?;
    Ok(Json(event))
}

// =====================================================================
// Test-only endpoints (gated on dev mode)
// =====================================================================
//
// These are exposed so in-process tests can drive background tasks
// deterministically. The main server enables them automatically; in
// production with DEV_MODE=0 they would 403. We don't currently
// deploy the binary in production, so we keep them simple.

pub async fn test_run_sweep(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    if !state.config.dev_mode {
        return Err(AppError::BadRequest("test endpoint disabled in non-dev".into()));
    }
    matching::run_sweep_for_test(&state.pool).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
