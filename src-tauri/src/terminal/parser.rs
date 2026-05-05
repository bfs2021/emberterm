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
        OperatingSystemCommand::SetWindowTitle(title) => {
            Some(OscEvent::Title(title.clone()))
        }
        OperatingSystemCommand::CurrentWorkingDirectory(cwd) => {
            Some(OscEvent::Cwd(cwd.clone()))
        }
        OperatingSystemCommand::Unspecified(parts) => {
            handle_unspecified_osc(parts)
        }
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
