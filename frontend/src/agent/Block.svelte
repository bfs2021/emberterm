<script lang="ts">
  import { slide } from "svelte/transition";
  import type { Snippet } from "svelte";
  import type { Block } from "../stores/app.js";

  interface Props {
    block: Block;
    children?: Snippet;
  }

  let { block, children }: Props = $props();

  let expanded = $state(true);

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

  const exitOk = $derived(block.exitCode === 0 || block.exitCode == null);
  const durationLabel = $derived(
    block.durationMs != null
      ? block.durationMs < 1000
        ? `${block.durationMs}ms`
        : `${(block.durationMs / 1000).toFixed(1)}s`
      : null
  );
</script>

<div class="block" class:has-agent={!!block.agentId}>
  <div class="block-head">
    <button
      class="block-toggle"
      aria-expanded={expanded}
      aria-label={expanded ? "Collapse block" : "Expand block"}
      onclick={() => (expanded = !expanded)}
    >
      <span class="block-chevron" class:open={expanded}>›</span>
    </button>

    <span class="block-cmd">{block.command || "(no command)"}</span>

    {#if block.agentId}
      <span class="block-badge agent-badge">{block.agentId}</span>
    {/if}

    {#if block.exitCode != null}
      <span class="block-badge exit-badge" class:ok={exitOk} class:err={!exitOk}>
        {block.exitCode}
      </span>
    {/if}

    {#if durationLabel}
      <span class="block-dur">{durationLabel}</span>
    {/if}
  </div>

  {#if expanded}
    <div class="block-body" transition:slide={{ duration: reducedMotion ? 0 : 150 }}>
      <!-- Terminal output is rendered by xterm.js into Grid.svelte.
           Block.svelte provides collapsible chrome; scrollback byte ranges are
           Phase 2 once the Rust block model is wired. -->
      {#if children}{@render children()}{/if}
    </div>
  {/if}
</div>

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

  .block-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .agent-badge {
    background: color-mix(in srgb, var(--accent, #FF6B35) 15%, transparent);
    color: var(--accent, #FF6B35);
    border: 1px solid color-mix(in srgb, var(--accent, #FF6B35) 30%, transparent);
  }

  .exit-badge {
    font-family: monospace;
  }

  .exit-badge.ok {
    background: color-mix(in srgb, var(--green, #3fb950) 12%, transparent);
    color: var(--green, #3fb950);
    border: 1px solid color-mix(in srgb, var(--green, #3fb950) 28%, transparent);
  }

  .exit-badge.err {
    background: color-mix(in srgb, var(--red, #f85149) 12%, transparent);
    color: var(--red, #f85149);
    border: 1px solid color-mix(in srgb, var(--red, #f85149) 28%, transparent);
  }

  .block-dur {
    font-size: 10px;
    font-family: monospace;
    color: var(--fg-muted, #7d8590);
    flex-shrink: 0;
  }

  .block-body {
    padding-left: 14px;
  }
</style>
