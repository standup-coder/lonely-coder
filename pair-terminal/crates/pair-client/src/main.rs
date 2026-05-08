mod share;
mod join;
mod match_cmd;
mod pty_host;
mod pty_guest;
mod raw_guard;
mod tui;
mod connect;

use clap::{Parser, Subcommand};

const DEFAULT_SERVER: &str = "wss://pair.dev/ws";
const DEFAULT_SIGNALING: &str = "wss://pair.dev/signal";

#[derive(Parser)]
#[command(name = "pair", version, about = "Terminal pair programming for two")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, env = "PAIR_SERVER", default_value = DEFAULT_SERVER)]
    server: String,

    #[arg(long, global = true, env = "PAIR_SIGNALING", default_value = DEFAULT_SIGNALING)]
    signaling: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Share your terminal with another developer
    Share {
        /// Command to run (default: $SHELL)
        #[arg(default_value = "", trailing_var_arg = true)]
        command: Vec<String>,

        /// Pair mode: driver, navigator, or collab
        #[arg(long, default_value = "collab")]
        mode: String,

        /// Record the session in asciinema v2 format
        #[arg(long)]
        record: bool,

        /// Connect via WebRTC P2P instead of relay server
        #[arg(long)]
        p2p: bool,
    },

    /// Join another developer's terminal session
    Join {
        /// Session URL (pair://session_id#bootstrap_key)
        url: String,
    },

    /// Get matched with a random developer
    Match {
        /// Preferred programming languages
        #[arg(long = "lang", num_args = 1..)]
        languages: Vec<String>,

        /// Skill level: beginner, intermediate, expert
        #[arg(long, default_value = "intermediate")]
        skill: String,

        /// Pair mode: driver, navigator, or collab
        #[arg(long, default_value = "collab")]
        mode: String,
    },

    /// Login with GitHub
    Login {
        #[arg(long, default_value = "github")]
        provider: String,
    },

    /// View your profile and stats
    Profile,

    /// View the leaderboard
    Leaderboard,

    /// Replay a recorded session
    Replay {
        /// Path to .cast file
        path: String,
    },

    /// Upload a recording to share
    Upload {
        /// Path to .cast file
        path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pair=debug,tokio=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Share { command, mode, record, p2p } => {
            share::run(&cli.server, &cli.signaling, &command, &mode, record, p2p).await
        }
        Commands::Join { url } => join::run(&cli.server, &url).await,
        Commands::Match { languages, skill, mode } => {
            match_cmd::run(&cli.server, &languages, &skill, &mode).await
        }
        Commands::Login { provider } => {
            println!("Login with {} is not yet implemented. Use pair profile --anonymous.", provider);
            Ok(())
        }
        Commands::Profile => {
            println!("Profile feature coming soon.");
            Ok(())
        }
        Commands::Leaderboard => {
            println!("Leaderboard feature coming soon.");
            Ok(())
        }
        Commands::Replay { path } => {
            replay_file(&path).await
        }
        Commands::Upload { path } => {
            println!("Upload feature coming soon. File: {}", path);
            Ok(())
        }
    }
}

async fn replay_file(path: &str) -> anyhow::Result<()> {
    use pair_common::recording::*;
    use std::io::{Read, Write};

    let reader = AsciiCastReader::from_file(&std::path::PathBuf::from(path))?;
    let events = reader.events();

    let _raw_guard = raw_guard::RawModeGuard::new()?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let mut prev_time = 0.0f64;

    for event in events {
        let delay = event.time - prev_time;
        if delay > 0.0 {
            tokio::time::sleep(std::time::Duration::from_secs_f64(delay)).await;
        }

        match event.event_type {
            AsciiCastEventType::Output => {
                let bytes: Vec<u8> = event.data.bytes().collect();
                stdout.write_all(&bytes)?;
                stdout.flush()?;
            }
            AsciiCastEventType::Input => {
                // skip input events during replay
            }
            AsciiCastEventType::Resize => {
                // could handle resize during replay
            }
        }

        prev_time = event.time;
    }

    println!("\n--- Replay finished ---");
    Ok(())
}
