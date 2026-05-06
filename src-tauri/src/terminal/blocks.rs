use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub type BlockId = String;

// ── BlockStore ────────────────────────────────────────────────────────────────

/// Ordered block store with O(1) lookup by `BlockId`.
///
/// Internally keeps both a `Vec<Block>` (for insertion order, needed by
/// `block_get` IPC) and a `HashMap<BlockId, usize>` index so individual
/// blocks can be mutated in O(1) time (used by the D mark and `block.annotate`).
/// Blocks are never removed during a session, so position indices stay valid.
#[derive(Debug, Default)]
pub struct BlockStore {
    blocks: Vec<Block>,
    index: HashMap<BlockId, usize>,
}

impl BlockStore {
    /// Append a new block; panics if the ID is already present (a bug, not a runtime error).
    pub fn push(&mut self, block: Block) {
        let pos = self.blocks.len();
        self.index.insert(block.id.clone(), pos);
        self.blocks.push(block);
    }

    /// O(1) mutable access by block ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Block> {
        let pos = *self.index.get(id)?;
        self.blocks.get_mut(pos)
    }

    /// Returns all blocks in insertion order (cheaply cloneable for IPC).
    pub fn all(&self) -> &[Block] {
        &self.blocks
    }
}

/// Persistent block record stored on the session for `block_get` IPC queries.
#[derive(Debug, Clone, serde::Serialize, Default)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "../../frontend/bindings/Block.ts"))]
pub struct Block {
    pub id: BlockId,
    pub command: String,
    pub started_at_ms: u64,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    /// Set by `block.annotate` OSC 7777 verb.
    pub agent_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub fold: Option<String>,
}

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
        /// Command text captured from the B mark (if any), used as fallback in `on_output_start`.
        pending_command: Option<String>,
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
            pending_command: None,
        };
    }

    /// OSC 133 B — end of prompt, start of user input.
    /// Some shells embed the typed command here via `aid=`; store it as a fallback
    /// for `on_output_start` in case C arrives without its own `aid=` parameter.
    pub fn on_command_start(&mut self, command: Option<String>) {
        if let State::Prompting { ref mut pending_command, .. } = self.state {
            *pending_command = command;
        }
    }

    /// OSC 133 C — end of user input, start of command output.
    /// `command_aid` is the optional `aid=...` value from the C mark (takes precedence
    /// over any command captured at the B mark).
    /// Returns `BlockStartData` to emit as `block:start`.
    pub fn on_output_start(&mut self, command_aid: Option<String>) -> Option<BlockStartData> {
        let prev = std::mem::replace(&mut self.state, State::Idle);
        match prev {
            State::Prompting { id, started_at, started_at_ms, pending_command } => {
                // C's aid= takes precedence; B's pending_command is the fallback.
                let command = command_aid.or(pending_command).unwrap_or_default();
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

    /// Returns the ID of the block currently executing, if any.
    /// Used by the OSC 7777 `block.annotate` handler to target the right block.
    pub fn current_block_id(&self) -> Option<&BlockId> {
        match &self.state {
            State::Executing { id, .. } => Some(id),
            _ => None,
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

#[cfg(test)]
#[path = "blocks_test.rs"]
mod tests;
