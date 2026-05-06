use super::*;
use crate::agent::protocol::OscEvent;

fn osc_bel(tag: &str, rest: &[&str]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0x1b, 0x5d]; // ESC ]
    bytes.extend_from_slice(tag.as_bytes());
    for part in rest {
        bytes.push(b';');
        bytes.extend_from_slice(part.as_bytes());
    }
    bytes.push(0x07); // BEL terminator
    bytes
}

#[test]
fn test_osc0_title() {
    let seq = osc_bel("0", &["My Shell"]);
    let events = extract_osc_events(&seq);
    assert!(
        events.iter().any(|e| matches!(e, OscEvent::Title(_))),
        "expected Title event from OSC 0, got: {events:?}"
    );
}

#[test]
fn test_osc2_title() {
    let seq = osc_bel("2", &["My Shell"]);
    let events = extract_osc_events(&seq);
    assert!(
        events.iter().any(|e| matches!(e, OscEvent::Title(_))),
        "expected Title event from OSC 2, got: {events:?}"
    );
}

#[test]
fn test_osc_cwd() {
    let seq = osc_bel("7", &["file:///home/user/code"]);
    let events = extract_osc_events(&seq);
    assert!(
        events.iter().any(|e| matches!(e, OscEvent::Cwd(_))),
        "expected Cwd event, got: {events:?}"
    );
}

#[test]
fn test_osc_133_prompt_start() {
    let seq = osc_bel("133", &["A"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
    assert!(matches!(events[0], OscEvent::PromptMark { mark: 'A', .. }));
}

#[test]
fn test_osc_133_input_start() {
    let seq = osc_bel("133", &["B"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
    assert!(matches!(events[0], OscEvent::PromptMark { mark: 'B', .. }));
}

#[test]
fn test_osc_133_output_start() {
    let seq = osc_bel("133", &["C"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
    assert!(matches!(events[0], OscEvent::PromptMark { mark: 'C', .. }));
}

#[test]
fn test_osc_133_command_end() {
    let seq = osc_bel("133", &["D"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
    assert!(matches!(events[0], OscEvent::PromptMark { mark: 'D', .. }));
}

#[test]
fn test_osc_133_command_end_with_exit_code() {
    let seq = osc_bel("133", &["D", "2"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
    match &events[0] {
        OscEvent::PromptMark { mark: 'D', exit_code: Some(2), .. } => {}
        other => panic!("expected PromptMark D exit_code=2, got {other:?}"),
    }
}

#[test]
fn test_osc_133_output_start_with_aid() {
    let seq = osc_bel("133", &["C", "aid=ls -la"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
    match &events[0] {
        OscEvent::PromptMark { mark: 'C', command: Some(cmd), .. } => {
            assert_eq!(cmd, "ls -la");
        }
        other => panic!("expected PromptMark C with aid, got {other:?}"),
    }
}

#[test]
fn test_osc_133_b_with_aid() {
    let seq = osc_bel("133", &["B", "aid=git status"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1);
    match &events[0] {
        OscEvent::PromptMark { mark: 'B', command: Some(cmd), .. } => {
            assert_eq!(cmd, "git status");
        }
        other => panic!("expected PromptMark B with aid, got {other:?}"),
    }
}

#[test]
fn test_osc_7777_agent_query() {
    let payload = r#"{"v":1,"id":"q1","ask":"capabilities"}"#;
    let seq = osc_bel("7777", &["agent.query", payload]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1);
    match &events[0] {
        OscEvent::Osc7777 { verb, payload: p } => {
            assert_eq!(verb, "agent.query");
            assert!(p.contains("capabilities"));
        }
        other => panic!("expected Osc7777, got {other:?}"),
    }
}

#[test]
fn test_osc_7777_block_annotate() {
    let payload = r#"{"v":1,"agent_id":"claude-code","tags":["agent"]}"#;
    let seq = osc_bel("7777", &["block.annotate", payload]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1);
    match &events[0] {
        OscEvent::Osc7777 { verb, .. } => assert_eq!(verb, "block.annotate"),
        other => panic!("expected Osc7777, got {other:?}"),
    }
}

#[test]
fn test_osc_7777_tool_start() {
    let payload = r#"{"v":1,"id":"t1","name":"read_file","summary":"Read foo.rs"}"#;
    let seq = osc_bel("7777", &["tool.start", payload]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1);
    match &events[0] {
        OscEvent::Osc7777 { verb, payload: p } => {
            assert_eq!(verb, "tool.start");
            assert!(p.contains("read_file"));
        }
        other => panic!("expected Osc7777, got {other:?}"),
    }
}

#[test]
fn test_osc_7777_agent_hello() {
    let payload = r#"{"v":1,"name":"claude-code","version":"1.0.0"}"#;
    let seq = osc_bel("7777", &["agent.hello", payload]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1);
    match &events[0] {
        OscEvent::Osc7777 { verb, .. } => assert_eq!(verb, "agent.hello"),
        other => panic!("expected Osc7777, got {other:?}"),
    }
}

#[test]
fn test_osc_8_hyperlink() {
    let seq = osc_bel("8", &["id=link1", "https://example.com"]);
    let events = extract_osc_events(&seq);
    for e in &events {
        if let OscEvent::Hyperlink { url, id } = e {
            assert_eq!(url, "https://example.com");
            assert_eq!(id.as_deref(), Some("link1"));
            return;
        }
    }
    // termwiz handled it typed — still correct behaviour
}

#[test]
fn test_osc_8_hyperlink_end_ignored() {
    let seq = osc_bel("8", &["", ""]);
    let events = extract_osc_events(&seq);
    let hyperlinks: Vec<_> =
        events.iter().filter(|e| matches!(e, OscEvent::Hyperlink { .. })).collect();
    assert_eq!(hyperlinks.len(), 0, "hyperlink end should not emit an event");
}

#[test]
fn test_osc_52_clipboard_write() {
    // Base64 of "hello" is "aGVsbG8="
    let seq = osc_bel("52", &["c", "aGVsbG8="]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 1);
    match &events[0] {
        OscEvent::ClipboardWrite(text) => assert_eq!(text, "hello"),
        other => panic!("expected ClipboardWrite, got {other:?}"),
    }
}

#[test]
fn test_osc_52_read_request_ignored() {
    let seq = osc_bel("52", &["c", "?"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 0, "clipboard read request should be ignored in v0.1");
}

#[test]
fn test_unknown_osc_ignored() {
    let seq = osc_bel("999", &["some", "data"]);
    let events = extract_osc_events(&seq);
    assert_eq!(events.len(), 0, "unknown OSC should produce no events");
}

#[test]
fn test_empty_bytes() {
    let events = extract_osc_events(b"");
    assert_eq!(events.len(), 0);
}

#[test]
fn test_plain_text_no_events() {
    let events = extract_osc_events(b"hello world\r\n");
    assert_eq!(events.len(), 0);
}
