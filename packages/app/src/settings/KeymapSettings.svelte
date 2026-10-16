<script lang="ts">
  import { Kbd } from "@emberterm/ui";
  import { SHORTCUTS, TERMINAL_SHORTCUTS } from "../utils/shortcuts.js";

  const ROWS = [
    ...SHORTCUTS.map((s) => ({ label: s.label, keys: s.keys })),
    ...TERMINAL_SHORTCUTS,
  ];

  let search = $state("");
  const visible = $derived(
    search.trim() ? ROWS.filter((r) => r.label.toLowerCase().includes(search.trim().toLowerCase())) : ROWS,
  );
</script>

<div class="keymap">
  <input
    class="search"
    placeholder="Search shortcuts…"
    aria-label="Search shortcuts"
    bind:value={search}
    autocomplete="off"
    spellcheck={false}
  />
  <table>
    <thead>
      <tr><th>Action</th><th>Shortcut</th></tr>
    </thead>
    <tbody>
      {#each visible as row (row.label)}
        <tr>
          <td>{row.label}</td>
          <td><Kbd>{row.keys}</Kbd></td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if visible.length === 0}
    <p class="empty">No shortcuts match "{search}"</p>
  {/if}
  <p class="note">On macOS, use Cmd in place of Ctrl.</p>
</div>

<style>
  .keymap { display: flex; flex-direction: column; gap: 10px; }

  .search {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--fg);
    font: inherit;
    font-size: 13px;
    padding: 7px 10px;
    outline: none;
    width: 100%;
    box-sizing: border-box;
  }

  .search::placeholder { color: var(--fg-muted); }
  .search:focus { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-glow); }

  table { width: 100%; border-collapse: collapse; font-size: 13px; }

  th {
    text-align: left;
    color: var(--fg-muted);
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0 0 8px;
    border-bottom: 1px solid var(--border);
  }

  td {
    padding: 8px 0;
    color: var(--fg);
    border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  }

  .empty { font-size: 12px; color: var(--fg-muted); text-align: center; padding: 12px 0; margin: 0; }
  .note { font-size: 11px; color: var(--fg-muted); margin: 4px 0 0; }
</style>
