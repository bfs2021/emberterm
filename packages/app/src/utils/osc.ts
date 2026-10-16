/**
 * Extracts the filesystem path from an OSC 7 payload (`file://<host>/<path>`).
 * Our prompt hooks emit raw paths, so decoding is best-effort: a literal `%`
 * that is not valid percent-encoding is kept as-is.
 */
export function parseOsc7(data: string): string | null {
  if (!data.startsWith("file://")) return null;
  const rest = data.slice("file://".length);
  const slash = rest.indexOf("/");
  if (slash < 0) return null;
  let path = rest.slice(slash);
  try {
    path = decodeURIComponent(path);
  } catch {
    // keep the raw path
  }
  // pwsh reports `/D:/x`; Windows needs `D:/x`.
  if (/^\/[A-Za-z]:(\/|$)/.test(path)) path = path.slice(1);
  return path;
}

/** Decodes an OSC 52 clipboard *write* (`<target>;<base64>`). Reads (`?`) are refused. */
export function parseOsc52(data: string): string | null {
  const sep = data.indexOf(";");
  if (sep < 0) return null;
  const b64 = data.slice(sep + 1);
  if (!b64 || b64 === "?") return null;
  try {
    const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  } catch {
    return null;
  }
}

/**
 * OSC 133 prompt marks from our bash hook: `C` = a command started (busy), `A` / `D` = the shell
 * is back at a prompt (idle). Anything else (e.g. `B`) says nothing about busyness.
 */
export function parseOsc133(data: string): boolean | null {
  const mark = data.split(";")[0];
  if (mark === "C") return true;
  if (mark === "A" || mark === "D") return false;
  return null;
}
