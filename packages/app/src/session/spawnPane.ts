import { sessionCreate } from "../ipc/client.js";
import { createOutputSink, registerOutputSink } from "../terminal/outputSink.js";
import { noteOutput } from "../stores/app.js";

const STARTUP_GRACE_MS = 2000;

/** Starts a PTY session and wires its output to a sink the Grid attaches to later. */
export async function spawnPane(profileId?: string, cwd?: string): Promise<string> {
  const sink = createOutputSink();
  const born = Date.now();
  let sid: string | null = null;
  const sessionId = await sessionCreate({
    profileId,
    cwd,
    onOutput: (bytes) => {
      sink.push(bytes);
      // The shell's startup banner is not news; only later output marks the tab unread.
      if (sid) noteOutput(sid, Date.now() - born > STARTUP_GRACE_MS);
    },
  });
  sid = sessionId;
  registerOutputSink(sessionId, sink);
  return sessionId;
}
