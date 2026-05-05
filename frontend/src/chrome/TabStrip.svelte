<script lang="ts">
  import { tabs, activateTab, renameTab, type Tab } from "../stores/app.js";

  interface Props {
    onNewTab: () => void;
    onCloseTab: (tabId: string, sessionId: string) => void;
  }

  let { onNewTab, onCloseTab }: Props = $props();

  // Inline rename state
  let renamingTabId = $state<string | null>(null);
  let renameValue = $state("");

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
</script>

<div class="tab-strip" data-tauri-drag-region>
  {#each $tabs as tab (tab.id)}
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="tab"
      class:active={tab.isActive}
      class:unread={tab.hasUnread && !tab.isActive}
      class:agent={tab.agentActive}
      role="tab"
      tabindex="0"
      aria-selected={tab.isActive}
      onclick={() => activateTab(tab.id)}
      onkeydown={(e) => handleTabKeydown(e, tab)}
      onmousedown={(e) => e.button === 1 && onCloseTab(tab.id, tab.sessionId)}
      ondblclick={(e) => startRename(tab, e)}
    >
      {#if tab.agentActive}
        <span class="tab-dot agent-dot" aria-hidden="true"></span>
      {:else if tab.isRunning}
        <span class="tab-dot running-dot" aria-hidden="true"></span>
      {/if}

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

      <span
        class="tab-close"
        role="button"
        tabindex="0"
        aria-label="Close tab"
        onclick={(e) => { e.stopPropagation(); onCloseTab(tab.id, tab.sessionId); }}
        onkeydown={(e) => handleCloseKeydown(e, tab)}
      >×</span>
    </div>
  {/each}
  <button class="tab-new" aria-label="New tab" onclick={onNewTab}>+</button>
</div>

<style>
  .tab-strip {
    display: flex;
    align-items: stretch;
    flex: 1;
    min-width: 0;
    -webkit-app-region: no-drag;
    overflow: hidden;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 0 10px 0 12px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--tab-inactive-fg, #7d8590);
    cursor: pointer;
    white-space: nowrap;
    font-size: 12px;
    transition: color 0.1s, border-color 0.1s, background 0.1s;
    min-width: 0;
    max-width: 200px;
    flex-shrink: 1;
  }

  .tab:hover { color: var(--tab-active-fg, #e6edf3); }

  .tab.active {
    color: var(--tab-active-fg, #e6edf3);
    border-bottom-color: var(--accent, #FF6B35);
    background: var(--tab-active-bg, #0d1117);
  }

  .tab.unread::after {
    content: "";
    display: inline-block;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent, #FF6B35);
    margin-left: 2px;
    flex-shrink: 0;
  }

  .tab-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .running-dot {
    background: var(--fg-muted, #7d8590);
    animation: pulse 1.4s ease-in-out infinite;
  }

  .agent-dot {
    background: var(--accent, #FF6B35);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
    flex: 1;
    min-width: 0;
  }

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

  .tab-close {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0 2px;
    opacity: 0;
    transition: opacity 0.1s;
    user-select: none;
    flex-shrink: 0;
  }

  .tab:hover .tab-close,
  .tab.active .tab-close { opacity: 0.5; }
  .tab-close:hover { opacity: 1 !important; }

  .tab-new {
    background: transparent;
    border: none;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    font-size: 18px;
    padding: 0 10px;
    height: 100%;
    transition: color 0.1s;
    flex-shrink: 0;
  }
  .tab-new:hover { color: var(--tab-active-fg, #e6edf3); }
</style>
