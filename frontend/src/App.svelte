<script lang="ts">
  import Grid from "./terminal/Grid.svelte";
  import TitleBar from "./chrome/TitleBar.svelte";
  import StatusBar from "./chrome/StatusBar.svelte";
  import ToolCard from "./agent/ToolCard.svelte";
  import CommandPalette from "./palette/CommandPalette.svelte";
  import SettingsPanel from "./settings/SettingsPanel.svelte";
  import { sessionCreate, sessionClose, agentRespond, gitBranch as gitBranchIpc, configIsFirstRun } from "./ipc/client.js";
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
  } from "./ipc/events.js";
  import WelcomeScreen from "./welcome/WelcomeScreen.svelte";
  import {
    tabs,
    activeCwd,
    activeAgentName,
    addTab,
    removeTab,
    activateTab,
    setTabCwd,
    setTabTitle,
    setTabAgent,
    addToolCall,
    updateToolCall,
    toolCalls,
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
        onAgentHello((p) => setTabAgent(p.session_id, p.name)),
        onToolStart((p) => {
          addToolCall(p.session_id, {
            id: p.id,
            name: p.name,
            summary: p.summary,
            icon: p.icon,
            status: "running",
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
        // OSC 52 clipboard write — termwiz already decoded base64; content is plain text
        onClipboardWrite((p) => {
          navigator.clipboard.writeText(p.data).catch(() => {});
        }),
        onTerminalBell(() => playBell()),
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

  async function openNewTab() {
    const sessionId = await sessionCreate();
    addTab(sessionId, "Shell");
  }

  async function closeTab(tabId: string, sessionId: string) {
    await sessionClose(sessionId).catch(() => {});
    removeTab(tabId);
    if ($tabs.length === 0) await openNewTab();
  }

  // ── Agent actions ────────────────────────────────────────────────────────

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
          <!-- Tool cards for this session -->
          {#if tab.isActive}
            {#each ($toolCalls.get(tab.sessionId) ?? []).slice(-5) as call (call.id)}
              <div class="tool-overlay">
                <ToolCard
                  id={call.id}
                  name={call.name}
                  summary={call.summary}
                  icon={call.icon}
                  status={call.status}
                  progressPct={call.progressPct}
                  progressMessage={call.progressMessage}
                  durationMs={call.durationMs}
                  onCancel={call.status === "running"
                    ? () => cancelTool(tab.sessionId, call.id)
                    : undefined}
                />
              </div>
            {/each}
          {/if}

          <Grid
            sessionId={tab.sessionId}
            active={tab.isActive}
            onExit={() => closeTab(tab.id, tab.sessionId)}
          />
        </div>
      {/each}
    {/if}
  </main>

  <StatusBar
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

  .tool-overlay {
    position: absolute;
    bottom: 8px;
    left: 8px;
    right: 8px;
    z-index: 10;
    pointer-events: none;
  }

  .tool-overlay > :global(*) { pointer-events: all; }

  .splash {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--fg-muted, #7d8590);
    font-size: 14px;
  }

  .splash.error { color: var(--danger, #f85149); }
</style>
