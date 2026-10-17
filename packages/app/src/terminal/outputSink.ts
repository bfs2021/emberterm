/** Hands bytes to xterm; `done` runs once xterm has parsed them. */
export type SinkWriter = (bytes: Uint8Array, done: () => void) => void;

/** Holds a session's PTY bytes until its Grid attaches, then streams live. */
export interface OutputSink {
  push(bytes: Uint8Array): void;
  attach(write: SinkWriter): void;
  detach(): void;
}

/**
 * Parsed bytes are acked in steps this size, so typing echo does not cost an IPC call per
 * key. It must stay well under Rust's `HIGH_WATERMARK`, since the remainder is never acked.
 */
export const ACK_EVERY = 32 * 1024;

/**
 * `ack` tells Rust how many bytes xterm has parsed; Rust stops sending while too many are
 * unacked. Buffered bytes are acked only once a Grid parses them, so output for a pane
 * without a Grid stalls the shell instead of growing this buffer.
 */
export function createOutputSink(ack: (bytes: number) => void): OutputSink {
  let buffered: Uint8Array[] = [];
  let writer: SinkWriter | null = null;
  // Bumped on detach so a late callback from a torn-down xterm is not counted twice.
  let generation = 0;
  let inFlight = 0;
  let parsed = 0;

  function settle(bytes: number) {
    parsed += bytes;
    if (parsed < ACK_EVERY) return;
    ack(parsed);
    parsed = 0;
  }

  function write(w: SinkWriter, bytes: Uint8Array) {
    const gen = generation;
    inFlight += bytes.length;
    w(bytes, () => {
      if (gen !== generation) return;
      inFlight -= bytes.length;
      settle(bytes.length);
    });
  }

  return {
    push(bytes) {
      if (writer) write(writer, bytes);
      else buffered.push(bytes);
    },
    attach(w) {
      for (const bytes of buffered) write(w, bytes);
      buffered = [];
      writer = w;
    },
    detach() {
      // A disposed xterm never calls back; ack what it dropped so the sender is not stuck.
      generation++;
      writer = null;
      settle(inFlight);
      inFlight = 0;
    },
  };
}

const sinks = new Map<string, OutputSink>();

export function registerOutputSink(sessionId: string, sink: OutputSink) {
  sinks.set(sessionId, sink);
}

export function getOutputSink(sessionId: string): OutputSink | undefined {
  return sinks.get(sessionId);
}

export function removeOutputSink(sessionId: string) {
  sinks.delete(sessionId);
}

/** Tauri delivers raw channel bodies as ArrayBuffer; accept the JSON fallback too. */
export function toBytes(msg: unknown): Uint8Array {
  if (msg instanceof Uint8Array) return msg;
  if (msg instanceof ArrayBuffer) return new Uint8Array(msg);
  if (Array.isArray(msg)) return Uint8Array.from(msg as number[]);
  return new Uint8Array();
}
