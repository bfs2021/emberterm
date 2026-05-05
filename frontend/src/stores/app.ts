import { writable, derived } from "svelte/store";

export interface Tab {
  id: string;
  sessionId: string;
  title: string;
  cwd?: string;
  isActive: boolean;
  hasUnread: boolean;
  isRunning: boolean;
  agentActive: boolean;
}

export interface AgentInfo {
  name: string;
  version: string;
  pid?: number;
}

export const tabs = writable<Tab[]>([]);
export const activeTabId = writable<string | null>(null);
export const config = writable<Record<string, unknown>>({});

export const activeTab = derived(
  [tabs, activeTabId],
  ([$tabs, $activeTabId]) => $tabs.find((t) => t.id === $activeTabId) ?? null,
);

export const activeSessionId = derived(
  activeTab,
  ($tab) => $tab?.sessionId ?? null,
);

let tabCounter = 0;

export function addTab(sessionId: string, title: string): string {
  const id = `tab-${++tabCounter}`;
  tabs.update((ts) => {
    const updated = ts.map((t) => ({ ...t, isActive: false }));
    return [
      ...updated,
      { id, sessionId, title, isActive: true, hasUnread: false, isRunning: true, agentActive: false },
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
    // Activate the adjacent tab after removal.
    const nextIdx = Math.min(idx, filtered.length - 1);
    filtered[nextIdx].isActive = true;
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
