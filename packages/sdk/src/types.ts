/**
 * TypeScript mirrors of the Rust structs in agent/protocol.rs.
 * Keep in sync with packages/desktop/src/agent/protocol.rs.
 */

export type ToolStatus = "ok" | "error" | "cancelled";
export type RiskLevel = "safe" | "caution" | "destructive";
export type FoldMode = "auto" | "open" | "closed";

export interface AgentHelloPayload {
  v: number;
  name: string;
  version: string;
  pid?: number;
}

export interface ToolStartPayload {
  v: number;
  id: string;
  name: string;
  summary: string;
  args?: unknown;
  icon?: string;
  parent_id?: string;
}

export interface ToolProgressPayload {
  v: number;
  id: string;
  pct?: number;
  message?: string;
  phase?: string;
}

export interface ToolEndPayload {
  v: number;
  id: string;
  status: ToolStatus;
  summary?: string;
  duration_ms?: number;
}

export interface DiffPayload {
  v: number;
  id: string;
  path: string;
  patch: string;
  actions: string[];
}

export interface ApprovalPayload {
  v: number;
  id: string;
  title: string;
  detail: string;
  risk: RiskLevel;
  actions: string[];
}

export interface AgentQueryPayload {
  v: number;
  id: string;
  ask: string;
}

export interface BlockAnnotatePayload {
  v: number;
  agent_id?: string;
  tags?: string[];
  fold?: FoldMode;
}

export interface SessionMarkPayload {
  v: number;
  label?: string;
  kind?: string;
}
