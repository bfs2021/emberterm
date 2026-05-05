<script lang="ts">
  import { allThemes, activeThemeId, setTheme, importTheme } from "../theme/store.js";
  import Btn from "../lib/Btn.svelte";

  interface Props {
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  let importJson = $state("");
  let importError = $state("");

  function handleImport() {
    importError = "";
    const theme = importTheme(importJson);
    if (theme) {
      setTheme(theme.id);
      importJson = "";
    } else {
      importError = "Invalid VS Code theme JSON.";
    }
  }
</script>

<div class="theme-editor">
  <h3>Themes</h3>

  <div class="theme-list">
    {#each $allThemes as theme (theme.id)}
      <button
        class="theme-item"
        class:active={$activeThemeId === theme.id}
        onclick={() => setTheme(theme.id)}
      >
        <span class="theme-swatch" style="background:{theme.vars['--bg']}; border-color:{theme.vars['--border']};">
          <span class="swatch-accent" style="background:{theme.vars['--accent']};"></span>
        </span>
        <span class="theme-name">{theme.name}</span>
        <span class="theme-kind">{theme.kind}</span>
        {#if $activeThemeId === theme.id}
          <span class="theme-check">✓</span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="import-section">
    <h4>Import VS Code Theme</h4>
    <textarea
      bind:value={importJson}
      placeholder="Paste VS Code theme JSON here…"
      rows="6"
    ></textarea>
    {#if importError}
      <p class="import-error">{importError}</p>
    {/if}
    <Btn variant="primary" onclick={handleImport} disabled={!importJson.trim()}>Import</Btn>
  </div>
</div>

<style>
  .theme-editor { display: flex; flex-direction: column; gap: 16px; }
  h3 { margin: 0 0 4px; font-size: 14px; color: var(--fg, #e6edf3); }
  h4 { margin: 0 0 8px; font-size: 12px; color: var(--fg-muted, #7d8590); text-transform: uppercase; letter-spacing: 0.05em; }

  .theme-list { display: flex; flex-direction: column; gap: 2px; }

  .theme-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 5px;
    cursor: pointer;
    color: var(--fg, #e6edf3);
    text-align: left;
  }

  .theme-item:hover { background: rgba(255,255,255,0.04); }

  .theme-item.active {
    border-color: var(--accent, #FF6B35);
    background: rgba(255,107,53,0.07);
  }

  .theme-swatch {
    width: 28px;
    height: 20px;
    border-radius: 3px;
    border: 1px solid;
    flex-shrink: 0;
    position: relative;
    overflow: hidden;
  }

  .swatch-accent {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 4px;
  }

  .theme-name { flex: 1; font-size: 13px; }

  .theme-kind { font-size: 11px; color: var(--fg-muted, #7d8590); }

  .theme-check { color: var(--accent, #FF6B35); font-size: 12px; }

  .import-section { display: flex; flex-direction: column; gap: 8px; }

  textarea {
    background: var(--bg, #0d1117);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    color: var(--fg, #e6edf3);
    font-size: 12px;
    font-family: monospace;
    padding: 8px 10px;
    resize: vertical;
    outline: none;
  }

  textarea:focus { border-color: var(--accent, #FF6B35); }

  .import-error { color: var(--danger, #f85149); font-size: 12px; margin: 0; }

  .import-section :global(.btn) { align-self: flex-start; }
</style>
