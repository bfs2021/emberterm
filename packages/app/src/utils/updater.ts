import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "downloading"; version: string; pct: number | null }
  | { kind: "installing"; version: string }
  | { kind: "error"; message: string };

type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

export interface UpdateDeps {
  check: () => Promise<{ version: string; downloadAndInstall: (cb: (e: DownloadEvent) => void) => Promise<void> } | null>;
  relaunch: () => Promise<void>;
}

export const tauriUpdateDeps: UpdateDeps = { check, relaunch };

/** One click: look for a newer release, and if there is one download it, install it and restart. */
export async function runUpdate(deps: UpdateDeps, set: (s: UpdateState) => void): Promise<void> {
  set({ kind: "checking" });
  try {
    const update = await deps.check();
    if (!update) { set({ kind: "current" }); return; }

    let total: number | undefined;
    let received = 0;
    const pct = () => (total ? Math.min(100, Math.round((received / total) * 100)) : null);
    await update.downloadAndInstall((e) => {
      if (e.event === "Started") { total = e.data.contentLength; received = 0; }
      else if (e.event === "Progress") received += e.data.chunkLength;
      else return;
      set({ kind: "downloading", version: update.version, pct: pct() });
    });
    set({ kind: "installing", version: update.version });
    await deps.relaunch();
  } catch (e) {
    set({ kind: "error", message: e instanceof Error ? e.message : String(e) });
  }
}
