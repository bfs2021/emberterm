import type { SavedSession } from "../stores/sessionSnapshot.js";

export interface Autosave {
  schedule(snapshot: SavedSession): void;
  flush(): Promise<void>;
  /** Stop saving for good — used on window close so dying shells can't empty the file. */
  suspend(): void;
}

export function createAutosave(opts: {
  delayMs: number;
  save: (s: SavedSession) => Promise<void>;
  onError: (e: unknown) => void;
}): Autosave {
  let pending: SavedSession | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let suspended = false;
  // Last snapshot scheduled. Title/prompt churn re-publishes identical snapshots; letting those
  // reset the debounce timer would starve the save, and rewrite the file for nothing.
  let lastKey: string | null = null;

  async function run() {
    timer = undefined;
    const snapshot = pending;
    pending = null;
    if (!snapshot || suspended) return;
    try {
      await opts.save(snapshot);
    } catch (e) {
      lastKey = null; // let the same content be retried
      opts.onError(e);
    }
  }

  return {
    schedule(snapshot) {
      if (suspended) return;
      const key = JSON.stringify(snapshot);
      if (key === lastKey) return;
      lastKey = key;
      pending = snapshot;
      clearTimeout(timer);
      timer = setTimeout(run, opts.delayMs);
    },
    async flush() {
      clearTimeout(timer);
      await run();
    },
    suspend() {
      suspended = true;
      clearTimeout(timer);
      pending = null;
    },
  };
}
