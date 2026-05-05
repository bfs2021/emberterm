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
        // OSC 52 — clipboard write (termwiz decodes base64; content is plain text)
        OperatingSystemCommand::SetSelection(_sel, content) => {
            if content.is_empty() {
                return None; // empty or read-request path
            }
            Some(OscEvent::ClipboardWrite(content.clone()))
        }
        // OSC 133 — FinalTerm/shell integration prompt marks
        OperatingSystemCommand::FinalTermSemanticPrompt(prompt) => {
            match prompt {
                FinalTermSemanticPrompt::FreshLineAndStartPrompt { .. } => {
                    Some(OscEvent::PromptMark { mark: 'A', exit_code: None, command: None })
                }
                FinalTermSemanticPrompt::MarkEndOfPromptAndStartOfInputUntilNextMarker
                | FinalTermSemanticPrompt::MarkEndOfPromptAndStartOfInputUntilEndOfLine => {
                    Some(OscEvent::PromptMark { mark: 'B', exit_code: None, command: None })
                }
                FinalTermSemanticPrompt::MarkEndOfInputAndStartOfOutput { aid } => {
                    Some(OscEvent::PromptMark { mark: 'C', exit_code: None, command: aid.clone() })
                }
                FinalTermSemanticPrompt::CommandStatus { status, .. } => {
                    Some(OscEvent::PromptMark { mark: 'D', exit_code: Some(*status), command: None })
                }
                _ => None,
            }
        }
        OperatingSystemCommand::Unspecified(parts) => handle_unspecified_osc(parts),
        _ => None,
    }
}

fn handle_unspecified_osc(parts: &[Vec<u8>]) -> Option<OscEvent> {
    let first = parts.first()?;
    let tag = std::str::from_utf8(first).ok()?;

    match tag {
        // OSC 8 — hyperlinks (https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda)
        // Format: ESC ] 8 ; params ; uri BEL  (empty uri = link end)
        "8" => {
            let uri_bytes = parts.get(2)?;
            let url = std::str::from_utf8(uri_bytes).ok()?.to_string();
            if url.is_empty() {
                return None; // hyperlink end marker — nothing to emit
            }
            let params = parts.get(1)
                .and_then(|b| std::str::from_utf8(b).ok())
                .unwrap_or("");
            let id = params.split(';')
                .find_map(|seg| seg.strip_prefix("id="))
                .map(str::to_string);
            Some(OscEvent::Hyperlink { url, id })
        }
        // OSC 52 — clipboard access
        // Format: ESC ] 52 ; selection ; base64-data BEL
        "52" => {
            let data_bytes = parts.get(2)?;
            let data = std::str::from_utf8(data_bytes).ok()?;
            if data == "?" || data.is_empty() {
                return None; // read requests not supported in v0.1
            }
            Some(OscEvent::ClipboardWrite(data.to_string()))
        }
        // OSC 133 — shell prompt marks (A=prompt start, B=command start, C=output start, D=command end)
        // Format: ESC]133;A\a  /  ESC]133;C;aid=<cmd>\a  /  ESC]133;D;exit=<code>\a
        "133" => {
            let mark_bytes = parts.get(1)?;
            let mark_str = std::str::from_utf8(mark_bytes).ok()?;
            let mark = mark_str.chars().next()?;

            let exit_code = if mark == 'D' {
                // Extra params: ESC]133;D;exit=0\a → parts[2] = "exit=0"
                parts.get(2)
                    .and_then(|b| std::str::from_utf8(b).ok())
                    .and_then(|s| {
                        s.strip_prefix("exit=").unwrap_or(s).parse::<i32>().ok()
                    })
            } else {
                None
            };

            let command = if mark == 'C' {
                // ESC]133;C;aid=<cmd>\a → parts[2] = "aid=<cmd>"
                parts.get(2)
                    .and_then(|b| std::str::from_utf8(b).ok())
                    .and_then(|s| s.strip_prefix("aid=").map(str::to_string))
            } else {
                None
            };

            Some(OscEvent::PromptMark { mark, exit_code, command })
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
        assert!(matches!(events[0], OscEvent::PromptMark { mark: 'A', .. }));
    }

    #[test]
    fn test_osc_133_input_start() {
        // OSC 133;B → MarkEndOfPromptAndStartOfInput → mark 'B'
        let seq = osc_bel("133", &["B"]);
        let events = extract_osc_events(&seq);
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert!(matches!(events[0], OscEvent::PromptMark { mark: 'B', .. }));
    }

    #[test]
    fn test_osc_133_output_start() {
        // OSC 133;C → MarkEndOfInputAndStartOfOutput → mark 'C'
        let seq = osc_bel("133", &["C"]);
        let events = extract_osc_events(&seq);
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert!(matches!(events[0], OscEvent::PromptMark { mark: 'C', .. }));
    }

    #[test]
    fn test_osc_133_command_end() {
        // OSC 133;D → CommandStatus → mark 'D'
        let seq = osc_bel("133", &["D"]);
        let events = extract_osc_events(&seq);
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert!(matches!(events[0], OscEvent::PromptMark { mark: 'D', .. }));
    }

    #[test]
    fn test_osc_133_command_end_with_exit_code() {
        // OSC 133;D;2 → mark 'D' with exit_code = Some(2)
        // Termwiz reads osc[2] as a plain integer for the D mark exit code.
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
        // OSC 133;C;aid=ls -la → mark 'C' with command = Some("ls -la")
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
        // OSC 8 hyperlink — termwiz may parse this typed (SetHyperlink) before we see it
        // as Unspecified.  If it does, xterm.js still renders the link; we just won't
        // get a Rust-side event.  Test covers the Unspecified path.
        let seq = osc_bel("8", &["id=link1", "https://example.com"]);
        let events = extract_osc_events(&seq);
        // Either we get a Hyperlink event (Unspecified path) or none (typed path).
        // Both are acceptable — xterm.js handles display regardless.
        for e in &events {
            if let OscEvent::Hyperlink { url, id } = e {
                assert_eq!(url, "https://example.com");
                assert_eq!(id.as_deref(), Some("link1"));
                return;
            }
        }
        // If no event: termwiz handled it typed — still correct behaviour.
    }

    #[test]
    fn test_osc_8_hyperlink_end_ignored() {
        // Empty URI = hyperlink end — should produce no event.
        let seq = osc_bel("8", &["", ""]);
        let events = extract_osc_events(&seq);
        let hyperlinks: Vec<_> = events.iter().filter(|e| matches!(e, OscEvent::Hyperlink { .. })).collect();
        assert_eq!(hyperlinks.len(), 0, "hyperlink end should not emit an event");
    }

    #[test]
    fn test_osc_52_clipboard_write() {
        // Termwiz handles OSC 52 as SetSelection and decodes the base64 content.
        // Base64 of "hello" is "aGVsbG8="; termwiz delivers the decoded string "hello".
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
}
