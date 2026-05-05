<script lang="ts">
  import { parseDiff, diffStats } from "./diff-utils.js";

  interface Props {
    id: string;
    path: string;
    patch: string;
    actions: string[];
    onAction: (id: string, result: string) => void;
  }

  let { id, path, patch, actions, onAction }: Props = $props();

  const lines = $derived(parseDiff(patch));
  const stats = $derived(diffStats(lines));

  const fileName = $derived(path.split(/[\\/]/).pop() ?? path);

  let resolved = $state(false);
  let resolvedResult = $state("");

  function act(result: string) {
    resolved = true;
    resolvedResult = result;
    onAction(id, result);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (resolved) return;
    const key = e.key.toLowerCase();
    if (key === "y" && actions.includes("accept")) { e.preventDefault(); act("accept"); }
    else if (key === "n" && actions.includes("reject")) { e.preventDefault(); act("reject"); }
    else if (key === "e" && actions.includes("edit")) { e.preventDefault(); act("edit"); }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="diff-view"
  class:resolved
  onkeydown={handleKeydown}
  tabindex={resolved ? -1 : 0}
  role="application"
  aria-label="Diff for {path}"
>
  <div class="diff-header">
    <span class="diff-path" title={path}>{fileName}</span>
    <span class="diff-stats">
      {#if stats.add > 0}<span class="stat-add">+{stats.add}</span>{/if}
      {#if stats.del > 0}<span class="stat-del">-{stats.del}</span>{/if}
    </span>
    {#if resolved}
      <span class="resolved-badge">
        {resolvedResult === "accept" ? "✓ Applied" : resolvedResult === "reject" ? "✕ Rejected" : "Editing…"}
      </span>
    {/if}
  </div>

  {#if !resolved}
    <div class="diff-body">
      {#each lines as line}
        {#if line.type === "hunk"}
          <div class="diff-hunk">{line.text}</div>
        {:else if line.type !== "meta"}
          <div class="diff-line" data-type={line.type}>
            <span class="line-gutter">{line.type !== "add" ? (line.oldNum ?? "") : ""}</span>
            <span class="line-gutter">{line.type !== "del" ? (line.newNum ?? "") : ""}</span>
            <span class="line-sign">{line.type === "add" ? "+" : line.type === "del" ? "−" : " "}</span>
            <span class="line-text">{line.text}</span>
          </div>
        {/if}
      {/each}
    </div>

    <div class="diff-footer">
      {#each actions as action}
        <button
          class="diff-action"
          class:primary={action === "accept"}
          onclick={() => act(action)}
        >
          {#if action === "accept"}Accept <kbd>Y</kbd>
          {:else if action === "reject"}Reject <kbd>N</kbd>
          {:else if action === "edit"}Edit <kbd>E</kbd>
          {:else}{action}
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .diff-view {
    background: var(--bg-block, rgba(255,255,255,0.03));
    border: 1px solid var(--border, #30363d);
    border-radius: 6px;
    overflow: hidden;
    font-size: 12px;
    font-family: monospace;
    margin: 4px 0;
  }

  .diff-view:focus { outline: 2px solid var(--accent, #FF6B35); outline-offset: -2px; }

  .diff-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: color-mix(in srgb, var(--bg-elev, #010409) 60%, transparent);
    border-bottom: 1px solid var(--border, #30363d);
  }

  .diff-path {
    flex: 1;
    color: var(--fg, #e6edf3);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .diff-stats { display: flex; gap: 6px; flex-shrink: 0; }
  .stat-add { color: var(--green, #3fb950); font-weight: 600; }
  .stat-del { color: var(--red, #f85149); font-weight: 600; }

  .resolved-badge {
    font-size: 11px;
    color: var(--fg-muted, #7d8590);
    flex-shrink: 0;
  }

  .diff-body {
    max-height: 300px;
    overflow-y: auto;
  }

  .diff-line {
    display: flex;
    min-width: 0;
  }

  .diff-line[data-type="add"] { background: var(--diff-add-bg, rgba(63,185,80,0.12)); }
  .diff-line[data-type="del"] { background: var(--diff-del-bg, rgba(248,81,73,0.12)); }

  .line-gutter {
    width: 36px;
    flex-shrink: 0;
    padding: 0 6px;
    text-align: right;
    color: var(--fg-muted, #7d8590);
    opacity: 0.5;
    user-select: none;
    font-size: 11px;
    line-height: 20px;
  }

  .line-sign {
    width: 16px;
    flex-shrink: 0;
    text-align: center;
    line-height: 20px;
    user-select: none;
  }

  [data-type="add"] .line-sign { color: var(--green, #3fb950); }
  [data-type="del"] .line-sign { color: var(--red, #f85149); }

  .line-text {
    flex: 1;
    padding: 0 8px 0 4px;
    line-height: 20px;
    color: var(--fg, #e6edf3);
    white-space: pre;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  [data-type="del"] .line-text { color: var(--diff-del-line, var(--fg, #e6edf3)); }
  [data-type="add"] .line-text { color: var(--diff-add-line, var(--fg, #e6edf3)); }

  .diff-hunk {
    padding: 2px 10px;
    color: var(--blue, #58a6ff);
    background: color-mix(in srgb, var(--blue, #58a6ff) 8%, transparent);
    font-size: 11px;
    line-height: 18px;
    opacity: 0.8;
  }

  .diff-footer {
    display: flex;
    gap: 6px;
    padding: 6px 10px;
    border-top: 1px solid var(--border, #30363d);
    background: color-mix(in srgb, var(--bg-elev, #010409) 60%, transparent);
  }

  .diff-action {
    padding: 4px 10px;
    border-radius: 4px;
    border: 1px solid var(--border, #30363d);
    background: transparent;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    font-size: 11px;
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .diff-action:hover { color: var(--fg, #e6edf3); background: rgba(255,255,255,0.05); }

  .diff-action.primary {
    background: color-mix(in srgb, var(--green, #3fb950) 15%, transparent);
    border-color: color-mix(in srgb, var(--green, #3fb950) 40%, transparent);
    color: var(--green, #3fb950);
  }

  .diff-action.primary:hover {
    background: color-mix(in srgb, var(--green, #3fb950) 25%, transparent);
  }

  kbd {
    background: rgba(255,255,255,0.07);
    border: 1px solid var(--border, #30363d);
    border-radius: 2px;
    font-size: 9px;
    padding: 0 3px;
    font-family: monospace;
    opacity: 0.7;
  }

  .resolved .diff-header { opacity: 0.6; }
</style>
