use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalId(pub String);

impl TerminalId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string().replace("-", "")[..24].to_string())
    }
}

impl SessionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string().replace('-', "")[..24].to_string())
    }
}

impl UserId {
    pub fn anonymous() -> Self {
        Self(format!("anon_{}", Uuid::new_v4().to_string()[..8].to_string()))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PairMode {
    Driver,
    Navigator,
    Collaborative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ServerMode {
    Public,
    SelfHosted,
    P2P,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPreferences {
    pub languages: Vec<String>,
    pub skill_level: SkillLevel,
    pub mode: PairMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: UserId,
    pub username: String,
    pub display_name: Option<String>,
    pub skill_level: SkillLevel,
    pub languages: Vec<String>,
    pub total_sessions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: SessionId,
    pub duration_secs: u64,
    pub mode: PairMode,
    pub recorded: bool,
}
