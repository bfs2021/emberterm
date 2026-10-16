import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

const gitStatus = vi.fn();
vi.mock("../ipc/client.js", () => ({ gitStatus: (cwd: string) => gitStatus(cwd) }));

import { gitInfo, refreshGit } from "./gitStatus.js";

const status = (branch: string) => ({ branch, dirty: false, insertions: 0, deletions: 0 });

beforeEach(() => { gitStatus.mockReset(); });

describe("refreshGit", () => {
  it("stores the branch for a folder", async () => {
    gitStatus.mockResolvedValue(status("main"));
    await refreshGit("/x");
    expect(get(gitInfo).branch).toBe("main");
  });

  it("clears the info when there is no folder or the lookup fails", async () => {
    gitStatus.mockResolvedValue(status("main"));
    await refreshGit("/x");
    await refreshGit("");
    expect(get(gitInfo).branch).toBe("");
    gitStatus.mockResolvedValue(status("dev"));
    await refreshGit("/y");
    gitStatus.mockImplementation(async () => { throw new Error("not a repo"); });
    await refreshGit("/z");
    expect(get(gitInfo).branch).toBe("");
  });

  it("ignores a slow answer for a folder the user already left", async () => {
    let finishSlow!: (v: unknown) => void;
    gitStatus.mockImplementationOnce(() => new Promise((r) => { finishSlow = r; }));
    const slow = refreshGit("/slow");
    gitStatus.mockResolvedValueOnce(status("fast"));
    await refreshGit("/fast");
    finishSlow(status("stale"));
    await slow;
    expect(get(gitInfo).branch).toBe("fast");
  });
});
