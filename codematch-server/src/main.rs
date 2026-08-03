//! Entry point. Wires config → DB → router → bind.
//!
//! `DEV_MODE=1` short-circuits the OAuth path so you can run a real
//! server with a real DB and a real session cookie without registering
//! a GitHub OAuth App first.

mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod seed;

use axum::{
    extract::FromRef,
    http::Method,
    routing::{get, patch, post},
    Router,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use auth::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "codematch_server=debug,tower_http=info,axum=info".into()),
        )
        .init();

    let cfg = Arc::new(config::Config::from_env()?);
    tracing::info!(
        host = %cfg.host,
        port = cfg.port,
        dev_mode = cfg.dev_mode,
        github = cfg.github.is_some(),
        "starting codematch-server"
    );

    let pool = db::connect(&cfg.database_url).await?;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // Seed dev users once at startup. Safe to call repeatedly — the
    // upsert is idempotent on github_id. We only run this in dev mode
    // to avoid accidentally mutating a real DB on every boot.
    if cfg.dev_mode {
        if let Err(e) = seed::run(&pool).await {
            tracing::warn!(error = %e, "dev seed failed (non-fatal)");
        }
    }

    let state = AppState { pool, config: cfg.clone(), http };

    let app = Router::new()
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
        );

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(s: &AppState) -> Self { s.pool.clone() }
}
