import type { StartupWarning } from "../ipc/client.js";

export function formatWarning(w: StartupWarning): string {
  return w.backup ? `${w.file}: ${w.message} — saved a copy as ${w.backup}` : `${w.file}: ${w.message}`;
}
