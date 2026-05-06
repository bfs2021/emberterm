<script lang="ts">
  import { StatusDot, ProgressBar } from "@emberterm/ui";
  import { stripAnsi } from "../utils/stripAnsi.js";

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
    args?: Record<string, unknown>;
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
    args,
    onCancel,
  }: Props = $props();

  // Strip escape sequences from agent-supplied strings before DOM rendering.
  const safeName = $derived(stripAnsi(name));
  const safeSummary = $derived(stripAnsi(summary));
  const safeProgressMsg = $derived(progressMessage != null ? stripAnsi(progressMessage) : undefined);

  let expanded = $state(false);

  // Minimal inline SVG paths keyed by icon name
  const ICON_PATHS: Record<string, string> = {
    pencil: "M9.3 2.7a1 1 0 0 1 1.4 1.4L4 10.8 1.5 11 1.7 8.5zm0 0",
    file: "M4 1h5l2 2v9a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1zM8.5 1.5V4H11",
    terminal: "M2 4l4 4-4 4M7 12h5",
    network: "M6 1a5 5 0 1 0 0 10A5 5 0 0 0 6 1zM1 6h10M6 1c-2 2-2 6 0 10M6 1c2 2 2 6 0 10",
    search: "M7.5 7.5a4 4 0 1 0-5.66-5.66 4 4 0 0 0 5.66 5.66zM7.5 7.5L11 11",
    bug: "M4 4a2 2 0 0 1 4 0v3H4zM1 7h3M8 7h3M2 4L1 2M10 4l1-2M6 7v4M4 11a2 2 0 0 0 4 0",
    default: "M6 1a5 5 0 1 0 0 10A5 5 0 0 0 6 1zM6 4v4M6 9v1",
  };

  function iconPath(key: string): string {
    return ICON_PATHS[key] ?? ICON_PATHS.default;
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }

  const hasArgs = $derived(!!args && Object.keys(args).length > 0);
</script>

<div class="tool-card" class:expanded data-status={status}>
  <button
    class="tool-header"
    aria-expanded={expanded}
    onclick={() => (expanded = !expanded)}
  >
    <StatusDot {status} />

    <!-- Tool icon -->
    <svg class="tool-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d={iconPath(icon)} />
    </svg>

    <span class="tool-name">{safeName}</span>
    <span class="tool-summary">{safeSummary}</span>

    <span class="tool-spacer"></span>

    <!-- Duration badge shown after completion -->
    {#if durationMs !== undefined && (status === "ok" || status === "error")}
      <span class="tool-time">{formatDuration(durationMs)}</span>
    {/if}

    <!-- Progress percentage during run -->
    {#if status === "running" && progressPct !== undefined}
      <span class="tool-pct">{progressPct}%</span>
    {/if}

    <!-- Cancel button (running only) -->
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

    <span class="tool-chevron" aria-hidden="true">{expanded ? "▴" : "▾"}</span>
  </button>

  {#if status === "running" && progressPct !== undefined}
    <ProgressBar pct={progressPct} />
  {/if}

  <!-- Expanded body -->
  {#if expanded}
    <div class="tool-body">
      <p class="tool-detail">{safeSummary}</p>

      {#if safeProgressMsg}
        <p class="tool-progress-msg">{safeProgressMsg}</p>
      {/if}

      {#if hasArgs}
        <dl class="tool-args">
          {#each Object.entries(args!) as [k, v]}
            <div class="arg-row">
              <dt class="arg-key">{k}</dt>
              <dd class="arg-val">{typeof v === "object" ? JSON.stringify(v) : String(v)}</dd>
            </div>
          {/each}
        </dl>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tool-card {
    background: color-mix(in srgb, var(--bg-elev, #010409) 80%, transparent);
    border: 1px solid var(--border, #30363d);
    border-radius: 6px;
    margin: 4px 0;
    font-size: 12px;
    overflow: hidden;
  }

  .tool-card[data-status="running"] {
    border-color: color-mix(in srgb, var(--accent, #FF6B35) 40%, var(--border, #30363d));
  }

  .tool-card[data-status="error"] {
    border-color: color-mix(in srgb, var(--danger, #f85149) 40%, var(--border, #30363d));
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
    background: var(--accent-soft, rgba(255, 255, 255, 0.03));
  }

  .tool-icon {
    width: 12px;
    height: 12px;
    flex-shrink: 0;
    opacity: 0.7;
  }

  .tool-name { font-weight: 500; flex-shrink: 0; }
  .tool-summary {
    color: var(--fg-muted, #7d8590);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tool-spacer { flex: 1; min-width: 8px; }

  .tool-time {
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: var(--fg-muted, #7d8590);
    background: var(--bg-block, rgba(255,255,255,0.04));
    border-radius: 3px;
    padding: 1px 5px;
    flex-shrink: 0;
  }

  .tool-pct {
    font-size: 10px;
    color: var(--accent, #FF6B35);
    flex-shrink: 0;
  }

  .tool-cancel {
    color: var(--fg-muted, #7d8590);
    font-size: 10px;
    flex-shrink: 0;
    cursor: pointer;
    padding: 1px 4px;
    border-radius: 3px;
  }

  .tool-cancel:hover {
    background: color-mix(in srgb, var(--danger, #f85149) 15%, transparent);
    color: var(--danger, #f85149);
  }

  .tool-chevron { color: var(--fg-muted, #7d8590); font-size: 9px; flex-shrink: 0; }

  .tool-body {
    padding: 6px 10px 8px;
    border-top: 1px solid var(--border, #30363d);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tool-detail {
    margin: 0;
    color: var(--fg-muted, #7d8590);
    font-size: 11px;
    line-height: 1.5;
  }

  .tool-progress-msg {
    margin: 0;
    color: var(--accent, #FF6B35);
    font-size: 11px;
    line-height: 1.5;
  }

  .tool-args {
    margin: 4px 0 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .arg-row {
    display: flex;
    gap: 8px;
    font-size: 11px;
  }

  .arg-key {
    color: var(--blue, #58a6ff);
    flex-shrink: 0;
    font-family: monospace;
    min-width: 60px;
  }

  .arg-val {
    color: var(--fg-muted, #7d8590);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
  }
</style>
