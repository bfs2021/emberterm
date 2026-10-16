import { describe, it, expect } from "vitest";
import { ACK_EVERY, createOutputSink, registerOutputSink, getOutputSink, removeOutputSink, toBytes } from "./outputSink.js";

const b = (s: string) => new TextEncoder().encode(s);
const s = (u: Uint8Array) => new TextDecoder().decode(u);

describe("OutputSink", () => {
  it("buffers until attached, then replays in order", () => {
    const sink = createOutputSink(() => {});
    sink.push(b("a"));
    sink.push(b("b"));
    const seen: string[] = [];
    sink.attach((u) => seen.push(s(u)));
    sink.push(b("c"));
    expect(seen).toEqual(["a", "b", "c"]);
  });

  it("buffers again after detach", () => {
    const sink = createOutputSink(() => {});
    const first: string[] = [];
    sink.attach((u) => first.push(s(u)));
    sink.detach();
    sink.push(b("x"));
    const second: string[] = [];
    sink.attach((u) => second.push(s(u)));
    expect(first).toEqual([]);
    expect(second).toEqual(["x"]);
  });

  it("registry stores and removes sinks", () => {
    const sink = createOutputSink(() => {});
    registerOutputSink("s1", sink);
    expect(getOutputSink("s1")).toBe(sink);
    removeOutputSink("s1");
    expect(getOutputSink("s1")).toBeUndefined();
  });
});

describe("OutputSink acks", () => {
  /** A writer that holds xterm's parse callbacks so a test decides when parsing ends. */
  function heldWriter() {
    const pending: (() => void)[] = [];
    return { pending, write: (_: Uint8Array, done: () => void) => void pending.push(done) };
  }
  const chunk = (n: number) => new Uint8Array(n);

  it("acks only after xterm has parsed, in steps of ACK_EVERY", () => {
    const acks: number[] = [];
    const sink = createOutputSink((n) => acks.push(n));
    const w = heldWriter();
    sink.attach(w.write);
    sink.push(chunk(ACK_EVERY / 2));
    sink.push(chunk(ACK_EVERY / 2));
    expect(acks).toEqual([]);
    w.pending[0]();
    expect(acks).toEqual([]);
    w.pending[1]();
    expect(acks).toEqual([ACK_EVERY]);
  });

  it("never acks bytes buffered for a Grid that has not attached", () => {
    const acks: number[] = [];
    const sink = createOutputSink((n) => acks.push(n));
    sink.push(chunk(ACK_EVERY * 4));
    expect(acks).toEqual([]);
    sink.attach((_, done) => done());
    expect(acks).toEqual([ACK_EVERY * 4]);
  });

  it("acks bytes a detached Grid dropped, and ignores its late callbacks", () => {
    const acks: number[] = [];
    const sink = createOutputSink((n) => acks.push(n));
    const w = heldWriter();
    sink.attach(w.write);
    sink.push(chunk(ACK_EVERY));
    sink.detach();
    expect(acks).toEqual([ACK_EVERY]);
    w.pending[0]();
    expect(acks).toEqual([ACK_EVERY]);
  });

  it("a remounted Grid acks only what it parses", () => {
    const acks: number[] = [];
    const sink = createOutputSink((n) => acks.push(n));
    const old = heldWriter();
    sink.attach(old.write);
    sink.push(chunk(10));
    sink.detach();
    sink.push(chunk(ACK_EVERY - 10));
    sink.attach((_, done) => done());
    expect(acks).toEqual([ACK_EVERY]);
  });
});

describe("toBytes", () => {
  it("accepts ArrayBuffer, Uint8Array and number[]", () => {
    expect(s(toBytes(b("hi").buffer))).toBe("hi");
    expect(s(toBytes(b("hi")))).toBe("hi");
    expect(s(toBytes([104, 105]))).toBe("hi");
  });
});
