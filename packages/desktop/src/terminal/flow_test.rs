use super::*;

use std::sync::Arc;
use std::time::Duration;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn room_below_the_watermark() {
    let flow = FlowControl::default();
    flow.sent(HIGH_WATERMARK - 1);
    assert!(flow.wait_for_room());
}

#[test]
fn sender_blocks_at_the_watermark_until_acked() {
    let flow = Arc::new(FlowControl::default());
    flow.sent(HIGH_WATERMARK);
    let sender = flow.clone();
    let t = std::thread::spawn(move || sender.wait_for_room());
    std::thread::sleep(ms(50));
    assert!(
        !t.is_finished(),
        "sender must wait while the webview is behind"
    );
    flow.ack(1);
    assert!(t.join().unwrap());
}

#[test]
fn a_partial_ack_that_stays_above_the_watermark_keeps_waiting() {
    let flow = Arc::new(FlowControl::default());
    flow.sent(HIGH_WATERMARK + 100);
    let sender = flow.clone();
    let t = std::thread::spawn(move || sender.wait_for_room());
    flow.ack(50);
    std::thread::sleep(ms(50));
    assert!(!t.is_finished());
    flow.ack(51);
    assert!(t.join().unwrap());
}

#[test]
fn closing_releases_a_blocked_sender() {
    let flow = Arc::new(FlowControl::default());
    flow.sent(HIGH_WATERMARK);
    let sender = flow.clone();
    let t = std::thread::spawn(move || sender.wait_for_room());
    std::thread::sleep(ms(20));
    flow.close();
    assert!(!t.join().unwrap(), "a closed session has no room");
}

#[test]
fn over_acking_saturates_at_zero() {
    let flow = FlowControl::default();
    flow.sent(10);
    flow.ack(100);
    flow.sent(HIGH_WATERMARK - 1);
    assert!(
        flow.wait_for_room(),
        "a stale ack must not leave a negative or wrapped count"
    );
}
