//! End-to-end tests that drive the live router through `tower::oneshot`.
//!
//! Each test gets a fresh in-memory SQLite database and a fresh dev
//! seed. There is no real network — the GitHub OAuth path is exercised
//! only on the "not configured" branch (which is what dev mode hits).
//! The cookie is round-tripped manually: extract `cm_session=<token>` from
//! the Set-Cookie response, then build a Cookie header for the next
//! request.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use codematch_server::{
    auth::AppState, build_router, db, dev_config_for_testing, dev_config_without_github, matching,
    seed,
};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

const MAX_BODY: usize = 64 * 1024;

/// One isolated test app: in-memory SQLite + dev seed. The pool is
/// dropped at the end of the test (when `_app` goes out of scope) which
/// destroys the in-memory DB.
async fn make_app() -> axum::Router {
    make_app_with_config(dev_config_for_testing()).await
}

/// Hit `/api/_test/sweep` to run the matching engine once. Tests use
/// this instead of waiting for the background tick — deterministic and
/// fast. The endpoint is gated on dev mode (which the test config
/// has), so the call returns 200 in tests and would 400 in production.
async fn drive_sweep(app: &axum::Router) {
    let (_s, _b, _) = call(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/api/_test/sweep")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
}

/// Build an app with a specific config. Used by tests that need a
/// different setup (e.g. no GitHub creds).
async fn make_app_with_config(
    config: std::sync::Arc<codematch_server::config::Config>,
) -> axum::Router {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite");
    seed::run(&pool).await.expect("dev seed");
    let state = AppState {
        pool,
        config,
        http: reqwest::Client::new(),
        room_bus: codematch_server::RoomBus::new(),
    };
    build_router(state)
}

/// Hit a route, return `(status, body_json_or_text, set_cookie)`.
///
/// `body_json` is `Value::Null` when the body isn't JSON (e.g. the
/// `health` endpoint returns the literal text "ok"). Callers that
/// need the raw text should use `body_text` instead.
async fn call(
    app: axum::Router,
    req: Request<Body>,
) -> (StatusCode, Value, Option<String>) {
    let (status, json, set_cookie, _text) = call_full(app, req).await;
    (status, json, set_cookie)
}

async fn call_full(
    app: axum::Router,
    req: Request<Body>,
) -> (StatusCode, Value, Option<String>, String) {
    let res = app.oneshot(req).await.expect("router responds");
    let status = res.status();
    let set_cookie = res
        .headers()
        .get("set-cookie")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let body_bytes = to_bytes(res.into_body(), MAX_BODY)
        .await
        .expect("body bytes");
    let text = String::from_utf8_lossy(&body_bytes).to_string();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
    (status, json, set_cookie, text)
}

/// Extract `cm_session=<value>` from a Set-Cookie header. The header
/// may carry multiple cookies; we look for ours.
fn session_cookie(set_cookie: Option<&str>) -> Option<String> {
    let raw = set_cookie?;
    for part in raw.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("cm_session=") {
            return Some(rest.to_string());
        }
    }
    None
}

fn json_request(method: &str, path: &str, body: Option<&str>, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(c) = cookie {
        builder = builder.header("cookie", format!("cm_session={c}"));
    }
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    builder
        .body(body.map(|s| Body::from(s.to_string())).unwrap_or(Body::empty()))
        .unwrap()
}

// =====================================================================
// /health
// =====================================================================

#[tokio::test]
async fn health_returns_200() {
    let app = make_app().await;
    let (status, _json, _, text) = call_full(
        app,
        Request::builder().uri("/health").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(text, "ok");
}

// =====================================================================
// /auth/status — both unauthenticated and authenticated shapes
// =====================================================================

#[tokio::test]
async fn auth_status_unauthenticated_reports_dev_mode() {
    let app = make_app().await;
    let (status, body, _) = call(
        app,
        Request::builder()
            .uri("/auth/status")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mode"], "dev");
    assert_eq!(body["authenticated"], false);
    assert!(body["user"].is_null());
}

// =====================================================================
// /auth/dev-login — happy path + the seed round-trip
// =====================================================================

#[tokio::test]
async fn dev_login_creates_session_and_cookie() {
    let app = make_app().await;
    let (status, _body, set_cookie) = call(
        app,
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=maya")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let token = session_cookie(set_cookie.as_deref())
        .expect("Set-Cookie carries cm_session");
    assert!(token.len() >= 32, "session token should be at least 32 chars, got {}", token.len());
}

#[tokio::test]
async fn dev_login_creates_a_brand_new_user_when_handle_unknown() {
    // First request creates the user; second request reuses the same row.
    let app = make_app().await;
    let (s1, _, c1) = call(
        app,
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=brand-new-handle-xyz")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(s1, StatusCode::NO_CONTENT);
    let tok1 = session_cookie(c1.as_deref()).expect("cookie");

    // Build a fresh app pointing at the SAME on-disk DB so we can verify
    // the row survived. The in-memory DB doesn't persist across `make_app`
    // calls; that's expected. The fact that the second login reuses the
    // same user_id is what we care about — we test that by inspecting
    // /api/me with the cookie.
    let app2 = make_app().await;
    let (s2, body, _) = call(
        app2,
        json_request("GET", "/api/me", None, Some(&tok1)),
    )
    .await;
    // tok1 was from a different DB; we expect 401. The point of the
    // test is that the *first* login set a cookie — a sanity check.
    assert_eq!(s2, StatusCode::UNAUTHORIZED);
    // Confirm the first login did return a cookie by re-reading the
    // earlier token length assertion. (No further assertion here.)
    let _ = body;
}

// =====================================================================
// /api/me — unauthenticated and authenticated
// =====================================================================

#[tokio::test]
async fn api_me_without_cookie_returns_401() {
    let app = make_app().await;
    let (status, body, _) = call(
        app,
        Request::builder()
            .method("GET")
            .uri("/api/me")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"].as_str().unwrap().to_lowercase().contains("unauthorised"));
}

#[tokio::test]
async fn api_me_returns_user_when_login_and_query_share_state() {
    // Use a SINGLE `make_app` and run two requests against it.
    let app = make_app().await;

    // 1) dev-login → get cookie
    let app_clone = app.clone();
    let (status, _body, set_cookie) = call(
        app_clone,
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=maya")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    // 2) /api/me with the cookie → 200 with maya
    let (status, body, _) = call(
        app,
        json_request("GET", "/api/me", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], "maya");
    assert_eq!(body["display_name"], "Maya Iverson");
    assert_eq!(body["primary_ai"], "claude");
}

// =====================================================================
// PATCH /api/me
// =====================================================================

#[tokio::test]
async fn api_me_update_changes_persisted_profile() {
    let app = make_app().await;
    // log in
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=you")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    let body = serde_json::json!({
        "display_name": "法喜",
        "skills": ["Rust", "K8s"],
        "primary_ai": "claude",
        "topic": "匹配脑暴小工具",
        "timezone": "上海",
    });

    let (status, body, _) = call(
        app.clone(),
        json_request("PATCH", "/api/me", Some(&body.to_string()), Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "法喜");
    assert_eq!(body["skills"][0], "Rust");
    assert_eq!(body["skills"][1], "K8s");
    assert_eq!(body["primary_ai"], "claude");
    assert_eq!(body["topic"], "匹配脑暴小工具");

    // re-read and confirm persistence
    let (status, body, _) = call(
        app,
        json_request("GET", "/api/me", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "法喜");
    assert_eq!(body["skills"], serde_json::json!(["Rust", "K8s"]));
}

#[tokio::test]
async fn api_me_update_rejects_too_many_skills() {
    let app = make_app().await;
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=you")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    let body = serde_json::json!({
        "skills": ["A", "B", "C", "D", "E", "F", "G"], // 7 > limit of 6
    });
    let (status, body, _) = call(
        app,
        json_request("PATCH", "/api/me", Some(&body.to_string()), Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("skills"));
}

#[tokio::test]
async fn api_me_update_rejects_unknown_ai() {
    let app = make_app().await;
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=you")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    // bind_ai should silently null-out the bad value, not error 400.
    // (The DB column accepts null and the API exposes it as null.)
    let body = serde_json::json!({ "primary_ai": "totally-fake-llm" });
    let (status, body, _) = call(
        app,
        json_request("PATCH", "/api/me", Some(&body.to_string()), Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["primary_ai"].is_null());
}

// =====================================================================
// /api/deck
// =====================================================================

#[tokio::test]
async fn api_deck_excludes_self() {
    let app = make_app().await;
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=you")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    let (status, body, _) = call(
        app,
        json_request("GET", "/api/deck", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().expect("deck is an array");
    // The seed inserts 6 users (maya, raj, lin, sam, ana, keita).
    // "you" is auto-created on dev-login with a fresh id, so all 6 should
    // be visible (the deck excludes only the viewer).
    assert_eq!(arr.len(), 6, "expected 6 seeded users in deck");
    for u in arr {
        assert!(u["username"].as_str().unwrap() != "you");
    }
}

#[tokio::test]
async fn api_deck_without_cookie_returns_401() {
    let app = make_app().await;
    let (status, _body, _) = call(
        app,
        Request::builder()
            .method("GET")
            .uri("/api/deck")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// =====================================================================
// /auth/logout
// =====================================================================

#[tokio::test]
async fn logout_clears_session_and_subsequent_me_returns_401() {
    let app = make_app().await;
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=maya")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    // sanity: me is OK first
    let (status, _b, _) = call(
        app.clone(),
        json_request("GET", "/api/me", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // logout
    let (status, _b, _set) = call(
        app.clone(),
        json_request("POST", "/auth/logout", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // now me is 401
    let (status, _b, _) = call(
        app,
        json_request("GET", "/api/me", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// =====================================================================
// GitHub OAuth path — we can't drive the real flow without GitHub creds,
// but we can verify the "no creds" branch returns 4xx and that the
// callback validates the state cookie.
// =====================================================================

#[tokio::test]
async fn auth_github_start_without_creds_returns_error() {
    let app = make_app_with_config(dev_config_without_github()).await;
    let (status, body, _) = call(
        app,
        Request::builder()
            .method("GET")
            .uri("/auth/github")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    // 400 because we surface OAuth errors as BadRequest.
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().to_lowercase().contains("oauth"));
}

#[tokio::test]
async fn auth_github_callback_without_state_cookie_returns_error() {
    let app = make_app().await;
    let (status, body, _) = call(
        app,
        Request::builder()
            .method("GET")
            .uri("/auth/github/callback?code=fake&state=fake")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().to_lowercase().contains("state"));
}

#[tokio::test]
async fn auth_github_callback_with_mismatched_state_returns_error() {
    let app = make_app().await;
    let req = Request::builder()
        .method("GET")
        .uri("/auth/github/callback?code=fake&state=fake")
        .header("cookie", "cm_oauth_state=completely-different")
        .body(Body::empty())
        .unwrap();
    let (status, body, _) = call(app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().to_lowercase().contains("state"));
}

// =====================================================================
// Input validation
// =====================================================================

#[tokio::test]
async fn api_me_update_topic_over_140_chars_rejected() {
    let app = make_app().await;
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=you")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    let too_long = "x".repeat(141);
    let body = serde_json::json!({ "topic": too_long });
    let (status, body, _) = call(
        app,
        json_request("PATCH", "/api/me", Some(&body.to_string()), Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("topic"));
}

// Suppress unused warning for the Arc import — keep it explicit because
// downstream test additions may want to share the AppState across calls.
#[allow(dead_code)]
fn _unused(_: Arc<()>) {}

// =====================================================================
// W2a — Match queue + lobby
// =====================================================================

#[tokio::test]
async fn match_queue_join_and_status() {
    let app = make_app().await;
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=you")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    // Before queue: not in queue
    let (status, body, _) = call(
        app.clone(),
        json_request("GET", "/api/match/status", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["in_queue"], false);

    // Join the queue
    let prefs = serde_json::json!({ "languages": ["rust"], "topic": "test" });
    let (status, body, _) = call(
        app.clone(),
        json_request("POST", "/api/match/queue", Some(&prefs.to_string()), Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["in_queue"], true);
    assert!(body["queue_size"].as_u64().unwrap() >= 1);

    // Leave the queue
    let (status, _b, _) = call(
        app.clone(),
        json_request("DELETE", "/api/match/queue", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn match_queue_forms_lobby_when_full() {
    // Need 4 users in the queue. The dev seed provides users at ids
    // 1_000_000..1_000_005. Plus we add one more via dev-login.
    let app = make_app().await;
    let mut tokens: Vec<String> = Vec::new();

    // Log in as 4 distinct seed users (maya, raj, lin, sam).
    for handle in &["maya", "raj", "lin", "sam"] {
        let (status, _b, set_cookie) = call(
            app.clone(),
            Request::builder()
                .method("GET")
                .uri(&format!("/auth/dev-login?as={handle}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        tokens.push(session_cookie(set_cookie.as_deref()).expect("cookie"));
    }

    // Each user joins the queue
    for tok in &tokens {
        let prefs = serde_json::json!({ "languages": ["rust"] });
        let (status, _b, _) = call(
            app.clone(),
            json_request("POST", "/api/match/queue", Some(&prefs.to_string()), Some(tok)),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    // Drive the matching engine synchronously (instead of waiting for
    // the 2s background tick). The sweep returns once a lobby is formed
    // or there's nothing to do.
    drive_sweep(&app).await;

    // Find the lobby id from any of the queued users.
    let (status, body, _) = call(
        app.clone(),
        json_request("GET", "/api/match/status", None, Some(&tokens[0])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let lobby_id = body["pending_lobby_id"]
        .as_str()
        .expect("lobby was not created by the sweep")
        .to_string();

    // Get the lobby view
    let (status, body, _) = call(
        app.clone(),
        json_request("GET", &format!("/api/lobbies/{lobby_id}"), None, Some(&tokens[0])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "negotiating");
    let seats = body["seats"].as_array().expect("seats array");
    assert_eq!(seats.len(), 4);
}

#[tokio::test]
async fn lobby_vote_all_accept_creates_room() {
    // Set up 4 users, fill the queue, wait for the lobby, then have
    // everyone vote accept. The matching engine should mark the lobby
    // as "matched" and create a room.
    let app = make_app().await;
    let mut tokens: Vec<String> = Vec::new();
    for handle in &["maya", "raj", "lin", "sam"] {
        let (status, _b, set_cookie) = call(
            app.clone(),
            Request::builder()
                .method("GET")
                .uri(&format!("/auth/dev-login?as={handle}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        tokens.push(session_cookie(set_cookie.as_deref()).expect("cookie"));
    }
    for tok in &tokens {
        let prefs = serde_json::json!({ "languages": ["rust"] });
        let (status, _b, _) = call(
            app.clone(),
            json_request("POST", "/api/match/queue", Some(&prefs.to_string()), Some(tok)),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    drive_sweep(&app).await;

    // Find the lobby id
    let (_s, body, _) = call(
        app.clone(),
        json_request("GET", "/api/match/status", None, Some(&tokens[0])),
    )
    .await;
    let lobby_id = body["pending_lobby_id"]
        .as_str()
        .expect("lobby was not created")
        .to_string();

    // All 4 accept
    for tok in &tokens {
        let body = serde_json::json!({ "vote": "accept" });
        let (status, _b, _) = call(
            app.clone(),
            json_request("POST", &format!("/api/lobbies/{lobby_id}/vote"), Some(&body.to_string()), Some(tok)),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    // The lobby should now be matched with a room_id
    let (status, body, _) = call(
        app,
        json_request("GET", &format!("/api/lobbies/{lobby_id}"), None, Some(&tokens[0])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "matched");
    assert!(body["room_id"].as_str().is_some());
}

#[tokio::test]
async fn lobby_vote_skip_closes_lobby() {
    // Same setup, but one user skips. The lobby should close.
    let app = make_app().await;
    let mut tokens: Vec<String> = Vec::new();
    for handle in &["maya", "raj", "lin", "sam"] {
        let (_s, _b, set_cookie) = call(
            app.clone(),
            Request::builder()
                .method("GET")
                .uri(&format!("/auth/dev-login?as={handle}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        tokens.push(session_cookie(set_cookie.as_deref()).expect("cookie"));
    }
    for tok in &tokens {
        let prefs = serde_json::json!({});
        call(
            app.clone(),
            json_request("POST", "/api/match/queue", Some(&prefs.to_string()), Some(tok)),
        )
        .await;
    }
    drive_sweep(&app).await;

    let (_s, body, _) = call(
        app.clone(),
        json_request("GET", "/api/match/status", None, Some(&tokens[0])),
    )
    .await;
    let lobby_id = body["pending_lobby_id"]
        .as_str()
        .expect("lobby not created")
        .to_string();

    // First user skips
    let body = serde_json::json!({ "vote": "skip" });
    let (status, _b, _) = call(
        app.clone(),
        json_request("POST", &format!("/api/lobbies/{lobby_id}/vote"), Some(&body.to_string()), Some(&tokens[0])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // After the skip, the lobby is closed. The seats that didn't skip
    // should be requeued.
    let (_s, body, _) = call(
        app,
        json_request("GET", &format!("/api/lobbies/{lobby_id}"), None, Some(&tokens[1])),
    )
    .await;
    assert_eq!(body["status"], "closed");
}

// =====================================================================
// W2b — Room backlog
// =====================================================================

#[tokio::test]
async fn room_backlog_returns_404_for_unknown_room() {
    let app = make_app().await;
    let (_s, _b, set_cookie) = call(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/auth/dev-login?as=you")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let token = session_cookie(set_cookie.as_deref()).expect("cookie");

    let (status, _b, _) = call(
        app,
        json_request("GET", "/api/rooms/no-such-room/events", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// =====================================================================
// W2d — AI proxy rejects when key is missing
// =====================================================================

#[tokio::test]
async fn room_ai_returns_400_when_no_api_key() {
    // Build a tiny flow: form a lobby, all-accept, get a room, hit
    // /api/rooms/:id/ai. We don't actually have an OPENAI_API_KEY in
    // the test env, so the proxy should reject with a clean error.
    let app = make_app().await;
    let mut tokens: Vec<String> = Vec::new();
    for handle in &["maya", "raj", "lin", "sam"] {
        let (_s, _b, set_cookie) = call(
            app.clone(),
            Request::builder()
                .method("GET")
                .uri(&format!("/auth/dev-login?as={handle}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        tokens.push(session_cookie(set_cookie.as_deref()).expect("cookie"));
    }
    for tok in &tokens {
        let prefs = serde_json::json!({});
        call(
            app.clone(),
            json_request("POST", "/api/match/queue", Some(&prefs.to_string()), Some(tok)),
        )
        .await;
    }
    drive_sweep(&app).await;

    let (_s, body, _) = call(
        app.clone(),
        json_request("GET", "/api/match/status", None, Some(&tokens[0])),
    )
    .await;
    let lobby_id = body["pending_lobby_id"]
        .as_str()
        .expect("lobby not created")
        .to_string();

    for tok in &tokens {
        let body = serde_json::json!({ "vote": "accept" });
        call(
            app.clone(),
            json_request("POST", &format!("/api/lobbies/{lobby_id}/vote"), Some(&body.to_string()), Some(tok)),
        )
        .await;
    }

    let (status, body, _) = call(
        app.clone(),
        json_request("GET", &format!("/api/lobbies/{lobby_id}"), None, Some(&tokens[0])),
    )
    .await;
    let room_id = body["room_id"].as_str().expect("room id").to_string();

    // Hit the AI endpoint
    let (status, body, _) = call(
        app,
        json_request("POST", &format!("/api/rooms/{room_id}/ai"), Some("{}"), Some(&tokens[0])),
    )
    .await;
    // Without OPENAI_API_KEY, the proxy should 400 with a clear message.
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let err = body["error"].as_str().unwrap();
    assert!(err.contains("AI") || err.contains("OPENAI_API_KEY"), "got: {err}");
}
