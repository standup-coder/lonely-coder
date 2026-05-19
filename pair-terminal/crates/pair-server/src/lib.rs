pub mod db;
pub mod matching;
pub mod session;
pub mod ws_handler;

use std::sync::Arc;

pub struct AppState {
    pub db: db::Db,
    pub session_mgr: Arc<session::SessionManager>,
    pub match_queue: Arc<matching::MatchQueue>,
    pub connection_count: Arc<std::sync::atomic::AtomicU32>,
}

const MAX_CONNECTIONS: u32 = 1000;

impl AppState {
    pub fn new(db: db::Db) -> Self {
        Self {
            db,
            session_mgr: Arc::new(session::SessionManager::new()),
            match_queue: Arc::new(matching::MatchQueue::new()),
            connection_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    pub fn try_connect(&self) -> bool {
        let count = self
            .connection_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count >= MAX_CONNECTIONS {
            self.connection_count
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            false
        } else {
            true
        }
    }

    pub fn disconnect(&self) {
        self.connection_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}
