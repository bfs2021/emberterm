use serde::{Deserialize, Serialize};

/// Events extracted from OSC sequences by the terminal parser.
#[derive(Debug, Clone)]
pub enum OscEvent {
    Title(String),
    Cwd(String),
    PromptMark(char),
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
        _ => OscVerb::Unknown {
            verb: verb.to_string(),
            payload: payload.to_string(),
        },
    }
}
