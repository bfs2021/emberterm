<script lang="ts">
  import { onMount } from "svelte";
  import Grid from "./terminal/Grid.svelte";
  import { sessionCreate, sessionClose } from "./ipc/client";
  import {
    tabs,
    activeTabId,
    addTab,
    removeTab,
    activateTab,
    activeSessionId,
  } from "./stores/app";

  // ── State ────────────────────────────────────────────────────────────────

  let loading = $state(true);
  let error = $state<string | null>(null);

  // ── Bootstrap ────────────────────────────────────────────────────────────

  onMount(async () => {
    try {
      await openNewTab();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  // ── Tab management ───────────────────────────────────────────────────────

  async function openNewTab() {
    const sessionId = await sessionCreate();
    addTab(sessionId, "Shell");
  }

  async function closeTab(tabId: string, sessionId: string) {
    await sessionClose(sessionId).catch(() => {});
    removeTab(tabId);
    if ($tabs.length === 0) {
      // Re-open a tab so the window is never empty.
      await openNewTab();
    }
  }

  function handleTabExit(tabId: string, sessionId: string) {
    closeTab(tabId, sessionId);
  }
</script>

<div class="app">
  <!-- Title bar (custom, Phase 1 component goes here) -->
  <header class="title-bar" data-tauri-drag-region>
    <div class="tab-strip" data-tauri-drag-region>
      {#each $tabs as tab (tab.id)}
        <div
          class="tab"
          class:active={tab.isActive}
          role="tab"
          tabindex="0"
          aria-selected={tab.isActive}
          onclick={() => activateTab(tab.id)}
          onkeydown={(e) => e.key === "Enter" && activateTab(tab.id)}
        >
          <span class="tab-title">{tab.title}</span>
          <span
            class="tab-close"
            role="button"
            tabindex="0"
            aria-label="Close tab"
            onclick={(e) => { e.stopPropagation(); closeTab(tab.id, tab.sessionId); }}
            onkeydown={(e) => e.key === "Enter" && (e.stopPropagation(), closeTab(tab.id, tab.sessionId))}
          >×</span>
        </div>
      {/each}
      <button class="tab-new" aria-label="New tab" onclick={openNewTab}>+</button>
    </div>
    <!-- Window controls live here in Phase 1 (TitleBar.svelte) -->
  </header>

  <!-- Terminal grid area -->
  <main class="terminal-area">
    {#if loading}
      <div class="splash">Starting shell…</div>
    {:else if error}
      <div class="splash error">{error}</div>
    {:else}
      {#each $tabs as tab (tab.id)}
        <Grid
          sessionId={tab.sessionId}
          active={tab.isActive}
          onExit={(code) => handleTabExit(tab.id, tab.sessionId)}
        />
      {/each}
    {/if}
  </main>

  <!-- Status bar (Phase 1 component goes here) -->
  <footer class="status-bar">
    <span class="status-cwd">{$activeSessionId ? "~" : ""}</span>
  </footer>
</div>

<style>
  /* ── Layout ──────────────────────────────────────────────────────────── */
  .app {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background: #0d1117;
    color: #e6edf3;
    font-family: system-ui, sans-serif;
    overflow: hidden;
  }

  /* ── Title bar ───────────────────────────────────────────────────────── */
  .title-bar {
    display: flex;
    align-items: stretch;
    height: 36px;
    background: #010409;
    border-bottom: 1px solid #21262d;
    flex-shrink: 0;
    /* Allow Tauri window drag on the title bar background. */
    -webkit-app-region: drag;
  }

  .tab-strip {
    display: flex;
    align-items: stretch;
    gap: 0;
    flex: 1;
    min-width: 0;
    -webkit-app-region: no-drag;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 12px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: #7d8590;
    cursor: pointer;
    white-space: nowrap;
    font-size: 12px;
    transition: color 0.1s, border-color 0.1s;
  }
  .tab:hover { color: #e6edf3; }
  .tab.active {
    color: #e6edf3;
    border-bottom-color: #FF6B35;
    background: #0d1117;
  }

  .tab-title { pointer-events: none; }

  .tab-close {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0 2px;
    opacity: 0.5;
    transition: opacity 0.1s;
    user-select: none;
  }
  .tab-close:hover { opacity: 1; }

  .tab-new {
    background: transparent;
    border: none;
    color: #7d8590;
    cursor: pointer;
    font-size: 18px;
    padding: 0 10px;
    line-height: 36px;
    transition: color 0.1s;
  }
  .tab-new:hover { color: #e6edf3; }

  /* ── Terminal area ───────────────────────────────────────────────────── */
  .terminal-area {
    flex: 1;
    min-height: 0;
    position: relative;
  }

  .splash {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #7d8590;
    font-size: 14px;
  }
  .splash.error { color: #f85149; }

  /* ── Status bar ──────────────────────────────────────────────────────── */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 24px;
    padding: 0 12px;
    background: #010409;
    border-top: 1px solid #21262d;
    font-size: 11px;
    color: #7d8590;
    flex-shrink: 0;
  }
</style>
