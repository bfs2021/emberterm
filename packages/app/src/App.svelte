<script lang="ts">
  import TabPane from "./terminal/TabPane.svelte";
  import TitleBar from "./chrome/TitleBar.svelte";
  import StatusBar from "./chrome/StatusBar.svelte";
  import CommandPalette from "./palette/CommandPalette.svelte";
  import SettingsPanel from "./settings/SettingsPanel.svelte";
  import WelcomeScreen from "./welcome/WelcomeScreen.svelte";
  import { playBell } from "./utils/bell.js";
  import { navigateBlocks } from "./utils/blockNav.js";
  import { sessionCreate, sessionClose, agentRespond, gitStatus as gitStatusIpc, configIsFirstRun, configGet, profileList, quakeToggle, queryAgentVersion, type Profile } from "./ipc/client.js";
  import { loadFontConfig } from "./theme/fontStore.js";
  import { loadTerminalConfig, applyOpacity } from "./stores/terminalConfigStore.js";
  import { loadAgentConfig } from "./stores/app.js";
  import {
    onTerminalOutput, onTerminalTitle, onTerminalCwd, onTerminalBell,
    onAgentHello, onToolStart, onToolProgress, onToolEnd, onClipboardWrite,
    onProfilesUpdated, onConfigUpdated, onAgentDiff, onAgentApproval, onAgentBye,
    onBlockStart, onBlockEnd, onBlockAnnotate,
    type AgentDiffPayload, type AgentApprovalPayload,
  } from "./ipc/events.js";
  import { get } from "svelte/store";
  import {
    tabs, activeCwd, activeAgentName, activeAgentVersion, activeShell, activeSessionId,
    addTab, removeTab, activateTab, setTabCwd, setTabTitle,
    setTabAgent, clearTabAgent, addToolCall, updateToolCall, toolCalls,
    addBlock, annotateBlock, closeBlock, blocks,
    setTabSplit, clearTabSplit, createOutputBuffer, appendOutputBuffer,
  } from "./stores/app.js";
  import "./theme/store.js";

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
  let profileCache: Profile[] = [];
  let defaultProfileId = $state("");

  function shellBasename(p: string) { return (p.split(/[\\/]/).pop() ?? p).replace(/\.exe$/i, ""); }

  // Known agents detectable by title change (for agents that don't emit OSC 7777).
  const AGENT_TITLE_PATTERNS: { pattern: RegExp; name: string; executable: string }[] = [
    { pattern: /claude[\s-]?code/i, name: "claude-code", executable: "claude" },
    { pattern: /\baider\b/i, name: "aider", executable: "aider" },
    { pattern: /\bcodex\b/i, name: "codex", executable: "codex" },
  ];

  let pendingHellos = $state(new Map<string, { name: string; version: string }>());
  let pendingDiffs = $state(new Map<string, AgentDiffPayload>());
  let pendingApprovals = $state(new Map<string, AgentApprovalPayload>());

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

  $effect(() => {
    let active = true;
    const cleanup: Array<() => void> = [];
    (async () => {
      cleanup.push(await onTerminalOutput((p) => appendOutputBuffer(p.session_id, new Uint8Array(p.data))));
      try {
        showWelcome = await configIsFirstRun();
        await Promise.all([openNewTab(), loadFontConfig(), loadTerminalConfig(), loadAgentConfig()]);
        // Apply persisted window opacity + read default profile id
        configGet().then((cfg: unknown) => {
          const c = cfg as { appearance?: { opacity?: number }; behavior?: { default_profile?: string } };
          if (typeof c?.appearance?.opacity === "number") applyOpacity(c.appearance.opacity);
          defaultProfileId = c?.behavior?.default_profile ?? "";
        }).catch(() => {});
      } catch (e) { error = String(e); }
      finally { loading = false; }
      if (!active) return;
      const fns = await Promise.all([
        onTerminalTitle((p) => {
          const t = p.title;
          setTabTitle(p.session_id, /^[A-Za-z]:[\\\/]|^\//.test(t) ? shellBasename(t) : (t || p.title));
          // Title-based agent detection for agents that don't emit OSC 7777.
          const tab = get(tabs).find((tb) => tb.sessionId === p.session_id);
          if (!tab?.agentActive) {
            const clean = t.replace(/^\*\s*/, "").trim();
            const match = AGENT_TITLE_PATTERNS.find(({ pattern }) => pattern.test(clean));
            if (match) {
              const { name, executable } = match;
              queryAgentVersion(executable)
                .then((ver) => setTabAgent(p.session_id, name, ver ?? undefined))
                .catch(() => setTabAgent(p.session_id, name));
            }
          } else {
            // If the tab has a title-detected agent and the title no longer matches, clear it.
            // Keep agent info when it was set via OSC 7777 (pendingHellos tracks those).
            if (!pendingHellos.has(p.session_id)) {
              const clean = t.replace(/^\*\s*/, "").trim();
              if (!AGENT_TITLE_PATTERNS.some(({ pattern }) => pattern.test(clean))) {
                clearTabAgent(p.session_id);
              }
            }
          }
        }),
        onTerminalCwd((p) => setTabCwd(p.session_id, p.cwd)),
        onAgentHello((p) => { setTabAgent(p.session_id, p.name, p.version); pendingHellos = new Map(pendingHellos).set(p.session_id, { name: p.name, version: p.version }); }),
        onToolStart((p) => addToolCall(p.session_id, { id: p.id, name: p.name, summary: p.summary, icon: p.icon, status: "running", args: p.args })),
        onToolProgress((p) => updateToolCall(p.session_id, p.id, { progressPct: p.pct, progressMessage: p.message })),
        onToolEnd((p) => updateToolCall(p.session_id, p.id, { status: p.status as "ok" | "error" | "cancelled", progressPct: undefined, durationMs: p.duration_ms, ...(p.summary ? { summary: p.summary } : {}) })),
        onClipboardWrite((p) => navigator.clipboard.writeText(p.data).catch(() => {})),
        onTerminalBell(() => playBell()),
        onAgentDiff((p) => { pendingDiffs = new Map(pendingDiffs).set(p.session_id, p); }),
        onAgentApproval((p) => { pendingApprovals = new Map(pendingApprovals).set(p.session_id, p); }),
        onAgentBye((p) => { clearTabAgent(p.session_id); pendingHellos = new Map([...pendingHellos].filter(([k]) => k !== p.session_id)); }),
        onBlockStart((p) => addBlock(p.session_id, { id: p.id, command: p.command, startedAt: p.started_at, agentId: p.agent_id })),
        onBlockEnd((p) => closeBlock(p.session_id, p.id, p.exit_code, p.duration_ms)),
        onBlockAnnotate((p) => annotateBlock(p.session_id, p.id, { agentId: p.agent_id, tags: p.tags, fold: p.fold })),
        onProfilesUpdated(() => { profileCache = []; }),
        onConfigUpdated(() => {
          configGet().then((cfg: unknown) => {
            const c = cfg as { appearance?: { opacity?: number }; behavior?: { default_profile?: string } };
            if (typeof c?.appearance?.opacity === "number") applyOpacity(c.appearance.opacity);
            defaultProfileId = c?.behavior?.default_profile ?? "";
          }).catch(() => {});
        }),
      ]);
      if (!active) { fns.forEach((f) => f()); return; }
      cleanup.push(...fns);
    })();
    return () => { active = false; cleanup.forEach((f) => f()); };
  });

  function handleDiffAction(sessionId: string, id: string, result: string) {
    agentRespond(sessionId, "reply", JSON.stringify({ v: 1, id, result })).catch(() => {});
    setTimeout(() => { pendingDiffs = new Map([...pendingDiffs].filter(([k]) => k !== sessionId)); }, 2000);
  }
  function handleApprovalAction(sessionId: string, id: string, result: string) {
    agentRespond(sessionId, "reply", JSON.stringify({ v: 1, id, result })).catch(() => {});
    pendingApprovals = new Map([...pendingApprovals].filter(([k]) => k !== sessionId));
  }
  function dismissHello(sid: string) { pendingHellos = new Map([...pendingHellos].filter(([k]) => k !== sid)); }
  function cancelTool(sid: string, id: string) { agentRespond(sid, "event", JSON.stringify({ v: 1, kind: "tool.cancel", id })).catch(() => {}); }

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
    if (mod && e.key === "`") { e.preventDefault(); quakeToggle().catch(() => {}); return; }
    if (e.altKey && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      navigateBlocks(document.querySelector(".tab-pane:not(.hidden)"), e.key === "ArrowDown" ? 1 : -1);
    }
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

  async function openNewTab(profileId?: string) {
    try {
      if (!profileCache.length) { try { profileCache = await profileList(); } catch { /* ignore */ } }
      // Resolve which profile to display: explicit id > config default > first in list
      const resolvedId = profileId ?? defaultProfileId;
      const profile = resolvedId
        ? profileCache.find((p) => p.id === resolvedId) ?? profileCache[0]
        : profileCache[0];
      const label = profile?.name ?? shellBasename(profile?.shell ?? "Shell");
      const shell  = profile ? shellBasename(profile.shell) : "";
      const sessionId = await sessionCreate(resolvedId || undefined);
      createOutputBuffer(sessionId);
      addTab(sessionId, label || "Shell", shell);
    } catch (e) { error = `Failed to open tab: ${e}`; }
  }

  async function closeTab(tabId: string, sessionId: string) {
    const tab = $tabs.find((t) => t.id === tabId);
    if (tab?.splitSessionId) await sessionClose(tab.splitSessionId).catch(() => {});
    await sessionClose(sessionId).catch(() => {});
    removeTab(tabId);
    if ($tabs.length === 0) showWelcome = true;
  }

  async function splitActiveTab(direction: "horizontal" | "vertical") {
    const tab = $tabs.find((t) => t.isActive);
    if (!tab || tab.splitSessionId) return;
    try {
      const sessionId = await sessionCreate();
      createOutputBuffer(sessionId);
      setTabSplit(tab.id, sessionId, direction);
    } catch (e) { error = `Failed to split pane: ${e}`; }
  }

  async function closeSplitPane(tabId: string) {
    const tab = $tabs.find((t) => t.id === tabId);
    if (!tab?.splitSessionId) return;
    await sessionClose(tab.splitSessionId).catch(() => {});
    clearTabSplit(tabId);
  }

  // ── Testing affordance ───────────────────────────────────────────────────
  // Expose the active session ID on window so E2E tests can call session_write
  // directly (e.g. to inject OSC sequences).  Only runs in dev/debug builds.
  if (import.meta.env.DEV) {
    $effect(() => {
      (window as unknown as Record<string, unknown>)["__EMBER_ACTIVE_SESSION__"] =
        $activeSessionId;
    });
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

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
          blocks={$blocks.get(tab.sessionId) ?? []}
          splitBlocks={tab.splitSessionId ? ($blocks.get(tab.splitSessionId) ?? []) : []}
          toolCalls={$toolCalls.get(tab.sessionId) ?? []}
          pendingDiff={pendingDiffs.get(tab.sessionId)}
          pendingApproval={pendingApprovals.get(tab.sessionId)}
          pendingHello={pendingHellos.get(tab.sessionId)}
          onExit={() => closeTab(tab.id, tab.sessionId)}
          onSplitExit={() => closeSplitPane(tab.id)}
          onResize={tab.isActive ? (c, r) => { termCols = c; termRows = r; } : undefined}
          onCancelTool={(id) => cancelTool(tab.sessionId, id)}
          onDiffAction={(id, result) => handleDiffAction(tab.sessionId, id, result)}
          onApprovalAction={(id, result) => handleApprovalAction(tab.sessionId, id, result)}
          onDismissHello={() => dismissHello(tab.sessionId)}
          onCloseSplit={() => closeSplitPane(tab.id)}
        />
      {/each}
    {/if}
  </main>
  <StatusBar shell={$activeShell} cwd={$activeCwd} gitBranch={currentGitBranch} {gitDirty} {gitInsertions} {gitDeletions} agentName={$activeAgentName} agentVersion={$activeAgentVersion} {termCols} {termRows} />
  <CommandPalette open={paletteOpen} onClose={() => (paletteOpen = false)} onNewTab={openNewTab} onOpenSettings={() => (settingsOpen = true)} />
  <SettingsPanel open={settingsOpen} onClose={() => (settingsOpen = false)} onTestProfile={(id) => openNewTab(id)} />
  {#if showWelcome}
    <WelcomeScreen onComplete={async (profileId) => { showWelcome = false; profileCache = []; if ($tabs.length === 0) await openNewTab(profileId); }} />
  {/if}
</div>

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
