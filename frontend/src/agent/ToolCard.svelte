<script lang="ts">
  export type ToolStatus = "pending" | "running" | "ok" | "error" | "cancelled";

  interface Props {
    id: string;
    name: string;
    summary: string;
    icon?: string;
    status?: ToolStatus;
    progressPct?: number;
    progressMessage?: string;
    durationMs?: number;
    onCancel?: () => void;
  }

  let {
    name,
    summary,
    icon = "terminal",
    status = "running",
    progressPct,
    progressMessage,
    durationMs,
    onCancel,
  }: Props = $props();

  let expanded = $state(false);

  const ICONS: Record<string, string> = {
    pencil: "✏",
    file: "📄",
    terminal: "⚡",
    network: "🌐",
    search: "🔍",
    bug: "🐛",
  };

  const statusIcon: Record<ToolStatus, string> = {
    pending: "⏳",
    running: "⟳",
    ok: "✓",
    error: "✗",
    cancelled: "⊘",
  };

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }
</script>

<div class="tool-card" class:expanded data-status={status}>
  <button
    class="tool-header"
    aria-expanded={expanded}
    onclick={() => (expanded = !expanded)}
  >
    <span class="tool-icon">{ICONS[icon] ?? "⚙"}</span>
    <span class="tool-name">{name}</span>
    <span class="tool-summary">{expanded ? "" : summary}</span>
    <span class="tool-spacer"></span>
    {#if status === "running" && progressPct !== undefined}
      <span class="tool-progress">{progressPct}%</span>
    {/if}
    <span class="tool-status" aria-label={status}>{statusIcon[status]}</span>
    {#if durationMs !== undefined && (status === "ok" || status === "error")}
      <span class="tool-duration">{formatDuration(durationMs)}</span>
    {/if}
    {#if status === "running" && onCancel}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span
        class="tool-cancel"
        title="Cancel"
        aria-label="Cancel tool call"
        onclick={(e) => { e.stopPropagation(); onCancel!(); }}
      >✕</span>
    {/if}
    <span class="tool-toggle" aria-hidden="true">{expanded ? "▴" : "▾"}</span>
  </button>

  {#if expanded}
    <div class="tool-body">
      <p class="tool-detail">{summary}</p>
      {#if progressMessage}
        <p class="tool-progress-msg">{progressMessage}</p>
      {/if}
    </div>
  {/if}

  {#if status === "running" && progressPct !== undefined}
    <div class="tool-progress-bar">
      <div class="tool-progress-fill" style="width: {progressPct}%"></div>
    </div>
  {/if}
</div>

<style>
  .tool-card {
    background: color-mix(in srgb, var(--bg-elevated, #010409) 80%, transparent);
    border: 1px solid var(--border, #21262d);
    border-radius: 6px;
    margin: 4px 0;
    font-size: 12px;
    overflow: hidden;
  }

  .tool-card[data-status="running"] {
    border-color: color-mix(in srgb, var(--accent, #FF6B35) 40%, var(--border, #21262d));
  }

  .tool-card[data-status="error"] {
    border-color: color-mix(in srgb, var(--danger, #f85149) 40%, var(--border, #21262d));
  }

  .tool-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    width: 100%;
    background: transparent;
    border: none;
    color: var(--fg, #e6edf3);
    cursor: pointer;
    text-align: left;
  }

  .tool-header:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .tool-icon { font-size: 13px; flex-shrink: 0; }
  .tool-name { font-weight: 500; flex-shrink: 0; }
  .tool-summary { color: var(--fg-muted, #7d8590); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tool-spacer { flex: 1; }
  .tool-progress { color: var(--accent, #FF6B35); font-size: 11px; flex-shrink: 0; }
  .tool-duration { color: var(--fg-muted, #7d8590); font-size: 11px; flex-shrink: 0; }
  .tool-toggle { color: var(--fg-muted, #7d8590); font-size: 10px; flex-shrink: 0; }

  .tool-cancel {
    color: var(--fg-muted, #7d8590);
    font-size: 10px;
    flex-shrink: 0;
    cursor: pointer;
    padding: 1px 4px;
    border-radius: 3px;
  }

  .tool-cancel:hover {
    background: rgba(248, 81, 73, 0.15);
    color: var(--danger, #f85149);
  }

  .tool-status {
    flex-shrink: 0;
    font-size: 13px;
  }

  [data-status="running"] .tool-status {
    color: var(--accent, #FF6B35);
    display: inline-block;
    animation: spin 1s linear infinite;
  }

  [data-status="ok"] .tool-status { color: var(--green, #3fb950); }
  [data-status="error"] .tool-status { color: var(--danger, #f85149); }
  [data-status="cancelled"] .tool-status { color: var(--fg-muted, #7d8590); }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .tool-body {
    padding: 6px 10px 8px;
    border-top: 1px solid var(--border, #21262d);
  }

  .tool-detail, .tool-progress-msg {
    margin: 0;
    color: var(--fg-muted, #7d8590);
    font-size: 11px;
    line-height: 1.5;
  }

  .tool-progress-msg { margin-top: 4px; color: var(--accent, #FF6B35); }

  .tool-progress-bar {
    height: 2px;
    background: var(--border, #21262d);
  }

  .tool-progress-fill {
    height: 100%;
    background: var(--accent, #FF6B35);
    transition: width 0.3s ease;
  }
</style>
