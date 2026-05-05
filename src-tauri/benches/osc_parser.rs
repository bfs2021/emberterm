use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use emberterm::terminal::parser::extract_osc_events;

// ── Fixture builders ──────────────────────────────────────────────────────────

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

/// Build a realistic PTY chunk that mixes plain text, OSC 133 marks, and an
/// OSC 7777 tool.start sequence — representative of a single command block.
fn make_shell_block() -> Vec<u8> {
    let mut buf = Vec::new();
    // Prompt start (A), some prompt text, input start (B), output start (C),
    // several lines of output, command end (D).
    buf.extend_from_slice(&osc_bel("133", &["A"]));
    buf.extend_from_slice(b"\x1b[32m$ \x1b[0m");
    buf.extend_from_slice(&osc_bel("133", &["B", "aid=cargo build --release"]));
    buf.extend_from_slice(b"cargo build --release\r\n");
    buf.extend_from_slice(&osc_bel("133", &["C", "aid=cargo build --release"]));
    for i in 0..20 {
        buf.extend_from_slice(
            format!("   Compiling crate_{i} v0.1.0\r\n").as_bytes(),
        );
    }
    buf.extend_from_slice(&osc_bel("133", &["D", "0"]));
    buf
}

/// Build a chunk containing N back-to-back OSC 7777 tool.start sequences.
fn make_tool_starts(n: usize) -> Vec<u8> {
    let mut buf = Vec::new();
    for i in 0..n {
        let payload = format!(
            r#"{{"v":1,"id":"t{i}","name":"read_file","summary":"Read file_{i}.rs"}}"#
        );
        buf.extend_from_slice(&osc_bel("7777", &["tool.start", &payload]));
    }
    buf
}

/// Build a chunk of pure plain-text output (no OSC sequences).
fn make_plain_text(lines: usize) -> Vec<u8> {
    let mut buf = Vec::new();
    for i in 0..lines {
        buf.extend_from_slice(format!("Output line {i}: some terminal text here\r\n").as_bytes());
    }
    buf
}

// ── Benchmarks ────────────────────────────────────────────────────────────────

fn bench_shell_block(c: &mut Criterion) {
    let chunk = make_shell_block();
    let bytes = chunk.len();
    let mut group = c.benchmark_group("shell_block");
    group.throughput(Throughput::Bytes(bytes as u64));
    group.bench_function("parse", |b| {
        b.iter(|| extract_osc_events(black_box(&chunk)));
    });
    group.finish();
}

fn bench_tool_starts(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_starts");
    for n in [1usize, 10, 100] {
        let chunk = make_tool_starts(n);
        let bytes = chunk.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("n", n), &chunk, |b, chunk| {
            b.iter(|| extract_osc_events(black_box(chunk)));
        });
    }
    group.finish();
}

fn bench_plain_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("plain_text");
    for lines in [100usize, 1_000, 10_000] {
        let chunk = make_plain_text(lines);
        let bytes = chunk.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("lines", lines), &chunk, |b, chunk| {
            b.iter(|| extract_osc_events(black_box(chunk)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_shell_block, bench_tool_starts, bench_plain_text);
criterion_main!(benches);
