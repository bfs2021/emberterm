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

// ── Subscriptions ──────────────────────────────────────────────────────────

export function onTerminalOutput(cb: (p: TerminalOutputPayload) => void): Promise<UnlistenFn> {
  return listen<TerminalOutputPayload>("terminal:output", (e) => cb(e.payload));
}

export function onSessionExit(cb: (p: SessionExitPayload) => void): Promise<UnlistenFn> {
  return listen<SessionExitPayload>("terminal:exit", (e) => cb(e.payload));
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
