/** Holds a session's PTY bytes until its Grid attaches, then streams live. */
export interface OutputSink {
  push(bytes: Uint8Array): void;
  attach(write: (bytes: Uint8Array) => void): void;
  detach(): void;
}

export function createOutputSink(): OutputSink {
  let buffered: Uint8Array[] = [];
  let writer: ((bytes: Uint8Array) => void) | null = null;
  return {
    push(bytes) {
      if (writer) writer(bytes);
      else buffered.push(bytes);
    },
    attach(write) {
      for (const bytes of buffered) write(bytes);
      buffered = [];
      writer = write;
    },
    detach() {
      writer = null;
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
