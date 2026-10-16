use std::sync::{Condvar, Mutex};

use crate::state::lock_or_recover;

/// The sender stops while this many sent bytes are not yet acked. `Channel::send` returns
/// before the webview has the data, so without acks a webview that falls behind queues
/// output without limit. One more batch can go out at the limit, so the webview holds at
/// most this plus `output::MAX_PENDING`.
pub(super) const HIGH_WATERMARK: usize = 512 * 1024;

/// Bytes sent to the webview that xterm has not yet parsed, as reported by `session_ack`.
#[derive(Default)]
pub(crate) struct FlowControl {
    state: Mutex<FlowState>,
    wake: Condvar,
}

#[derive(Default)]
struct FlowState {
    unacked: usize,
    closed: bool,
}

impl FlowControl {
    /// Sender side. Blocks while `HIGH_WATERMARK` bytes are unacked; `false` once closed.
    pub(super) fn wait_for_room(&self) -> bool {
        let mut s = lock_or_recover(&self.state);
        while s.unacked >= HIGH_WATERMARK && !s.closed {
            s = self.wake.wait(s).unwrap_or_else(|e| e.into_inner());
        }
        !s.closed
    }

    /// Records bytes about to be sent. Called before the send, so an ack can never arrive
    /// for bytes not yet counted.
    pub(super) fn sent(&self, bytes: usize) {
        lock_or_recover(&self.state).unacked += bytes;
    }

    /// Bytes xterm has parsed. Saturates: the webview acks bytes it dropped (a Grid torn
    /// down mid-write) and a stale ack must not wrap the count.
    pub(crate) fn ack(&self, bytes: usize) {
        let mut s = lock_or_recover(&self.state);
        s.unacked = s.unacked.saturating_sub(bytes);
        self.wake.notify_all();
    }

    /// The session is gone, so no ack will come; release the sender.
    pub(crate) fn close(&self) {
        lock_or_recover(&self.state).closed = true;
        self.wake.notify_all();
    }
}

#[cfg(test)]
#[path = "flow_test.rs"]
mod tests;
