use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Ping,
    ListSources,
    AddSource { name: String, url: String },
    RemoveSource { id: String },
    Start { id: String },
    Stop,
    Status,
    Doctor,
    InstallDriver,
    Snapshot, // returns base64 jpeg of latest frame if available
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Response {
    pub fn ok(msg: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: msg.into(),
            data: None,
        }
    }
    pub fn ok_data(msg: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            ok: true,
            message: msg.into(),
            data: Some(data),
        }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: msg.into(),
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInfo {
    pub running: bool,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub fps: Option<f64>,
    pub frames_written: u64,
    pub device: Option<String>,
    pub error: Option<String>,
}

pub fn socket_path() -> std::path::PathBuf {
    let mut p = dirs::runtime_dir().unwrap_or_else(|| {
        let mut q = std::env::temp_dir();
        q.push("berrydesk");
        let _ = std::fs::create_dir_all(&q);
        q
    });
    p.push("berrydesk.sock");
    p
}
