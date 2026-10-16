import type { Profile } from "../ipc/client.js";

/**
 * Requested profile, else the configured default, else the first profile. A saved
 * profile id that no longer exists must land on the user's default, not on
 * whichever file sorts first.
 */
export function pickProfile(
  profiles: Profile[],
  id: string | undefined,
  defaultId: string,
): Profile | undefined {
  return (
    (id ? profiles.find((p) => p.id === id) : undefined) ??
    (defaultId ? profiles.find((p) => p.id === defaultId) : undefined) ??
    profiles[0]
  );
}
