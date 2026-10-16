use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::state::lock_or_recover;

/// Each channel send is a `webview.eval` (plus a fetch round trip above 1 KB), so a flood
/// sent chunk by chunk is bound by IPC. Holding output this long caps a flood at ~125 sends
/// a second; xterm paints once per 16 ms frame anyway, so the delay is not visible.
pub(super) const FLUSH_AFTER: Duration = Duration::from_millis(8);
/// Send early at this size: large enough that the per-send cost vanishes, small enough that
/// one `term.write` stays a short parse slice and does not stall the webview.
pub(super) const FLUSH_AT: usize = 64 * 1024;
/// The reader blocks above this, which stops it draining ConPTY and in turn stalls the
/// shell, so a flood the webview cannot keep up with does not grow memory here.
pub(super) const MAX_PENDING: usize = 1024 * 1024;

/// Output waiting to be sent. Plain data so the batching rules can be tested with fake clocks.
#[derive(Default)]
pub(super) struct Batch {
    pending: Vec<u8>,
    first_at: Option<Instant>,
    last_sent: Option<Instant>,
    in_sync: bool,
    sync_buf: Vec<u8>,
}

impl Batch {
    /// Queues a PTY chunk. A DEC 2026 synchronized frame is held until it ends, then queued
    /// whole; `take` always drains everything, so a frame never straddles two sends.
    pub(super) fn push(&mut self, chunk: &[u8], now: Instant) {
        if detect_sync(chunk, &mut self.in_sync, &mut self.sync_buf) {
            let frame = std::mem::take(&mut self.sync_buf);
            self.queue(&frame, now);
        } else if !self.in_sync {
            self.queue(chunk, now);
        }
    }

    fn queue(&mut self, bytes: &[u8], now: Instant) {
        if self.pending.is_empty() {
            self.first_at = Some(now);
        }
        self.pending.extend_from_slice(bytes);
    }

    pub(super) fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// When the pending bytes should go out, `None` when there are none. Output after a quiet
    /// spell (typing echo) goes at once; only bursts wait out `FLUSH_AFTER`.
    pub(super) fn due_at(&self) -> Option<Instant> {
        let first = self.first_at?;
        let quiet = self
            .last_sent
            .is_none_or(|sent| first.duration_since(sent) >= FLUSH_AFTER);
        if quiet || self.pending.len() >= FLUSH_AT {
            Some(first)
        } else {
            Some(first + FLUSH_AFTER)
        }
    }

    pub(super) fn take(&mut self, now: Instant) -> Vec<u8> {
        self.first_at = None;
        self.last_sent = Some(now);
        std::mem::take(&mut self.pending)
    }
}

/// Hands output from the PTY reader thread to the sender thread.
#[derive(Default)]
pub(super) struct OutputPipe {
    state: Mutex<PipeState>,
    sender_wake: Condvar,
    reader_wake: Condvar,
}

#[derive(Default)]
struct PipeState {
    batch: Batch,
    closed: bool,
}

impl OutputPipe {
    /// Reader side. Blocks while `MAX_PENDING` bytes wait; `false` once the pipe is closed.
    pub(super) fn push(&self, chunk: &[u8]) -> bool {
        let mut s = lock_or_recover(&self.state);
        while s.batch.pending_len() >= MAX_PENDING && !s.closed {
            s = self.reader_wake.wait(s).unwrap_or_else(|e| e.into_inner());
        }
        if s.closed {
            return false;
        }
        s.batch.push(chunk, Instant::now());
        self.sender_wake.notify_one();
        true
    }

    /// Sender side. Blocks until a batch is due; after `close` it drains what is left, then
    /// returns `None`.
    pub(super) fn next_batch(&self) -> Option<Vec<u8>> {
        let mut s = lock_or_recover(&self.state);
        loop {
            let now = Instant::now();
            match s.batch.due_at() {
                None if s.closed => return None,
                None => s = self.sender_wake.wait(s).unwrap_or_else(|e| e.into_inner()),
                Some(due) if due <= now || s.closed => {
                    let bytes = s.batch.take(now);
                    self.reader_wake.notify_one();
                    return Some(bytes);
                }
                Some(due) => {
                    s = self
                        .sender_wake
                        .wait_timeout(s, due - now)
                        .unwrap_or_else(|e| e.into_inner())
                        .0;
                }
            }
        }
    }

    /// Called by whichever side stops first, so the other one does not wait forever.
    pub(super) fn close(&self) {
        lock_or_recover(&self.state).closed = true;
        self.sender_wake.notify_all();
        self.reader_wake.notify_all();
    }
}

/// DEC 2026 synchronized output — update sync state based on `chunk`.
///
/// - Detects `ESC [ ? 2026 h` (begin sync) and `ESC [ ? 2026 l` (end sync).
/// - While in sync mode, appends incoming bytes to `sync_buf` instead of emitting.
/// - Returns `true` when an end-sync marker is found; at that point `sync_buf`
///   holds the complete coalesced frame and the caller should emit it.
pub(super) fn detect_sync(chunk: &[u8], in_sync: &mut bool, sync_buf: &mut Vec<u8>) -> bool {
    const BEGIN: &[u8] = b"\x1b[?2026h";
    const END: &[u8] = b"\x1b[?2026l";

    let has_begin = chunk.windows(BEGIN.len()).any(|w| w == BEGIN);
    let has_end = chunk.windows(END.len()).any(|w| w == END);

    if has_begin && !*in_sync {
        *in_sync = true;
    }

    if *in_sync {
        sync_buf.extend_from_slice(chunk);
    }

    if has_end && *in_sync {
        *in_sync = false;
        return true; // caller should emit sync_buf
    }

    false
}

#[cfg(test)]
#[path = "output_test.rs"]
mod tests;
