use super::*;
use crate::terminal::parser::extract_osc_events;
use crate::agent::protocol::OscEvent;

// ── parse_verb unit tests ─────────────────────────────────────────────────────

#[test]
fn parse_agent_hello() {
    let p = r#"{"v":1,"name":"claude-code","version":"1.2.0","pid":42}"#;
    match parse_verb("agent.hello", p) {
        OscVerb::AgentHello(h) => {
            assert_eq!(h.name, "claude-code");
            assert_eq!(h.version, "1.2.0");
            assert_eq!(h.pid, Some(42));
        }
        other => panic!("expected AgentHello, got {other:?}"),
    }
}

#[test]
fn parse_tool_start_minimal() {
    let p = r#"{"v":1,"id":"t1","name":"read_file","summary":"Read foo"}"#;
    match parse_verb("tool.start", p) {
        OscVerb::ToolStart(t) => {
            assert_eq!(t.id, "t1");
            assert_eq!(t.name, "read_file");
            assert!(t.args.is_none());
        }
        other => panic!("expected ToolStart, got {other:?}"),
    }
}

#[test]
fn parse_agent_query_capabilities() {
    let p = r#"{"v":1,"id":"q1","ask":"capabilities"}"#;
    match parse_verb("agent.query", p) {
        OscVerb::AgentQuery(q) => {
            assert_eq!(q.id, "q1");
            assert_eq!(q.ask, "capabilities");
        }
        other => panic!("expected AgentQuery, got {other:?}"),
    }
}

#[test]
fn parse_block_annotate_with_tags() {
    let p = r#"{"v":1,"agent_id":"aider","tags":["refactor","agent"],"fold":"auto"}"#;
    match parse_verb("block.annotate", p) {
        OscVerb::BlockAnnotate(a) => {
            assert_eq!(a.agent_id.as_deref(), Some("aider"));
            assert_eq!(a.tags, vec!["refactor", "agent"]);
            assert_eq!(a.fold.as_deref(), Some("auto"));
        }
        other => panic!("expected BlockAnnotate, got {other:?}"),
    }
}

#[test]
fn parse_block_annotate_tags_default_to_empty() {
    let p = r#"{"v":1}"#;
    match parse_verb("block.annotate", p) {
        OscVerb::BlockAnnotate(a) => {
            assert!(a.tags.is_empty());
            assert!(a.agent_id.is_none());
        }
        other => panic!("expected BlockAnnotate, got {other:?}"),
    }
}

#[test]
fn parse_unknown_verb() {
    match parse_verb("future.feature", r#"{"v":1}"#) {
        OscVerb::Unknown { verb, .. } => assert_eq!(verb, "future.feature"),
        other => panic!("expected Unknown, got {other:?}"),
    }
}

#[test]
fn parse_agent_bye() {
    assert!(matches!(parse_verb("agent.bye", "{}"), OscVerb::AgentBye));
    assert!(matches!(parse_verb("agent.bye", ""), OscVerb::AgentBye));
}

// ── OSC 7777 roundtrip tests ──────────────────────────────────────────────────
//
// These tests verify that:
// 1. An OSC 7777 sequence written by an agent is correctly parsed end-to-end.
// 2. The reply sequence format (written back to PTY stdin) is well-formed.

fn osc_bel(tag: &str, params: &[&str]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0x1b, 0x5d]; // ESC ]
    bytes.extend_from_slice(tag.as_bytes());
    for p in params {
        bytes.push(b';');
        bytes.extend_from_slice(p.as_bytes());
    }
    bytes.push(0x07); // BEL
    bytes
}

/// Build the reply sequence that the terminal writes back to PTY stdin.
fn reply_sequence(verb: &str, payload_json: &serde_json::Value) -> Vec<u8> {
    format!("\x1b]7777;{};{}\x1b\\", verb, payload_json).into_bytes()
}

#[test]
fn diff_payload_parses_correctly() {
    let json = r#"{"v":1,"id":"d_1","path":"src/main.rs","patch":"--- a/src/main.rs\n+++ b/src/main.rs\n","actions":["accept","reject"]}"#;
    match parse_verb("diff", json) {
        OscVerb::Diff(d) => {
            assert_eq!(d.id, "d_1");
            assert_eq!(d.path, "src/main.rs");
            assert_eq!(d.actions, ["accept", "reject"]);
        }
        other => panic!("expected Diff, got {other:?}"),
    }
}

#[test]
fn diff_accept_reply_is_valid_osc7777() {
    // Simulate: user clicks "Accept" → terminal writes reply to PTY stdin.
    let payload = serde_json::json!({ "v": 1, "id": "d_1", "result": "accept" });
    let seq = reply_sequence("reply", &payload);

    // Verify OSC wire format
    assert!(seq.starts_with(b"\x1b]7777;reply;"), "must start with OSC 7777 reply prefix");
    assert!(seq.ends_with(b"\x1b\\"), "must end with ST terminator");

    // Extract and parse the payload from the sequence
    let raw = std::str::from_utf8(&seq).unwrap();
    let prefix = "\x1b]7777;reply;";
    let suffix = "\x1b\\";
    let json_part = &raw[prefix.len()..raw.len() - suffix.len()];
    let parsed: serde_json::Value = serde_json::from_str(json_part).unwrap();
    assert_eq!(parsed["id"], "d_1");
    assert_eq!(parsed["result"], "accept");
}

#[test]
fn approval_payload_parses_correctly() {
    let json = r#"{"v":1,"id":"a_1","title":"Run command","detail":"rm -rf node_modules","risk":"destructive","actions":["allow_once","deny"]}"#;
    match parse_verb("approval", json) {
        OscVerb::Approval(a) => {
            assert_eq!(a.id, "a_1");
            assert_eq!(a.risk, "destructive");
            assert_eq!(a.actions, ["allow_once", "deny"]);
        }
        other => panic!("expected Approval, got {other:?}"),
    }
}

#[test]
fn approval_deny_reply_format() {
    let payload = serde_json::json!({ "v": 1, "id": "a_1", "result": "deny" });
    let seq = reply_sequence("reply", &payload);

    let raw = std::str::from_utf8(&seq).unwrap();
    assert!(raw.contains("\"result\":\"deny\""));
    assert!(raw.contains("\"id\":\"a_1\""));
}

#[test]
fn agent_query_roundtrip_via_parser() {
    // Agent emits: ESC ] 7777 ; agent.query ; {"v":1,"id":"q1","ask":"capabilities"} BEL
    let query_json = r#"{"v":1,"id":"q1","ask":"capabilities"}"#;
    let seq = osc_bel("7777", &["agent.query", query_json]);

    // Parser extracts the Osc7777 event
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1);
    let (verb, payload_str) = match &events[0] {
        OscEvent::Osc7777 { verb, payload } => (verb.as_str(), payload.as_str()),
        other => panic!("expected Osc7777, got {other:?}"),
    };
    assert_eq!(verb, "agent.query");

    // parse_verb converts it to the typed AgentQuery
    let osc_verb = parse_verb(verb, payload_str);
    let query = match osc_verb {
        OscVerb::AgentQuery(q) => q,
        other => panic!("expected AgentQuery, got {other:?}"),
    };

    // Terminal builds capability reply and writes to PTY stdin
    let reply_payload = serde_json::json!({
        "v": 1,
        "id": query.id,
        "result": {
            "osc_7777": ["tool.start","tool.progress","tool.end","diff","approval","session.mark","agent.bye"],
            "blocks": true
        }
    });
    let reply_seq = reply_sequence("reply", &reply_payload);

    // Validate reply is a well-formed OSC sequence
    assert!(reply_seq.starts_with(b"\x1b]7777;reply;"));
    assert!(reply_seq.ends_with(b"\x1b\\"));
    let raw = std::str::from_utf8(&reply_seq).unwrap();
    assert!(raw.contains("\"id\":\"q1\""));
    assert!(raw.contains("\"blocks\":true"));
    assert!(raw.contains("\"osc_7777\""));
}

#[test]
fn tool_cancel_event_format() {
    // Simulate: user cancels a running tool — terminal emits OSC 7777 event
    let payload = serde_json::json!({ "v": 1, "kind": "tool.cancel", "id": "t_42" });
    let seq = reply_sequence("event", &payload);
    let raw = std::str::from_utf8(&seq).unwrap();
    assert!(raw.starts_with("\x1b]7777;event;"));
    assert!(raw.contains("\"kind\":\"tool.cancel\""));
    assert!(raw.contains("\"id\":\"t_42\""));
}
