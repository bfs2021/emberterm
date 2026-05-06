/**
 * Move keyboard focus between `.block-head-nav` elements in the given container.
 * dir=1 moves forward (toward the newest block), dir=-1 moves backward.
 */
export function navigateBlocks(container: HTMLElement | null, dir: 1 | -1): void {
  if (!container) return;
  const heads = Array.from(container.querySelectorAll<HTMLElement>(".block-head-nav"));
  if (!heads.length) return;
  const focused = document.activeElement as HTMLElement | null;
  const idx = focused ? heads.indexOf(focused) : -1;
  const next = idx === -1
    ? (dir === 1 ? heads.length - 1 : 0)
    : (idx + dir + heads.length) % heads.length;
  heads[next].focus();
  heads[next].scrollIntoView({ block: "nearest" });
}
