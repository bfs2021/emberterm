use super::*;

// ── Mock PTY ─────────────────────────────────────────────────────────────────

struct MockMasterPty;

impl portable_pty::MasterPty for MockMasterPty {
    fn resize(&self, _size: portable_pty::PtySize) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_size(&self) -> anyhow::Result<portable_pty::PtySize> {
        Ok(portable_pty::PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
    }

    fn try_clone_reader(&self) -> anyhow::Result<Box<dyn std::io::Read + Send>> {
        Ok(Box::new(std::io::empty()))
    }

    fn take_writer(&self) -> anyhow::Result<Box<dyn std::io::Write + Send>> {
        Ok(Box::new(std::io::sink()))
    }

    // Unix-only trait items; a mock has no process group, fd or tty.
    #[cfg(unix)]
    fn process_group_leader(&self) -> Option<i32> {
        None
    }

    #[cfg(unix)]
    fn as_raw_fd(&self) -> Option<i32> {
        None
    }

    #[cfg(unix)]
    fn tty_name(&self) -> Option<std::path::PathBuf> {
        None
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_session() -> Session {
    Session {
        master: Box::new(MockMasterPty),
        writer: Arc::new(Mutex::new(Box::new(std::io::sink()))),
        pid: None,
    }
}

// ── Session struct lifecycle ──────────────────────────────────────────────────

#[test]
fn mock_pty_resize_succeeds() {
    let s = make_session();
    let result = s.master.resize(portable_pty::PtySize {
        rows: 40,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    });
    assert!(result.is_ok());
}

#[test]
fn mock_pty_get_size_returns_default() {
    let s = make_session();
    let size = s.master.get_size().unwrap();
    assert_eq!(size.rows, 24);
    assert_eq!(size.cols, 80);
}

#[test]
fn writer_accepts_bytes() {
    let s = make_session();
    let mut w = s.writer.lock().unwrap();
    use std::io::Write;
    let result = w.write_all(b"hello");
    assert!(result.is_ok());
}
