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

export interface TerminalTitlePayload {
  session_id: string;
  title: string;
}

export interface TerminalCwdPayload {
  session_id: string;
  cwd: string;
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

export function onTerminalTitle(cb: (p: TerminalTitlePayload) => void): Promise<UnlistenFn> {
  return listen<TerminalTitlePayload>("terminal:title", (e) => cb(e.payload));
}

export function onTerminalCwd(cb: (p: TerminalCwdPayload) => void): Promise<UnlistenFn> {
  return listen<TerminalCwdPayload>("terminal:cwd", (e) => cb(e.payload));
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
