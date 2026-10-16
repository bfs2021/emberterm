import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Terminal } from "@xterm/xterm";

const addons: FakeAddon[] = [];

class FakeAddon {
  disposed = 0;
  lose: () => void = () => {};
  constructor() { addons.push(this); }
  onContextLoss(cb: () => void) { this.lose = cb; }
  dispose() { this.disposed++; }
}

vi.mock("@xterm/addon-webgl", () => ({ WebglAddon: FakeAddon }));

const { attachWebgl } = await import("./webglRenderer.js");

function fakeTerm(loadAddon: (a: unknown) => void = () => {}) {
  return { loadAddon: vi.fn(loadAddon) } as unknown as Terminal;
}

describe("attachWebgl", () => {
  beforeEach(() => { addons.length = 0; });

  it("loads the addon and disposes it on detach", () => {
    const term = fakeTerm();
    const detach = attachWebgl(term);
    expect(term.loadAddon).toHaveBeenCalledWith(addons[0]);
    detach();
    expect(addons[0].disposed).toBe(1);
  });

  it("falls back to the DOM renderer when WebGL cannot start", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const detach = attachWebgl(fakeTerm(() => { throw new Error("no webgl2"); }));
    expect(addons[0].disposed).toBe(1);
    expect(warn).toHaveBeenCalled();
    detach();
    expect(addons[0].disposed).toBe(1);
    warn.mockRestore();
  });

  it("disposes the addon once when the context is lost", () => {
    const detach = attachWebgl(fakeTerm());
    addons[0].lose();
    expect(addons[0].disposed).toBe(1);
    detach();
    expect(addons[0].disposed).toBe(1);
  });
});
