//! Library crate root. Exposes modules + a `build_router` so tests can
//! drive the app in-process without going through `main`.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod seed;

use axum::{
    extract::FromRef,
    http::Method,
    routing::{get, patch, post},
    Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub use auth::AppState;
pub use config::Config;

/// Build the axum router with the standard middleware stack. Used by
/// `main` and by tests; if you want a stripped-down router (e.g. without
/// CORS), call `Router::new()` yourself and add only the routes you need.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/auth/status", get(handlers::auth_status))
        .route("/auth/github", get(handlers::auth_github_start))
        .route(
            "/auth/github/callback",
            get(handlers::auth_github_callback),
        )
        .route("/auth/dev-login", get(handlers::auth_dev_login))
        .route("/auth/logout", post(handlers::auth_logout))
        .route("/api/me", get(handlers::api_me))
        .route("/api/me", patch(handlers::api_me_update))
        .route("/api/deck", get(handlers::api_deck))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_origin(Any)
                .allow_credentials(false)
                .allow_headers(Any),
        )
}

/// Apply the CORS layer to a router. Kept separate so tests can build a
/// router without CORS (irrelevant in-process) or with a custom origin
/// policy.
pub fn with_cors(router: Router) -> Router {
    router.layer(
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_origin(Any)
            .allow_credentials(false)
            .allow_headers(Any),
    )
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(s: &AppState) -> Self {
        s.pool.clone()
    }
}

/// Test / harness helper: build a config wired for dev mode against
/// `127.0.0.1:0`. The `Config::from_env()` path enforces non-loopback
/// refusal; tests need to bypass that. GitHub creds are present but
/// fake — enough to drive the OAuth handler past the "is configured"
/// guard into the state-cookie check, which is what tests assert on.
pub fn dev_config_for_testing() -> Arc<Config> {
    dev_config_with_github(Some(config::GitHubOAuth {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
    }))
}

/// Test variant: no GitHub creds. Use to assert the "OAuth not
/// configured" branch of the auth handlers.
pub fn dev_config_without_github() -> Arc<Config> {
    dev_config_with_github(None)
}

fn dev_config_with_github(github: Option<config::GitHubOAuth>) -> Arc<Config> {
    Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        database_url: "sqlite::memory:".to_string(),
        dev_mode: true,
        public_url: "http://127.0.0.1:0".to_string(),
        github,
        session_ttl_hours: 24,
        session_cookie_name: "cm_session".to_string(),
    })
}
