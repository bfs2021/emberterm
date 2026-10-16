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

  async function run() {
    timer = undefined;
    const snapshot = pending;
    pending = null;
    if (!snapshot || suspended) return;
    try {
      await opts.save(snapshot);
    } catch (e) {
      opts.onError(e);
    }
  }

  return {
    schedule(snapshot) {
      if (suspended) return;
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
