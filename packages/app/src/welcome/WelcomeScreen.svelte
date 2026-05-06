<script lang="ts">
  import { configUpdate } from "../ipc/client.js";
  import { Kbd } from "@emberterm/ui";

  interface Props {
    onComplete: () => void;
  }

  let { onComplete }: Props = $props();

  // ── Theme selection ─────────────────────────────────────────────────────────

  const THEMES = [
    { id: "ember-dark",  label: "Ember Dark",  bg: "#0d1117", fg: "#e6edf3", accent: "#FF6B35" },
    { id: "ember-light", label: "Ember Light",  bg: "#f6f8fa", fg: "#24292f", accent: "#FF6B35" },
    { id: "tokyo-night", label: "Tokyo Night",  bg: "#1a1b2e", fg: "#a9b1d6", accent: "#7aa2f7" },
  ] as const;

  let selectedTheme = $state("ember-dark");

  // ── Shell selection ─────────────────────────────────────────────────────────

  const isWindows = navigator.userAgent.includes("Windows");

  const SHELLS = isWindows
    ? ["pwsh.exe", "powershell.exe", "cmd.exe"]
    : ["/bin/zsh", "/bin/bash", "/bin/sh"];

  let selectedShell = $state(SHELLS[0]);

  // ── Agent toggle ────────────────────────────────────────────────────────────

  let agentEnabled = $state(true);

  // ── Submit ──────────────────────────────────────────────────────────────────

  let saving = $state(false);

  async function handleGetStarted() {
    saving = true;
    try {
      await configUpdate({
        appearance: { theme: selectedTheme },
        behavior: { default_profile: "default" },
        agent: { osc_7777_enabled: agentEnabled },
      });
      onComplete();
    } catch {
      saving = false;
    }
  }
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label="Welcome to EmberTerm">
  <div class="card">
    <div class="header">
      <span class="logo" aria-hidden="true">🔥</span>
      <h1>Welcome to EmberTerm</h1>
      <p class="tagline">Where shells and agents meet.</p>
    </div>

    <!-- Theme picker -->
    <section class="section">
      <h2>Choose a theme</h2>
      <div class="theme-row">
        {#each THEMES as theme}
          <button
            class="theme-swatch"
            class:selected={selectedTheme === theme.id}
            style="--swatch-bg: {theme.bg}; --swatch-accent: {theme.accent}; --swatch-fg: {theme.fg}"
            onclick={() => (selectedTheme = theme.id)}
            aria-pressed={selectedTheme === theme.id}
          >
            <span class="swatch-preview" aria-hidden="true" style="background:{theme.bg}">
              <span class="swatch-topbar" style="background:color-mix(in srgb,{theme.bg} 55%,#000 45%)">
                <span class="swatch-td"></span>
                <span class="swatch-td"></span>
                <span class="swatch-td"></span>
              </span>
              <span class="swatch-term">
                <span class="swatch-prow">
                  <span class="swatch-ps" style="color:{theme.accent}">$</span>
                  <span class="swatch-cur" style="background:{theme.fg}"></span>
                </span>
                <span class="swatch-ol" style="background:{theme.fg};width:72%"></span>
                <span class="swatch-ol" style="background:{theme.fg};width:50%"></span>
              </span>
            </span>
            <span class="swatch-label">{theme.label}</span>
          </button>
        {/each}
      </div>
    </section>

    <!-- Shell picker -->
    <section class="section">
      <h2>Default shell</h2>
      <select class="shell-select" bind:value={selectedShell}>
        {#each SHELLS as shell}
          <option value={shell}>{shell}</option>
        {/each}
      </select>
    </section>

    <!-- Agent integration -->
    <section class="section agent-section">
      <label class="toggle-row">
        <span class="toggle-label">
          <strong>Enable agent integration</strong>
          <span class="toggle-hint">OSC 7777 — inline tool cards, diff viewer, approvals</span>
        </span>
        <span class="toggle-wrap">
          <input type="checkbox" class="toggle-input" bind:checked={agentEnabled} id="agent-toggle" />
          <span class="toggle-track" aria-hidden="true"></span>
        </span>
      </label>
    </section>

    <!-- CTA -->
    <button class="cta" onclick={handleGetStarted} disabled={saving}>
      {saving ? "Setting up…" : "Get Started"}
    </button>

    <p class="hint">Press <Kbd>Ctrl+Shift+P</Kbd> to open the command palette at any time.</p>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
    animation: fade-in 160ms ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .card {
    background: var(--bg-elev, #010409);
    border: 1px solid var(--border, #21262d);
    border-radius: 12px;
    padding: 32px 36px;
    width: 420px;
    max-width: calc(100vw - 48px);
    display: flex;
    flex-direction: column;
    gap: 24px;
    animation: slide-up 260ms cubic-bezier(.2,.8,.2,1);
  }

  @keyframes slide-up {
    from { opacity: 0; transform: translateY(12px) scale(0.97); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .header {
    text-align: center;
  }

  .logo {
    font-size: 40px;
    display: block;
    margin-bottom: 10px;
  }

  h1 {
    margin: 0 0 6px;
    font-size: 20px;
    font-weight: 600;
    color: var(--fg, #e6edf3);
  }

  .tagline {
    margin: 0;
    font-size: 13px;
    color: var(--fg-muted, #7d8590);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  h2 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted, #7d8590);
  }

  /* Theme swatches */
  .theme-row {
    display: flex;
    gap: 12px;
  }

  .theme-swatch {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    background: var(--swatch-bg);
    border: 2px solid transparent;
    border-radius: 8px;
    padding: 10px 8px;
    cursor: pointer;
    transition: border-color 0.15s;
  }

  .theme-swatch.selected {
    border-color: var(--swatch-accent);
  }

  .theme-swatch:hover:not(.selected) {
    border-color: var(--border, #21262d);
  }

  .swatch-preview {
    width: 100%;
    height: 54px;
    border-radius: 4px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border: 1px solid rgba(255,255,255,0.06);
  }

  .swatch-topbar {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 0 5px;
    height: 10px;
    flex-shrink: 0;
  }

  .swatch-td {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: rgba(255,255,255,0.2);
    flex-shrink: 0;
  }

  .swatch-term {
    flex: 1;
    padding: 4px 6px 3px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .swatch-prow {
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .swatch-ps {
    font-size: 8px;
    font-family: monospace;
    line-height: 1;
    flex-shrink: 0;
  }

  .swatch-cur {
    width: 4px;
    height: 7px;
    opacity: 0.85;
    flex-shrink: 0;
  }

  .swatch-ol {
    display: block;
    height: 2px;
    border-radius: 1px;
    opacity: 0.22;
  }

  .swatch-label {
    font-size: 11px;
    color: var(--fg-muted, #7d8590);
    white-space: nowrap;
  }

  /* Shell select */
  .shell-select {
    background: var(--bg, #0d1117);
    border: 1px solid var(--border, #21262d);
    border-radius: 6px;
    color: var(--fg, #e6edf3);
    font-size: 13px;
    padding: 7px 10px;
    cursor: pointer;
    outline: none;
    transition: border-color 0.1s;
  }

  .shell-select:focus {
    border-color: var(--accent, #FF6B35);
  }

  /* Agent toggle */
  .agent-section {
    background: color-mix(in srgb, var(--accent, #FF6B35) 6%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent, #FF6B35) 20%, transparent);
    border-radius: 8px;
    padding: 12px 14px;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
  }

  .toggle-label {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-label strong {
    font-size: 13px;
    color: var(--fg, #e6edf3);
  }

  .toggle-hint {
    font-size: 11px;
    color: var(--fg-muted, #7d8590);
  }

  .toggle-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .toggle-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-track {
    display: block;
    width: 36px;
    height: 20px;
    border-radius: 10px;
    background: var(--border, #21262d);
    cursor: pointer;
    transition: background 0.2s;
    position: relative;
  }

  .toggle-track::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: white;
    transition: transform 0.2s;
  }

  .toggle-input:checked + .toggle-track {
    background: var(--accent, #FF6B35);
  }

  .toggle-input:checked + .toggle-track::after {
    transform: translateX(16px);
  }

  /* CTA */
  .cta {
    background: var(--accent, #FF6B35);
    border: none;
    border-radius: 8px;
    color: white;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    padding: 11px;
    transition: opacity 0.15s;
  }

  .cta:hover:not(:disabled) { opacity: 0.88; }
  .cta:disabled { opacity: 0.5; cursor: not-allowed; }

  .hint {
    margin: 0;
    font-size: 11px;
    color: var(--fg-muted, #7d8590);
    text-align: center;
  }

</style>
