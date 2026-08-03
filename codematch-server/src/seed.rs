//! Dev-only seed: pre-populate the database with a small set of users
//! that mirror the prototype's mock deck. The current session user is
//! excluded from the deck they see, so the seed has to be at least 5 users
//! for the prototype's "find 3 yes-swipes" flow to be testable.

use crate::db::upsert_github_user;
use crate::models::{GitHubUser, UpdateProfileRequest};
use sqlx::SqlitePool;

/// A canonical list of fake users. We use synthetic, large github_id values
/// (1_000_000 + N) so they can't accidentally collide with real signups.
const SEED_USERS: &[(&str, &str, &str, &str, &[&str])] = &[
    (
        "maya",
        "Maya Iverson",
        "Berlin",
        "Go / Distributed / Observability",
        &["go", "distributed", "observability"],
    ),
    (
        "raj",
        "Raj Venkatesh",
        "Bangalore",
        "Python / ML / Search",
        &["python", "ml", "search"],
    ),
    (
        "lin",
        "林夏",
        "北京",
        "边缘节点跑 LLM 推理 — 真的现实吗？",
        &["typescript", "edge", "infra"],
    ),
    (
        "sam",
        "Sam Okonkwo",
        "Lagos",
        "Betting against token-burn — 10×-cheaper coding agents?",
        &["rust", "compilers", "perf"],
    ),
    (
        "ana",
        "Ana Reis",
        "Lisbon",
        "Open-source maintainer burnout — what could tooling actually do?",
        &["ruby", "devex", "oss"],
    ),
    (
        "keita",
        "Keita Mori",
        "東京",
        "i18n is solved. l10n is not — what is everyone missing?",
        &["typescript", "i18n", "design"],
    ),
];

pub async fn run(pool: &SqlitePool) -> anyhow::Result<()> {
    for (i, (handle, name, tz, topic, skills)) in SEED_USERS.iter().enumerate() {
        let gh = GitHubUser {
            id: 1_000_000 + i as i64,
            login: (*handle).to_string(),
            name: Some((*name).to_string()),
            email: Some(format!("{handle}@devseed.local")),
            avatar_url: Some(format!(
                "https://api.dicebear.com/9.x/initials/png?seed={name}"
            )),
            bio: Some((*topic).to_string()),
        };

        let user = upsert_github_user(pool, &gh).await?;

        // Mark as dev seed so we don't accidentally show "fake" markers.
        sqlx::query("UPDATE users SET is_dev_seed = 1 WHERE id = ?")
            .bind(user.id)
            .execute(pool)
            .await?;

        // Apply the deck-visible fields (skills, timezone, topic).
        let req = UpdateProfileRequest {
            display_name: Some((*name).to_string()),
            bio: Some((*topic).to_string()),
            skills: Some(skills.iter().map(|s| (*s).to_string()).collect()),
            timezone: Some((*tz).to_string()),
            primary_ai: Some("claude".to_string()),
            topic: Some((*topic).to_string()),
        };
        crate::db::update_profile(pool, user.id, &req).await?;
    }
    tracing::info!("dev seed: inserted {} users", SEED_USERS.len());
    Ok(())
}
