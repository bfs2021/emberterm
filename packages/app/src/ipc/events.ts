import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { StartupWarning } from "./client.js";

// ── Payload types ──────────────────────────────────────────────────────────

export interface SessionExitPayload {
  session_id: string;
  code: number;
}

// ── Subscriptions ──────────────────────────────────────────────────────────

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

/** Fired when a watched config/profile file changes to something that fails to parse. */
export function onAppWarning(cb: (w: StartupWarning) => void): Promise<UnlistenFn> {
  return listen<StartupWarning>("app:warning", (e) => cb(e.payload));
}
