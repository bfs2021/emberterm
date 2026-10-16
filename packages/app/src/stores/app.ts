import { writable, derived, get } from "svelte/store";

export interface Tab {
  id: string;
  sessionId: string;
  title: string;
  shell?: string;
  profileId?: string;
  /** True once the user renamed the tab; shell titles no longer apply. */
  customTitle: boolean;
  cwd?: string;
  isActive: boolean;
  hasUnread: boolean;
  isRunning: boolean;
  // Split-pane state — present only when the tab is split.
  splitSessionId?: string;
  splitDirection?: "horizontal" | "vertical";
  splitProfileId?: string;
  splitCwd?: string;
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

export function addTab(sessionId: string, title: string, shell?: string, profileId?: string): string {
  const id = `tab-${++tabCounter}`;
  tabs.update((ts) => {
    const updated = ts.map((t) => ({ ...t, isActive: false }));
    return [
      ...updated,
      { id, sessionId, title, shell, profileId, customTitle: false, isActive: true, hasUnread: false, isRunning: true },
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

/** Applies `fn` to the tab list and notifies subscribers only if it returned a new list. */
function updateTabsIfChanged(fn: (ts: Tab[]) => Tab[] | null) {
  const next = fn(get(tabs));
  if (next) tabs.set(next);
}

/** OSC 7 from either pane of a tab. */
export function setPaneCwd(sessionId: string, cwd: string) {
  updateTabsIfChanged((ts) => {
    let changed = false;
    const next = ts.map((t) => {
      if (t.sessionId === sessionId && t.cwd !== cwd) { changed = true; return { ...t, cwd }; }
      if (t.splitSessionId === sessionId && t.splitCwd !== cwd) { changed = true; return { ...t, splitCwd: cwd }; }
      return t;
    });
    return changed ? next : null;
  });
}

// Shells re-send the title on every prompt; an unchanged title must not touch the store.
export function setTabTitle(sessionId: string, title: string) {
  updateTabsIfChanged((ts) => {
    let changed = false;
    const next = ts.map((t) => {
      if (t.sessionId !== sessionId || t.customTitle || t.title === title) return t;
      changed = true;
      return { ...t, title };
    });
    return changed ? next : null;
  });
}

export function renameTab(tabId: string, newTitle: string) {
  const trimmed = newTitle.trim();
  if (!trimmed) return;
  tabs.update((ts) => ts.map((t) => (t.id === tabId ? { ...t, title: trimmed, customTitle: true } : t)));
}

export function restoreCustomTitle(tabId: string, title: string) {
  renameTab(tabId, title);
}

// ── Split pane helpers ────────────────────────────────────────────────────────

/** Record the secondary session on a split tab. */
export function setTabSplit(
  tabId: string,
  splitSessionId: string,
  direction: "horizontal" | "vertical",
  splitProfileId?: string,
) {
  tabs.update((ts) =>
    ts.map((t) =>
      t.id === tabId ? { ...t, splitSessionId, splitDirection: direction, splitProfileId } : t,
    ),
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
        ? { ...t, splitSessionId: undefined, splitDirection: undefined, splitProfileId: undefined, splitCwd: undefined }
        : t,
    ),
  );
}

// ── Toasts ────────────────────────────────────────────────────────────────────
export interface Toast {
  id: number;
  message: string;
}

export const toasts = writable<Toast[]>([]);
let toastCounter = 0;

export function pushToast(message: string, ttlMs = 8000): number {
  const id = ++toastCounter;
  toasts.update((ts) => [...ts, { id, message }]);
  setTimeout(() => dismissToast(id), ttlMs);
  return id;
}

export function dismissToast(id: number) {
  toasts.update((ts) => ts.filter((t) => t.id !== id));
}
