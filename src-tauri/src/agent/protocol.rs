use serde::{Deserialize, Serialize};

/// Events extracted from OSC sequences by the terminal parser.
#[derive(Debug, Clone)]
pub enum OscEvent {
    Title(String),
    Cwd(String),
    /// OSC 133 shell integration prompt marks.
    PromptMark {
        mark: char,
        /// Populated on 'D' marks: the command exit code.
        exit_code: Option<i32>,
        /// Populated on 'C' marks: optional command text from the `aid` parameter.
        command: Option<String>,
    },
    /// OSC 8 — hyperlink start (`id` from params, `url` is the href).
    Hyperlink { url: String, id: Option<String> },
    /// OSC 52 — clipboard write request (base64-encoded data).
    ClipboardWrite(String),
    Osc7777 { verb: String, payload: String },
}

/// Parsed OSC 7777 verbs from agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OscVerb {
    AgentHello(AgentHelloPayload),
    ToolStart(ToolStartPayload),
    ToolProgress(ToolProgressPayload),
    ToolEnd(ToolEndPayload),
    Diff(DiffPayload),
    Approval(ApprovalPayload),
    AgentBye,
    AgentQuery(AgentQueryPayload),
    BlockAnnotate(BlockAnnotatePayload),
    SessionMark(SessionMarkPayload),
    Unknown { verb: String, payload: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHelloPayload {
    pub v: u32,
    pub name: String,
    pub version: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStartPayload {
    pub v: u32,
    pub id: String,
    pub name: String,
    pub summary: String,
    pub args: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProgressPayload {
    pub v: u32,
    pub id: String,
    pub pct: Option<u8>,
    pub message: Option<String>,
    pub phase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEndPayload {
    pub v: u32,
    pub id: String,
    pub status: String,
    pub summary: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPayload {
    pub v: u32,
    pub id: String,
    pub path: String,
    pub patch: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPayload {
    pub v: u32,
    pub id: String,
    pub title: String,
    pub detail: String,
    pub risk: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentQueryPayload {
    pub v: u32,
    pub id: String,
    pub ask: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAnnotatePayload {
    pub v: u32,
    pub agent_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub fold: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMarkPayload {
    pub v: u32,
    pub label: Option<String>,
    pub kind: Option<String>,
}

/// Parses a raw OSC 7777 verb + payload string into a typed OscVerb.
pub fn parse_verb(verb: &str, payload: &str) -> OscVerb {
    match verb {
        "agent.hello" => serde_json::from_str::<AgentHelloPayload>(payload)
            .map(OscVerb::AgentHello)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "tool.start" => serde_json::from_str::<ToolStartPayload>(payload)
            .map(OscVerb::ToolStart)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "tool.progress" => serde_json::from_str::<ToolProgressPayload>(payload)
            .map(OscVerb::ToolProgress)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "tool.end" => serde_json::from_str::<ToolEndPayload>(payload)
            .map(OscVerb::ToolEnd)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "diff" => serde_json::from_str::<DiffPayload>(payload)
            .map(OscVerb::Diff)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "approval" => serde_json::from_str::<ApprovalPayload>(payload)
            .map(OscVerb::Approval)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "agent.bye" => OscVerb::AgentBye,
        "block.annotate" => serde_json::from_str::<BlockAnnotatePayload>(payload)
            .map(OscVerb::BlockAnnotate)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "agent.query" => serde_json::from_str::<AgentQueryPayload>(payload)
            .map(OscVerb::AgentQuery)
            .unwrap_or_else(|_| OscVerb::Unknown {
                verb: verb.to_string(),
                payload: payload.to_string(),
            }),
        "session.mark" => serde_json::from_str::<SessionMarkPayload>(payload)
            .map(OscVerb::SessionMark)
            .unwrap_or(OscVerb::SessionMark(SessionMarkPayload { v: 1, label: None, kind: None })),
        _ => OscVerb::Unknown {
            verb: verb.to_string(),
            payload: payload.to_string(),
        },
    }
}

#[cfg(test)]
#[path = "protocol_test.rs"]
mod tests;
