<script lang="ts">
  import AppearanceSettings from "./AppearanceSettings.svelte";
  import TerminalSettings from "./TerminalSettings.svelte";
  import ThemeEditor from "./ThemeEditor.svelte";
  import AgentSettings from "./AgentSettings.svelte";
  import ProfileEditor from "./ProfileEditor.svelte";
  import { Btn, Kbd } from "@emberterm/ui";
  import { profileList, configUpdate, type Profile } from "../ipc/client.js";

  interface Props {
    open: boolean;
    onClose: () => void;
    /** Called when the user clicks Test in the profile editor. Receives the profile id. */
    onTestProfile?: (profileId: string) => void;
  }

  let { open, onClose, onTestProfile }: Props = $props();

  type Category = "appearance" | "profiles" | "terminal" | "colorscheme" | "keymap" | "agent";

  let activeCategory = $state<Category>("appearance");
  let profiles = $state<Profile[]>([]);
  let editingProfile = $state<Profile | null>(null);
  let creatingProfile = $state(false);
  let searchQuery = $state("");
  let defaultProfile = $state("");

  const CATEGORIES: { id: Category; label: string; keywords: string[]; icon: string }[] = [
    {
      id: "appearance", label: "Appearance",
      keywords: ["theme", "font", "color", "style", "opacity", "acrylic", "padding", "window"],
      icon: "M2 9a1 1 0 0 1 1-1h1.5l1-3h3l1 3H11a1 1 0 0 1 1 1v1H2zM4 10v2h8v-2M6 4l.5 1.5h3L10 4",
    },
    {
      id: "profiles", label: "Profiles",
      keywords: ["shell", "profile", "pwsh", "bash", "zsh", "cmd", "env", "default"],
      icon: "M6 7a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM1 11a5 5 0 0 1 10 0",
    },
    {
      id: "terminal", label: "Terminal",
      keywords: ["cursor", "scrollback", "blink", "bold", "close", "behavior", "rendering"],
      icon: "M1 4l3 3-3 3M7 10h4",
    },
    {
      id: "colorscheme", label: "Color Scheme",
      keywords: ["theme", "color", "scheme", "dark", "light", "import", "vscode"],
      icon: "M6 1a5 5 0 1 0 0 10A5 5 0 0 0 6 1zM6 3v6M3 6h6",
    },
    {
      id: "keymap", label: "Keymap",
      keywords: ["key", "shortcut", "hotkey", "ctrl", "cmd", "binding"],
      icon: "M1 3h10v6H1zM3 3v6M5 3v6M7 3v6M9 3v6M1 6h10",
    },
    {
      id: "agent", label: "Agent",
      keywords: ["agent", "osc", "tool", "claude", "ai", "7777", "auto", "accept"],
      icon: "M6 1l1.5 3 3.5.5-2.5 2.5.5 3.5L6 9l-3 1.5.5-3.5L1 4.5 4.5 4z",
    },
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
    if (!open) return;
    profileList().then((p) => {
      profiles = p;
      defaultProfile = p.find((x) => x.id === "default")?.id ?? (p[0]?.id ?? "");
    }).catch(() => {});
  });

  async function onDefaultProfileChange(e: Event) {
    const id = (e.currentTarget as HTMLSelectElement).value;
    defaultProfile = id;
    await configUpdate({ behavior: { default_profile: id } });
  }

  const KEYMAP = [
    { action: "New tab", win: "Ctrl+Shift+T", mac: "Cmd+T" },
    { action: "Close tab", win: "Ctrl+Shift+W", mac: "Cmd+W" },
    { action: "Next tab", win: "Ctrl+Tab", mac: "Cmd+Opt+→" },
    { action: "Prev tab", win: "Ctrl+Shift+Tab", mac: "Cmd+Opt+←" },
    { action: "Command palette", win: "Ctrl+Shift+P", mac: "Cmd+Shift+P" },
    { action: "Settings", win: "Ctrl+,", mac: "Cmd+," },
    { action: "Copy", win: "Ctrl+Shift+C", mac: "Cmd+C" },
    { action: "Paste", win: "Ctrl+Shift+V", mac: "Cmd+V" },
    { action: "Accept diff", win: "Y", mac: "Y" },
    { action: "Reject diff", win: "N", mac: "N" },
    { action: "Approve action", win: "Y", mac: "Y" },
    { action: "Deny action", win: "N", mac: "N" },
  ];

  let keymapSearch = $state("");
  const visibleKeymap = $derived(
    keymapSearch.trim()
      ? KEYMAP.filter((k) => k.action.toLowerCase().includes(keymapSearch.toLowerCase()))
      : KEYMAP
  );

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
            title={cat.label}
          >
            <svg class="nav-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d={cat.icon} />
            </svg>
            <span>{cat.label}</span>
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
          <h2 class="section-title">Appearance</h2>
          <AppearanceSettings />

        {:else if activeCategory === "profiles"}
          {#if creatingProfile || editingProfile}
            <ProfileEditor
              profile={editingProfile ?? undefined}
              onSave={(p) => {
                profiles = [...profiles.filter((x) => x.id !== p.id), p];
                creatingProfile = false;
                editingProfile = null;
              }}
              onCancel={() => { creatingProfile = false; editingProfile = null; }}
              onTest={(p) => {
                creatingProfile = false;
                editingProfile = null;
                onClose();
                onTestProfile?.(p.id);
              }}
              onDelete={(deletedId) => {
                profiles = profiles.filter((p) => p.id !== deletedId);
                creatingProfile = false;
                editingProfile = null;
              }}
            />
          {:else}
            <h2 class="section-title">Profiles</h2>
            <div class="profiles-section">
              <div class="default-profile-row">
                <label class="default-label" for="default-profile-select">Default profile for new tabs</label>
                <select id="default-profile-select" class="default-select" value={defaultProfile} onchange={onDefaultProfileChange}>
                  {#each profiles as p}
                    <option value={p.id}>{p.name}</option>
                  {/each}
                </select>
              </div>
              <div class="profiles-list">
                {#each profiles as p}
                  <div class="profile-row">
                    <span class="profile-name">{p.name}</span>
                    <span class="profile-shell">{p.shell}</span>
                    <Btn variant="ghost" size="sm" onclick={() => (editingProfile = p)}>Edit</Btn>
                  </div>
                {/each}
                <Btn variant="primary" onclick={() => (creatingProfile = true)}>+ New Profile</Btn>
              </div>
            </div>
          {/if}

        {:else if activeCategory === "terminal"}
          <h2 class="section-title">Terminal</h2>
          <TerminalSettings />

        {:else if activeCategory === "colorscheme"}
          <h2 class="section-title">Color Scheme</h2>
          <ThemeEditor />

        {:else if activeCategory === "keymap"}
          <h2 class="section-title">Keymap</h2>
          <div class="keymap-section">
            <input
              class="keymap-search"
              placeholder="Search shortcuts…"
              bind:value={keymapSearch}
              autocomplete="off"
              spellcheck={false}
            />
            <table class="keymap-table">
              <thead>
                <tr><th>Action</th><th>Windows / Linux</th><th>macOS</th></tr>
              </thead>
              <tbody>
                {#each visibleKeymap as row}
                  <tr>
                    <td>{row.action}</td>
                    <td><Kbd>{row.win}</Kbd></td>
                    <td><Kbd>{row.mac}</Kbd></td>
                  </tr>
                {/each}
              </tbody>
            </table>
            {#if visibleKeymap.length === 0}
              <p class="keymap-empty">No shortcuts match "{keymapSearch}"</p>
            {/if}
          </div>

        {:else if activeCategory === "agent"}
          <h2 class="section-title">Agent</h2>
          <AgentSettings />
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
    width: 820px;
    max-width: calc(100vw - 48px);
    height: 580px;
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

  /* ── Left nav ───────────────────────────────────────────── */
  .settings-nav {
    display: flex;
    flex-direction: column;
    width: 200px;
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
    padding: 8px 14px;
    text-align: left;
    border-radius: 4px;
    margin: 0 4px;
    transition: color 0.1s, background 0.1s;
    display: flex;
    align-items: center;
    gap: 9px;
  }

  .nav-item:hover { color: var(--fg, #e6edf3); background: rgba(255,255,255,0.04); }
  .nav-item.active { color: var(--fg, #e6edf3); background: rgba(255,255,255,0.08); }

  .nav-icon {
    width: 13px;
    height: 13px;
    flex-shrink: 0;
    opacity: 0.75;
  }

  .nav-item.active .nav-icon { opacity: 1; }

  /* ── Right content ──────────────────────────────────────── */
  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
    position: relative;
  }

  .section-title {
    margin: 0 0 16px;
    font-size: 15px;
    font-weight: 600;
    color: var(--fg, #e6edf3);
    padding-right: 32px; /* don't overlap close button */
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

  /* ── Profiles ───────────────────────────────────────────── */
  .profiles-section { display: flex; flex-direction: column; gap: 14px; }

  .default-profile-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: rgba(255,255,255,0.03);
    border: 1px solid var(--border, #21262d);
    border-radius: 6px;
  }

  .default-label {
    flex: 1;
    font-size: 13px;
    color: var(--fg, #e6edf3);
    white-space: nowrap;
  }

  .default-select {
    background: var(--bg, #0d1117);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    color: var(--fg, #e6edf3);
    font-size: 13px;
    padding: 5px 8px;
    outline: none;
    cursor: pointer;
  }

  .default-select:focus { border-color: var(--accent, #FF6B35); }

  .profiles-list { display: flex; flex-direction: column; gap: 6px; }

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

  /* ── Keymap ─────────────────────────────────────────────── */
  .keymap-section { display: flex; flex-direction: column; gap: 10px; }

  .keymap-search {
    background: rgba(255,255,255,0.04);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    color: var(--fg, #e6edf3);
    font-size: 12px;
    padding: 5px 8px;
    outline: none;
    width: 100%;
    box-sizing: border-box;
  }

  .keymap-search::placeholder { color: var(--fg-muted, #7d8590); }
  .keymap-search:focus { border-color: var(--accent, #FF6B35); }

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

  .keymap-empty {
    font-size: 12px;
    color: var(--fg-muted, #7d8590);
    text-align: center;
    padding: 12px 0;
    margin: 0;
  }
</style>
