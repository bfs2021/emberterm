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

<header class="title-bar">
  <!--
    Drag region is a dedicated absolute overlay that sits BEHIND all interactive
    content (z-index 0). Buttons, tabs, and controls are z-index 1 and receive
    clicks normally. The overlay only activates for clicks on empty title-bar space.
    This is the correct Tauri 2 pattern — putting data-tauri-drag-region on the
    outer container causes it to swallow clicks on non-native-interactive children
    (div/span) before their own handlers fire.
  -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="drag-region"
    data-tauri-drag-region
    ondblclick={toggleMaximize}
  ></div>

  <!-- Branding -->
  <div class="title-brand">
    <svg class="brand-flame" viewBox="0 0 16 20" fill="none" aria-hidden="true">
      <path d="M8 0 C8 0, 14 5, 14 11 C14 15.4 11.3 18 8 18 C4.7 18 2 15.4 2 11 C2 7 6 3 6 3 C6 3 6 7 8 8 C8 8 10 6 8 0Z" fill="#FF6B35"/>
      <path d="M8 9 C8 9, 10 10.5 10 13 C10 14.7 9.1 16 8 16 C6.9 16 6 14.7 6 13 C6 11 8 9 8 9Z" fill="rgba(255,255,255,0.45)"/>
    </svg>
    <span class="brand-name">EMBER</span>
  </div>

  <!-- Tabs -->
  <div class="tab-area">
    <TabStrip {onNewTab} {onCloseTab} />
  </div>

  <!-- Window controls -->
  <div class="win-controls">
    <button class="win-btn" aria-label="Minimize" onclick={minimize}>
      <svg width="10" height="1" viewBox="0 0 10 1" aria-hidden="true">
        <line x1="0" y1="0.5" x2="10" y2="0.5" stroke="currentColor" stroke-width="1.5"/>
      </svg>
    </button>
    <button class="win-btn" aria-label="Maximize" onclick={toggleMaximize}>
      <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
        <rect x="0.75" y="0.75" width="8.5" height="8.5" fill="none" stroke="currentColor" stroke-width="1.5"/>
      </svg>
    </button>
    <button class="win-btn win-close" aria-label="Close" onclick={close}>
      <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
        <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" stroke-width="1.5"/>
        <line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" stroke-width="1.5"/>
      </svg>
    </button>
  </div>
</header>

<style>
  .title-bar {
    position: relative;
    display: flex;
    align-items: stretch;
    height: 36px;
    background: var(--bg-elev, #010409);
    border-bottom: 1px solid var(--border, #21262d);
    flex-shrink: 0;
    user-select: none;
  }

  /* Drag overlay: absolute, fills header, z-index 0 so everything else is above it */
  .drag-region {
    position: absolute;
    inset: 0;
    z-index: 0;
  }

  /* All interactive sections sit above the drag overlay */
  .title-brand,
  .tab-area,
  .win-controls {
    position: relative;
    z-index: 1;
  }

  /* ── Branding ─────────────────────────────── */
  .title-brand {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 14px 0 12px;
    flex-shrink: 0;
    border-right: 1px solid var(--border, #21262d);
  }

  .brand-flame {
    width: 12px;
    height: 15px;
    flex-shrink: 0;
  }

  .brand-name {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.14em;
    color: var(--fg-muted, #7d8590);
    text-transform: uppercase;
  }

  /* ── Tab area ─────────────────────────────── */
  .tab-area {
    flex: 1;
    min-width: 0;
    display: flex;
    overflow: hidden;
  }

  /* ── Window controls ──────────────────────── */
  .win-controls {
    display: flex;
    align-items: stretch;
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
    background: rgba(255, 255, 255, 0.08);
    color: var(--fg, #e6edf3);
  }

  .win-close:hover {
    background: #c42b1c;
    color: white;
  }
</style>
