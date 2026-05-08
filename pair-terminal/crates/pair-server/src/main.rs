mod ws_handler;
mod session;
mod matching;
mod db;

use axum::{routing::{get, post}, Router};
use clap::Parser;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "pair-server", version, about = "pair-terminal relay server")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value = "pair.db")]
    database: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pair_server=debug,tower=info,axum=info".into()),
        )
        .init();

    let cli = Cli::parse();

    let db = db::Db::new(&cli.database).await?;
    let app_state = Arc::new(AppState::new(db));
    let session_mgr = Arc::new(session::SessionManager::new());
    let match_queue = Arc::new(matching::MatchQueue::new());

    let shared = Arc::new((app_state.clone(), session_mgr.clone()));

    let app = Router::new()
        .route("/ws", get(ws_handler::handle_ws))
        .route("/health", get(health))
        .route("/match/register", post(matching::register_match))
        .with_state(shared)
        .with_state(match_queue);

    let addr = format!("{}:{}", cli.host, cli.port);
    tracing::info!("pair-server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

pub struct AppState {
    pub db: db::Db,
}

impl AppState {
    pub fn new(db: db::Db) -> Self {
        Self { db }
    }
}
