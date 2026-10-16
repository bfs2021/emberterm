import { writable } from "svelte/store";
import { gitStatus } from "../ipc/client.js";

export interface GitInfo {
  branch: string;
  dirty: boolean;
  insertions: number;
  deletions: number;
}

const EMPTY: GitInfo = { branch: "", dirty: false, insertions: 0, deletions: 0 };

export const gitInfo = writable<GitInfo>({ ...EMPTY });

let latest = 0;

/** Looks up git info for `cwd`; an answer for a folder the user has since left is dropped. */
export async function refreshGit(cwd: string): Promise<void> {
  const mine = ++latest;
  if (!cwd) {
    gitInfo.set({ ...EMPTY });
    return;
  }
  try {
    const s = await gitStatus(cwd);
    if (mine === latest) gitInfo.set({ branch: s.branch ?? "", dirty: s.dirty, insertions: s.insertions, deletions: s.deletions });
  } catch {
    if (mine === latest) gitInfo.set({ ...EMPTY });
  }
}
