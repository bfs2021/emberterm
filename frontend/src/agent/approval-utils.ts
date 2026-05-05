/// Pure logic extracted from ApprovalSheet.svelte for testing.

/** Convert a snake_case action key to a Title Case label. */
export function formatAction(a: string): string {
  return a.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

/** Return the CSS class string for an action button at index `i`. */
export function btnClass(a: string, i: number): string {
  if (a === "deny") return "btn btn-danger";
  if (i === 0 || a.startsWith("allow_session") || a === "allow_always") return "btn btn-primary";
  if (a.startsWith("allow")) return "btn btn-ghost";
  return "btn";
}

/**
 * Map a keyboard key press to an action from the available list.
 * Returns the matched action string or `null` if no binding applies.
 *
 * Bindings:
 *  - `y/Y`   → first "allow*" action
 *  - `a/A`   → "allow_always" or "allow_session"
 *  - `n/N`   → "deny"
 *  - `Escape` → "deny" if present, otherwise last action
 */
export function resolveKey(key: string, actions: string[]): string | null {
  switch (key) {
    case "y":
    case "Y": {
      const allow = actions.find((a) => a.startsWith("allow"));
      return allow ?? null;
    }
    case "a":
    case "A": {
      const always = actions.find(
        (a) => a === "allow_always" || a === "allow_session"
      );
      return always ?? null;
    }
    case "n":
    case "N": {
      const deny = actions.find((a) => a === "deny");
      return deny ?? null;
    }
    case "Escape": {
      const deny = actions.find((a) => a === "deny");
      return deny ?? actions.at(-1) ?? null;
    }
    default:
      return null;
  }
}
