<script lang="ts">
  import { onMount } from "svelte";

  interface Props {
    x: number;
    y: number;
    canCopy: boolean;
    onCopy: () => void;
    onPaste: () => void;
    onSelectAll: () => void;
    onFind: () => void;
    onClose: () => void;
  }

  let { x, y, canCopy, onCopy, onPaste, onSelectAll, onFind, onClose }: Props = $props();

  let menu: HTMLDivElement | undefined = $state();

  // Keep the menu on screen when opened near the right/bottom edge.
  const left = $derived(Math.max(4, Math.min(x, window.innerWidth - 164)));
  const top = $derived(Math.max(4, Math.min(y, window.innerHeight - 140)));

  onMount(() => menu?.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus());

  function pick(action: () => void) {
    onClose();
    action();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); onClose(); return; }
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const items = [...(menu?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
    const i = items.indexOf(document.activeElement as HTMLButtonElement);
    items[(i + (e.key === "ArrowDown" ? 1 : -1) + items.length) % items.length]?.focus();
  }
</script>

<svelte:window onmousedown={(e) => { if (!menu?.contains(e.target as Node)) onClose(); }} onblur={onClose} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div bind:this={menu} class="menu" role="menu" tabindex="-1" style="left:{left}px;top:{top}px" onkeydown={onKeydown}>
  <button role="menuitem" disabled={!canCopy} onclick={() => pick(onCopy)}>Copy</button>
  <button role="menuitem" onclick={() => pick(onPaste)}>Paste</button>
  <button role="menuitem" onclick={() => pick(onSelectAll)}>Select all</button>
  <hr />
  <button role="menuitem" onclick={() => pick(onFind)}>Find…</button>
</div>

<style>
  .menu {
    position: fixed;
    z-index: 800;
    min-width: 150px;
    padding: 4px;
    display: flex;
    flex-direction: column;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.4);
    outline: none;
  }

  button {
    text-align: left;
    padding: 6px 10px;
    background: transparent;
    border: 0;
    border-radius: 5px;
    color: var(--fg);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }

  button:hover:not(:disabled), button:focus-visible { background: var(--accent-soft); outline: none; }
  button:disabled { color: var(--fg-muted); opacity: 0.5; cursor: default; }
  hr { margin: 4px 2px; border: 0; border-top: 1px solid var(--border); }
</style>
