<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TabStrip from "./TabStrip.svelte";

  interface Props {
    onNewTab: () => void;
    onCloseTab: (tabId: string, sessionId: string) => void;
  }

  let { onNewTab, onCloseTab }: Props = $props();

  const win = getCurrentWindow();

  async function minimize() { await win.minimize(); }
  async function toggleMaximize() { await win.toggleMaximize(); }
  async function close() { await win.close(); }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header class="title-bar" data-tauri-drag-region ondblclick={toggleMaximize}>
  <TabStrip {onNewTab} {onCloseTab} />

  <!-- Window controls -->
  <div class="win-controls">
    <button class="win-btn" aria-label="Minimize" onclick={minimize} title="Minimize">
      <svg width="10" height="1" viewBox="0 0 10 1"><line x1="0" y1="0.5" x2="10" y2="0.5" stroke="currentColor" stroke-width="1.5"/></svg>
    </button>
    <button class="win-btn" aria-label="Maximize" onclick={toggleMaximize} title="Maximize">
      <svg width="10" height="10" viewBox="0 0 10 10"><rect x="0.75" y="0.75" width="8.5" height="8.5" fill="none" stroke="currentColor" stroke-width="1.5"/></svg>
    </button>
    <button class="win-btn close-btn" aria-label="Close" onclick={close} title="Close">
      <svg width="10" height="10" viewBox="0 0 10 10"><line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" stroke-width="1.5"/><line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" stroke-width="1.5"/></svg>
    </button>
  </div>
</header>

<style>
  .title-bar {
    display: flex;
    align-items: stretch;
    height: 36px;
    background: var(--bg-elevated, #010409);
    border-bottom: 1px solid var(--border, #21262d);
    flex-shrink: 0;
    -webkit-app-region: drag;
  }

  .win-controls {
    display: flex;
    align-items: stretch;
    -webkit-app-region: no-drag;
    flex-shrink: 0;
  }

  .win-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    background: transparent;
    border: none;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .win-btn:hover {
    background: rgba(255, 255, 255, 0.07);
    color: var(--fg, #e6edf3);
  }

  .close-btn:hover {
    background: var(--win-close-hover, #c42b1c);
    color: white;
  }
</style>
