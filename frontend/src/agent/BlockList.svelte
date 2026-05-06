<script lang="ts">
  import Block from "./Block.svelte";
  import type { Block as BlockData } from "../stores/app.js";

  interface Props {
    blocks: BlockData[];
  }

  let { blocks }: Props = $props();

  // Show most recent MAX_VISIBLE blocks; earlier ones are scrollable.
  const MAX_VISIBLE = 8;
  let showAll = $state(false);

  const visible = $derived(
    showAll ? blocks : blocks.slice(-MAX_VISIBLE)
  );
  const hiddenCount = $derived(blocks.length - MAX_VISIBLE);
</script>

{#if blocks.length > 0}
  <section class="block-list" aria-label="Command history">
    {#if !showAll && hiddenCount > 0}
      <button class="show-all-btn" onclick={() => (showAll = true)}>
        ↑ {hiddenCount} earlier block{hiddenCount !== 1 ? "s" : ""}
      </button>
    {/if}
    {#each visible as block (block.id)}
      <Block {block} />
    {/each}
  </section>
{/if}

<style>
  .block-list {
    flex-shrink: 0;
    max-height: 200px;
    overflow-y: auto;
    padding: 4px 8px;
    border-top: 1px solid var(--border, rgba(255,255,255,0.1));
    background: var(--bg, #0d1117);
    /* Scrollbar styling */
    scrollbar-width: thin;
    scrollbar-color: var(--border, rgba(255,255,255,0.15)) transparent;
  }

  .show-all-btn {
    width: 100%;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border, rgba(255,255,255,0.08));
    color: var(--fg-muted, #7d8590);
    font-size: 11px;
    padding: 4px 8px;
    cursor: pointer;
    text-align: left;
    margin-bottom: 2px;
  }

  .show-all-btn:hover { color: var(--fg, #e6edf3); }
</style>
