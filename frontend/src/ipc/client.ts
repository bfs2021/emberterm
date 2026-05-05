import { invoke } from "@tauri-apps/api/core";

// ── Types (mirrors Rust structs in src-tauri/src) ──────────────────────────

export type SessionId = string;

// ── Commands (webview → Rust) ──────────────────────────────────────────────

export async function sessionCreate(profileId?: string): Promise<SessionId> {
  return invoke("session_create", { profileId: profileId ?? null });
}

export async function sessionWrite(id: SessionId, data: Uint8Array): Promise<void> {
  return invoke("session_write", { id, data: Array.from(data) });
}

export async function sessionResize(
  id: SessionId,
  cols: number,
  rows: number,
): Promise<void> {
  return invoke("session_resize", { id, cols, rows });
}

export async function sessionClose(id: SessionId): Promise<void> {
  return invoke("session_close", { id });
}
