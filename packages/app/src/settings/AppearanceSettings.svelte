<script lang="ts">
  import { fontConfig, updateFontConfig } from "../theme/fontStore.js";
  import { configUpdate, configGet } from "../ipc/client.js";
  import { applyOpacity } from "../stores/terminalConfigStore.js";
  import { onMount } from "svelte";

  // Window appearance config — separate from fontStore
  let windowPadding = $state(8);
  let acrylic = $state(true);
  let opacity = $state(95); // 0-100 integer for slider

  onMount(async () => {
    try {
      const cfg = await configGet() as {
        appearance?: { window_padding?: number; acrylic?: boolean; opacity?: number };
      };
      const a = cfg?.appearance;
      if (a) {
        windowPadding = a.window_padding ?? 8;
        acrylic = a.acrylic ?? true;
        opacity = Math.max(75, Math.round((a.opacity ?? 0.95) * 100));
        applyOpacity(opacity / 100);
      }
    } catch { /* keep defaults */ }
  });

  async function saveWindow() {
    await configUpdate({
      appearance: {
        window_padding: windowPadding,
        acrylic,
        opacity: opacity / 100,
      },
    });
  }

  function onFontFamilyInput(e: Event) {
    updateFontConfig({ fontFamily: (e.currentTarget as HTMLInputElement).value }).catch(() => {});
  }

  function onFontSizeInput(e: Event) {
    const n = parseInt((e.currentTarget as HTMLInputElement).value, 10);
    if (n >= 8 && n <= 72) updateFontConfig({ fontSize: n }).catch(() => {});
  }

  function onLineHeightInput(e: Event) {
    const n = parseFloat((e.currentTarget as HTMLInputElement).value);
    if (n >= 0.8 && n <= 3.0) updateFontConfig({ lineHeight: n }).catch(() => {});
  }

  function onPaddingInput(e: Event) {
    const n = parseInt((e.currentTarget as HTMLInputElement).value, 10);
    if (n >= 0 && n <= 64) { windowPadding = n; saveWindow().catch(() => {}); }
  }

  function onOpacityInput(e: Event) {
    opacity = parseInt((e.currentTarget as HTMLInputElement).value, 10);
    applyOpacity(opacity / 100);
    saveWindow().catch(() => {});
  }

  function toggleAcrylic() {
    acrylic = !acrylic;
    saveWindow().catch(() => {});
  }
</script>

<div class="appearance-settings">
  <!-- Font -->
  <section class="section">
    <h3>Font</h3>
    <label class="field">
      <span class="label">Family</span>
      <input
        type="text"
        value={$fontConfig.fontFamily}
        oninput={onFontFamilyInput}
        placeholder="JetBrains Mono, monospace"
        spellcheck={false}
      />
    </label>
    <div class="field-row">
      <label class="field">
        <span class="label">Size (px)</span>
        <input
          type="number"
          min="8" max="72"
          value={$fontConfig.fontSize}
          oninput={onFontSizeInput}
        />
      </label>
      <label class="field">
        <span class="label">Line height</span>
        <input
          type="number"
          min="0.8" max="3.0" step="0.05"
          value={$fontConfig.lineHeight}
          oninput={onLineHeightInput}
        />
      </label>
    </div>
  </section>

  <!-- Window -->
  <section class="section">
    <h3>Window</h3>

    <label class="field">
      <span class="label">Padding (px)</span>
      <input
        type="number"
        min="0" max="64"
        value={windowPadding}
        oninput={onPaddingInput}
        style="max-width: 80px;"
      />
    </label>

    <div class="toggle-row" role="button" tabindex="0"
      onclick={toggleAcrylic}
      onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleAcrylic(); } }}
    >
      <span class="toggle-label">
        <span class="toggle-title">Acrylic / Mica</span>
        <span class="toggle-hint">Translucent window effect (Windows 11 / macOS)</span>
      </span>
      <span class="toggle-wrap">
        <input type="checkbox" class="toggle-input" checked={acrylic} readonly id="acrylic-toggle" />
        <span class="toggle-track" aria-hidden="true"></span>
      </span>
    </div>

    <label class="field">
      <span class="label">Opacity — {opacity}%</span>
      <input
        type="range"
        min="75" max="100"
        value={opacity}
        oninput={onOpacityInput}
        class="range-input"
      />
    </label>
  </section>
</div>

<style>
  .appearance-settings { display: flex; flex-direction: column; gap: 20px; }

  .section { display: flex; flex-direction: column; gap: 10px; }

  h3 {
    margin: 0 0 2px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted, #7d8590);
  }

  .field { display: flex; flex-direction: column; gap: 4px; }
  .field-row { display: flex; gap: 10px; }

  .label {
    font-size: 12px;
    color: var(--fg-muted, #7d8590);
  }

  input[type="text"], input[type="number"] {
    background: var(--bg, #0d1117);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    color: var(--fg, #e6edf3);
    font-size: 13px;
    padding: 5px 8px;
    outline: none;
    width: 100%;
    box-sizing: border-box;
    font-family: inherit;
  }

  input[type="text"]:focus, input[type="number"]:focus {
    border-color: var(--accent, #FF6B35);
  }

  /* Toggle row */
  .toggle-row {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    outline: none;
    padding: 4px 0;
  }

  .toggle-row:focus-visible { outline: 2px solid var(--accent-glow, rgba(255,107,53,0.5)); border-radius: 4px; }

  .toggle-label {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-title { font-size: 13px; color: var(--fg, #e6edf3); }

  .toggle-hint { font-size: 11px; color: var(--fg-muted, #7d8590); }

  .toggle-wrap { position: relative; flex-shrink: 0; }

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

  .toggle-input:checked + .toggle-track { background: var(--accent, #FF6B35); }
  .toggle-input:checked + .toggle-track::after { transform: translateX(16px); }

  /* Range slider */
  .range-input {
    -webkit-appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--border, #21262d);
    outline: none;
    cursor: pointer;
    margin-top: 4px;
  }

  .range-input::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent, #FF6B35);
    cursor: pointer;
  }
</style>
