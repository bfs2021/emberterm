/**
 * Whether closing a tab should ask first: only when the setting is on and a program is still
 * running in one of its panes. A failed check never blocks closing.
 */
export async function shouldConfirmClose(
  sessionIds: string[],
  enabled: boolean,
  isBusy: (sessionId: string) => Promise<boolean>,
): Promise<boolean> {
  if (!enabled) return false;
  for (const id of sessionIds) {
    try {
      if (await isBusy(id)) return true;
    } catch {
      // unknown: don't nag
    }
  }
  return false;
}
