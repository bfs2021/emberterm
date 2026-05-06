<script lang="ts">
  import { tabs, activateTab, renameTab, reorderTab, type Tab } from "../stores/app.js";

  interface Props {
    onNewTab: () => void;
    onCloseTab: (tabId: string, sessionId: string) => void;
  }

  let { onNewTab, onCloseTab }: Props = $props();

  // Inline rename state
  let renamingTabId = $state<string | null>(null);
  let renameValue = $state("");

  // Shell icon type
  type IconKey = "pwsh" | "bash" | "agent" | "default";

  function shellIcon(tab: Tab): IconKey {
    if (tab.agentActive) return "agent";
    const s = (tab.shell ?? "").toLowerCase();
    if (s.includes("pwsh") || s.includes("powershell")) return "pwsh";
    if (s.includes("bash") || s.includes("zsh") || s.includes("sh")) return "bash";
    return "default";
  }

  // Inline SVG strings for tab shell icons (12×12 viewBox="0 0 24 24")
  const ICONS: Record<IconKey, string> = {
    pwsh: `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><polyline points="4 17 10 11 4 5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><line x1="12" y1="19" x2="20" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>`,
    bash: `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="4" width="18" height="16" rx="2" stroke="currentColor" stroke-width="2"/><path d="M7 9l3 3-3 3M13 15h4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    agent: `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 2.5 C 13 6, 16 7, 16.5 11 C 17 14.5, 14.8 17, 12 17 C 9.2 17, 7 14.5, 7.5 11 C 8 7.5, 12 7, 12 2.5 Z" fill="currentColor" stroke="none"/></svg>`,
    default: `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><polyline points="4 17 10 11 4 5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><line x1="12" y1="19" x2="20" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>`,
  };

  function startRename(tab: Tab, e: MouseEvent) {
    e.stopPropagation();
    renamingTabId = tab.id;
    renameValue = tab.title;
  }

  function commitRename() {
    if (renamingTabId) {
      renameTab(renamingTabId, renameValue);
      renamingTabId = null;
    }
  }

  function cancelRename() {
    renamingTabId = null;
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); commitRename(); }
    if (e.key === "Escape") { e.preventDefault(); cancelRename(); }
  }

  function handleRenameMount(node: HTMLInputElement) {
    node.select();
  }

  function handleTabKeydown(e: KeyboardEvent, tab: Tab) {
    if (e.key === "Enter" || e.key === " ") activateTab(tab.id);
  }

  function handleCloseKeydown(e: KeyboardEvent, tab: Tab) {
    if (e.key === "Enter") {
      e.stopPropagation();
      onCloseTab(tab.id, tab.sessionId);
    }
  }

  // ── Drag-to-reorder ────────────────────────────────────────────────────────
  let dragTabId = $state<string | null>(null);
  let dragOverId = $state<string | null>(null);

  function onDragStart(e: DragEvent, tabId: string) {
    dragTabId = tabId;
    e.dataTransfer!.effectAllowed = "move";
  }
  function onDragOver(e: DragEvent, tabId: string) {
    e.preventDefault();
    if (dragTabId && dragTabId !== tabId) dragOverId = tabId;
  }
  function onDragEnd() { dragTabId = null; dragOverId = null; }
  function onDrop(e: DragEvent, targetId: string) {
    e.preventDefault();
    if (dragTabId && dragTabId !== targetId) reorderTab(dragTabId, targetId);
    dragTabId = null;
    dragOverId = null;
  }

  // Dot visibility: agent > unread (inactive) > running (inactive)
  function showAgentDot(tab: Tab) { return tab.agentActive; }
  function showUnreadDot(tab: Tab) { return tab.hasUnread && !tab.isActive && !tab.agentActive; }
  function showRunningDot(tab: Tab) { return tab.isRunning && !tab.isActive && !tab.agentActive && !tab.hasUnread; }
</script>

<div class="tab-strip">
  {#each $tabs as tab (tab.id)}
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="tab"
      class:active={tab.isActive}
      class:drag-over={dragOverId === tab.id}
      role="tab"
      tabindex="0"
      aria-selected={tab.isActive}
      draggable="true"
      onclick={() => activateTab(tab.id)}
      onkeydown={(e) => handleTabKeydown(e, tab)}
      onmousedown={(e) => e.button === 1 && onCloseTab(tab.id, tab.sessionId)}
      ondblclick={(e) => startRename(tab, e)}
      ondragstart={(e) => onDragStart(e, tab.id)}
      ondragover={(e) => onDragOver(e, tab.id)}
      ondrop={(e) => onDrop(e, tab.id)}
      ondragend={onDragEnd}
    >
      <!-- Top accent bar for active tab (rendered via ::after in CSS) -->

      <!-- Shell icon -->
      <span
        class="tab-icon"
        class:icon-active={tab.isActive}
        aria-hidden="true"
      >
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html ICONS[shellIcon(tab)]}
      </span>

      <!-- Title or rename input -->
      {#if renamingTabId === tab.id}
        <!-- svelte-ignore a11y_autofocus -->
        <input
          class="tab-rename-input"
          type="text"
          bind:value={renameValue}
          onblur={commitRename}
          onkeydown={handleRenameKeydown}
          onclick={(e) => e.stopPropagation()}
          use:handleRenameMount
          autofocus
        />
      {:else}
        <span class="tab-title">{tab.title}</span>
      {/if}

      <!-- Status dot -->
      {#if showAgentDot(tab)}
        <span class="tab-dot agent-dot" aria-hidden="true"></span>
      {:else if showUnreadDot(tab)}
        <span class="tab-dot unread-dot" aria-hidden="true"></span>
      {:else if showRunningDot(tab)}
        <span class="tab-dot running-dot" aria-hidden="true"></span>
      {/if}

      <!-- Close button -->
      <span
        class="tab-close"
        role="button"
        tabindex="0"
        aria-label="Close tab"
        onclick={(e) => { e.stopPropagation(); onCloseTab(tab.id, tab.sessionId); }}
        onkeydown={(e) => handleCloseKeydown(e, tab)}
      >
        <svg width="10" height="10" viewBox="0 0 12 12" aria-hidden="true">
          <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
        </svg>
      </span>
    </div>
  {/each}

  <button class="tab-add" aria-label="New tab" onclick={() => onNewTab()}>
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <line x1="7" y1="2" x2="7" y2="12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
      <line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
    </svg>
  </button>
</div>

<style>
  .tab-strip {
    display: flex;
    align-items: flex-end;
    flex: 1;
    min-width: 0;
    padding-top: 6px;
    overflow: hidden;
    -webkit-app-region: no-drag;
    height: 100%;
  }

  .tab {
    position: relative;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 0 8px 0 10px;
    height: 30px;
    background: transparent;
    border: 1px solid transparent;
    border-bottom: none;
    border-radius: 8px 8px 0 0;
    color: var(--tab-inactive-fg, #7d8590);
    cursor: pointer;
    white-space: nowrap;
    font-size: 12px;
    transition: color 0.1s, background 0.1s, border-color 0.1s;
    min-width: 120px;
    max-width: 220px;
    flex-shrink: 1;
    box-sizing: border-box;
  }

  .tab:hover {
    background: rgba(255, 255, 255, 0.03);
    color: var(--tab-active-fg, #e6edf3);
  }

  .tab.active {
    background: var(--tab-active-bg, #0d1117);
    border-color: var(--border, #21262d);
    color: var(--tab-active-fg, #e6edf3);
  }

  .tab.drag-over {
    border-left: 2px solid var(--accent, #FF6B35);
  }

  /* Top accent line for active tab */
  .tab.active::after {
    content: "";
    position: absolute;
    top: 0;
    left: 10px;
    right: 10px;
    height: 2px;
    background: var(--accent, #FF6B35);
    border-radius: 0 0 2px 2px;
    box-shadow: 0 0 8px var(--accent-glow, rgba(255, 107, 53, 0.35));
  }

  /* ── Shell icon ───────────────────────────── */
  .tab-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    color: var(--fg-dim, #484f58);
    transition: color 0.1s;
  }

  .tab-icon.icon-active {
    color: var(--fg-muted, #7d8590);
  }

  .tab:hover .tab-icon {
    color: var(--fg-muted, #7d8590);
  }

  /* ── Title ────────────────────────────────── */
  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
    flex: 1;
    min-width: 0;
  }

  /* ── Rename input ─────────────────────────── */
  .tab-rename-input {
    background: var(--bg, #0d1117);
    border: 1px solid var(--accent, #FF6B35);
    border-radius: 3px;
    color: var(--fg, #e6edf3);
    font-size: 12px;
    font-family: inherit;
    height: 20px;
    padding: 0 4px;
    flex: 1;
    min-width: 40px;
    max-width: 160px;
    outline: none;
  }

  /* ── Status dots ──────────────────────────── */
  .tab-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .agent-dot {
    background: var(--accent, #FF6B35);
    box-shadow: 0 0 6px var(--accent-glow, rgba(255, 107, 53, 0.35));
  }

  .unread-dot {
    background: var(--blue, #58a6ff);
  }

  .running-dot {
    background: var(--green, #7ee787);
    animation: running-pulse 1.4s ease-in-out infinite;
  }

  @keyframes running-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.8); }
  }

  /* ── Close button ─────────────────────────── */
  .tab-close {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border-radius: 4px;
    cursor: pointer;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.1s, background 0.1s;
    user-select: none;
    color: inherit;
  }

  .tab:hover .tab-close,
  .tab.active .tab-close {
    opacity: 1;
  }

  .tab-close:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  /* ── New-tab button ──────────────────────── */
  .tab-add {
    align-self: flex-end;
    width: 28px;
    height: 28px;
    margin: 0 0 1px 2px;
    display: grid;
    place-items: center;
    border-radius: 6px;
    background: transparent;
    border: none;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s;
    pointer-events: auto;
    -webkit-app-region: no-drag;
  }

  .tab-add:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--fg, #e6edf3);
  }
</style>
