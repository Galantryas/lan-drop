use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerNode {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub os_type: String,
    pub last_seen: u64,
    pub is_app_user: bool,
    pub is_blocked: bool,
    pub bandwidth_limit_kbps: Option<u32>, // None = unlimited, Some(kbps) = limit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub ip: String,
    pub name: String,
    pub port: u16,
    pub os_type: String,
    pub is_app_user: bool,
    pub is_online: bool,
    pub is_blocked: bool,
    pub bandwidth_limit_kbps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNodeInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub os_type: String,
    pub download_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardPayload {
    pub content: String,
    pub sender_name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    pub device_name: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgressPayload {
    pub transfer_id: String,
    pub file_name: String,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub is_complete: bool,
    pub is_error: bool,
    pub error_message: Option<String>,
    pub direction: String, // "outgoing" or "incoming"
    pub peer_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedFileRecord {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub sender_name: String,
    pub timestamp: String,
}

pub type PeerStore = Arc<RwLock<HashMap<String, PeerNode>>>;
