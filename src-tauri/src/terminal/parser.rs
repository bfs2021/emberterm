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

            // Both B and C marks may carry an `aid=<command>` parameter.
            // B is emitted by some shells when the user hits Enter (command known at that point).
            // C's aid= takes precedence if both are present.
            let command = if mark == 'B' || mark == 'C' {
                // ESC]133;C;aid=ls -la\a → parts[2] = "aid=ls -la"
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
#[path = "parser_test.rs"]
mod tests;
