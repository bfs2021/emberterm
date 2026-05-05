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
