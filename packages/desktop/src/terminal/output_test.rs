use super::*;

use std::sync::Arc;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

// ── Batch: when output is due ───────────────────────────────────────────────

#[test]
fn nothing_pending_is_never_due() {
    assert_eq!(Batch::default().due_at(), None);
}

#[test]
fn first_output_after_a_quiet_spell_goes_at_once() {
    let t0 = Instant::now();
    let mut b = Batch::default();
    b.push(b"a", t0);
    assert_eq!(b.due_at(), Some(t0));
    b.take(t0);
    b.push(b"b", t0 + FLUSH_AFTER);
    assert_eq!(b.due_at(), Some(t0 + FLUSH_AFTER));
}

#[test]
fn a_burst_waits_from_its_first_unsent_byte() {
    let t0 = Instant::now();
    let mut b = Batch::default();
    b.push(b"a", t0);
    b.take(t0);
    b.push(b"b", t0 + ms(1));
    b.push(b"c", t0 + ms(5));
    assert_eq!(b.due_at(), Some(t0 + ms(1) + FLUSH_AFTER));
    assert_eq!(b.take(t0 + ms(9)), b"bc");
    assert_eq!(b.due_at(), None);
}

#[test]
fn a_full_batch_is_due_without_waiting() {
    let t0 = Instant::now();
    let mut b = Batch::default();
    b.push(b"a", t0);
    b.take(t0);
    b.push(&vec![b'x'; FLUSH_AT - 1], t0 + ms(1));
    assert_eq!(b.due_at(), Some(t0 + ms(1) + FLUSH_AFTER));
    b.push(b"y", t0 + ms(2));
    assert_eq!(b.due_at(), Some(t0 + ms(1)));
}

#[test]
fn take_returns_bytes_in_order() {
    let t0 = Instant::now();
    let mut b = Batch::default();
    for chunk in [&b"one "[..], b"two ", b"three"] {
        b.push(chunk, t0);
    }
    assert_eq!(b.take(t0), b"one two three");
}

// ── Batch: DEC 2026 frames ──────────────────────────────────────────────────

#[test]
fn a_synchronized_frame_is_held_until_it_ends() {
    let t0 = Instant::now();
    let mut b = Batch::default();
    b.push(b"\x1b[?2026hpart1", t0);
    b.push(b"part2", t0 + ms(20));
    assert_eq!(b.due_at(), None);
    b.push(b"part3\x1b[?2026l", t0 + ms(40));
    assert_eq!(
        b.take(t0 + ms(40)),
        b"\x1b[?2026hpart1part2part3\x1b[?2026l"
    );
}

#[test]
fn a_frame_is_queued_whole_behind_plain_output() {
    let t0 = Instant::now();
    let mut b = Batch::default();
    b.push(b"before ", t0);
    b.push(b"\x1b[?2026hframe", t0);
    assert_eq!(b.pending_len(), b"before ".len());
    b.push(b"\x1b[?2026l", t0);
    assert_eq!(b.take(t0), b"before \x1b[?2026hframe\x1b[?2026l");
}

// ── OutputPipe: threads ─────────────────────────────────────────────────────

#[test]
fn pipe_delivers_everything_in_order_then_ends_after_close() {
    let pipe = Arc::new(OutputPipe::default());
    let reader = pipe.clone();
    let t = std::thread::spawn(move || {
        for i in 0..2000u32 {
            assert!(reader.push(format!("{i}\n").as_bytes()));
        }
        reader.close();
    });
    let mut got = Vec::new();
    let mut sends = 0;
    while let Some(bytes) = pipe.next_batch() {
        got.extend(bytes);
        sends += 1;
    }
    t.join().unwrap();
    let want: String = (0..2000u32).map(|i| format!("{i}\n")).collect();
    assert_eq!(got, want.as_bytes());
    assert!(sends < 2000, "chunks were not coalesced: {sends} sends");
}

#[test]
fn reader_blocks_while_the_pipe_is_full() {
    let pipe = Arc::new(OutputPipe::default());
    assert!(pipe.push(&vec![b'x'; MAX_PENDING]));
    let reader = pipe.clone();
    let t = std::thread::spawn(move || reader.push(b"more"));
    std::thread::sleep(ms(50));
    assert!(!t.is_finished(), "reader should wait for the sender");
    assert_eq!(pipe.next_batch().map(|b| b.len()), Some(MAX_PENDING));
    assert!(t.join().unwrap());
}

#[test]
fn closing_releases_a_blocked_reader() {
    let pipe = Arc::new(OutputPipe::default());
    assert!(pipe.push(&vec![b'x'; MAX_PENDING]));
    let reader = pipe.clone();
    let t = std::thread::spawn(move || reader.push(b"more"));
    std::thread::sleep(ms(20));
    pipe.close();
    assert!(!t.join().unwrap());
}

#[test]
fn close_flushes_pending_output_without_waiting() {
    let pipe = OutputPipe::default();
    assert!(pipe.push(b"a"));
    assert_eq!(pipe.next_batch().as_deref(), Some(&b"a"[..]));
    assert!(pipe.push(b"tail"));
    pipe.close();
    assert_eq!(pipe.next_batch().as_deref(), Some(&b"tail"[..]));
    assert_eq!(pipe.next_batch(), None);
}

// ── detect_sync ─────────────────────────────────────────────────────────────

#[test]
fn dec2026_begin_and_end_in_same_chunk_returns_true() {
    let mut in_sync = false;
    let mut buf = Vec::new();
    let chunk = b"\x1b[?2026hsome output\x1b[?2026l";
    assert!(detect_sync(chunk, &mut in_sync, &mut buf));
    assert!(!in_sync);
    assert_eq!(buf, chunk);
}

#[test]
fn dec2026_begin_chunk_does_not_emit() {
    let mut in_sync = false;
    let mut buf = Vec::new();
    let chunk1 = b"\x1b[?2026hpart1";
    assert!(!detect_sync(chunk1, &mut in_sync, &mut buf));
    assert!(in_sync);
    assert_eq!(&buf[..chunk1.len()], chunk1.as_ref());
}

#[test]
fn dec2026_end_chunk_coalesces_and_returns_true() {
    let mut in_sync = false;
    let mut buf = Vec::new();
    let chunk1 = b"\x1b[?2026hpart1";
    let chunk2 = b"part2\x1b[?2026l";
    detect_sync(chunk1, &mut in_sync, &mut buf);
    assert!(detect_sync(chunk2, &mut in_sync, &mut buf));
    assert!(!in_sync);
    let expected: Vec<u8> = [chunk1.as_ref(), chunk2.as_ref()].concat();
    assert_eq!(buf, expected);
}

#[test]
fn dec2026_no_markers_plain_passthrough() {
    let mut in_sync = false;
    let mut buf = Vec::new();
    assert!(!detect_sync(b"plain output", &mut in_sync, &mut buf));
    assert!(!in_sync);
    assert!(buf.is_empty());
}
