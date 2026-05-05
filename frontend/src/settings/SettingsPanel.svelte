<script lang="ts">
  import ThemeEditor from "./ThemeEditor.svelte";
  import ProfileEditor from "./ProfileEditor.svelte";
  import Btn from "../lib/Btn.svelte";
  import { profileList, type Profile } from "../ipc/client.js";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  type Category = "appearance" | "profiles" | "keymap" | "agent";

  let activeCategory = $state<Category>("appearance");
  let profiles = $state<Profile[]>([]);
  let editingProfile = $state<Profile | null>(null);
  let creatingProfile = $state(false);
  let searchQuery = $state("");

  const CATEGORIES: { id: Category; label: string; keywords: string[] }[] = [
    { id: "appearance", label: "Appearance", keywords: ["theme", "font", "color", "style", "opacity", "acrylic"] },
    { id: "profiles", label: "Profiles", keywords: ["shell", "profile", "pwsh", "bash", "zsh", "cmd", "env"] },
    { id: "keymap", label: "Keymap", keywords: ["key", "shortcut", "hotkey", "ctrl", "cmd"] },
    { id: "agent", label: "Agent", keywords: ["agent", "osc", "tool", "claude", "ai"] },
  ];

  const visibleCategories = $derived(
    searchQuery.trim()
      ? CATEGORIES.filter((c) => {
          const q = searchQuery.toLowerCase();
          return (
            c.label.toLowerCase().includes(q) ||
            c.keywords.some((k) => k.includes(q))
          );
        })
      : CATEGORIES
  );

  $effect(() => {
    profileList().then((p) => { profiles = p; }).catch(() => {});
  });

  const KEYMAP = [
    { action: "New tab", win: "Ctrl+Shift+T", mac: "Cmd+T" },
    { action: "Close tab", win: "Ctrl+Shift+W", mac: "Cmd+W" },
    { action: "Next tab", win: "Ctrl+Tab", mac: "Cmd+Opt+→" },
    { action: "Prev tab", win: "Ctrl+Shift+Tab", mac: "Cmd+Opt+←" },
    { action: "Command palette", win: "Ctrl+Shift+P", mac: "Cmd+Shift+P" },
    { action: "Settings", win: "Ctrl+,", mac: "Cmd+," },
    { action: "Copy", win: "Ctrl+Shift+C", mac: "Cmd+C" },
    { action: "Paste", win: "Ctrl+Shift+V", mac: "Cmd+V" },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="settings-overlay" onclick={onClose} onkeydown={handleKeydown}>
    <div class="settings-panel" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" aria-label="Settings">
      <!-- Left nav -->
      <nav class="settings-nav">
        <div class="nav-search-wrap">
          <input
            class="nav-search"
            placeholder="Search…"
            bind:value={searchQuery}
            autocomplete="off"
            spellcheck={false}
          />
        </div>
        {#each visibleCategories as cat}
          <button
            class="nav-item"
            class:active={activeCategory === cat.id}
            onclick={() => { activeCategory = cat.id; }}
          >
            {cat.label}
          </button>
        {/each}
        {#if visibleCategories.length === 0}
          <span class="nav-empty">No matches</span>
        {/if}
      </nav>

      <!-- Right content -->
      <div class="settings-content">
        <button class="close-btn" aria-label="Close settings" onclick={onClose}>×</button>

        {#if activeCategory === "appearance"}
          <ThemeEditor />

        {:else if activeCategory === "profiles"}
          {#if creatingProfile || editingProfile}
            <ProfileEditor
              profile={editingProfile ?? undefined}
              onSave={(p) => {
                profiles = [...profiles.filter((x: any) => x.id !== p.id), p];
                creatingProfile = false;
                editingProfile = null;
              }}
              onCancel={() => { creatingProfile = false; editingProfile = null; }}
            />
          {:else}
            <div class="profiles-list">
              <h3>Profiles</h3>
              {#each profiles as p}
                <div class="profile-row">
                  <span class="profile-name">{p.name}</span>
                  <span class="profile-shell">{p.shell}</span>
                  <Btn variant="ghost" size="sm" onclick={() => (editingProfile = p)}>Edit</Btn>
                </div>
              {/each}
              <Btn variant="primary" onclick={() => (creatingProfile = true)}>+ New Profile</Btn>
            </div>
          {/if}

        {:else if activeCategory === "keymap"}
          <div class="keymap-section">
            <h3>Keyboard Shortcuts</h3>
            <table class="keymap-table">
              <thead>
                <tr><th>Action</th><th>Windows / Linux</th><th>macOS</th></tr>
              </thead>
              <tbody>
                {#each KEYMAP as row}
                  <tr>
                    <td>{row.action}</td>
                    <td><kbd>{row.win}</kbd></td>
                    <td><kbd>{row.mac}</kbd></td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>

        {:else if activeCategory === "agent"}
          <div class="agent-section">
            <h3>Agent Integration</h3>
            <p class="agent-desc">OSC 7777 protocol is enabled. Any agent that implements the spec will automatically display tool cards and integration UI.</p>
            <p class="agent-version">Spec version: 0.1</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 900;
    animation: fade-in 120ms ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .settings-panel {
    display: flex;
    width: 700px;
    max-width: calc(100vw - 48px);
    height: 500px;
    max-height: calc(100vh - 96px);
    background: var(--bg-elev, #010409);
    border: 1px solid var(--border, #21262d);
    border-radius: 10px;
    box-shadow: 0 24px 64px rgba(0,0,0,0.6);
    overflow: hidden;
    position: relative;
    animation: slide-in 220ms cubic-bezier(.2,.8,.2,1);
  }

  @keyframes slide-in {
    from { opacity: 0; transform: translateY(-8px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .settings-nav {
    display: flex;
    flex-direction: column;
    width: 160px;
    flex-shrink: 0;
    border-right: 1px solid var(--border, #21262d);
    padding: 8px 0;
    gap: 2px;
  }

  .nav-search-wrap {
    padding: 0 8px 8px;
    border-bottom: 1px solid var(--border, #21262d);
    margin-bottom: 4px;
  }

  .nav-search {
    width: 100%;
    background: rgba(255,255,255,0.04);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    color: var(--fg, #e6edf3);
    font-size: 12px;
    padding: 5px 8px;
    outline: none;
    box-sizing: border-box;
  }

  .nav-search::placeholder { color: var(--fg-muted, #7d8590); }
  .nav-search:focus { border-color: var(--accent, #FF6B35); }

  .nav-empty {
    font-size: 11px;
    color: var(--fg-muted, #7d8590);
    padding: 8px 16px;
    opacity: 0.6;
  }

  .nav-item {
    background: transparent;
    border: none;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    font-size: 13px;
    padding: 8px 16px;
    text-align: left;
    border-radius: 4px;
    margin: 0 4px;
    transition: color 0.1s, background 0.1s;
  }

  .nav-item:hover { color: var(--fg, #e6edf3); background: rgba(255,255,255,0.04); }
  .nav-item.active { color: var(--fg, #e6edf3); background: rgba(255,255,255,0.08); }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    position: relative;
  }

  .close-btn {
    position: absolute;
    top: 12px;
    right: 14px;
    background: transparent;
    border: none;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    font-size: 20px;
    line-height: 1;
    padding: 0;
  }

  .close-btn:hover { color: var(--fg, #e6edf3); }

  /* Profiles */
  .profiles-list { display: flex; flex-direction: column; gap: 8px; }
  h3 { margin: 0 0 12px; font-size: 14px; color: var(--fg, #e6edf3); }

  .profile-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: rgba(255,255,255,0.03);
    border: 1px solid var(--border, #21262d);
    border-radius: 5px;
  }

  .profile-name { flex: 1; font-size: 13px; color: var(--fg, #e6edf3); }
  .profile-shell { font-size: 11px; color: var(--fg-muted, #7d8590); font-family: monospace; }

  .profiles-list :global(.btn) { align-self: flex-start; margin-top: 4px; }

  /* Keymap */
  .keymap-section { display: flex; flex-direction: column; gap: 12px; }

  .keymap-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .keymap-table th {
    text-align: left;
    color: var(--fg-muted, #7d8590);
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0 0 8px;
    border-bottom: 1px solid var(--border, #21262d);
  }

  .keymap-table td {
    padding: 7px 0;
    color: var(--fg, #e6edf3);
    border-bottom: 1px solid color-mix(in srgb, var(--border, #21262d) 50%, transparent);
  }

  kbd {
    background: rgba(255,255,255,0.07);
    border: 1px solid var(--border, #21262d);
    border-radius: 3px;
    font-family: monospace;
    font-size: 11px;
    padding: 2px 6px;
  }

  /* Agent */
  .agent-section { display: flex; flex-direction: column; gap: 8px; }
  .agent-desc { color: var(--fg-muted, #7d8590); font-size: 13px; line-height: 1.6; margin: 0; }
  .agent-version { color: var(--fg-muted, #7d8590); font-size: 11px; margin: 0; }
</style>
