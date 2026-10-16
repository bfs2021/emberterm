interface CopyKeyEvent {
  type: string;
  code: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
}

/**
 * Ctrl+Shift+C always copies. Plain Ctrl+C copies only while text is selected; otherwise it
 * must reach the shell as the interrupt (^C).
 */
export function isCopyChord(e: CopyKeyEvent, hasSelection: boolean): boolean {
  if (e.type !== "keydown" || e.code !== "KeyC" || !e.ctrlKey || e.altKey) return false;
  return e.shiftKey || hasSelection;
}

function ctrlChord(e: CopyKeyEvent, code: string): boolean {
  return e.type === "keydown" && e.code === code && e.ctrlKey && !e.altKey;
}

/** Ctrl+V and Ctrl+Shift+V paste (Ctrl+V is not ^V here, matching Windows Terminal). */
export function isPasteChord(e: CopyKeyEvent): boolean {
  return ctrlChord(e, "KeyV");
}

/** Ctrl+Shift+F; plain Ctrl+F belongs to the shell and programs like less and vim. */
export function isFindChord(e: CopyKeyEvent): boolean {
  return ctrlChord(e, "KeyF") && e.shiftKey;
}
