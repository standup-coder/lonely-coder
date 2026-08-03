//! Database access. SQLite via sqlx. The schema lives in
//! `migrations/0001_init.sql` and is applied at startup; the file is
//! idempotent (CREATE TABLE IF NOT EXISTS) so re-runs are safe.

use crate::error::{AppError, AppResult};
use crate::models::{GitHubUser, UpdateProfileRequest, User};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::time::Duration;

const INIT_SQL: &str = include_str!("../migrations/0001_init.sql");

pub async fn connect(database_url: &str) -> AppResult<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|e| AppError::Internal(format!("connect {database_url}: {e}")))?;

    // The schema file is small and self-contained, so we execute the whole
    // thing in one round-trip rather than splitting on `;` (which breaks on
    // semicolons inside SQL comments — and we have one for context).
    sqlx::query(INIT_SQL)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "schema apply failed");
            AppError::Internal(format!("schema: {e}"))
        })?;
    Ok(pool)
}

pub async fn upsert_github_user(
    pool: &SqlitePool,
    gh: &GitHubUser,
) -> AppResult<User> {
    // SQLite's INSERT ... ON CONFLICT (col) DO UPDATE requires the column
    // to be UNIQUE. github_id is. We do the upsert in a single statement
    // so concurrent first-login races don't double-insert.
    sqlx::query(
        r#"
        INSERT INTO users
            (github_id, username, display_name, email, avatar_url, bio, last_active_at)
        VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
        ON CONFLICT(github_id) DO UPDATE SET
            username      = excluded.username,
            display_name  = excluded.display_name,
            email         = excluded.email,
            avatar_url    = excluded.avatar_url,
            bio           = excluded.bio,
            last_active_at = datetime('now')
        "#,
    )
    .bind(gh.id)
    .bind(&gh.login)
    .bind(gh.name.as_deref())
    .bind(gh.email.as_deref())
    .bind(gh.avatar_url.as_deref())
    .bind(gh.bio.as_deref())
    .execute(pool)
    .await?;

    fetch_by_github_id(pool, gh.id)
        .await?
        .ok_or_else(|| AppError::Internal("upsert succeeded but row missing".into()))
}

pub async fn fetch_by_github_id(
    pool: &SqlitePool,
    github_id: i64,
) -> AppResult<Option<User>> {
    let row = sqlx::query_as::<_, User>(
        r#"SELECT id, github_id, username, display_name, email, avatar_url, bio,
                  skills, timezone, primary_ai, topic, is_dev_seed,
                  created_at, last_active_at
           FROM users WHERE github_id = ?"#,
    )
    .bind(github_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn fetch_by_id(pool: &SqlitePool, id: i64) -> AppResult<Option<User>> {
    let row = sqlx::query_as::<_, User>(
        r#"SELECT id, github_id, username, display_name, email, avatar_url, bio,
                  skills, timezone, primary_ai, topic, is_dev_seed,
                  created_at, last_active_at
           FROM users WHERE id = ?"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn fetch_by_username(
    pool: &SqlitePool,
    username: &str,
) -> AppResult<Option<User>> {
    let row = sqlx::query_as::<_, User>(
        r#"SELECT id, github_id, username, display_name, email, avatar_url, bio,
                  skills, timezone, primary_ai, topic, is_dev_seed,
                  created_at, last_active_at
           FROM users WHERE username = ?"#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn update_profile(
    pool: &SqlitePool,
    user_id: i64,
    req: &UpdateProfileRequest,
) -> AppResult<User> {
    // Build the SET clause dynamically based on which fields are present.
    // We accept partial updates — only non-None fields are written.
    let mut sets: Vec<&str> = Vec::new();
    if req.display_name.is_some() { sets.push("display_name = ?"); }
    if req.bio.is_some()          { sets.push("bio = ?"); }
    if req.skills.is_some()       { sets.push("skills = ?"); }
    if req.timezone.is_some()     { sets.push("timezone = ?"); }
    if req.primary_ai.is_some()   { sets.push("primary_ai = ?"); }
    if req.topic.is_some()        { sets.push("topic = ?"); }
    sets.push("last_active_at = datetime('now')");

    let sql = format!("UPDATE users SET {} WHERE id = ?", sets.join(", "));
    let mut q = sqlx::query(&sql);
    if let Some(v) = &req.display_name { q = q.bind(v); }
    if let Some(v) = &req.bio          { q = q.bind(v); }
    if let Some(v) = &req.skills       { q = q.bind(serde_json::to_string(v).unwrap_or_else(|_| "[]".into())); }
    if let Some(v) = &req.timezone     { q = q.bind(v); }
    if let Some(v) = &req.primary_ai   { q = bind_ai(q, v.as_str()); }
    if let Some(v) = &req.topic        { q = q.bind(v); }
    q = q.bind(user_id);
    q.execute(pool).await?;

    fetch_by_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound)
}

fn bind_ai<'q>(
    q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    raw: &'q str,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    // Reject anything that isn't one of the 4 known values — protects the
    // DB column from arbitrary strings and keeps the client's enum honest.
    match raw {
        "claude" | "gpt4" | "gemini" | "deepseek" => q.bind(raw),
        _ => q.bind(Option::<&str>::None),
    }
}

pub async fn deck_for(
    pool: &SqlitePool,
    user_id: i64,
) -> AppResult<Vec<User>> {
    // Naive: return all other users ordered by last_active_at. The real
    // matching algorithm (skill overlap, timezone, complementary skills)
    // will replace this; the surface stays the same.
    let rows = sqlx::query_as::<_, User>(
        r#"SELECT id, github_id, username, display_name, email, avatar_url, bio,
                  skills, timezone, primary_ai, topic, is_dev_seed,
                  created_at, last_active_at
           FROM users
           WHERE id != ?
           ORDER BY last_active_at DESC
           LIMIT 50"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
