//! Library crate root. Exposes modules + a `build_router` so tests can
//! drive the app in-process without going through `main`.

pub mod ai;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod matching;
pub mod models;
pub mod room;
pub mod seed;

use axum::{
    extract::FromRef,
    http::Method,
    routing::{get, patch, post},
    Router,
};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

pub use auth::AppState;
pub use config::Config;
pub use room::RoomBus;

/// Build the axum router with the standard middleware stack. Used by
/// `main` and by tests; if you want a stripped-down router (e.g. without
/// CORS), call `Router::new()` yourself and add only the routes you need.
pub fn build_router(state: AppState) -> Router {
    let bus = state.room_bus.clone();
    let mut app = Router::new()
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
        // W2: matching + lobby + room
        .route("/api/match/queue", post(handlers::api_match_queue))
        .route("/api/match/queue", axum::routing::delete(handlers::api_match_dequeue))
        .route("/api/match/status", get(handlers::api_match_status))
        .route("/api/lobbies/:id", get(handlers::api_lobby_get))
        .route("/api/lobbies/:id/join", post(handlers::api_lobby_join))
        .route("/api/lobbies/:id/leave", post(handlers::api_lobby_leave))
        .route("/api/lobbies/:id/vote", post(handlers::api_lobby_vote))
        .route("/api/rooms/:id/events", get(handlers::api_room_backlog))
        .route("/api/rooms/:id/ws", get(handlers::api_room_ws))
        .route("/api/rooms/:id/ai", post(handlers::api_room_ai))
        // Test-only: drive the matching sweep deterministically.
        .route("/api/_test/sweep", post(handlers::test_run_sweep))
        .with_state(state)
        .layer(axum::Extension(bus))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                // `allow_credentials(false)` is required for `*` per the
                // CORS spec. We rely on the prototype being served from
                // the same origin (no CORS) for the credentials path;
                // this CORS layer is just for the WebSocket upgrade
                // handshake and dev-mode cross-origin testing.
                .allow_origin(Any)
                .allow_credentials(false)
                .allow_headers(Any),
        );

    // Serve the prototype at `/app/*` from the repo's
    // `codematch-prototype/` directory. This avoids the entire CORS
    // credential problem: same origin → no CORS → cookies flow freely.
    if let Ok(prototype_root) = std::env::var("PROTOTYPE_DIR") {
        let p = PathBuf::from(prototype_root);
        if p.exists() {
            let index = p.join("index.html");
            let serve = ServeDir::new(&p).fallback(ServeFile::new(&index));
            app = app.fallback_service(serve);
            tracing::info!(path = %p.display(), "serving prototype at /app/");
        }
    }
    app
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
