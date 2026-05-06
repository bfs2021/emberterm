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
  agentActive: boolean;
  agentName?: string;
  // Split-pane state — present only when the tab is split.
  splitSessionId?: string;
  splitDirection?: "horizontal" | "vertical";
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
  args?: Record<string, unknown>;
}

export interface Block {
  id: string;
  command: string;
  startedAt: number;
  endedAt?: number;
  exitCode?: number;
  durationMs?: number;
  agentId?: string;
  tags?: string[];
  fold?: string;
}

export const tabs = writable<Tab[]>([]);
export const activeTabId = writable<string | null>(null);
// Map sessionId → list of tool calls for that session
export const toolCalls = writable<Map<string, ToolCall[]>>(new Map());
// Map sessionId → list of blocks for that session (Phase 2)
export const blocks = writable<Map<string, Block[]>>(new Map());

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
      { id, sessionId, title, shell, isActive: true, hasUnread: false, isRunning: true, agentActive: false },
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

export function setTabAgent(sessionId: string, agentName: string) {
  tabs.update((ts) =>
    ts.map((t) =>
      t.sessionId === sessionId
        ? { ...t, agentActive: true, agentName }
        : t,
    ),
  );
}

export function clearTabAgent(sessionId: string) {
  tabs.update((ts) =>
    ts.map((t) =>
      t.sessionId === sessionId
        ? { ...t, agentActive: false, agentName: undefined }
        : t,
    ),
  );
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

export function addBlock(sessionId: string, block: Block) {
  blocks.update((m) => {
    const existing = m.get(sessionId) ?? [];
    m.set(sessionId, [...existing, block]);
    return new Map(m);
  });
}

export function annotateBlock(
  sessionId: string,
  blockId: string,
  patch: Pick<Block, "agentId" | "tags" | "fold">,
) {
  blocks.update((m) => {
    const list = m.get(sessionId) ?? [];
    m.set(sessionId, list.map((b) => (b.id === blockId ? { ...b, ...patch } : b)));
    return new Map(m);
  });
}

export function closeBlock(sessionId: string, blockId: string, exitCode: number, durationMs: number) {
  blocks.update((m) => {
    const list = m.get(sessionId) ?? [];
    m.set(sessionId, list.map((b) =>
      b.id === blockId ? { ...b, endedAt: Date.now(), exitCode, durationMs } : b,
    ));
    return new Map(m);
  });
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
