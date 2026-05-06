<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<kbd class="kbd">{@render children()}</kbd>

<style>
  .kbd {
    display: inline-block;
    padding: 1px 5px;
    background: var(--bg-elev-2, #161b22);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    font-family: var(--mono, monospace);
    font-size: 10.5px;
    color: var(--fg-muted, #7d8590);
    line-height: 1.4;
    font-style: normal;
  }
</style>
