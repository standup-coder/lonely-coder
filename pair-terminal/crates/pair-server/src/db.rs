use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqlitePool};
use anyhow::Result;

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(path: &str) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                display_name TEXT,
                skill_level TEXT DEFAULT 'intermediate',
                languages TEXT DEFAULT '[]',
                total_sessions INTEGER DEFAULT 0,
                total_duration_secs INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                host_id TEXT NOT NULL,
                guest_id TEXT,
                mode TEXT NOT NULL,
                started_at TEXT DEFAULT (datetime('now')),
                ended_at TEXT,
                duration_secs INTEGER DEFAULT 0,
                recorded INTEGER DEFAULT 0
            )"
        ).execute(&pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS matches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_a TEXT NOT NULL,
                user_b TEXT,
                preferences TEXT,
                matched_at TEXT,
                accepted INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
