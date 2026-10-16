import { tick } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** The last tab is gone: quit. The tick lets autosave see the empty list before the close-time flush. */
export async function closeWindowWhenEmpty(): Promise<void> {
  await tick();
  await getCurrentWindow().close();
}
