<script lang="ts">
  import { profileUpsert } from "../ipc/client.js";

  interface Profile {
    id: string;
    name: string;
    icon: string;
    shell: string;
    args: string[];
    cwd: string;
    env: Record<string, string>;
    theme?: string;
    font_family?: string;
    font_size?: number;
  }

  interface Props {
    profile?: Profile;
    onSave?: (profile: Profile) => void;
    onCancel?: () => void;
  }

  let { profile, onSave, onCancel }: Props = $props();

  let id = $state(profile?.id ?? crypto.randomUUID().slice(0, 8));
  let name = $state(profile?.name ?? "New Profile");
  let shell = $state(profile?.shell ?? "");
  let argsStr = $state((profile?.args ?? []).join(" "));
  let cwd = $state(profile?.cwd ?? "~");
  let saving = $state(false);
  let error = $state("");

  async function save() {
    if (!name.trim() || !shell.trim()) {
      error = "Name and shell are required.";
      return;
    }
    saving = true;
    error = "";
    const p: Profile = {
      id,
      name: name.trim(),
      icon: "terminal",
      shell: shell.trim(),
      args: argsStr.trim() ? argsStr.trim().split(/\s+/) : [],
      cwd: cwd.trim() || "~",
      env: {},
    };
    try {
      await profileUpsert(p);
      onSave?.(p);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="profile-editor">
  <h3>Profile</h3>

  <label class="field">
    <span class="label">Name</span>
    <input bind:value={name} placeholder="My Shell" />
  </label>

  <label class="field">
    <span class="label">Shell</span>
    <input bind:value={shell} placeholder="pwsh.exe, bash, zsh…" />
  </label>

  <label class="field">
    <span class="label">Arguments</span>
    <input bind:value={argsStr} placeholder="-NoLogo" />
  </label>

  <label class="field">
    <span class="label">Working directory</span>
    <input bind:value={cwd} placeholder="~" />
  </label>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="actions">
    <button class="btn-secondary" onclick={onCancel} disabled={saving}>Cancel</button>
    <button class="btn-primary" onclick={save} disabled={saving}>
      {saving ? "Saving…" : "Save"}
    </button>
  </div>
</div>

<style>
  .profile-editor { display: flex; flex-direction: column; gap: 12px; }
  h3 { margin: 0 0 4px; font-size: 14px; color: var(--fg, #e6edf3); }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .label {
    font-size: 11px;
    color: var(--fg-muted, #7d8590);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  input {
    background: var(--bg, #0d1117);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    color: var(--fg, #e6edf3);
    font-size: 13px;
    padding: 6px 10px;
    outline: none;
  }

  input:focus { border-color: var(--accent, #FF6B35); }

  .error { color: var(--danger, #f85149); font-size: 12px; margin: 0; }

  .actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 4px; }

  .btn-primary, .btn-secondary {
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    padding: 6px 16px;
  }

  .btn-primary {
    background: var(--accent, #FF6B35);
    color: #fff;
  }

  .btn-primary:disabled { opacity: 0.5; }

  .btn-secondary {
    background: transparent;
    border: 1px solid var(--border, #21262d);
    color: var(--fg-muted, #7d8590);
  }

  .btn-secondary:hover { color: var(--fg, #e6edf3); }
</style>
