<script lang="ts">
  import type { Profile } from "../ipc/client.js";

  interface Props {
    profiles: Profile[];
    defaultProfileId: string;
    onPick: (profileId: string) => void;
  }

  let { profiles, defaultProfileId, onPick }: Props = $props();

  let open = $state(false);
  let button: HTMLButtonElement | undefined = $state();
  let menu: HTMLDivElement | undefined = $state();
  let pos = $state({ left: 0, top: 0 });

  function toggle() {
    if (open) { open = false; return; }
    const r = button!.getBoundingClientRect();
    pos = { left: Math.max(4, Math.min(r.left, window.innerWidth - 204)), top: r.bottom + 4 };
    open = true;
    queueMicrotask(() => menu?.querySelector<HTMLButtonElement>("button")?.focus());
  }

  function pick(id: string) {
    open = false;
    onPick(id);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); open = false; button?.focus(); return; }
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const items = [...(menu?.querySelectorAll<HTMLButtonElement>("button") ?? [])];
    const i = items.indexOf(document.activeElement as HTMLButtonElement);
    items[(i + (e.key === "ArrowDown" ? 1 : -1) + items.length) % items.length]?.focus();
  }
</script>

<svelte:window onmousedown={(e) => { if (open && !menu?.contains(e.target as Node) && !button?.contains(e.target as Node)) open = false; }} />

<button
  bind:this={button}
  class="chevron"
  aria-label="Choose a shell for the new tab"
  aria-haspopup="menu"
  aria-expanded={open}
  disabled={profiles.length === 0}
  onclick={toggle}
>
  <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
    <path d="M2 3.5 L5 6.5 L8 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
  </svg>
</button>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div bind:this={menu} class="menu" role="menu" tabindex="-1" style="left:{pos.left}px;top:{pos.top}px" onkeydown={onKeydown}>
    {#each profiles as p (p.id)}
      <button role="menuitem" onclick={() => pick(p.id)}>
        <span>{p.name}</span>
        {#if p.id === defaultProfileId}<small>default</small>{/if}
      </button>
    {/each}
  </div>
{/if}

<style>
  .chevron {
    align-self: flex-end;
    width: 18px;
    height: 28px;
    margin: 0 0 1px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    -webkit-app-region: no-drag;
  }

  .chevron:hover:not(:disabled), .chevron[aria-expanded="true"] { background: rgba(255, 255, 255, 0.05); color: var(--fg, #e6edf3); }
  .chevron:focus-visible { outline: 2px solid var(--accent-glow); }
  .chevron:disabled { opacity: 0.4; cursor: default; }

  .menu {
    position: fixed;
    z-index: 800;
    min-width: 180px;
    padding: 4px;
    display: flex;
    flex-direction: column;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.4);
    outline: none;
  }

  .menu button {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding: 6px 10px;
    background: transparent;
    border: 0;
    border-radius: 5px;
    color: var(--fg);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .menu button:hover, .menu button:focus-visible { background: var(--accent-soft); outline: none; }
  small { color: var(--fg-muted); font-size: 11px; }
</style>
