import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Payload types (mirrors Rust event payloads) ────────────────────────────

export interface TerminalOutputPayload {
  session_id: string;
  data: number[]; // Vec<u8> serialized as JSON array
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

// ── Subscriptions ──────────────────────────────────────────────────────────

export function onTerminalOutput(
  cb: (payload: TerminalOutputPayload) => void,
): Promise<UnlistenFn> {
  return listen<TerminalOutputPayload>("terminal:output", (e) => cb(e.payload));
}

export function onSessionExit(
  cb: (payload: SessionExitPayload) => void,
): Promise<UnlistenFn> {
  return listen<SessionExitPayload>("terminal:exit", (e) => cb(e.payload));
}

export function onTerminalTitle(
  cb: (payload: TerminalTitlePayload) => void,
): Promise<UnlistenFn> {
  return listen<TerminalTitlePayload>("terminal:title", (e) => cb(e.payload));
}

export function onTerminalCwd(
  cb: (payload: TerminalCwdPayload) => void,
): Promise<UnlistenFn> {
  return listen<TerminalCwdPayload>("terminal:cwd", (e) => cb(e.payload));
}
