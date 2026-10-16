import { WebglAddon } from "@xterm/addon-webgl";
import type { Terminal } from "@xterm/xterm";

/**
 * Renders `term` with WebGL until the returned function is called. Disposing the addon hands the
 * terminal back to xterm's DOM renderer, which is also what happens when WebGL cannot start or the
 * GPU drops the context.
 */
export function attachWebgl(term: Terminal): () => void {
  let addon: WebglAddon | undefined = new WebglAddon();
  const release = () => {
    addon?.dispose();
    addon = undefined;
  };
  addon.onContextLoss(release);
  try {
    term.loadAddon(addon);
  } catch (e) {
    console.warn("WebGL renderer unavailable, using the DOM renderer", e);
    release();
  }
  return release;
}
