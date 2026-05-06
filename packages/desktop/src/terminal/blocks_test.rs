use super::*;
use crate::agent::protocol::OscEvent;
use crate::terminal::parser::extract_osc_events;

// ── Unit tests: BlockManager state machine ────────────────────────────────────

fn full_cycle(mgr: &mut BlockManager, command_b: Option<&str>, command_c: Option<&str>) -> String {
    mgr.on_prompt_start();
    mgr.on_command_start(command_b.map(str::to_string));
    let start = mgr.on_output_start(command_c.map(str::to_string)).unwrap();
    start.command
}

#[test]
fn b_mark_command_used_as_fallback() {
    let mut mgr = BlockManager::default();
    let cmd = full_cycle(&mut mgr, Some("ls -la"), None);
    assert_eq!(cmd, "ls -la");
}

#[test]
fn c_mark_command_takes_precedence_over_b() {
    let mut mgr = BlockManager::default();
    let cmd = full_cycle(&mut mgr, Some("from_b"), Some("from_c"));
    assert_eq!(cmd, "from_c");
}

#[test]
fn b_mark_without_command_falls_back_to_empty() {
    let mut mgr = BlockManager::default();
    let cmd = full_cycle(&mut mgr, None, None);
    assert_eq!(cmd, "");
}

#[test]
fn current_block_id_returns_executing_id() {
    let mut mgr = BlockManager::default();
    assert!(mgr.current_block_id().is_none());
    mgr.on_prompt_start();
    assert!(mgr.current_block_id().is_none());
    mgr.on_command_start(None);
    let data = mgr.on_output_start(None).unwrap();
    assert_eq!(mgr.current_block_id(), Some(&data.id));
    mgr.on_command_end(0);
    assert!(mgr.current_block_id().is_none());
}

// ── Pipeline integration tests: extract_osc_events + BlockManager ─────────────
//
// These tests simulate what the PTY reader thread does: feeding raw OSC bytes
// from a real shell through the parser, then dispatching events to BlockManager.

/// Build a BEL-terminated OSC sequence: ESC ] <tag> [; <param>...] BEL
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

/// Feed OSC bytes through the parser and drive BlockManager; returns the block
/// events produced by a complete A → B → C → D sequence.
fn run_pipeline(
    mgr: &mut BlockManager,
    sequences: &[Vec<u8>],
) -> (Option<BlockStartData>, Option<BlockEndData>) {
    let mut start_data: Option<BlockStartData> = None;
    let mut end_data: Option<BlockEndData> = None;

    for bytes in sequences {
        for event in extract_osc_events(bytes) {
            match event {
                OscEvent::PromptMark { mark: 'A', .. } => {
                    mgr.on_prompt_start();
                }
                OscEvent::PromptMark { mark: 'B', command, .. } => {
                    mgr.on_command_start(command);
                }
                OscEvent::PromptMark { mark: 'C', command, .. } => {
                    start_data = mgr.on_output_start(command);
                }
                OscEvent::PromptMark { mark: 'D', exit_code, .. } => {
                    end_data = mgr.on_command_end(exit_code.unwrap_or(-1));
                }
                _ => {}
            }
        }
    }

    (start_data, end_data)
}

#[test]
fn pipeline_a_b_c_d_full_cycle() {
    let mut mgr = BlockManager::default();
    let seqs = vec![
        osc_bel("133", &["A"]),
        osc_bel("133", &["B", "aid=ls -la"]),
        osc_bel("133", &["C"]),
        osc_bel("133", &["D", "0"]),
    ];
    let (start, end) = run_pipeline(&mut mgr, &seqs);

    let start = start.expect("expected BlockStartData from C mark");
    assert_eq!(start.command, "ls -la", "command from B mark should carry over");
    assert!(!start.id.is_empty());

    let end = end.expect("expected BlockEndData from D mark");
    assert_eq!(end.id, start.id, "block IDs must match");
    assert_eq!(end.exit_code, 0);
}

#[test]
fn pipeline_c_mark_aid_overrides_b_mark() {
    let mut mgr = BlockManager::default();
    let seqs = vec![
        osc_bel("133", &["A"]),
        osc_bel("133", &["B", "aid=from_b"]),
        osc_bel("133", &["C", "aid=from_c"]),
        osc_bel("133", &["D", "0"]),
    ];
    let (start, _) = run_pipeline(&mut mgr, &seqs);
    assert_eq!(start.unwrap().command, "from_c");
}

#[test]
fn pipeline_exit_code_non_zero() {
    let mut mgr = BlockManager::default();
    let seqs = vec![
        osc_bel("133", &["A"]),
        osc_bel("133", &["C"]),
        osc_bel("133", &["D", "1"]),
    ];
    let (_, end) = run_pipeline(&mut mgr, &seqs);
    assert_eq!(end.unwrap().exit_code, 1);
}

#[test]
fn pipeline_multiple_sequential_blocks() {
    let mut mgr = BlockManager::default();

    // First block
    let seqs1 = vec![
        osc_bel("133", &["A"]),
        osc_bel("133", &["B", "aid=cmd1"]),
        osc_bel("133", &["C"]),
        osc_bel("133", &["D", "0"]),
    ];
    let (s1, e1) = run_pipeline(&mut mgr, &seqs1);
    let s1 = s1.unwrap();
    assert_eq!(s1.command, "cmd1");
    assert_eq!(e1.unwrap().id, s1.id);

    // Second block — new A resets state
    let seqs2 = vec![
        osc_bel("133", &["A"]),
        osc_bel("133", &["B", "aid=cmd2"]),
        osc_bel("133", &["C"]),
        osc_bel("133", &["D", "0"]),
    ];
    let (s2, e2) = run_pipeline(&mut mgr, &seqs2);
    let s2 = s2.unwrap();
    assert_eq!(s2.command, "cmd2");
    assert_ne!(s2.id, s1.id, "each block should have a unique ID");
    assert_eq!(e2.unwrap().id, s2.id);
}

#[test]
fn pipeline_d_without_a_b_c_is_noop() {
    let mut mgr = BlockManager::default();
    let seqs = vec![osc_bel("133", &["D", "0"])];
    let (start, end) = run_pipeline(&mut mgr, &seqs);
    assert!(start.is_none());
    assert!(end.is_none());
}

#[test]
fn pipeline_interleaved_osc_other_events_ignored() {
    let mut mgr = BlockManager::default();
    // Mix OSC 133 marks with unrelated OSC sequences
    let seqs = vec![
        osc_bel("0", &["My Shell"]),         // title — ignored
        osc_bel("133", &["A"]),
        osc_bel("7", &["file:///home/user"]), // cwd — ignored
        osc_bel("133", &["C", "aid=echo hello"]),
        osc_bel("133", &["D", "0"]),
    ];
    let (start, end) = run_pipeline(&mut mgr, &seqs);
    let start = start.expect("expected block start");
    assert_eq!(start.command, "echo hello");
    assert_eq!(end.unwrap().exit_code, 0);
}

// ── Unit tests: BlockStore ────────────────────────────────────────────────────

#[test]
fn block_store_push_and_all_preserve_order() {
    let mut store = BlockStore::default();
    for i in 0..3u32 {
        store.push(Block { id: format!("blk-{i}"), command: format!("cmd{i}"), ..Default::default() });
    }
    let ids: Vec<&str> = store.all().iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, ["blk-0", "blk-1", "blk-2"]);
}

#[test]
fn block_store_get_mut_o1_update() {
    let mut store = BlockStore::default();
    store.push(Block { id: "blk-1".into(), command: "ls".into(), ..Default::default() });
    store.push(Block { id: "blk-2".into(), command: "cat".into(), ..Default::default() });

    // Update blk-1 without touching blk-2.
    if let Some(b) = store.get_mut("blk-1") {
        b.exit_code = Some(0);
        b.duration_ms = Some(42);
    }
    assert_eq!(store.all()[0].exit_code, Some(0));
    assert_eq!(store.all()[1].exit_code, None, "blk-2 should be untouched");
}

#[test]
fn block_store_get_mut_missing_returns_none() {
    let mut store = BlockStore::default();
    assert!(store.get_mut("nonexistent").is_none());
}
