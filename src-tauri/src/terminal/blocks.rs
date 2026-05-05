use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub type BlockId = String;

/// Data returned by `on_output_start` — used by the caller to emit `block:start`.
pub struct BlockStartData {
    pub id: BlockId,
    pub command: String,
    /// Unix timestamp in milliseconds (when OSC 133 A was seen).
    pub started_at_ms: u64,
}

/// Data returned by `on_command_end` — used by the caller to emit `block:end`.
pub struct BlockEndData {
    pub id: BlockId,
    pub exit_code: i32,
    pub duration_ms: u64,
}

enum State {
    Idle,
    /// OSC 133 A received — prompt is being displayed.
    Prompting {
        id: BlockId,
        started_at: Instant,
        started_at_ms: u64,
    },
    /// OSC 133 C received — command is executing, waiting for D.
    Executing {
        id: BlockId,
        started_at: Instant,
        command: String,
    },
}

/// Tracks the OSC 133 prompt-mark state machine for one terminal session.
/// Methods return data payloads; the caller is responsible for emitting Tauri events.
pub struct BlockManager {
    state: State,
    counter: u64,
}

impl Default for BlockManager {
    fn default() -> Self {
        Self { state: State::Idle, counter: 0 }
    }
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

impl BlockManager {
    fn next_id(&mut self) -> BlockId {
        self.counter += 1;
        format!("blk-{}", self.counter)
    }

    /// OSC 133 A — fresh line + start of prompt. Prepare state for a new block.
    pub fn on_prompt_start(&mut self) {
        let id = self.next_id();
        self.state = State::Prompting {
            id,
            started_at: Instant::now(),
            started_at_ms: unix_now_ms(),
        };
    }

    /// OSC 133 C — end of user input, start of command output.
    /// `command_aid` is the optional `aid=...` value the shell may embed.
    /// Returns `BlockStartData` to emit as `block:start`.
    pub fn on_output_start(&mut self, command_aid: Option<String>) -> Option<BlockStartData> {
        let prev = std::mem::replace(&mut self.state, State::Idle);
        match prev {
            State::Prompting { id, started_at, started_at_ms } => {
                let command = command_aid.unwrap_or_default();
                self.state = State::Executing {
                    id: id.clone(),
                    started_at,
                    command: command.clone(),
                };
                Some(BlockStartData { id, command, started_at_ms })
            }
            other => {
                self.state = other;
                None
            }
        }
    }

    /// OSC 133 D — command ended with `exit_code`.
    /// Returns `BlockEndData` to emit as `block:end`.
    pub fn on_command_end(&mut self, exit_code: i32) -> Option<BlockEndData> {
        let prev = std::mem::replace(&mut self.state, State::Idle);
        match prev {
            State::Executing { id, started_at, .. } => {
                let duration_ms = started_at.elapsed().as_millis() as u64;
                Some(BlockEndData { id, exit_code, duration_ms })
            }
            other => {
                self.state = other;
                None
            }
        }
    }
}
