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
