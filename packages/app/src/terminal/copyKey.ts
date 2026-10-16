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
