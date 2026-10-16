import { describe, it, expect, vi } from "vitest";
import { runUpdate, type UpdateState, type UpdateDeps } from "./updater.js";

type Event = { event: "Started"; data: { contentLength?: number } } | { event: "Progress"; data: { chunkLength: number } } | { event: "Finished" };

function setup(update: { version: string; events?: Event[]; fail?: Error } | null, checkFail?: Error) {
  const states: UpdateState[] = [];
  const relaunch = vi.fn().mockResolvedValue(undefined);
  const deps: UpdateDeps = {
    check: async () => {
      if (checkFail) throw checkFail;
      if (!update) return null;
      return {
        version: update.version,
        downloadAndInstall: async (cb) => {
          for (const e of update.events ?? []) cb(e);
          if (update.fail) throw update.fail;
        },
      };
    },
    relaunch,
  };
  return { states, relaunch, run: () => runUpdate(deps, (s) => states.push(s)) };
}

describe("runUpdate", () => {
  it("reports up to date when there is no newer release", async () => {
    const t = setup(null);
    await t.run();
    expect(t.states.map((s) => s.kind)).toEqual(["checking", "current"]);
    expect(t.relaunch).not.toHaveBeenCalled();
  });

  it("downloads, installs and restarts when a newer release exists", async () => {
    const t = setup({
      version: "0.2.0",
      events: [
        { event: "Started", data: { contentLength: 200 } },
        { event: "Progress", data: { chunkLength: 50 } },
        { event: "Progress", data: { chunkLength: 150 } },
        { event: "Finished" },
      ],
    });
    await t.run();
    expect(t.states).toEqual([
      { kind: "checking" },
      { kind: "downloading", version: "0.2.0", pct: 0 },
      { kind: "downloading", version: "0.2.0", pct: 25 },
      { kind: "downloading", version: "0.2.0", pct: 100 },
      { kind: "installing", version: "0.2.0" },
    ]);
    expect(t.relaunch).toHaveBeenCalledOnce();
  });

  it("shows no percentage when the server sends no length", async () => {
    const t = setup({ version: "0.2.0", events: [{ event: "Started", data: {} }, { event: "Progress", data: { chunkLength: 10 } }] });
    await t.run();
    expect(t.states.filter((s) => s.kind === "downloading").every((s) => s.kind === "downloading" && s.pct === null)).toBe(true);
  });

  it("reports a failed check without restarting", async () => {
    const t = setup(null, new Error("offline"));
    await t.run();
    expect(t.states.at(-1)).toEqual({ kind: "error", message: "offline" });
    expect(t.relaunch).not.toHaveBeenCalled();
  });

  it("reports a failed download and does not restart", async () => {
    const t = setup({ version: "0.2.0", fail: new Error("bad signature") });
    await t.run();
    expect(t.states.at(-1)).toEqual({ kind: "error", message: "bad signature" });
    expect(t.relaunch).not.toHaveBeenCalled();
  });
});
