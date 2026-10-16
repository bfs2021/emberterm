<script lang="ts">
  import { onMount } from "svelte";
  import type { SearchAddon } from "@xterm/addon-search";

  interface Props {
    search: SearchAddon;
    onClose: () => void;
  }

  let { search, onClose }: Props = $props();

  let query = $state("");
  let input: HTMLInputElement | undefined = $state();
  let noMatch = $state(false);

  onMount(() => { input?.focus(); input?.select(); });

  function run(backwards: boolean, incremental = false) {
    if (!query) { noMatch = false; search.clearDecorations(); return; }
    noMatch = !(backwards ? search.findPrevious(query) : search.findNext(query, { incremental }));
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); close(); }
    else if (e.key === "Enter") { e.preventDefault(); run(e.shiftKey); }
  }

  function close() {
    search.clearDecorations();
    onClose();
  }
</script>

<div class="find" role="search">
  <input
    bind:this={input}
    bind:value={query}
    class:bad={noMatch}
    oninput={() => run(false, true)}
    onkeydown={onKeydown}
    placeholder="Find in scrollback"
    aria-label="Find in scrollback"
    autocomplete="off"
    spellcheck={false}
  />
  <button type="button" aria-label="Previous match" title="Previous (Shift+Enter)" onclick={() => run(true)}>↑</button>
  <button type="button" aria-label="Next match" title="Next (Enter)" onclick={() => run(false)}>↓</button>
  <button type="button" aria-label="Close find" title="Close (Esc)" onclick={close}>✕</button>
</div>

<style>
  .find {
    position: absolute;
    top: 8px;
    right: 18px;
    z-index: 10;
    display: flex;
    gap: 4px;
    padding: 6px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35);
  }

  input {
    width: 200px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--fg);
    font: inherit;
    font-size: 12px;
    padding: 4px 8px;
    outline: none;
  }

  input:focus { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-glow); }
  input.bad { border-color: var(--red); }

  button {
    width: 26px;
    background: transparent;
    border: 0;
    border-radius: 5px;
    color: var(--fg-muted);
    font-size: 12px;
    cursor: pointer;
  }

  button:hover { background: var(--accent-soft); color: var(--fg); }
  button:focus-visible { outline: 2px solid var(--accent-glow); }
</style>
