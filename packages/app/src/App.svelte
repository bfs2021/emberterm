<script lang="ts">
  import TabPane from "./terminal/TabPane.svelte";
  import TitleBar from "./chrome/TitleBar.svelte";
  import StatusBar from "./chrome/StatusBar.svelte";
  import Toasts from "./chrome/Toasts.svelte";
  import CommandPalette from "./palette/CommandPalette.svelte";
  import SettingsPanel from "./settings/SettingsPanel.svelte";
  import WelcomeScreen from "./welcome/WelcomeScreen.svelte";
  import { playBell } from "./utils/bell.js";
  import { formatWarning } from "./utils/warnings.js";
  import { sessionClose, gitStatus as gitStatusIpc, configIsFirstRun, configGet, profileList, startupWarnings, sessionLoad, sessionSave, type Profile } from "./ipc/client.js";
  import { loadFontConfig } from "./theme/fontStore.js";
  import { loadTerminalConfig } from "./stores/terminalConfigStore.js";
  import { removeOutputSink } from "./terminal/outputSink.js";
  import { spawnPane } from "./session/spawnPane.js";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { createAutosave } from "./session/autosave.js";
  import { pickProfile } from "./session/profiles.js";
  import { toSnapshot, clampActiveTab, type SavedSession } from "./stores/sessionSnapshot.js";
  import { onTerminalBell, onProfilesUpdated, onConfigUpdated, onAppWarning } from "./ipc/events.js";
  import {
    tabs, activeCwd, activeShell,
    addTab, removeTab, activateTab, setPaneCwd, setTabTitle,
    setTabSplit, clearTabSplit, pushToast, restoreCustomTitle,
  } from "./stores/app.js";

  let loading = $state(true);
  let error = $state<string | null>(null);
  let paletteOpen = $state(false);
  let settingsOpen = $state(false);
  let currentGitBranch = $state("");
  let gitDirty = $state(false);
  let gitInsertions = $state(0);
  let gitDeletions = $state(0);
  let showWelcome = $state(false);
  let termCols = $state(0);
  let termRows = $state(0);
  let profileCache = $state<Profile[]>([]);
  let defaultProfileId = $state("");

  function shellBasename(p: string) { return (p.split(/[\\/]/).pop() ?? p).replace(/\.exe$/i, ""); }

  function handlePaneTitle(sessionId: string, title: string) {
    setTabTitle(sessionId, /^[A-Za-z]:[\\/]|^\//.test(title) ? shellBasename(title) : title);
  }


  $effect(() => {
    const cwd = $activeCwd;
    if (cwd) {
      gitStatusIpc(cwd).then((s) => {
        currentGitBranch = s.branch ?? "";
        gitDirty = s.dirty;
        gitInsertions = s.insertions;
        gitDeletions = s.deletions;
      }).catch(() => {
        currentGitBranch = "";
        gitDirty = false;
        gitInsertions = 0;
        gitDeletions = 0;
      });
    } else {
      currentGitBranch = "";
      gitDirty = false;
      gitInsertions = 0;
      gitDeletions = 0;
    }
  });

  async function loadDefaultProfileId() {
    try {
      const c = (await configGet()) as { behavior?: { default_profile?: string } };
      defaultProfileId = c?.behavior?.default_profile ?? "";
    } catch { /* keep the previous value */ }
  }

  let autosaveReady = $state(false);
  let lastSaveErrorAt = 0;
  const autosave = createAutosave({
    delayMs: 500,
    save: sessionSave,
    onError: (e) => {
      console.error("session save failed", e);
      if (Date.now() - lastSaveErrorAt > 60_000) {
        lastSaveErrorAt = Date.now();
        pushToast(`Could not save session: ${e}`);
      }
    },
  });

  // Snapshot on every tab change once restore has finished (a partial restore must not be saved).
  $effect(() => {
    const snapshot = toSnapshot($tabs);
    if (autosaveReady) autosave.schedule(snapshot);
  });

  // The title bar's close button calls win.close(), which lands here too.
  $effect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow()
      .onCloseRequested(async () => {
        await autosave.flush();
        autosave.suspend();
      })
      .then((fn) => { unlisten = fn; })
      .catch(console.error);
    return () => unlisten?.();
  });

  $effect(() => {
    let active = true;
    const cleanup: Array<() => void> = [];
    (async () => {
      try {
        showWelcome = await configIsFirstRun();
        await Promise.all([loadFontConfig(), loadTerminalConfig(), loadDefaultProfileId()]);
        const { session, warning } = await sessionLoad();
        if (warning) pushToast(formatWarning(warning));
        const restored = session ? await restoreSession(session) : false;
        if (!restored) await openNewTab();
        autosaveReady = true;
      } catch (e) { error = String(e); }
      finally { loading = false; }
      if (!active) return;
      const fns = await Promise.all([
        onTerminalBell(() => playBell()),
        onAppWarning((w) => pushToast(formatWarning(w))),
        onProfilesUpdated(() => { profileList().then((p) => { profileCache = p; }).catch(() => { profileCache = []; }); }),
        onConfigUpdated(() => { loadDefaultProfileId(); }),
      ]);
      if (!active) { fns.forEach((f) => f()); return; }
      cleanup.push(...fns);
      startupWarnings().then((ws) => ws.forEach((w) => pushToast(formatWarning(w)))).catch(console.error);
    })();
    return () => { active = false; cleanup.forEach((f) => f()); };
  });

  function handleWindowKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.shiftKey && e.key === "P") { e.preventDefault(); paletteOpen = !paletteOpen; return; }
    if (mod && e.shiftKey && e.key === "T") { e.preventDefault(); openNewTab(); return; }
    if (mod && e.shiftKey && e.key === "W") { e.preventDefault(); closeActiveTab(); return; }
    if (mod && e.key === ",") { e.preventDefault(); settingsOpen = true; return; }
    if (mod && e.key === "Tab") { e.preventDefault(); cycleTab(1); return; }
    if (mod && e.shiftKey && e.key === "Tab") { e.preventDefault(); cycleTab(-1); return; }
    if (mod && e.shiftKey && e.key === "\\") { e.preventDefault(); splitActiveTab("horizontal"); return; }
    if (mod && e.shiftKey && e.key === "-") { e.preventDefault(); splitActiveTab("vertical"); return; }
  }

  function cycleTab(dir: 1 | -1) {
    const ts = $tabs;
    if (ts.length < 2) return;
    activateTab(ts[(ts.findIndex((t) => t.isActive) + dir + ts.length) % ts.length].id);
  }
  async function closeActiveTab() {
    const tab = $tabs.find((t) => t.isActive);
    if (tab) await closeTab(tab.id, tab.sessionId);
  }

  /** Requested id, else the configured default, else the first profile (see pickProfile). */
  function resolveProfile(profileId?: string): Profile | undefined {
    return pickProfile(profileCache, profileId, defaultProfileId);
  }

  async function openNewTab(profileId?: string, cwd?: string): Promise<string | null> {
    try {
      if (!profileCache.length) { try { profileCache = await profileList(); } catch { /* ignore */ } }
      const profile = resolveProfile(profileId);
      const label = profile?.name ?? shellBasename(profile?.shell ?? "Shell");
      const shell  = profile ? shellBasename(profile.shell) : "";
      // Spawn the resolved profile, not the requested id: a dangling default_profile
      // must not make Rust pick a different shell than the tab is labelled with.
      const sessionId = await spawnPane(profile?.id, cwd);
      return addTab(sessionId, label || "Shell", shell, profile?.id);
    } catch (e) {
      // A toast, not the full-screen error splash: other (restored) tabs must stay usable.
      pushToast(`Failed to open tab: ${e}`);
      return null;
    }
  }

  async function splitTab(tabId: string, direction: "horizontal" | "vertical", profileId?: string, cwd?: string) {
    try {
      const profile = resolveProfile(profileId);
      const sessionId = await spawnPane(profile?.id, cwd);
      setTabSplit(tabId, sessionId, direction, profile?.id);
    } catch (e) { pushToast(`Failed to split pane: ${e}`); }
  }

  async function restoreSession(saved: SavedSession): Promise<boolean> {
    const ids: string[] = [];
    for (const t of saved.tabs) {
      const id = await openNewTab(t.pane.profile_id, t.pane.cwd);
      if (!id) continue;
      ids.push(id);
      if (t.custom_title && t.title) restoreCustomTitle(id, t.title);
      if (t.split) {
        const dir = t.split.direction === "vertical" ? "vertical" : "horizontal";
        await splitTab(id, dir, t.split.pane.profile_id, t.split.pane.cwd);
      }
    }
    if (!ids.length) return false;
    activateTab(ids[Math.min(clampActiveTab(saved), ids.length - 1)]);
    return true;
  }

  async function closeTab(tabId: string, sessionId: string) {
    const tab = $tabs.find((t) => t.id === tabId);
    if (tab?.splitSessionId) {
      await sessionClose(tab.splitSessionId).catch(() => {});
      removeOutputSink(tab.splitSessionId);
    }
    await sessionClose(sessionId).catch(() => {});
    removeOutputSink(sessionId);
    removeTab(tabId);
    if ($tabs.length === 0) showWelcome = true;
  }

  async function splitActiveTab(direction: "horizontal" | "vertical") {
    const tab = $tabs.find((t) => t.isActive);
    if (!tab || tab.splitSessionId) return;
    await splitTab(tab.id, direction);
  }

  async function closeSplitPane(tabId: string) {
    const tab = $tabs.find((t) => t.id === tabId);
    if (!tab?.splitSessionId) return;
    await sessionClose(tab.splitSessionId).catch(() => {});
    removeOutputSink(tab.splitSessionId);
    clearTabSplit(tabId);
  }

</script>

<svelte:window onkeydown={handleWindowKeydown} />

<!--
  .app is the window chrome (title bar + terminals + status bar).
  CSS opacity is applied here so the entire shell fades together.
  Modals are rendered OUTSIDE this div so they are NOT affected by opacity
  and always appear fully opaque, regardless of the user's transparency setting.
-->
<div class="app">
  <TitleBar onNewTab={openNewTab} onCloseTab={closeTab} onOpenSettings={() => (settingsOpen = true)} />
  <main class="terminal-area">
    {#if loading}
      <div class="splash">Starting shell…</div>
    {:else if error}
      <div class="splash error">{error}</div>
    {:else}
      {#each $tabs as tab (tab.id)}
        <TabPane
          {tab}
          isActive={tab.isActive}
          onExit={() => closeTab(tab.id, tab.sessionId)}
          onSplitExit={() => closeSplitPane(tab.id)}
          onResize={tab.isActive ? (c, r) => { termCols = c; termRows = r; } : undefined}
          onCloseSplit={() => closeSplitPane(tab.id)}
          onPaneCwd={setPaneCwd}
          onPaneTitle={handlePaneTitle}
        />
      {/each}
    {/if}
  </main>
  <StatusBar shell={$activeShell} cwd={$activeCwd} gitBranch={currentGitBranch} {gitDirty} {gitInsertions} {gitDeletions} {termCols} {termRows} onOpenPalette={() => (paletteOpen = true)} />
</div>

<!-- Modals outside .app — unaffected by window opacity -->
<CommandPalette open={paletteOpen} profiles={profileCache} {defaultProfileId} onClose={() => (paletteOpen = false)} onNewTab={openNewTab} onOpenSettings={() => (settingsOpen = true)} />
<SettingsPanel open={settingsOpen} onClose={() => (settingsOpen = false)} onTestProfile={(id) => openNewTab(id)} />
{#if showWelcome}
  <WelcomeScreen onComplete={async (profileId) => { showWelcome = false; if ($tabs.length === 0) await openNewTab(profileId); }} />
{/if}
<Toasts />

<style>
  .app {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background: transparent;
    color: var(--fg, #e6edf3);
    font-family: system-ui, sans-serif;
    overflow: hidden;
  }
  /* isolation:isolate creates a stacking context so tab-pane children
     cannot visually or event-wise escape above the title bar / status bar. */
  .terminal-area { flex: 1; min-height: 0; position: relative; isolation: isolate; }
  .splash { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--fg-muted, #7d8590); font-size: 14px; }
  .splash.error { color: var(--red, #f85149); }
</style>
