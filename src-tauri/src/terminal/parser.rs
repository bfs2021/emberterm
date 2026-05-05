use termwiz::escape::osc::FinalTermSemanticPrompt;
use termwiz::escape::{Action, OperatingSystemCommand, parser::Parser};

use crate::agent::protocol::OscEvent;

/// Feeds a chunk of raw PTY bytes through the termwiz parser and extracts
/// any OSC sequences we care about. Returns parsed OSC events; all bytes
/// (including OSC sequences) are still forwarded to xterm.js unchanged.
pub fn extract_osc_events(bytes: &[u8]) -> Vec<OscEvent> {
    let mut parser = Parser::new();
    let mut events = Vec::new();

    parser.parse(bytes, |action| {
        if let Action::OperatingSystemCommand(osc) = action {
            if let Some(event) = handle_osc(&osc) {
                events.push(event);
            }
        }
    });

    events
}

fn handle_osc(osc: &OperatingSystemCommand) -> Option<OscEvent> {
    match osc {
        // OSC 0 sets both icon name and window title
        OperatingSystemCommand::SetIconNameAndWindowTitle(title) |
        OperatingSystemCommand::SetWindowTitle(title) => {
            Some(OscEvent::Title(title.clone()))
        }
        OperatingSystemCommand::CurrentWorkingDirectory(cwd) => {
            Some(OscEvent::Cwd(cwd.clone()))
        }
        // OSC 133 — FinalTerm/shell integration prompt marks
        OperatingSystemCommand::FinalTermSemanticPrompt(prompt) => {
            let mark = match prompt {
                FinalTermSemanticPrompt::FreshLineAndStartPrompt { .. } => 'A',
                FinalTermSemanticPrompt::MarkEndOfPromptAndStartOfInputUntilNextMarker
                | FinalTermSemanticPrompt::MarkEndOfPromptAndStartOfInputUntilEndOfLine => 'B',
                FinalTermSemanticPrompt::MarkEndOfInputAndStartOfOutput { .. } => 'C',
                FinalTermSemanticPrompt::CommandStatus { .. } => 'D',
                _ => return None,
            };
            Some(OscEvent::PromptMark(mark))
        }
        OperatingSystemCommand::Unspecified(parts) => handle_unspecified_osc(parts),
        _ => None,
    }
}

fn handle_unspecified_osc(parts: &[Vec<u8>]) -> Option<OscEvent> {
    let first = parts.first()?;
    let tag = std::str::from_utf8(first).ok()?;

    match tag {
        // OSC 133 — shell prompt marks (A=prompt start, B=command start, C=output start, D=command end)
        "133" => {
            let mark = parts.get(1)?;
            let mark_str = std::str::from_utf8(mark).ok()?;
            Some(OscEvent::PromptMark(mark_str.chars().next()?))
        }
        // OSC 7777 — EmberTerm agent protocol
        "7777" => {
            let verb_bytes = parts.get(1)?;
            let payload_bytes = parts.get(2)?;
            let verb = std::str::from_utf8(verb_bytes).ok()?.to_string();
            let payload = std::str::from_utf8(payload_bytes).ok()?.to_string();
            Some(OscEvent::Osc7777 { verb, payload })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // OSC 0 sets icon name AND window title → SetIconNameAndWindowTitle
        let seq = osc_bel("0", &["My Shell"]);
        let events = extract_osc_events(&seq);
        assert!(
            events.iter().any(|e| matches!(e, OscEvent::Title(_))),
            "expected Title event from OSC 0, got: {events:?}"
        );
    }

    #[test]
    fn test_osc2_title() {
        // OSC 2 sets window title only → SetWindowTitle
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
        // OSC 133;A → FreshLineAndStartPrompt → mark 'A'
        let seq = osc_bel("133", &["A"]);
        let events = extract_osc_events(&seq);
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert!(matches!(events[0], OscEvent::PromptMark('A')));
    }

    #[test]
    fn test_osc_133_input_start() {
        // OSC 133;B → MarkEndOfPromptAndStartOfInput → mark 'B'
        let seq = osc_bel("133", &["B"]);
        let events = extract_osc_events(&seq);
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert!(matches!(events[0], OscEvent::PromptMark('B')));
    }

    #[test]
    fn test_osc_133_output_start() {
        // OSC 133;C → MarkEndOfInputAndStartOfOutput → mark 'C'
        let seq = osc_bel("133", &["C"]);
        let events = extract_osc_events(&seq);
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert!(matches!(events[0], OscEvent::PromptMark('C')));
    }

    #[test]
    fn test_osc_133_command_end() {
        // OSC 133;D → CommandStatus → mark 'D'
        let seq = osc_bel("133", &["D"]);
        let events = extract_osc_events(&seq);
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert!(matches!(events[0], OscEvent::PromptMark('D')));
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
}
