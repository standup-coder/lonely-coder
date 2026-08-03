//! Data types crossing the wire and crossing the database boundary.
//!
//! `User` is the single source of truth — used both in the DB layer (with
//! `sqlx::FromRow`) and in the API response (via `serde`). Keeping them
//! the same shape avoids "two definitions of a user" drift; the small
//! downside is that internal-only fields would have to use `#[serde(skip)]`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub github_id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    /// JSON-encoded array, e.g. `["Rust","K8s"]`.
    pub skills: String,
    pub timezone: Option<String>,
    pub primary_ai: Option<String>,
    pub topic: Option<String>,
    #[serde(default)]
    pub is_dev_seed: i64,
    pub created_at: String,
    pub last_active_at: String,
}

impl User {
    /// Helper for handlers that need a display name with a fallback.
    /// Reserved for future use; current handlers prefer `display_name`
    /// via `UserPublic` which carries the Option through.
    #[allow(dead_code)]
    pub fn display_name_or(&self) -> &str {
        self.display_name
            .as_deref()
            .unwrap_or(self.username.as_str())
    }
}

/// `User` as the client sees it. Same data, but the `is_dev_seed` internal
/// flag is hidden so the prototype can't accidentally show "this is a fake
/// user" markers in the UI.
#[derive(Debug, Clone, Serialize)]
pub struct UserPublic {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub skills: Vec<String>,
    pub timezone: Option<String>,
    pub primary_ai: Option<String>,
    pub topic: Option<String>,
    pub created_at: String,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        let skills: Vec<String> = serde_json::from_str(&u.skills).unwrap_or_default();
        Self {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            bio: u.bio,
            skills,
            timezone: u.timezone,
            primary_ai: u.primary_ai,
            topic: u.topic,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub skills: Option<Vec<String>>,
    pub timezone: Option<String>,
    pub primary_ai: Option<String>,
    pub topic: Option<String>,
}

/// GitHub's `/user` response — only the fields we actually consume.
#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[allow(dead_code)]
pub struct Session {
    pub token: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
