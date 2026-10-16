import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Payload types ──────────────────────────────────────────────────────────

export interface TerminalOutputPayload {
  session_id: string;
  data: number[];
}

export interface SessionExitPayload {
  session_id: string;
  code: number;
}

export interface AgentHelloPayload {
  session_id: string;
  name: string;
  version: string;
  pid?: number;
}

export interface ToolStartPayload {
  session_id: string;
  id: string;
  name: string;
  summary: string;
  icon?: string;
  parent_id?: string;
  args?: Record<string, unknown>;
}

export interface ToolProgressPayload {
  session_id: string;
  id: string;
  pct?: number;
  message?: string;
}

export interface ToolEndPayload {
  session_id: string;
  id: string;
  status: string;
  summary?: string;
  duration_ms?: number;
}

// ── Subscriptions ──────────────────────────────────────────────────────────

export function onTerminalOutput(cb: (p: TerminalOutputPayload) => void): Promise<UnlistenFn> {
  return listen<TerminalOutputPayload>("terminal:output", (e) => cb(e.payload));
}

export function onSessionExit(cb: (p: SessionExitPayload) => void): Promise<UnlistenFn> {
  return listen<SessionExitPayload>("terminal:exit", (e) => cb(e.payload));
}

export function onAgentHello(cb: (p: AgentHelloPayload) => void): Promise<UnlistenFn> {
  return listen<AgentHelloPayload>("agent:hello", (e) => cb(e.payload));
}

export function onToolStart(cb: (p: ToolStartPayload) => void): Promise<UnlistenFn> {
  return listen<ToolStartPayload>("agent:tool_start", (e) => cb(e.payload));
}

export function onToolProgress(cb: (p: ToolProgressPayload) => void): Promise<UnlistenFn> {
  return listen<ToolProgressPayload>("agent:tool_progress", (e) => cb(e.payload));
}

export function onToolEnd(cb: (p: ToolEndPayload) => void): Promise<UnlistenFn> {
  return listen<ToolEndPayload>("agent:tool_end", (e) => cb(e.payload));
}

// ── Agent diff / approval (Phase 2) ──────────────────────────────────────

export interface AgentDiffPayload {
  session_id: string;
  id: string;
  path: string;
  patch: string;
  actions: string[];
}

export interface AgentApprovalPayload {
  session_id: string;
  id: string;
  title: string;
  detail: string;
  risk: "safe" | "caution" | "destructive";
  actions: string[];
}

export function onAgentDiff(cb: (p: AgentDiffPayload) => void): Promise<UnlistenFn> {
  return listen<AgentDiffPayload>("agent:diff", (e) => cb(e.payload));
}

export function onAgentApproval(cb: (p: AgentApprovalPayload) => void): Promise<UnlistenFn> {
  return listen<AgentApprovalPayload>("agent:approval", (e) => cb(e.payload));
}

// ── Agent bye ─────────────────────────────────────────────────────────────

export interface AgentByePayload {
  session_id: string;
}

export function onAgentBye(cb: (p: AgentByePayload) => void): Promise<UnlistenFn> {
  return listen<AgentByePayload>("agent:bye", (e) => cb(e.payload));
}

// ── Bell ──────────────────────────────────────────────────────────────────

export interface TerminalBellPayload {
  session_id: string;
}

export function onTerminalBell(cb: (p: TerminalBellPayload) => void): Promise<UnlistenFn> {
  return listen<TerminalBellPayload>("terminal:bell", (e) => cb(e.payload));
}

// ── Hot-reload events ──────────────────────────────────────────────────────

/** Fired when ~/.config/ember/profiles/ changes on disk. */
export function onProfilesUpdated(cb: () => void): Promise<UnlistenFn> {
  return listen("profiles:updated", () => cb());
}

/** Fired when ~/.config/ember/config.toml changes on disk. */
export function onConfigUpdated(cb: () => void): Promise<UnlistenFn> {
  return listen("config:updated", () => cb());
}

// ── Block events (Phase 2 — emitted when Rust block tracking is enabled) ──

export interface BlockStartPayload {
  session_id: string;
  id: string;
  command: string;
  started_at: number;
  agent_id?: string;
}

export interface BlockEndPayload {
  session_id: string;
  id: string;
  exit_code: number;
  duration_ms: number;
}

export interface BlockAnnotatePayload {
  session_id: string;
  id: string;
  agent_id?: string;
  tags: string[];
  fold?: string;
}

export function onBlockStart(cb: (p: BlockStartPayload) => void): Promise<UnlistenFn> {
  return listen<BlockStartPayload>("block:start", (e) => cb(e.payload));
}

export function onBlockEnd(cb: (p: BlockEndPayload) => void): Promise<UnlistenFn> {
  return listen<BlockEndPayload>("block:end", (e) => cb(e.payload));
}

export function onBlockAnnotate(cb: (p: BlockAnnotatePayload) => void): Promise<UnlistenFn> {
  return listen<BlockAnnotatePayload>("block:annotate", (e) => cb(e.payload));
}
