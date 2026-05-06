use super::*;
use crate::agent::tools::ToolStatus;
use crate::terminal::blocks::Block;

// ── Mock PTY ─────────────────────────────────────────────────────────────────

struct MockMasterPty;

impl portable_pty::MasterPty for MockMasterPty {
    fn resize(&self, _size: portable_pty::PtySize) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_size(&self) -> anyhow::Result<portable_pty::PtySize> {
        Ok(portable_pty::PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
    }

    fn try_clone_reader(&self) -> anyhow::Result<Box<dyn std::io::Read + Send>> {
        Ok(Box::new(std::io::empty()))
    }

    fn take_writer(&self) -> anyhow::Result<Box<dyn std::io::Write + Send>> {
        Ok(Box::new(std::io::sink()))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_session(id: &str) -> Session {
    Session {
        id: id.to_string(),
        master: Box::new(MockMasterPty),
        writer: Arc::new(Mutex::new(Box::new(std::io::sink()))),
        title: String::new(),
        cwd: None,
        active_tools: Mutex::new(ActiveTools::default()),
        agent_name: Mutex::new(None),
        blocks: Arc::new(Mutex::new(BlockStore::default())),
    }
}

// ── Session struct lifecycle ──────────────────────────────────────────────────

#[test]
fn session_id_is_stored_correctly() {
    let s = make_session("ses-42");
    assert_eq!(s.id, "ses-42");
}

#[test]
fn session_starts_with_no_agent() {
    let s = make_session("s1");
    assert!(s.agent_name.lock().unwrap().is_none());
}

#[test]
fn session_starts_with_empty_tools() {
    let s = make_session("s1");
    assert!(s.active_tools.lock().unwrap().calls.is_empty());
}

#[test]
fn session_starts_with_empty_blocks() {
    let s = make_session("s1");
    assert!(s.blocks.lock().unwrap().all().is_empty());
}

#[test]
fn mock_pty_resize_succeeds() {
    let s = make_session("s1");
    let result = s.master.resize(portable_pty::PtySize { rows: 40, cols: 120, pixel_width: 0, pixel_height: 0 });
    assert!(result.is_ok());
}

#[test]
fn mock_pty_get_size_returns_default() {
    let s = make_session("s1");
    let size = s.master.get_size().unwrap();
    assert_eq!(size.rows, 24);
    assert_eq!(size.cols, 80);
}

#[test]
fn writer_accepts_bytes() {
    let s = make_session("s1");
    let mut w = s.writer.lock().unwrap();
    use std::io::Write;
    let result = w.write_all(b"hello");
    assert!(result.is_ok());
}

// ── Tool call lifecycle ───────────────────────────────────────────────────────

#[test]
fn tool_call_start_adds_entry() {
    let s = make_session("s1");
    s.active_tools.lock().unwrap().start("t1".into(), "read_file".into(), "Reading foo.rs".into(), None);
    assert_eq!(s.active_tools.lock().unwrap().calls.len(), 1);
    assert_eq!(s.active_tools.lock().unwrap().calls["t1"].name, "read_file");
}

#[test]
fn tool_call_progress_updates_fields() {
    let s = make_session("s1");
    s.active_tools.lock().unwrap().start("t1".into(), "build".into(), "Building…".into(), None);
    s.active_tools.lock().unwrap().progress("t1", Some(50), Some("Compiling…".into()));
    let tools = s.active_tools.lock().unwrap();
    let call = &tools.calls["t1"];
    assert_eq!(call.progress_pct, Some(50));
    assert_eq!(call.progress_message.as_deref(), Some("Compiling…"));
}

#[test]
fn tool_call_end_sets_status_and_duration() {
    let s = make_session("s1");
    s.active_tools.lock().unwrap().start("t1".into(), "edit".into(), "Editing…".into(), None);
    s.active_tools.lock().unwrap().end("t1", ToolStatus::Ok, Some("Edited 5 lines".into()), Some(120));
    let tools = s.active_tools.lock().unwrap();
    let call = &tools.calls["t1"];
    assert_eq!(call.status, ToolStatus::Ok);
    assert_eq!(call.duration_ms, Some(120));
    assert_eq!(call.summary, "Edited 5 lines");
}

#[test]
fn multiple_tool_calls_are_independent() {
    let s = make_session("s1");
    {
        let mut t = s.active_tools.lock().unwrap();
        t.start("t1".into(), "read".into(), "Reading".into(), None);
        t.start("t2".into(), "write".into(), "Writing".into(), None);
        t.end("t1", ToolStatus::Ok, None, None);
    }
    let tools = s.active_tools.lock().unwrap();
    assert_eq!(tools.calls["t1"].status, ToolStatus::Ok);
    assert_eq!(tools.calls["t2"].status, ToolStatus::Running);
}

// ── Block lifecycle ───────────────────────────────────────────────────────────

#[test]
fn block_push_inserts_in_order() {
    let s = make_session("s1");
    {
        let mut bs = s.blocks.lock().unwrap();
        bs.push(Block { id: "blk-1".into(), command: "ls".into(), started_at_ms: 100, ..Default::default() });
        bs.push(Block { id: "blk-2".into(), command: "pwd".into(), started_at_ms: 200, ..Default::default() });
    }
    let bs = s.blocks.lock().unwrap();
    let all = bs.all();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].id, "blk-1");
    assert_eq!(all[1].id, "blk-2");
}

#[test]
fn block_get_mut_allows_exit_code_update() {
    let s = make_session("s1");
    s.blocks.lock().unwrap().push(Block { id: "blk-1".into(), command: "make".into(), started_at_ms: 0, ..Default::default() });
    if let Some(b) = s.blocks.lock().unwrap().get_mut("blk-1") {
        b.exit_code = Some(0);
        b.duration_ms = Some(2500);
    }
    let bs = s.blocks.lock().unwrap();
    let b = &bs.all()[0];
    assert_eq!(b.exit_code, Some(0));
    assert_eq!(b.duration_ms, Some(2500));
}

#[test]
fn blocks_arc_is_shared() {
    let s = make_session("s1");
    let blocks_clone = Arc::clone(&s.blocks);
    s.blocks.lock().unwrap().push(Block { id: "blk-1".into(), command: "echo".into(), started_at_ms: 0, ..Default::default() });
    // Both the session field and the clone see the same data.
    assert_eq!(blocks_clone.lock().unwrap().all().len(), 1);
}
