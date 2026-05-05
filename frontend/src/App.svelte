<script lang="ts">
  import Grid from "./terminal/Grid.svelte";
  import TitleBar from "./chrome/TitleBar.svelte";
  import StatusBar from "./chrome/StatusBar.svelte";
  import HelloBanner from "./chrome/HelloBanner.svelte";
  import ToolCard from "./agent/ToolCard.svelte";
  import DiffView from "./agent/DiffView.svelte";
  import ApprovalSheet from "./agent/ApprovalSheet.svelte";
  import CommandPalette from "./palette/CommandPalette.svelte";
  import SettingsPanel from "./settings/SettingsPanel.svelte";
  import WelcomeScreen from "./welcome/WelcomeScreen.svelte";
  import { sessionCreate, sessionClose, agentRespond, gitBranch as gitBranchIpc, configIsFirstRun, profileList, type Profile } from "./ipc/client.js";
  import {
    onTerminalTitle,
    onTerminalCwd,
    onTerminalBell,
    onAgentHello,
    onToolStart,
    onToolProgress,
    onToolEnd,
    onClipboardWrite,
    onProfilesUpdated,
    onConfigUpdated,
    onAgentDiff,
    onAgentApproval,
    onBlockStart,
    onBlockEnd,
    type AgentDiffPayload,
    type AgentApprovalPayload,
  } from "./ipc/events.js";
  import {
    tabs,
    activeCwd,
    activeAgentName,
    activeShell,
    addTab,
    removeTab,
    activateTab,
    setTabCwd,
    setTabTitle,
    setTabAgent,
    addToolCall,
    updateToolCall,
    toolCalls,
    addBlock,
    closeBlock,
  } from "./stores/app.js";
  // Initialize the theme system on import — applies ember-dark CSS vars.
  import "./theme/store.js";

  // ── State ────────────────────────────────────────────────────────────────

  let loading = $state(true);
  let error = $state<string | null>(null);
  let paletteOpen = $state(false);
  let settingsOpen = $state(false);
  let currentGitBranch = $state("");
  let showWelcome = $state(false);

  // Profile cache — populated lazily on first tab open
  let profileCache: Profile[] = [];

  function shellBasename(path: string): string {
    return (path.split(/[\\/]/).pop() ?? path).replace(/\.exe$/i, "");
  }

  // Per-session pending agent UI (Map keyed by sessionId)
  let pendingHellos = $state(new Map<string, { name: string; version: string }>());
  let pendingDiffs = $state(new Map<string, AgentDiffPayload>());
  let pendingApprovals = $state(new Map<string, AgentApprovalPayload>());

  // ── Git branch — refresh when cwd changes ────────────────────────────────

  $effect(() => {
    const cwd = $activeCwd;
    if (cwd) {
      gitBranchIpc(cwd).then((b) => { currentGitBranch = b ?? ""; }).catch(() => {
        currentGitBranch = "";
      });
    } else {
      currentGitBranch = "";
    }
  });

  // ── Bell ─────────────────────────────────────────────────────────────────

  function playBell() {
    try {
      const ctx = new AudioContext();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.frequency.value = 880;
      gain.gain.setValueAtTime(0.08, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.15);
      osc.start(ctx.currentTime);
      osc.stop(ctx.currentTime + 0.15);
    } catch {
      // AudioContext unavailable — silently ignore
    }
  }

  // ── IPC listeners ────────────────────────────────────────────────────────

  $effect(() => {
    let active = true;
    const cleanup: Array<() => void> = [];

    (async () => {
      try {
        showWelcome = await configIsFirstRun();
        await openNewTab();
      } catch (e) {
        error = String(e);
      } finally {
        loading = false;
      }

      if (!active) return;

      const fns = await Promise.all([
        onTerminalTitle((p) => setTabTitle(p.session_id, p.title)),
        onTerminalCwd((p) => setTabCwd(p.session_id, p.cwd)),
        onAgentHello((p) => {
          setTabAgent(p.session_id, p.name);
          pendingHellos = new Map(pendingHellos).set(p.session_id, {
            name: p.name,
            version: p.version,
          });
        }),
        onToolStart((p) => {
          addToolCall(p.session_id, {
            id: p.id,
            name: p.name,
            summary: p.summary,
            icon: p.icon,
            status: "running",
            args: p.args,
          });
        }),
        onToolProgress((p) => {
          updateToolCall(p.session_id, p.id, {
            progressPct: p.pct,
            progressMessage: p.message,
          });
        }),
        onToolEnd((p) => {
          updateToolCall(p.session_id, p.id, {
            status: p.status as "ok" | "error" | "cancelled",
            progressPct: undefined,
            durationMs: p.duration_ms,
            ...(p.summary ? { summary: p.summary } : {}),
          });
        }),
        onClipboardWrite((p) => {
          navigator.clipboard.writeText(p.data).catch(() => {});
        }),
        onTerminalBell(() => playBell()),
        onAgentDiff((p) => {
          pendingDiffs = new Map(pendingDiffs).set(p.session_id, p);
        }),
        onAgentApproval((p) => {
          pendingApprovals = new Map(pendingApprovals).set(p.session_id, p);
        }),
        onBlockStart((p) => {
          addBlock(p.session_id, {
            id: p.id,
            command: p.command,
            startedAt: p.started_at,
            agentId: p.agent_id,
          });
        }),
        onBlockEnd((p) => {
          closeBlock(p.session_id, p.id, p.exit_code, p.duration_ms);
        }),
        onProfilesUpdated(() => {}),
        onConfigUpdated(() => {}),
      ]);

      if (!active) {
        fns.forEach((f) => f());
        return;
      }
      cleanup.push(...fns);
    })();

    return () => {
      active = false;
      cleanup.forEach((f) => f());
    };
  });

  // ── Agent action helpers ─────────────────────────────────────────────────

  function handleDiffAction(sessionId: string, id: string, result: string) {
    agentRespond(sessionId, "reply", JSON.stringify({ v: 1, id, result })).catch(() => {});
    pendingDiffs = new Map([...pendingDiffs].filter(([k]) => k !== sessionId));
  }

  function handleApprovalAction(sessionId: string, id: string, result: string) {
    agentRespond(sessionId, "reply", JSON.stringify({ v: 1, id, result })).catch(() => {});
    pendingApprovals = new Map([...pendingApprovals].filter(([k]) => k !== sessionId));
  }

  function dismissHello(sessionId: string) {
    pendingHellos = new Map([...pendingHellos].filter(([k]) => k !== sessionId));
  }

  // ── Keyboard shortcuts ───────────────────────────────────────────────────

  function handleWindowKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.shiftKey && e.key === "P") { e.preventDefault(); paletteOpen = !paletteOpen; return; }
    if (mod && e.shiftKey && e.key === "T") { e.preventDefault(); openNewTab(); return; }
    if (mod && e.shiftKey && e.key === "W") { e.preventDefault(); closeActiveTab(); return; }
    if (mod && e.key === ",") { e.preventDefault(); settingsOpen = true; return; }
    if (mod && e.key === "Tab") { e.preventDefault(); cycleTab(1); return; }
    if (mod && e.shiftKey && e.key === "Tab") { e.preventDefault(); cycleTab(-1); return; }
  }

  function cycleTab(dir: 1 | -1) {
    const ts = $tabs;
    if (ts.length < 2) return;
    const idx = ts.findIndex((t) => t.isActive);
    const next = (idx + dir + ts.length) % ts.length;
    activateTab(ts[next].id);
  }

  async function closeActiveTab() {
    const tab = $tabs.find((t) => t.isActive);
    if (tab) await closeTab(tab.id, tab.sessionId);
  }

  // ── Tab management ───────────────────────────────────────────────────────

  async function openNewTab(profileId?: string) {
    if (!profileCache.length) {
      try { profileCache = await profileList(); } catch { /* ignore */ }
    }
    const profile = profileId
      ? profileCache.find((p) => p.id === profileId)
      : profileCache[0];
    const shell = profile ? shellBasename(profile.shell) : "";

    const sessionId = await sessionCreate(profileId);
    addTab(sessionId, "Shell", shell);
  }

  async function closeTab(tabId: string, sessionId: string) {
    await sessionClose(sessionId).catch(() => {});
    removeTab(tabId);
    if ($tabs.length === 0) await openNewTab();
  }

  // ── Tool cancel ──────────────────────────────────────────────────────────

  function cancelTool(sessionId: string, toolId: string) {
    const payload = JSON.stringify({ v: 1, kind: "tool.cancel", id: toolId });
    agentRespond(sessionId, "event", payload).catch(() => {});
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
        <div class="tab-pane" class:hidden={!tab.isActive}>
          <Grid
            sessionId={tab.sessionId}
            active={tab.isActive}
            onExit={() => closeTab(tab.id, tab.sessionId)}
          />

          {#if tab.isActive}
            <!-- Tool cards -->
            {#each ($toolCalls.get(tab.sessionId) ?? []).slice(-5) as call (call.id)}
              <div class="agent-overlay agent-overlay--cards">
                <ToolCard
                  id={call.id}
                  name={call.name}
                  summary={call.summary}
                  icon={call.icon}
                  status={call.status}
                  progressPct={call.progressPct}
                  progressMessage={call.progressMessage}
                  durationMs={call.durationMs}
                  args={call.args}
                  onCancel={call.status === "running"
                    ? () => cancelTool(tab.sessionId, call.id)
                    : undefined}
                />
              </div>
            {/each}

            <!-- Diff viewer -->
            {@const diff = pendingDiffs.get(tab.sessionId)}
            {#if diff}
              <div class="agent-overlay agent-overlay--diff">
                <DiffView
                  id={diff.id}
                  path={diff.path}
                  patch={diff.patch}
                  actions={diff.actions}
                  onAction={(id, result) => handleDiffAction(tab.sessionId, id, result)}
                />
              </div>
            {/if}

            <!-- Approval sheet -->
            {@const approval = pendingApprovals.get(tab.sessionId)}
            {#if approval}
              <ApprovalSheet
                id={approval.id}
                title={approval.title}
                detail={approval.detail}
                risk={approval.risk}
                actions={approval.actions}
                onAction={(id, result) => handleApprovalAction(tab.sessionId, id, result)}
              />
            {/if}

            <!-- Agent hello banner -->
            {@const hello = pendingHellos.get(tab.sessionId)}
            {#if hello}
              <HelloBanner
                name={hello.name}
                version={hello.version}
                onDismiss={() => dismissHello(tab.sessionId)}
              />
            {/if}
          {/if}
        </div>
      {/each}
    {/if}
  </main>

  <StatusBar
    shell={$activeShell}
    cwd={$activeCwd}
    gitBranch={currentGitBranch}
    agentName={$activeAgentName}
  />

  <CommandPalette
    open={paletteOpen}
    onClose={() => (paletteOpen = false)}
    onNewTab={openNewTab}
    onOpenSettings={() => (settingsOpen = true)}
  />

  <SettingsPanel
    open={settingsOpen}
    onClose={() => (settingsOpen = false)}
  />

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

  .terminal-area {
    flex: 1;
    min-height: 0;
    position: relative;
  }

  .tab-pane {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
  }

  .tab-pane.hidden { display: none; }

  /* Shared overlay base — all agent UI floats above the terminal */
  .agent-overlay {
    position: absolute;
    left: 8px;
    right: 8px;
    z-index: 10;
    pointer-events: none;
  }

  .agent-overlay > :global(*) { pointer-events: all; }

  /* Tool cards: bottom-right stack */
  .agent-overlay--cards {
    bottom: 8px;
  }

  /* Diff viewer: above the tool card area */
  .agent-overlay--diff {
    bottom: 80px;
    max-width: 680px;
  }

  .splash {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--fg-muted, #7d8590);
    font-size: 14px;
  }

  .splash.error { color: var(--red, #f85149); }
</style>
