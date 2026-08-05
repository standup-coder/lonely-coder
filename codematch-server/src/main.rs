//! Entry point. Wires config → DB → router → bind.
//!
//! `DEV_MODE=1` short-circuits the OAuth path so you can run a real
//! server with a real DB and a real session cookie without registering
//! a GitHub OAuth App first.

use codematch_server::{
    auth::AppState, build_router, config, db, matching, room::RoomBus, seed,
};
use std::net::SocketAddr;
use std::sync::Arc;

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

    if cfg.dev_mode {
        if let Err(e) = seed::run(&pool).await {
            tracing::warn!(error = %e, "dev seed failed (non-fatal)");
        }
    }

    let room_bus = RoomBus::new();
    // Background matching engine: polls every 2s.
    matching::spawn_background_tasks(pool.clone());

    let state = AppState { pool, config: cfg.clone(), http, room_bus };
    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}
