export interface DiffLine {
  type: "add" | "del" | "ctx" | "hunk" | "meta";
  text: string;
  oldNum?: number;
  newNum?: number;
}

export interface DiffStats {
  add: number;
  del: number;
}

/** Parse a unified diff patch string into typed diff lines. */
export function parseDiff(patch: string): DiffLine[] {
  const result: DiffLine[] = [];
  let oldN = 0;
  let newN = 0;
  for (const raw of patch.split("\n")) {
    if (raw.startsWith("@@")) {
      const m = raw.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
      if (m) { oldN = parseInt(m[1]); newN = parseInt(m[2]); }
      result.push({ type: "hunk", text: raw });
    } else if (raw.startsWith("+") && !raw.startsWith("+++")) {
      result.push({ type: "add", text: raw.slice(1), newNum: newN++ });
    } else if (raw.startsWith("-") && !raw.startsWith("---")) {
      result.push({ type: "del", text: raw.slice(1), oldNum: oldN++ });
    } else if (raw.startsWith(" ")) {
      result.push({ type: "ctx", text: raw.slice(1), oldNum: oldN++, newNum: newN++ });
    } else if (
      raw.startsWith("---") ||
      raw.startsWith("+++") ||
      raw.startsWith("diff ") ||
      raw.startsWith("index ")
    ) {
      result.push({ type: "meta", text: raw });
    }
  }
  return result;
}

// ── Side-by-side view ─────────────────────────────────────────────────────────

export interface SideBySideLine {
  type: "hunk" | "meta" | "content";
  text?: string; // for hunk / meta rows
  // Left (old) side
  oldNum?: number;
  oldType?: "del" | "ctx" | "empty";
  oldText?: string;
  // Right (new) side
  newNum?: number;
  newType?: "add" | "ctx" | "empty";
  newText?: string;
}

/**
 * Converts a flat list of unified-diff lines into side-by-side rows.
 * Consecutive del/add blocks are paired so changed lines sit on the same row.
 */
export function toSideBySide(lines: DiffLine[]): SideBySideLine[] {
  const result: SideBySideLine[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (line.type === "meta" || line.type === "hunk") {
      result.push({ type: line.type, text: line.text });
      i++;
    } else if (line.type === "ctx") {
      result.push({ type: "content", oldNum: line.oldNum, oldType: "ctx", oldText: line.text,
                    newNum: line.newNum, newType: "ctx", newText: line.text });
      i++;
    } else {
      // Consume a contiguous del+add block and pair them row-by-row.
      const dels: DiffLine[] = [];
      const adds: DiffLine[] = [];
      while (i < lines.length && (lines[i].type === "del" || lines[i].type === "add")) {
        if (lines[i].type === "del") dels.push(lines[i]);
        else adds.push(lines[i]);
        i++;
      }
      for (let j = 0; j < Math.max(dels.length, adds.length); j++) {
        const d = dels[j];
        const a = adds[j];
        result.push({ type: "content",
          oldNum: d?.oldNum, oldType: d ? "del" : "empty", oldText: d?.text ?? "",
          newNum: a?.newNum, newType: a ? "add" : "empty", newText: a?.text ?? "" });
      }
    }
  }
  return result;
}

/** Count add/del lines in a parsed diff. */
export function diffStats(lines: DiffLine[]): DiffStats {
  let add = 0;
  let del = 0;
  for (const l of lines) {
    if (l.type === "add") add++;
    else if (l.type === "del") del++;
  }
  return { add, del };
}
