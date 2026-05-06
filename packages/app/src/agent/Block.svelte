<script lang="ts">
  import { slide } from "svelte/transition";
  import { Badge } from "@emberterm/ui";
  import type { Block } from "../stores/app.js";

  interface Props {
    block: Block;
  }

  let { block }: Props = $props();

  let expanded = $state(true);
  let contextMenu = $state<{ x: number; y: number } | null>(null);

  // Respect prefers-reduced-motion — drive transition duration to 0.
  let reducedMotion = $state(
    typeof window !== "undefined"
      ? window.matchMedia("(prefers-reduced-motion: reduce)").matches
      : false,
  );

  $effect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    reducedMotion = mq.matches;
    const handler = (e: MediaQueryListEvent) => { reducedMotion = e.matches; };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  });

  // Close context menu on any outside click.
  $effect(() => {
    if (!contextMenu) return;
    const close = () => { contextMenu = null; };
    window.addEventListener("click", close, { once: true });
    return () => window.removeEventListener("click", close);
  });

  const exitOk = $derived(block.exitCode === 0 || block.exitCode == null);
  const durationLabel = $derived(
    block.durationMs != null
      ? block.durationMs < 1000
        ? `${block.durationMs}ms`
        : `${(block.durationMs / 1000).toFixed(1)}s`
      : null
  );

  function openContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = { x: e.clientX, y: e.clientY };
  }

  function copyCommand() {
    navigator.clipboard.writeText(block.command).catch(() => {});
    contextMenu = null;
  }

  function copyOutput() {
    // Terminal output lives in xterm.js; copy available block metadata as summary.
    const lines = [
      `$ ${block.command}`,
      block.exitCode != null ? `Exit: ${block.exitCode}` : null,
      block.durationMs != null ? `Duration: ${durationLabel}` : null,
    ].filter(Boolean).join("\n");
    navigator.clipboard.writeText(lines).catch(() => {});
    contextMenu = null;
  }

  function shareBlock() {
    const meta = [
      block.exitCode != null ? `exit \`${block.exitCode}\`` : null,
      block.durationMs != null ? durationLabel : null,
      block.agentId ? `via ${block.agentId}` : null,
    ].filter(Boolean).join(" · ");
    const text = ["```", `$ ${block.command}`, "```", meta].filter(Boolean).join("\n");
    navigator.clipboard.writeText(text).catch(() => {});
    contextMenu = null;
  }
</script>

<div class="block" class:has-agent={!!block.agentId}>
  <!-- block-head-nav class is the selector used by App.svelte for Alt+Up/Down nav -->
  <div
    class="block-head block-head-nav"
    role="button"
    tabindex="0"
    aria-expanded={expanded}
    oncontextmenu={openContextMenu}
    onkeydown={(e) => {
      if (e.key === "Enter" || e.key === " ") { e.preventDefault(); expanded = !expanded; }
    }}
  >
    <button
      class="block-toggle"
      aria-label={expanded ? "Collapse block" : "Expand block"}
      onclick={() => (expanded = !expanded)}
      tabindex="-1"
    >
      <span class="block-chevron" class:open={expanded}>›</span>
    </button>

    <span class="block-cmd">{block.command || "(no command)"}</span>

    {#if block.agentId}
      <Badge variant="agent">{block.agentId}</Badge>
    {/if}

    {#if block.exitCode != null}
      <Badge variant={exitOk ? "ok" : "error"} mono>{block.exitCode}</Badge>
    {/if}

    {#if durationLabel}
      <span class="block-dur">{durationLabel}</span>
    {/if}
  </div>

  {#if expanded}
    <div class="block-body" transition:slide={{ duration: reducedMotion ? 0 : 150 }}>
      <!-- Terminal output is rendered by xterm.js into Grid.svelte.
           Block.svelte provides collapsible chrome; scrollback byte ranges are
           Phase 3 once the native renderer lands. -->
    </div>
  {/if}
</div>

<!-- Fixed-position context menu, rendered above all content -->
{#if contextMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="ctx-menu"
    style:left="{contextMenu.x}px"
    style:top="{contextMenu.y}px"
    role="menu"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <button role="menuitem" onclick={copyCommand}>Copy command</button>
    <button role="menuitem" onclick={copyOutput}>Copy output</button>
    <hr class="ctx-sep" />
    <button role="menuitem" onclick={shareBlock}>Share block</button>
  </div>
{/if}

<style>
  .block {
    display: flex;
    flex-direction: column;
    border-left: 2px solid transparent;
    margin: 2px 0;
    transition: border-color 0.15s;
  }

  .block.has-agent {
    border-left-color: var(--accent, #FF6B35);
  }

  .block-head {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px 3px 6px;
    min-height: 26px;
    background: var(--bg-block, rgba(255,255,255,0.03));
    border-radius: 4px;
    cursor: default;
    outline: none;
  }

  .block-head:focus-visible {
    outline: 2px solid var(--accent-glow, rgba(255,107,53,0.5));
    outline-offset: -2px;
  }

  .block-toggle {
    background: transparent;
    border: none;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }

  .block-chevron {
    display: inline-block;
    font-size: 14px;
    transition: transform 0.15s;
    transform: rotate(0deg);
  }

  .block-chevron.open {
    transform: rotate(90deg);
  }

  .block-cmd {
    flex: 1;
    font-family: monospace;
    font-size: 12px;
    color: var(--fg, #e6edf3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .block-dur {
    font-size: 10px;
    font-family: monospace;
    color: var(--fg-muted, #7d8590);
    flex-shrink: 0;
  }

  .block-body {
    padding-left: 14px;
    /* Placeholder — output is rendered by xterm.js */
  }

  /* Context menu */
  .ctx-menu {
    position: fixed;
    z-index: 1000;
    min-width: 160px;
    background: var(--bg-elev, #161b22);
    border: 1px solid var(--border, rgba(255,255,255,0.12));
    border-radius: 6px;
    padding: 4px 0;
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    display: flex;
    flex-direction: column;
  }

  .ctx-menu button {
    background: none;
    border: none;
    color: var(--fg, #e6edf3);
    font-size: 12px;
    padding: 6px 14px;
    text-align: left;
    cursor: pointer;
    width: 100%;
  }

  .ctx-menu button:hover { background: var(--bg-elev-2, rgba(255,255,255,0.06)); }

  .ctx-sep {
    border: none;
    border-top: 1px solid var(--border, rgba(255,255,255,0.1));
    margin: 3px 0;
  }
</style>
