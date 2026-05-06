/**
 * Emit helpers — write OSC 7777 sequences to process.stdout.
 *
 * Use these in your agent process to communicate with EmberTerm.
 * Wire format: ESC ] 7777 ; <verb> ; <json-payload> ESC \
 *
 * Zero runtime dependencies.
 */

import type {
  AgentHelloPayload,
  AgentQueryPayload,
  ApprovalPayload,
  BlockAnnotatePayload,
  DiffPayload,
  SessionMarkPayload,
  ToolEndPayload,
  ToolProgressPayload,
  ToolStartPayload,
} from "./types.js";

// ST = String Terminator  ESC \
const ST = "\x1b\\";

function osc7777(verb: string, payload: unknown): void {
  process.stdout.write(`\x1b]7777;${verb};${JSON.stringify(payload)}${ST}`);
}

/** Announce the agent on startup. */
export function emitHello(p: AgentHelloPayload): void {
  osc7777("agent.hello", p);
}

/** Begin a tool call. */
export function emitToolStart(p: ToolStartPayload): void {
  osc7777("tool.start", p);
}

/** Send a progress update for an in-flight tool call. */
export function emitProgress(p: ToolProgressPayload): void {
  osc7777("tool.progress", p);
}

/** Finish a tool call. */
export function emitToolEnd(p: ToolEndPayload): void {
  osc7777("tool.end", p);
}

/** Show an inline diff viewer. */
export function emitDiff(p: DiffPayload): void {
  osc7777("diff", p);
}

/** Show an approval sheet (user must confirm before agent proceeds). */
export function emitApproval(p: ApprovalPayload): void {
  osc7777("approval", p);
}

/** Annotate the current OSC 133 block. */
export function emitBlockAnnotate(p: BlockAnnotatePayload): void {
  osc7777("block.annotate", p);
}

/** Insert a session checkpoint marker. */
export function emitSessionMark(p: SessionMarkPayload): void {
  osc7777("session.mark", p);
}

/** Query the terminal's capabilities. */
export function emitQuery(p: AgentQueryPayload): void {
  osc7777("agent.query", p);
}

/** Signal that the agent session has ended. */
export function emitBye(): void {
  osc7777("agent.bye", {});
}
