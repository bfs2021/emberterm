import { describe, it, expect } from "vitest";
import { createOutputSink, registerOutputSink, getOutputSink, removeOutputSink, toBytes } from "./outputSink.js";

const b = (s: string) => new TextEncoder().encode(s);
const s = (u: Uint8Array) => new TextDecoder().decode(u);

describe("OutputSink", () => {
  it("buffers until attached, then replays in order", () => {
    const sink = createOutputSink();
    sink.push(b("a"));
    sink.push(b("b"));
    const seen: string[] = [];
    sink.attach((u) => seen.push(s(u)));
    sink.push(b("c"));
    expect(seen).toEqual(["a", "b", "c"]);
  });

  it("buffers again after detach", () => {
    const sink = createOutputSink();
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
    const sink = createOutputSink();
    registerOutputSink("s1", sink);
    expect(getOutputSink("s1")).toBe(sink);
    removeOutputSink("s1");
    expect(getOutputSink("s1")).toBeUndefined();
  });
});

describe("toBytes", () => {
  it("accepts ArrayBuffer, Uint8Array and number[]", () => {
    expect(s(toBytes(b("hi").buffer))).toBe("hi");
    expect(s(toBytes(b("hi")))).toBe("hi");
    expect(s(toBytes([104, 105]))).toBe("hi");
  });
});
