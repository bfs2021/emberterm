import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// We test the wire output by spying on process.stdout.write.
// Each emit* call should produce exactly one OSC 7777 sequence.

// OSC 7777 wire format: ESC ] 7777 ; <verb> ; <json> ESC \
const OSC_PREFIX = "\x1b]7777;";
const ST = "\x1b\\";

function parseOsc(raw: string): { verb: string; payload: unknown } {
  expect(raw.startsWith(OSC_PREFIX)).toBe(true);
  expect(raw.endsWith(ST)).toBe(true);
  const inner = raw.slice(OSC_PREFIX.length, raw.length - ST.length);
  const semi = inner.indexOf(";");
  expect(semi).toBeGreaterThan(0);
  const verb = inner.slice(0, semi);
  const payload = JSON.parse(inner.slice(semi + 1));
  return { verb, payload };
}

describe("SDK emit helpers", () => {
  let written: string;
  let spy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    written = "";
    spy = vi.spyOn(process.stdout, "write").mockImplementation((chunk) => {
      written += chunk as string;
      return true;
    });
  });

  afterEach(() => {
    spy.mockRestore();
  });

  // ── emitHello ─────────────────────────────────────────────────────────────

  it("emitHello writes a valid OSC 7777 agent.hello frame", async () => {
    const { emitHello } = await import("./emit.js");
    emitHello({ v: 1, name: "test-agent", version: "1.2.3", pid: 42 });

    const { verb, payload } = parseOsc(written);
    expect(verb).toBe("agent.hello");
    expect(payload).toMatchObject({ v: 1, name: "test-agent", version: "1.2.3", pid: 42 });
  });

  it("emitHello serialises name and version correctly", async () => {
    const { emitHello } = await import("./emit.js");
    emitHello({ v: 1, name: "my-agent", version: "0.0.1" });

    const { payload } = parseOsc(written);
    expect((payload as { name: string }).name).toBe("my-agent");
    expect((payload as { version: string }).version).toBe("0.0.1");
    expect((payload as { pid?: number }).pid).toBeUndefined();
  });

  // ── emitToolStart ──────────────────────────────────────────────────────────

  it("emitToolStart produces the correct wire format", async () => {
    const { emitToolStart } = await import("./emit.js");
    emitToolStart({ v: 1, id: "t1", name: "read_file", summary: "Reading src/main.rs" });

    const { verb, payload } = parseOsc(written);
    expect(verb).toBe("tool.start");
    expect(payload).toMatchObject({ v: 1, id: "t1", name: "read_file", summary: "Reading src/main.rs" });
  });

  it("emitToolStart includes optional fields when provided", async () => {
    const { emitToolStart } = await import("./emit.js");
    emitToolStart({
      v: 1,
      id: "t2",
      name: "edit_file",
      summary: "Editing",
      args: { path: "a.ts" },
      icon: "pencil",
      parent_id: "t1",
    });

    const { payload } = parseOsc(written);
    const p = payload as Record<string, unknown>;
    expect(p.args).toEqual({ path: "a.ts" });
    expect(p.icon).toBe("pencil");
    expect(p.parent_id).toBe("t1");
  });

  // ── emitToolEnd ───────────────────────────────────────────────────────────

  it("emitToolEnd with status ok produces correct frame", async () => {
    const { emitToolEnd } = await import("./emit.js");
    emitToolEnd({ v: 1, id: "t1", status: "ok", summary: "Done", duration_ms: 150 });

    const { verb, payload } = parseOsc(written);
    expect(verb).toBe("tool.end");
    expect((payload as { status: string }).status).toBe("ok");
    expect((payload as { duration_ms: number }).duration_ms).toBe(150);
  });

  it("emitToolEnd accepts all three status values", async () => {
    const { emitToolEnd } = await import("./emit.js");

    for (const status of ["ok", "error", "cancelled"] as const) {
      written = "";
      emitToolEnd({ v: 1, id: "t1", status });
      const { payload } = parseOsc(written);
      expect((payload as { status: string }).status).toBe(status);
    }
  });

  // ── emitProgress ──────────────────────────────────────────────────────────

  it("emitProgress writes verb tool.progress with pct and message", async () => {
    const { emitProgress } = await import("./emit.js");
    emitProgress({ v: 1, id: "t1", pct: 50, message: "Halfway there" });

    const { verb, payload } = parseOsc(written);
    expect(verb).toBe("tool.progress");
    expect((payload as { pct: number }).pct).toBe(50);
    expect((payload as { message: string }).message).toBe("Halfway there");
  });

  // ── emitDiff ──────────────────────────────────────────────────────────────

  it("emitDiff writes verb diff with required fields", async () => {
    const { emitDiff } = await import("./emit.js");
    emitDiff({ v: 1, id: "d1", path: "src/main.rs", patch: "@@ -1 +1 @@\n-old\n+new\n", actions: ["accept", "reject"] });

    const { verb, payload } = parseOsc(written);
    expect(verb).toBe("diff");
    const p = payload as Record<string, unknown>;
    expect(p.path).toBe("src/main.rs");
    expect(p.actions).toEqual(["accept", "reject"]);
  });

  // ── emitApproval ──────────────────────────────────────────────────────────

  it("emitApproval writes verb approval with risk field", async () => {
    const { emitApproval } = await import("./emit.js");
    emitApproval({
      v: 1,
      id: "a1",
      title: "Run rm -rf",
      detail: "rm -rf node_modules",
      risk: "destructive",
      actions: ["allow_once", "deny"],
    });

    const { verb, payload } = parseOsc(written);
    expect(verb).toBe("approval");
    expect((payload as { risk: string }).risk).toBe("destructive");
  });

  // ── emitBye ───────────────────────────────────────────────────────────────

  it("emitBye writes verb agent.bye", async () => {
    const { emitBye } = await import("./emit.js");
    emitBye();

    const { verb } = parseOsc(written);
    expect(verb).toBe("agent.bye");
  });

  // ── Frame integrity ───────────────────────────────────────────────────────

  it("every frame starts with ESC ] 7777 ; and ends with ESC \\", async () => {
    const { emitHello, emitBye, emitToolStart, emitToolEnd } = await import("./emit.js");
    const calls = [
      () => emitHello({ v: 1, name: "a", version: "1.0.0" }),
      () => emitBye(),
      () => emitToolStart({ v: 1, id: "x", name: "n", summary: "s" }),
      () => emitToolEnd({ v: 1, id: "x", status: "ok" }),
    ];

    for (const call of calls) {
      written = "";
      call();
      expect(written.startsWith(OSC_PREFIX)).toBe(true);
      expect(written.endsWith(ST)).toBe(true);
    }
  });

  it("JSON payload in each frame is valid and parseable", async () => {
    const { emitToolStart } = await import("./emit.js");
    emitToolStart({ v: 1, id: "t42", name: "search", summary: "Searching files" });

    const inner = written.slice(OSC_PREFIX.length, written.length - ST.length);
    const semi = inner.indexOf(";");
    const jsonStr = inner.slice(semi + 1);
    expect(() => JSON.parse(jsonStr)).not.toThrow();
  });
});
