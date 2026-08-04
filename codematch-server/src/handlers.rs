//! HTTP handlers. Every handler returns `AppResult<T>` so the
//! `IntoResponse` impl in `error.rs` does the heavy lifting.

use crate::auth::{
    build_session_cookie, clear_session_cookie, create_session, delete_session,
    exchange_code_for_token, fetch_github_user, generate_oauth_state, AuthUser, AppState,
};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{UpdateProfileRequest, UserPublic};
use axum::{
    extract::{Query, State},
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};

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
