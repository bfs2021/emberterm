import { invoke } from "@tauri-apps/api/core";

export type SessionId = string;

// ── Session commands ──────────────────────────────────────────────────────────

export async function sessionCreate(profileId?: string): Promise<SessionId> {
  return invoke("session_create", { profileId: profileId ?? null });
}

export async function sessionWrite(id: SessionId, data: Uint8Array): Promise<void> {
  return invoke("session_write", { id, data: Array.from(data) });
}

export async function sessionResize(id: SessionId, cols: number, rows: number): Promise<void> {
  return invoke("session_resize", { id, cols, rows });
}

export async function sessionClose(id: SessionId): Promise<void> {
  return invoke("session_close", { id });
}

// ── Config commands ───────────────────────────────────────────────────────────

export async function configGet(): Promise<unknown> {
  return invoke("config_get");
}

export async function configUpdate(patch: Record<string, unknown>): Promise<unknown> {
  return invoke("config_update", { patch });
}

// ── Profile commands ──────────────────────────────────────────────────────────

export interface Profile {
  id: string;
  name: string;
  icon: string;
  shell: string;
  args: string[];
  cwd: string;
  env: Record<string, string>;
  theme?: string;
  font_family?: string;
  font_size?: number;
}

export async function profileList(): Promise<Profile[]> {
  return invoke("profile_list");
}

export async function profileUpsert(profile: Profile): Promise<void> {
  return invoke("profile_upsert", { profile });
}

export async function profileDelete(id: string): Promise<void> {
  return invoke("profile_delete", { id });
}

export interface BuiltinProfileSpec {
  id: string;
  name: string;
  icon: string;
  shell: string;
  args: string[];
  description: string;
}

/** Probe the system for known shells and return those that are installed. */
export async function detectBuiltinProfiles(): Promise<BuiltinProfileSpec[]> {
  return invoke("detect_builtin_profiles");
}

// ── Block commands ────────────────────────────────────────────────────────────

export interface BlockRecord {
  id: string;
  command: string;
  started_at_ms: number;
  exit_code: number | null;
  duration_ms: number | null;
}

/** Returns all blocks recorded for a session in start order. */
export async function blockGet(sessionId: SessionId): Promise<BlockRecord[]> {
  return invoke("block_get", { sessionId });
}

/**
 * Searches blocks by command text (case-insensitive substring match).
 * An empty query returns all blocks.
 */
export async function blockSearch(sessionId: SessionId, query: string): Promise<BlockRecord[]> {
  return invoke("block_search", { sessionId, query });
}

// ── Agent commands ────────────────────────────────────────────────────────────

/**
 * Write an OSC 7777 sequence back into the agent's PTY stdin.
 * verb: "reply" | "event"
 * payload: JSON string
 */
export async function agentRespond(
  sessionId: SessionId,
  verb: string,
  payload: string
): Promise<void> {
  return invoke("agent_respond", { sessionId, verb, payload });
}

// ── Quake commands ────────────────────────────────────────────────────────────

/**
 * Toggle quake mode: hide the window if visible, or show and reposition it
 * at the top of the screen. Mirrors the OS-level Ctrl+` global shortcut.
 */
export async function quakeToggle(): Promise<void> {
  return invoke("quake_toggle");
}

// ── Utility commands ──────────────────────────────────────────────────────────

export interface GitStatus {
  branch: string | null;
  dirty: boolean;
  insertions: number;
  deletions: number;
}

/** Returns git status (branch, dirty flag, line diff counts) for the given directory. */
export async function gitStatus(cwd: string): Promise<GitStatus> {
  return invoke<GitStatus>("git_status", { cwd });
}

/** Runs `executable --version` and returns a semver string like "v2.1.128", or null. */
export async function queryAgentVersion(executable: string): Promise<string | null> {
  return invoke<string | null>("query_agent_version", { executable });
}

/** Returns true when no config.toml exists yet (first run). */
export async function configIsFirstRun(): Promise<boolean> {
  return invoke<boolean>("config_is_first_run");
}

/**
 * Set the OS-level window opacity (0.0–1.0). On Windows this uses
 * SetLayeredWindowAttributes so the entire compositor surface — including
 * WebGL canvases — becomes transparent. On other platforms this is a no-op;
 * CSS rgba transparency handles it instead.
 */
export async function setWindowOpacity(value: number): Promise<void> {
  return invoke("set_window_opacity", { value });
}
