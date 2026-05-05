use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::protocol::{OscVerb, parse_verb};
use super::tools::{ActiveTools, ToolStatus};
use crate::terminal::session::SessionId;

/// IPC event payloads emitted to the webview.

#[derive(Clone, Serialize)]
pub struct AgentHelloEvent {
    pub session_id: SessionId,
    pub name: String,
    pub version: String,
    pub pid: Option<u32>,
}

#[derive(Clone, Serialize)]
pub struct ToolStartEvent {
    pub session_id: SessionId,
    pub id: String,
    pub name: String,
    pub summary: String,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct ToolProgressEvent {
    pub session_id: SessionId,
    pub id: String,
    pub pct: Option<u8>,
    pub message: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct ToolEndEvent {
    pub session_id: SessionId,
    pub id: String,
    pub status: String,
    pub summary: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Dispatches a parsed OSC 7777 verb to the webview and updates tool state.
pub fn dispatch(
    verb: &str,
    payload: &str,
    session_id: &SessionId,
    tools: &mut ActiveTools,
    app: &AppHandle,
) {
    match parse_verb(verb, payload) {
        OscVerb::AgentHello(p) => {
            tracing::info!(session_id = %session_id, agent = %p.name, "agent connected");
            let _ = app.emit("agent:hello", AgentHelloEvent {
                session_id: session_id.clone(),
                name: p.name,
                version: p.version,
                pid: p.pid,
            });
        }
        OscVerb::ToolStart(p) => {
            tools.start(p.id.clone(), p.name.clone(), p.summary.clone(), p.icon.clone());
            let _ = app.emit("agent:tool_start", ToolStartEvent {
                session_id: session_id.clone(),
                id: p.id,
                name: p.name,
                summary: p.summary,
                icon: p.icon,
                parent_id: p.parent_id,
            });
        }
        OscVerb::ToolProgress(p) => {
            tools.progress(&p.id, p.pct, p.message.clone());
            let _ = app.emit("agent:tool_progress", ToolProgressEvent {
                session_id: session_id.clone(),
                id: p.id,
                pct: p.pct,
                message: p.message,
            });
        }
        OscVerb::ToolEnd(p) => {
            let status = match p.status.as_str() {
                "ok" => ToolStatus::Ok,
                "error" => ToolStatus::Error,
                "cancelled" => ToolStatus::Cancelled,
                _ => ToolStatus::Ok,
            };
            tools.end(&p.id, status, p.summary.clone(), p.duration_ms);
            let _ = app.emit("agent:tool_end", ToolEndEvent {
                session_id: session_id.clone(),
                id: p.id,
                status: p.status,
                summary: p.summary,
                duration_ms: p.duration_ms,
            });
        }
        OscVerb::Unknown { verb, .. } => {
            tracing::debug!(session_id = %session_id, verb = %verb, "unknown OSC 7777 verb");
        }
    }
}
