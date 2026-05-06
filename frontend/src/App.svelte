<script lang="ts">
  import TabPane from "./terminal/TabPane.svelte";
  import TitleBar from "./chrome/TitleBar.svelte";
  import StatusBar from "./chrome/StatusBar.svelte";
  import CommandPalette from "./palette/CommandPalette.svelte";
  import SettingsPanel from "./settings/SettingsPanel.svelte";
  import WelcomeScreen from "./welcome/WelcomeScreen.svelte";
  import { playBell } from "./utils/bell.js";
  import { navigateBlocks } from "./utils/blockNav.js";
  import { sessionCreate, sessionClose, agentRespond, gitBranch as gitBranchIpc, configIsFirstRun, profileList, quakeToggle, type Profile } from "./ipc/client.js";
  import {
    onTerminalOutput, onTerminalTitle, onTerminalCwd, onTerminalBell,
    onAgentHello, onToolStart, onToolProgress, onToolEnd, onClipboardWrite,
    onProfilesUpdated, onConfigUpdated, onAgentDiff, onAgentApproval, onAgentBye,
    onBlockStart, onBlockEnd, onBlockAnnotate,
    type AgentDiffPayload, type AgentApprovalPayload,
  } from "./ipc/events.js";
  import {
    tabs, activeCwd, activeAgentName, activeShell,
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
  let showWelcome = $state(false);
  let termCols = $state(0);
  let termRows = $state(0);
  let profileCache: Profile[] = [];

  function shellBasename(p: string) { return (p.split(/[\\/]/).pop() ?? p).replace(/\.exe$/i, ""); }

  let pendingHellos = $state(new Map<string, { name: string; version: string }>());
  let pendingDiffs = $state(new Map<string, AgentDiffPayload>());
  let pendingApprovals = $state(new Map<string, AgentApprovalPayload>());

  $effect(() => {
    const cwd = $activeCwd;
    if (cwd) { gitBranchIpc(cwd).then((b) => { currentGitBranch = b ?? ""; }).catch(() => { currentGitBranch = ""; }); }
    else { currentGitBranch = ""; }
  });

  $effect(() => {
    let active = true;
    const cleanup: Array<() => void> = [];
    (async () => {
      cleanup.push(await onTerminalOutput((p) => appendOutputBuffer(p.session_id, new Uint8Array(p.data))));
      try { showWelcome = await configIsFirstRun(); await openNewTab(); }
      catch (e) { error = String(e); }
      finally { loading = false; }
      if (!active) return;
      const fns = await Promise.all([
        onTerminalTitle((p) => { const t = p.title; setTabTitle(p.session_id, /^[A-Za-z]:[\\\/]|^\//.test(t) ? shellBasename(t) : (t || p.title)); }),
        onTerminalCwd((p) => setTabCwd(p.session_id, p.cwd)),
        onAgentHello((p) => { setTabAgent(p.session_id, p.name); pendingHellos = new Map(pendingHellos).set(p.session_id, { name: p.name, version: p.version }); }),
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
        onProfilesUpdated(() => {}),
        onConfigUpdated(() => {}),
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
      const profile = profileId ? profileCache.find((p) => p.id === profileId) : profileCache[0];
      const shell = profile ? shellBasename(profile.shell) : "";
      const sessionId = await sessionCreate(profileId);
      createOutputBuffer(sessionId);
      addTab(sessionId, shell || "Shell", shell);
    } catch (e) { error = `Failed to open tab: ${e}`; }
  }

  async function closeTab(tabId: string, sessionId: string) {
    const tab = $tabs.find((t) => t.id === tabId);
    if (tab?.splitSessionId) await sessionClose(tab.splitSessionId).catch(() => {});
    await sessionClose(sessionId).catch(() => {});
    removeTab(tabId);
    if ($tabs.length === 0) await openNewTab();
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
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="app">
  <TitleBar onNewTab={openNewTab} onCloseTab={closeTab} />
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
  <StatusBar shell={$activeShell} cwd={$activeCwd} gitBranch={currentGitBranch} agentName={$activeAgentName} {termCols} {termRows} />
  <CommandPalette open={paletteOpen} onClose={() => (paletteOpen = false)} onNewTab={openNewTab} onOpenSettings={() => (settingsOpen = true)} />
  <SettingsPanel open={settingsOpen} onClose={() => (settingsOpen = false)} />
  {#if showWelcome}
    <WelcomeScreen onComplete={() => (showWelcome = false)} />
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background: var(--bg, #0d1117);
    color: var(--fg, #e6edf3);
    font-family: system-ui, sans-serif;
    overflow: hidden;
  }
  .terminal-area { flex: 1; min-height: 0; position: relative; }
  .splash { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--fg-muted, #7d8590); font-size: 14px; }
  .splash.error { color: var(--red, #f85149); }
</style>
