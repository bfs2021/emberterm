import { invoke, Channel } from "@tauri-apps/api/core";
import { toBytes } from "../terminal/outputSink.js";

export type SessionId = string;

// ── Session commands ──────────────────────────────────────────────────────────

export async function sessionCreate(opts: {
  profileId?: string;
  onOutput: (bytes: Uint8Array) => void;
}): Promise<SessionId> {
  const channel = new Channel<ArrayBuffer>();
  channel.onmessage = (msg) => opts.onOutput(toBytes(msg));
  return invoke("session_create", { profileId: opts.profileId ?? null, onOutput: channel });
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

export interface StartupWarning {
  file: string;
  message: string;
  backup?: string | null;
}

/** Warnings from loading config/profiles at startup (drained on read). */
export async function startupWarnings(): Promise<StartupWarning[]> {
  return invoke("startup_warnings");
}

/** Returns true when no config.toml exists yet (first run). */
export async function configIsFirstRun(): Promise<boolean> {
  return invoke<boolean>("config_is_first_run");
}
