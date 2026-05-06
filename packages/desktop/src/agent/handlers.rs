use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::protocol::{OscVerb, parse_verb};
pub use super::protocol::BlockAnnotatePayload;
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
    pub args: Option<serde_json::Value>,
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

#[derive(Clone, Serialize)]
pub struct AgentDiffEvent {
    pub session_id: SessionId,
    pub id: String,
    pub path: String,
    pub patch: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct AgentApprovalEvent {
    pub session_id: SessionId,
    pub id: String,
    pub title: String,
    pub detail: String,
    pub risk: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct AgentByeEvent {
    pub session_id: SessionId,
}

#[derive(Clone, Serialize)]
pub struct AgentSessionMarkEvent {
    pub session_id: SessionId,
    pub label: Option<String>,
    pub kind: Option<String>,
}

/// Effect returned by `dispatch()` telling the PTY reader thread what to do
/// with any bytes the terminal needs to produce.
pub enum DispatchEffect {
    /// Inject bytes into the terminal output stream (renders in xterm.js).
    TerminalOutput(Vec<u8>),
    /// Write bytes into the PTY input stream so the agent reads them as stdin.
    PtyInput(Vec<u8>),
    /// Annotate the current executing block with agent metadata.
    AnnotateBlock(BlockAnnotatePayload),
}

/// Dispatches a parsed OSC 7777 verb to the webview and updates tool state.
/// Returns an optional `DispatchEffect` describing side-channel bytes to produce.
pub fn dispatch(
    verb: &str,
    payload: &str,
    session_id: &SessionId,
    tools: &mut ActiveTools,
    app: &AppHandle,
) -> Option<DispatchEffect> {
    match parse_verb(verb, payload) {
        OscVerb::AgentHello(p) => {
            tracing::info!(session_id = %session_id, agent = %p.name, "agent connected");
            let _ = app.emit("agent:hello", AgentHelloEvent {
                session_id: session_id.clone(),
                name: p.name,
                version: p.version,
                pid: p.pid,
            });
            None
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
                args: p.args,
            });
            None
        }
        OscVerb::ToolProgress(p) => {
            tools.progress(&p.id, p.pct, p.message.clone());
            let _ = app.emit("agent:tool_progress", ToolProgressEvent {
                session_id: session_id.clone(),
                id: p.id,
                pct: p.pct,
                message: p.message,
            });
            None
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
            None
        }
        OscVerb::Diff(p) => {
            tracing::info!(session_id = %session_id, id = %p.id, path = %p.path, "agent diff");
            let _ = app.emit("agent:diff", AgentDiffEvent {
                session_id: session_id.clone(),
                id: p.id,
                path: p.path,
                patch: p.patch,
                actions: p.actions,
            });
            None
        }
        OscVerb::Approval(p) => {
            tracing::info!(session_id = %session_id, id = %p.id, "agent approval request");
            let _ = app.emit("agent:approval", AgentApprovalEvent {
                session_id: session_id.clone(),
                id: p.id,
                title: p.title,
                detail: p.detail,
                risk: p.risk,
                actions: p.actions,
            });
            None
        }
        OscVerb::AgentBye => {
            tracing::info!(session_id = %session_id, "agent disconnected");
            let _ = app.emit("agent:bye", AgentByeEvent { session_id: session_id.clone() });
            None
        }
        OscVerb::BlockAnnotate(p) => {
            tracing::debug!(
                session_id = %session_id,
                agent_id = ?p.agent_id,
                "block annotate"
            );
            Some(DispatchEffect::AnnotateBlock(p))
        }
        OscVerb::AgentQuery(p) => {
            tracing::debug!(session_id = %session_id, id = %p.id, ask = %p.ask, "agent query");
            let reply_json = serde_json::json!({
                "v": 1,
                "id": p.id,
                "result": {
                    "osc_7777": [
                        "tool.start", "tool.progress", "tool.end",
                        "diff", "approval", "session.mark", "agent.bye"
                    ],
                    "blocks": true,
                    "kitty_keyboard": true,
                    "graphics": ["kitty"]
                }
            });
            let sequence = format!("\x1b]7777;reply;{}\x1b\\", reply_json);
            Some(DispatchEffect::PtyInput(sequence.into_bytes()))
        }
        OscVerb::SessionMark(p) => {
            tracing::debug!(session_id = %session_id, label = ?p.label, "session mark");
            let _ = app.emit("agent:session_mark", AgentSessionMarkEvent {
                session_id: session_id.clone(),
                label: p.label.clone(),
                kind: p.kind,
            });
            Some(DispatchEffect::TerminalOutput(make_separator(p.label.as_deref())))
        }
        OscVerb::Unknown { verb, .. } => {
            tracing::debug!(session_id = %session_id, verb = %verb, "unknown OSC 7777 verb");
            None
        }
    }
}

fn make_separator(label: Option<&str>) -> Vec<u8> {
    const WIDTH: usize = 50;
    const LINE: &str = "─";
    let text = match label {
        Some(l) if !l.is_empty() => format!(" {} ", l),
        _ => String::new(),
    };
    let text_len = text.chars().count();
    let half = (WIDTH.saturating_sub(text_len)) / 2;
    let left = LINE.repeat(half);
    let right = LINE.repeat(WIDTH.saturating_sub(text_len + half));
    let separator = format!("\r\n\x1b[2m{}{}{}\x1b[0m\r\n", left, text, right);
    separator.into_bytes()
}
