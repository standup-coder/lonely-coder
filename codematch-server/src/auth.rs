//! Session and OAuth plumbing.
//!
//! The session is a 32-byte random token stored in `sessions` and presented
//! via an HTTP-only cookie. The token is the *only* thing the cookie holds
//! — no user data is encoded. We never set `Secure` in dev (loopback HTTP)
//! but the code path is there for production.
//!
//! GitHub OAuth is a standard authorization-code flow. The state parameter
//! carries a CSRF nonce so a third-party redirect can't trick a logged-in
//! user into linking the attacker's account.

use crate::config::GitHubOAuth;
use crate::error::{AppError, AppResult};
use crate::models::{GitHubUser, User};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts, HeaderMap, StatusCode},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration as ChronoDuration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::sync::Arc;

#[allow(dead_code)]
pub const SESSION_COOKIE_DEFAULT: &str = "cm_session";

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub async fn create_session(
    pool: &SqlitePool,
    user_id: i64,
    ttl_hours: i64,
) -> AppResult<String> {
    let token = generate_token();
    let expires_at = (Utc::now() + ChronoDuration::hours(ttl_hours))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    sqlx::query(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)",
    )
    .bind(&token)
    .bind(user_id)
    .bind(&expires_at)
    .execute(pool)
    .await?;
    Ok(token)
}

pub async fn delete_session(pool: &SqlitePool, token: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn user_for_session_token(
    pool: &SqlitePool,
    token: &str,
) -> AppResult<Option<User>> {
    let row: Option<(i64, String)> = sqlx::query_as(
        r#"SELECT user_id, expires_at FROM sessions WHERE token = ?"#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    let Some((user_id, expires_at)) = row else {
        return Ok(None);
    };

    // Cheap client-side expiry check. A cron-style purge of expired rows
    // can be added later; for now the lookup just stops returning users.
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    if expires_at < now {
        let _ = delete_session(pool, token).await;
        return Ok(None);
    }

    crate::db::fetch_by_id(pool, user_id).await
}

pub fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim() == name {
                return Some(v.trim());
            }
        }
    }
    None
}

pub fn build_session_cookie(
    name: &str,
    token: &str,
    ttl_hours: i64,
    is_secure: bool,
) -> String {
    let max_age = ttl_hours * 3600;
    let secure = if is_secure { "; Secure" } else { "" };
    format!(
        "{name}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}",
    )
}

pub fn clear_session_cookie(name: &str) -> String {
    format!("{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

/// Extractor: pulls the session token from the Cookie header and resolves
/// it to a `User`. Handlers that need a logged-in user list this in their
/// signature; missing/invalid sessions turn into `401 Unauthorised`.
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app = AppState::from_ref(state);
        let token = read_cookie(&parts.headers, &app.config.session_cookie_name)
            .ok_or(AppError::Unauthorised)?;
        let user = user_for_session_token(&app.pool, token)
            .await?
            .ok_or(AppError::Unauthorised)?;
        Ok(AuthUser(user))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct OAuthState {
    /// Random nonce to defeat CSRF on the OAuth callback.
    pub nonce: String,
}

pub fn generate_oauth_state() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[allow(dead_code)]
pub fn hash_state(state: &str) -> String {
    let mut h = Sha256::new();
    h.update(state.as_bytes());
    URL_SAFE_NO_PAD.encode(h.finalize())
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GhAccessTokenResponse {
    access_token: String,
    scope: Option<String>,
    token_type: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhAccessTokenError {
    error: String,
    #[allow(dead_code)]
    error_description: Option<String>,
}

pub async fn exchange_code_for_token(
    http: &reqwest::Client,
    gh: &GitHubOAuth,
    code: &str,
) -> AppResult<String> {
    let res = http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("User-Agent", "codematch-server")
        .form(&[
            ("client_id", gh.client_id.as_str()),
            ("client_secret", gh.client_secret.as_str()),
            ("code", code),
        ])
        .send()
        .await
        .map_err(|e| AppError::Upstream(format!("github token: {e}")))?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(AppError::OAuth(format!("token exchange: {body}")));
    }

    // GitHub sometimes returns the error wrapped in the success-shape;
    // handle both.
    let text = res.text().await.unwrap_or_default();
    if let Ok(ok) = serde_json::from_str::<GhAccessTokenResponse>(&text) {
        if let Some(err) = ok.error {
            return Err(AppError::OAuth(format!("github: {err}")));
        }
        if !ok.access_token.is_empty() {
            return Ok(ok.access_token);
        }
    }
    if let Ok(err) = serde_json::from_str::<GhAccessTokenError>(&text) {
        return Err(AppError::OAuth(err.error));
    }
    Err(AppError::OAuth(format!("unexpected response: {text}")))
}

pub async fn fetch_github_user(
    http: &reqwest::Client,
    access_token: &str,
) -> AppResult<GitHubUser> {
    let res = http
        .get("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "codematch-server")
        .send()
        .await
        .map_err(|e| AppError::Upstream(format!("github user: {e}")))?;

    if res.status() == StatusCode::UNAUTHORIZED {
        return Err(AppError::OAuth("github rejected the access token".into()));
    }
    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(AppError::Upstream(format!("github user: {body}")));
    }
    let user: GitHubUser = res
        .json()
        .await
        .map_err(|e| AppError::Upstream(format!("decode github user: {e}")))?;
    Ok(user)
}

/// Shared application state. `axum::extract::FromRef` lets handlers
/// extract sub-pieces (e.g. just the pool) without cloning the whole
/// thing around.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Arc<crate::config::Config>,
    pub http: reqwest::Client,
}
