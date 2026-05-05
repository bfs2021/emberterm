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

// ── Utility commands ──────────────────────────────────────────────────────────

/** Returns the current git branch for the given directory, or null. */
export async function gitBranch(cwd: string): Promise<string | null> {
  return invoke<string | null>("git_branch", { cwd });
}

/** Returns true when no config.toml exists yet (first run). */
export async function configIsFirstRun(): Promise<boolean> {
  return invoke<boolean>("config_is_first_run");
}
