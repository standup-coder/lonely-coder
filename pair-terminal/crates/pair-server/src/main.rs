use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use pair_server::AppState;
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

    let db = pair_server::db::Db::new(&cli.database).await?;
    let app_state = Arc::new(AppState::new(db));

    // Start the matching background task
    pair_server::matching::start_matching_task(app_state.match_queue.clone(), |pair| {
        tracing::info!(
            "Matched users: {} and {} (session: {})",
            pair.user_a,
            pair.user_b,
            pair.session_id
        );
    });

    let app = Router::new()
        .route("/ws", get(pair_server::ws_handler::handle_ws))
        .route("/health", get(health))
        .route(
            "/match/register",
            post(pair_server::matching::register_match),
        )
        .with_state(app_state);

    let addr = format!("{}:{}", cli.host, cli.port);
    tracing::info!("pair-server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
