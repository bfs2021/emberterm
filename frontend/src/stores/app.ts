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
  agentName?: string;
}

export interface AgentInfo {
  name: string;
  version: string;
  pid?: number;
}

export interface ToolCall {
  id: string;
  name: string;
  summary: string;
  icon?: string;
  status: "pending" | "running" | "ok" | "error" | "cancelled";
  progressPct?: number;
  progressMessage?: string;
  durationMs?: number;
}

export const tabs = writable<Tab[]>([]);
export const activeTabId = writable<string | null>(null);
// Map sessionId → list of tool calls for that session
export const toolCalls = writable<Map<string, ToolCall[]>>(new Map());

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

export const activeAgentName = derived(
  activeTab,
  ($tab) => $tab?.agentName ?? "",
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

export function setTabAgent(sessionId: string, agentName: string) {
  tabs.update((ts) =>
    ts.map((t) =>
      t.sessionId === sessionId
        ? { ...t, agentActive: true, agentName }
        : t,
    ),
  );
}

export function addToolCall(sessionId: string, call: ToolCall) {
  toolCalls.update((m) => {
    const existing = m.get(sessionId) ?? [];
    m.set(sessionId, [...existing, call]);
    return new Map(m);
  });
}

export function updateToolCall(sessionId: string, id: string, patch: Partial<ToolCall>) {
  toolCalls.update((m) => {
    const calls = m.get(sessionId) ?? [];
    m.set(
      sessionId,
      calls.map((c) => (c.id === id ? { ...c, ...patch } : c)),
    );
    return new Map(m);
  });
}
