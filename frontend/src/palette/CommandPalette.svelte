<script lang="ts">
  import { allThemes, setTheme } from "../theme/store.js";

  interface Props {
    open: boolean;
    onClose: () => void;
    onNewTab?: () => void;
    onOpenSettings?: () => void;
  }

  let { open, onClose, onNewTab, onOpenSettings }: Props = $props();

  interface PaletteItem {
    id: string;
    label: string;
    description?: string;
    section: string;
    action: () => void;
  }

  let query = $state("");
  let selectedIdx = $state(0);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);
  let paletteEl = $state<HTMLDivElement | undefined>(undefined);

  function buildItems(): PaletteItem[] {
    const items: PaletteItem[] = [
      {
        id: "tab.new",
        label: "New Tab",
        description: "Open a new terminal tab",
        section: "Commands",
        action: () => { onNewTab?.(); onClose(); },
      },
      {
        id: "settings.open",
        label: "Open Settings",
        description: "Ctrl+,",
        section: "Commands",
        action: () => { onOpenSettings?.(); onClose(); },
      },
    ];

    for (const theme of $allThemes) {
      items.push({
        id: `theme.${theme.id}`,
        label: `Theme: ${theme.name}`,
        description: theme.kind,
        section: "Themes",
        action: () => { setTheme(theme.id); onClose(); },
      });
    }

    return items;
  }

  const allItems = $derived(buildItems());

  function fuzzyMatch(text: string, q: string): boolean {
    if (!q) return true;
    const tl = text.toLowerCase();
    const ql = q.toLowerCase();
    let ti = 0;
    for (let qi = 0; qi < ql.length; qi++) {
      const idx = tl.indexOf(ql[qi], ti);
      if (idx < 0) return false;
      ti = idx + 1;
    }
    return true;
  }

  const filtered = $derived(
    query
      ? allItems.filter((i) => fuzzyMatch(i.label, query) || fuzzyMatch(i.description ?? "", query))
      : allItems
  );

  // Group filtered items into sections
  const sections = $derived.by(() => {
    const map = new Map<string, PaletteItem[]>();
    for (const item of filtered) {
      const list = map.get(item.section) ?? [];
      list.push(item);
      map.set(item.section, list);
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }));
  });

  // Flat index for keyboard navigation (sections are visual only)
  $effect(() => {
    if (open) {
      query = "";
      selectedIdx = 0;
      requestAnimationFrame(() => inputEl?.focus());
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { onClose(); return; }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIdx = Math.min(selectedIdx + 1, filtered.length - 1);
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIdx = Math.max(selectedIdx - 1, 0);
    }
    if (e.key === "Enter") { filtered[selectedIdx]?.action(); return; }
    // Focus trap: keep Tab within the palette
    if (e.key === "Tab") {
      e.preventDefault();
      if (e.shiftKey) {
        selectedIdx = Math.max(selectedIdx - 1, 0);
      } else {
        selectedIdx = Math.min(selectedIdx + 1, filtered.length - 1);
      }
    }
  }

  // Map from item id to flat index
  function itemFlatIndex(sectionIdx: number, itemIdx: number): number {
    let n = 0;
    for (let s = 0; s < sectionIdx; s++) {
      n += sections.at(s)?.items.length ?? 0;
    }
    return n + itemIdx;
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="palette-overlay" onclick={onClose}>
    <div
      bind:this={paletteEl}
      class="palette"
      role="dialog"
      tabindex="-1"
      aria-label="Command palette"
      aria-modal="true"
      onclick={(e) => e.stopPropagation()}
      onkeydown={handleKeydown}
    >
      <input
        bind:this={inputEl}
        bind:value={query}
        class="palette-input"
        placeholder="Type a command or search themes…"
        autocomplete="off"
        spellcheck={false}
      />
      <div class="palette-list" role="listbox">
        {#each sections as section, si}
          <div class="section-header">{section.label}</div>
          {#each section.items as item, ii (item.id)}
            {@const flatIdx = itemFlatIndex(si, ii)}
            <button
              class="palette-item"
              class:selected={flatIdx === selectedIdx}
              role="option"
              aria-selected={flatIdx === selectedIdx}
              onclick={item.action}
              onmouseenter={() => (selectedIdx = flatIdx)}
            >
              <span class="item-label">{item.label}</span>
              {#if item.description}
                <span class="item-desc">{item.description}</span>
              {/if}
            </button>
          {/each}
        {/each}
        {#if filtered.length === 0}
          <div class="palette-empty">No results</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 80px;
    z-index: 1000;
  }

  .palette {
    width: 600px;
    max-width: calc(100vw - 32px);
    background: var(--bg-elevated, #010409);
    border: 1px solid var(--border, #21262d);
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    outline: none;
  }

  .palette-input {
    width: 100%;
    padding: 12px 16px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border, #21262d);
    color: var(--fg, #e6edf3);
    font-size: 14px;
    outline: none;
    box-sizing: border-box;
  }

  .palette-input::placeholder { color: var(--fg-muted, #7d8590); }

  .palette-list {
    max-height: 360px;
    overflow-y: auto;
    padding: 4px 0;
  }

  .section-header {
    padding: 8px 16px 4px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted, #7d8590);
    opacity: 0.6;
  }

  .palette-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 16px;
    background: transparent;
    border: none;
    color: var(--fg, #e6edf3);
    cursor: pointer;
    text-align: left;
    font-size: 13px;
  }

  .palette-item:hover,
  .palette-item.selected {
    background: rgba(255, 255, 255, 0.06);
  }

  .item-label { flex: 1; }

  .item-desc {
    color: var(--fg-muted, #7d8590);
    font-size: 11px;
    margin-left: 12px;
    flex-shrink: 0;
  }

  .palette-empty {
    padding: 16px;
    text-align: center;
    color: var(--fg-muted, #7d8590);
    font-size: 13px;
  }
</style>
