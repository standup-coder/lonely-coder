//! Runtime configuration loaded from environment variables (and .env in dev).
//!
//! Every value is optional with a sensible default except the GitHub OAuth
//! credentials, which are *only* required when DEV_MODE is not set. The dev
//! path is intentionally low-friction so the user can run the binary and
//! see a working app before they have registered an OAuth App on GitHub.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,

    /// When true, /auth/dev-login?as=USERNAME creates a session without
    /// GitHub. Use only for local development. Refuse to enable in any
    /// non-loopback bind.
    pub dev_mode: bool,

    /// Public URL of the running server. Used to build the OAuth callback
    /// and the GitHub redirect URL.
    pub public_url: String,

    pub github: Option<GitHubOAuth>,
    pub session_ttl_hours: i64,
    /// Cookie name for the session token. Domain-scoped by the browser.
    pub session_cookie_name: String,
}

#[derive(Debug, Clone)]
pub struct GitHubOAuth {
    pub client_id: String,
    pub client_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Best-effort .env load; missing file is fine.
        let _ = dotenvy::dotenv();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| anyhow::anyhow!("PORT must be u16"))?;
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://codematch.db".to_string());
        let dev_mode = env::var("DEV_MODE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        // Public URL: where the running app believes it lives. Used to build
        // OAuth callback. Defaults to http://{host}:{port} when unset.
        let public_url = env::var("PUBLIC_URL")
            .unwrap_or_else(|_| format!("http://{}:{}", host, port));

        let github = match (
            env::var("GITHUB_CLIENT_ID").ok(),
            env::var("GITHUB_CLIENT_SECRET").ok(),
        ) {
            (Some(id), Some(secret)) if !id.is_empty() && !secret.is_empty() => Some(GitHubOAuth {
                client_id: id,
                client_secret: secret,
            }),
            _ => None,
        };

        if !dev_mode && github.is_none() {
            anyhow::bail!(
                "Either DEV_MODE=1 or both GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET must be set"
            );
        }

        if dev_mode && (host != "127.0.0.1" && host != "localhost" && host != "::1") {
            anyhow::bail!(
                "DEV_MODE=1 refuses to bind on a non-loopback host ({}) — set HOST=127.0.0.1",
                host
            );
        }

        let session_ttl_hours: i64 = env::var("SESSION_TTL_HOURS")
            .unwrap_or_else(|_| "720".to_string()) // 30 days
            .parse()
            .map_err(|_| anyhow::anyhow!("SESSION_TTL_HOURS must be integer"))?;

        let session_cookie_name =
            env::var("SESSION_COOKIE_NAME").unwrap_or_else(|_| "cm_session".to_string());

        Ok(Self {
            host,
            port,
            database_url,
            dev_mode,
            public_url,
            github,
            session_ttl_hours,
            session_cookie_name,
        })
    }
}
