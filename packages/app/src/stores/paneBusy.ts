// Per-pane "is a command running" as announced by the shell itself (OSC 133). Not reactive:
// it is only read at the moment the user closes a tab.
const marks = new Map<string, boolean>();

const idleListeners = new Set<() => void>();

/** Runs `fn` whenever a shell reports it is back at a prompt; returns the unsubscribe. */
export function onPaneIdle(fn: () => void): () => void {
  idleListeners.add(fn);
  return () => idleListeners.delete(fn);
}

export function markPaneBusy(sessionId: string, busy: boolean) {
  marks.set(sessionId, busy);
  if (!busy) idleListeners.forEach((fn) => fn());
}

export function forgetPane(sessionId: string) {
  marks.delete(sessionId);
}

/** The shell's own word if it gives one, otherwise the process-tree check (cmd, pwsh, ...). */
export async function isPaneBusy(sessionId: string, fallback: (id: string) => Promise<boolean>): Promise<boolean> {
  const marked = marks.get(sessionId);
  return marked !== undefined ? marked : fallback(sessionId);
}
