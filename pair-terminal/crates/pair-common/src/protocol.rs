use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    Handshake(HandshakePayload),
    PtyOutput(PtyOutputPayload),
    KeyInput(KeyInputPayload),
    Resize(ResizePayload),
    ModeChange(ModeChangePayload),
    AesKeys(AesKeysPayload),
    Chat(ChatPayload),
    Ping,
    MatchRegister(MatchRegisterPayload),
    MatchCancel,
    SnapshotRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    HandshakeOk(HandshakeOkPayload),
    PtyOutput(PtyOutputPayload),
    KeyInput(KeyInputPayload),
    Resize(ResizePayload),
    ModeChange(ModeChangePayload),
    AesKeys(AesKeysPayload),
    AesKeyRotation(AesKeysPayload),
    Chat(ChatPayload),
    Ping,
    Pong,
    SessionEnd(SessionEndPayload),
    Snapshot(SnapshotPayload),
    MatchStatus(MatchStatusPayload),
    Matched(MatchedPayload),
    MatchTimeout,
    FatalError(String),
    NewPeerConnected,
    NumClients(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    pub user_id: String,
    pub role: Role,
    pub cols: u16,
    pub rows: u16,
    pub terminal_id: Option<String>,
    pub mode: PairMode,
    pub allow_guest_control: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeOkPayload {
    pub session_id: String,
    pub role: Role,
    pub terminal_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyOutputPayload {
    pub data: String,
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInputPayload {
    pub data: String,
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizePayload {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeChangePayload {
    pub mode: PairMode,
    pub changed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AesKeysPayload {
    pub b64_output_key: String,
    pub b64_input_key: String,
    pub iv_count: u64,
    pub max_iv_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPayload {
    pub from: String,
    pub text: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndPayload {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRegisterPayload {
    pub user_id: String,
    pub preferences: MatchPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStatusPayload {
    pub position: u32,
    pub eta_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedPayload {
    pub peer: PeerInfo,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub user_id: String,
    pub username: String,
    pub skill_level: SkillLevel,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Host,
    Guest,
}
