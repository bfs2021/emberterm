use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Pending,
    Running,
    Ok,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallState {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub icon: Option<String>,
    pub status: ToolStatus,
    pub progress_pct: Option<u8>,
    pub progress_message: Option<String>,
    pub duration_ms: Option<u64>,
    pub started_at_ms: u64,
}

pub type ToolId = String;

#[derive(Debug, Default)]
pub struct ActiveTools {
    pub calls: HashMap<ToolId, ToolCallState>,
}

impl ActiveTools {
    pub fn start(&mut self, id: String, name: String, summary: String, icon: Option<String>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.calls.insert(
            id.clone(),
            ToolCallState {
                id,
                name,
                summary,
                icon,
                status: ToolStatus::Running,
                progress_pct: None,
                progress_message: None,
                duration_ms: None,
                started_at_ms: now,
            },
        );
    }

    pub fn progress(&mut self, id: &str, pct: Option<u8>, message: Option<String>) {
        if let Some(call) = self.calls.get_mut(id) {
            call.progress_pct = pct;
            call.progress_message = message;
        }
    }

    pub fn end(&mut self, id: &str, status: ToolStatus, summary: Option<String>, duration_ms: Option<u64>) {
        if let Some(call) = self.calls.get_mut(id) {
            call.status = status;
            if let Some(s) = summary {
                call.summary = s;
            }
            call.duration_ms = duration_ms;
        }
    }
}
