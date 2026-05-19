use pair_common::types::{MatchPreferences, PairMode, SkillLevel};
use pair_server::matching::{calculate_match_score, skill_value, MatchQueue, MatchRequest};
use std::time::Instant;

#[tokio::test]
async fn test_match_queue_new() {
    let queue = MatchQueue::new();
    assert_eq!(queue.size().await, 0);
}

#[tokio::test]
async fn test_match_queue_enqueue_dequeue() {
    let queue = MatchQueue::new();
    let request = MatchRequest {
        user_id: "user1".to_string(),
        username: "User One".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let pos_before = queue.position(&request.user_id).await;
    assert!(pos_before.is_none());

    queue.enqueue(request.clone()).await;

    let size = queue.size().await;
    assert_eq!(size, 1);

    let pos = queue.position(&request.user_id).await;
    assert_eq!(pos, Some(0));

    let dequeued = queue.dequeue(&request.user_id).await;
    assert!(dequeued.is_some());
    assert_eq!(dequeued.unwrap().user_id, "user1");
}

#[tokio::test]
async fn test_match_queue_position() {
    let queue = MatchQueue::new();

    let request1 = MatchRequest {
        user_id: "user1".to_string(),
        username: "User One".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let request2 = MatchRequest {
        user_id: "user2".to_string(),
        username: "User Two".to_string(),
        preferences: MatchPreferences {
            languages: vec!["python".to_string()],
            skill_level: SkillLevel::Beginner,
            mode: PairMode::Navigator,
        },
        enqueued_at: Instant::now(),
    };

    queue.enqueue(request1.clone()).await;
    queue.enqueue(request2.clone()).await;

    let pos1 = queue.position(&request1.user_id).await;
    let pos2 = queue.position(&request2.user_id).await;

    assert_eq!(pos1, Some(0));
    assert_eq!(pos2, Some(1));
}

#[test]
fn test_calculate_match_score_language_overlap() {
    let request1 = MatchRequest {
        user_id: "u1".to_string(),
        username: "U1".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string(), "python".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let request2 = MatchRequest {
        user_id: "u2".to_string(),
        username: "U2".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string(), "go".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let score = calculate_match_score(&request1, &request2);
    assert!(score > 1.0);
}

#[test]
fn test_calculate_match_score_no_language_overlap() {
    let request1 = MatchRequest {
        user_id: "u1".to_string(),
        username: "U1".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let request2 = MatchRequest {
        user_id: "u2".to_string(),
        username: "U2".to_string(),
        preferences: MatchPreferences {
            languages: vec!["python".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let score = calculate_match_score(&request1, &request2);
    assert_eq!(score, 1.2);
}

#[test]
fn test_calculate_match_score_mode_mismatch() {
    let request1 = MatchRequest {
        user_id: "u1".to_string(),
        username: "U1".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let request2 = MatchRequest {
        user_id: "u2".to_string(),
        username: "U2".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Navigator,
        },
        enqueued_at: Instant::now(),
    };

    let score = calculate_match_score(&request1, &request2);
    assert_eq!(score, 2.0);
}

#[test]
fn test_skill_value() {
    assert_eq!(skill_value(&SkillLevel::Beginner), 0.0);
    assert_eq!(skill_value(&SkillLevel::Intermediate), 1.0);
    assert_eq!(skill_value(&SkillLevel::Expert), 2.0);
}

#[tokio::test]
async fn test_match_queue_try_match_insufficient_users() {
    let queue = MatchQueue::new();

    let request = MatchRequest {
        user_id: "user1".to_string(),
        username: "User One".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    queue.enqueue(request).await;

    let matched = queue.try_match().await;
    assert!(matched.is_none());
}

#[tokio::test]
async fn test_match_queue_try_match_sufficient_users() {
    let queue = MatchQueue::new();

    let request1 = MatchRequest {
        user_id: "user1".to_string(),
        username: "User One".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    let request2 = MatchRequest {
        user_id: "user2".to_string(),
        username: "User Two".to_string(),
        preferences: MatchPreferences {
            languages: vec!["rust".to_string()],
            skill_level: SkillLevel::Intermediate,
            mode: PairMode::Driver,
        },
        enqueued_at: Instant::now(),
    };

    queue.enqueue(request1.clone()).await;
    queue.enqueue(request2.clone()).await;

    let matched = queue.try_match().await;
    assert!(matched.is_some());
    let matched = matched.unwrap();
    assert!((matched.user_a == "user1" && matched.user_b == "user2") || (matched.user_a == "user2" && matched.user_b == "user1"));
    assert_eq!(matched.session_id.len(), 24);
}
