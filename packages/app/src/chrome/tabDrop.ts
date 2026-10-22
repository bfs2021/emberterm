/** Which half of a tab the pointer is over decides whether the dragged tab lands before or after it. */
export type DropSide = "before" | "after";

export function dropSide(clientX: number, rect: { left: number; width: number }): DropSide {
  return clientX < rect.left + rect.width / 2 ? "before" : "after";
}

/** Index the dragged tab ends up at once it has been removed from `fromIdx` and inserted beside `overIdx`. */
export function dropIndex(fromIdx: number, overIdx: number, side: DropSide): number {
  const slot = side === "after" ? overIdx + 1 : overIdx;
  return slot > fromIdx ? slot - 1 : slot;
}
