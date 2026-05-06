<script lang="ts">
  import { profileUpsert, profileDelete, type Profile } from "../ipc/client.js";
  import { Btn } from "@emberterm/ui";

  interface Props {
    profile?: Profile;
    onSave?: (profile: Profile) => void;
    onCancel?: () => void;
    onTest?: (profile: Profile) => void;
    onDelete?: (id: string) => void;
  }

  let { profile, onSave, onCancel, onTest, onDelete }: Props = $props();

  const ICON_OPTIONS = [
    { value: "terminal", label: "Terminal" },
    { value: "pwsh", label: "PowerShell" },
    { value: "bash", label: "Bash / Zsh" },
    { value: "agent", label: "Agent" },
  ];

  let id = $state(profile?.id ?? crypto.randomUUID().slice(0, 8));
  let name = $state(profile?.name ?? "New Profile");
  let icon = $state(profile?.icon ?? "terminal");
  let shell = $state(profile?.shell ?? "");
  let argsStr = $state((profile?.args ?? []).join(" "));
  let cwd = $state(profile?.cwd ?? "~");
  let envRows = $state<{ key: string; val: string }[]>(
    Object.entries(profile?.env ?? {}).map(([key, val]) => ({ key, val })),
  );
  let fontFamily = $state(profile?.font_family ?? "");
  let fontSize = $state(profile?.font_size?.toString() ?? "");
  let themeOverride = $state(profile?.theme ?? "");

  let saving = $state(false);
  let testing = $state(false);
  let deleting = $state(false);
  let confirmDelete = $state(false);
  let error = $state("");

  function buildProfile(): Profile {
    const env: Record<string, string> = {};
    for (const row of envRows) {
      const k = row.key.trim();
      if (k) env[k] = row.val;
    }
    return {
      id,
      name: name.trim(),
      icon,
      shell: shell.trim(),
      args: argsStr.trim() ? argsStr.trim().split(/\s+/) : [],
      cwd: cwd.trim() || "~",
      env,
      font_family: fontFamily.trim() || undefined,
      font_size: fontSize ? parseInt(fontSize, 10) : undefined,
      theme: themeOverride.trim() || undefined,
    };
  }

  function validate(): boolean {
    if (!name.trim() || !shell.trim()) { error = "Name and shell are required."; return false; }
    error = "";
    return true;
  }

  async function save() {
    if (!validate()) return;
    saving = true;
    try { const p = buildProfile(); await profileUpsert(p); onSave?.(p); }
    catch (e) { error = String(e); } finally { saving = false; }
  }

  async function test() {
    if (!validate()) return;
    testing = true;
    try { const p = buildProfile(); await profileUpsert(p); onTest?.(p); }
    catch (e) { error = String(e); } finally { testing = false; }
  }

  async function doDelete() {
    deleting = true;
    try { await profileDelete(id); onDelete?.(id); }
    catch (e) { error = String(e); deleting = false; confirmDelete = false; }
  }

  function addEnvRow() { envRows = [...envRows, { key: "", val: "" }]; }
  function removeEnvRow(i: number) { envRows = envRows.filter((_, idx) => idx !== i); }
</script>

<div class="profile-editor">
  <h3>{profile ? "Edit Profile" : "New Profile"}</h3>

  <!-- Basic fields -->
  <div class="field-row">
    <label class="field" style="flex:1">
      <span class="label">Name</span>
      <input bind:value={name} placeholder="My Shell" />
    </label>
    <label class="field" style="flex:0 0 140px">
      <span class="label">Icon</span>
      <select bind:value={icon}>
        {#each ICON_OPTIONS as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </label>
  </div>

  <label class="field">
    <span class="label">Shell</span>
    <input bind:value={shell} placeholder="pwsh.exe, bash, zsh…" />
  </label>

  <div class="field-row">
    <label class="field" style="flex:1">
      <span class="label">Arguments</span>
      <input bind:value={argsStr} placeholder="-NoLogo" />
    </label>
    <label class="field" style="flex:1">
      <span class="label">Working directory</span>
      <input bind:value={cwd} placeholder="~" />
    </label>
  </div>

  <!-- Environment variables -->
  <div class="field">
    <div class="env-header">
      <span class="label">Environment variables</span>
      <button class="add-env-btn" onclick={addEnvRow} type="button">+ Add</button>
    </div>
    {#if envRows.length > 0}
      <div class="env-rows">
        {#each envRows as row, i}
          <div class="env-row">
            <input class="env-key" bind:value={row.key} placeholder="KEY" spellcheck={false} />
            <span class="env-eq">=</span>
            <input class="env-val" bind:value={row.val} placeholder="value" spellcheck={false} />
            <button class="env-remove" onclick={() => removeEnvRow(i)} type="button" aria-label="Remove">×</button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Optional overrides -->
  <div class="overrides">
    <span class="label overrides-label">Overrides (optional)</span>
    <div class="field-row">
      <label class="field" style="flex:2">
        <span class="label">Font family</span>
        <input bind:value={fontFamily} placeholder="JetBrains Mono" />
      </label>
      <label class="field" style="flex:0 0 80px">
        <span class="label">Font size</span>
        <input bind:value={fontSize} type="number" min="6" max="72" placeholder="13" />
      </label>
    </div>
    <label class="field">
      <span class="label">Theme</span>
      <input bind:value={themeOverride} placeholder="tokyo-night" />
    </label>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="actions">
    {#if confirmDelete}
      <span class="confirm-msg">Delete "{name}"?</span>
      <Btn variant="danger" onclick={doDelete} disabled={deleting}>{deleting ? "Deleting…" : "Yes, delete"}</Btn>
      <Btn variant="ghost" onclick={() => (confirmDelete = false)} disabled={deleting}>Cancel</Btn>
    {:else}
      <div class="actions-left">
        {#if profile}
          <Btn variant="danger" onclick={() => (confirmDelete = true)}>Delete</Btn>
        {/if}
      </div>
      <div class="actions-right">
        <Btn variant="ghost" onclick={onCancel} disabled={saving || testing}>Cancel</Btn>
        {#if onTest}
          <Btn variant="ghost" onclick={test} disabled={saving || testing}>{testing ? "Opening…" : "Test"}</Btn>
        {/if}
        <Btn variant="primary" onclick={save} disabled={saving || testing}>{saving ? "Saving…" : "Save"}</Btn>
      </div>
    {/if}
  </div>
</div>

<style>
  .profile-editor { display: flex; flex-direction: column; gap: 12px; }
  h3 { margin: 0 0 4px; font-size: 14px; color: var(--fg, #e6edf3); }

  .field { display: flex; flex-direction: column; gap: 4px; }
  .field-row { display: flex; gap: 10px; }

  .label {
    font-size: 11px;
    color: var(--fg-muted, #7d8590);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  input, select {
    background: var(--bg, #0d1117);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    color: var(--fg, #e6edf3);
    font-size: 13px;
    padding: 6px 10px;
    outline: none;
    font-family: inherit;
    width: 100%;
    box-sizing: border-box;
  }

  input:focus, select:focus { border-color: var(--accent, #FF6B35); }
  input[type="number"] { padding: 6px 8px; }

  /* Env vars */
  .env-header { display: flex; align-items: center; justify-content: space-between; }
  .add-env-btn {
    background: transparent;
    border: none;
    color: var(--accent, #FF6B35);
    cursor: pointer;
    font-size: 11px;
    padding: 0;
  }
  .add-env-btn:hover { opacity: 0.8; }

  .env-rows { display: flex; flex-direction: column; gap: 4px; margin-top: 4px; }
  .env-row { display: flex; align-items: center; gap: 4px; }
  .env-key { flex: 0 0 120px; font-family: monospace !important; font-size: 12px !important; }
  .env-val { flex: 1; font-family: monospace !important; font-size: 12px !important; }
  .env-eq { color: var(--fg-muted, #7d8590); font-size: 13px; flex-shrink: 0; }
  .env-remove {
    background: transparent; border: none; color: var(--fg-muted, #7d8590);
    cursor: pointer; font-size: 16px; padding: 0 2px; line-height: 1; flex-shrink: 0;
  }
  .env-remove:hover { color: var(--red, #f85149); }

  /* Overrides */
  .overrides {
    display: flex; flex-direction: column; gap: 8px;
    border: 1px solid var(--border, #21262d);
    border-radius: 5px; padding: 10px;
  }
  .overrides-label { display: block; margin-bottom: 2px; }

  .error { color: var(--red, #f85149); font-size: 12px; margin: 0; }

  /* Actions */
  .actions { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
  .actions-left { flex: 1; }
  .actions-right { display: flex; gap: 8px; }
  .confirm-msg { flex: 1; font-size: 13px; color: var(--fg, #e6edf3); }
</style>
