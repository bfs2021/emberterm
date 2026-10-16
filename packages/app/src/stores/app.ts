import { writable, derived } from "svelte/store";

export interface Tab {
  id: string;
  sessionId: string;
  title: string;
  shell?: string;
  cwd?: string;
  isActive: boolean;
  hasUnread: boolean;
  isRunning: boolean;
  // Split-pane state — present only when the tab is split.
  splitSessionId?: string;
  splitDirection?: "horizontal" | "vertical";
}

export const tabs = writable<Tab[]>([]);
export const activeTabId = writable<string | null>(null);

export const activeTab = derived(
  [tabs, activeTabId],
  ([$tabs, $activeTabId]) => $tabs.find((t) => t.id === $activeTabId) ?? null,
);

export const activeSessionId = derived(
  activeTab,
  ($tab) => $tab?.sessionId ?? null,
);

export const activeCwd = derived(
  activeTab,
  ($tab) => $tab?.cwd ?? "",
);

export const activeShell = derived(
  activeTab,
  ($tab) => $tab?.shell ?? "",
);

let tabCounter = 0;

export function addTab(sessionId: string, title: string, shell?: string): string {
  const id = `tab-${++tabCounter}`;
  tabs.update((ts) => {
    const updated = ts.map((t) => ({ ...t, isActive: false }));
    return [
      ...updated,
      { id, sessionId, title, shell, isActive: true, hasUnread: false, isRunning: true },
    ];
  });
  activeTabId.set(id);
  return id;
}

export function removeTab(tabId: string) {
  tabs.update((ts) => {
    const idx = ts.findIndex((t) => t.id === tabId);
    const filtered = ts.filter((t) => t.id !== tabId);
    if (filtered.length === 0) return filtered;
    const nextIdx = Math.min(idx, filtered.length - 1);
    filtered[nextIdx] = { ...filtered[nextIdx], isActive: true };
    activeTabId.set(filtered[nextIdx].id);
    return filtered;
  });
}

export function updateTab(tabId: string, patch: Partial<Tab>) {
  tabs.update((ts) => ts.map((t) => (t.id === tabId ? { ...t, ...patch } : t)));
}

export function activateTab(tabId: string) {
  tabs.update((ts) =>
    ts.map((t) => ({ ...t, isActive: t.id === tabId, hasUnread: t.id === tabId ? false : t.hasUnread })),
  );
  activeTabId.set(tabId);
}

export function setTabCwd(sessionId: string, cwd: string) {
  tabs.update((ts) =>
    ts.map((t) => (t.sessionId === sessionId ? { ...t, cwd } : t)),
  );
}

export function setTabTitle(sessionId: string, title: string) {
  tabs.update((ts) =>
    ts.map((t) => (t.sessionId === sessionId ? { ...t, title } : t)),
  );
}

export function renameTab(tabId: string, newTitle: string) {
  const trimmed = newTitle.trim();
  if (!trimmed) return;
  tabs.update((ts) => ts.map((t) => (t.id === tabId ? { ...t, title: trimmed } : t)));
}

// ── Split pane helpers ────────────────────────────────────────────────────────

/** Record the secondary session on a split tab. */
export function setTabSplit(
  tabId: string,
  splitSessionId: string,
  direction: "horizontal" | "vertical",
) {
  tabs.update((ts) =>
    ts.map((t) => (t.id === tabId ? { ...t, splitSessionId, splitDirection: direction } : t)),
  );
}

/** Move a tab to a new position in the tab strip by reordering around a target tab. */
export function reorderTab(tabId: string, targetTabId: string) {
  tabs.update((ts) => {
    const fromIdx = ts.findIndex((t) => t.id === tabId);
    const toIdx = ts.findIndex((t) => t.id === targetTabId);
    if (fromIdx < 0 || toIdx < 0 || fromIdx === toIdx) return ts;
    const result = [...ts];
    const [moved] = result.splice(fromIdx, 1);
    // After removing the element, indices after fromIdx shift left by 1.
    const insertAt = toIdx > fromIdx ? toIdx - 1 : toIdx;
    result.splice(insertAt, 0, moved);
    return result;
  });
}

/** Remove the split — the secondary session has already been closed by the caller. */
export function clearTabSplit(tabId: string) {
  tabs.update((ts) =>
    ts.map((t) =>
      t.id === tabId
        ? { ...t, splitSessionId: undefined, splitDirection: undefined }
        : t,
    ),
  );
}

// ── Pre-mount output buffer ──────────────────────────────────────────────────
// App.svelte creates a buffer for each session right after sessionCreate().
// Grid.svelte drains it on mount (before the live listener takes over) to
// recover PTY bytes emitted before xterm.js was ready.
const _outputBuffers = new Map<string, Uint8Array[]>();

export function createOutputBuffer(sessionId: string) {
  _outputBuffers.set(sessionId, []);
}

export function appendOutputBuffer(sessionId: string, bytes: Uint8Array) {
  _outputBuffers.get(sessionId)?.push(bytes);
}

export function drainOutputBuffer(sessionId: string): Uint8Array[] {
  const buf = _outputBuffers.get(sessionId) ?? [];
  _outputBuffers.delete(sessionId);
  return buf;
}
