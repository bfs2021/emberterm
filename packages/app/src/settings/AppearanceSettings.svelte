<script lang="ts">
  import { onMount } from "svelte";
  import { fontConfig, updateFontConfig } from "../theme/fontStore.js";
  import { configUpdate, configGet, fontList } from "../ipc/client.js";
  import { primaryFamily, stackFor, pickableFamilies, type InstalledFont } from "../utils/fontFamily.js";
  import NumberStepper from "./NumberStepper.svelte";

  let windowPadding = $state(8);

  // Installed fonts come from the OS via Rust. `null` = still loading, `[]` + loadFailed = fall back to typing.
  let installed = $state<InstalledFont[] | null>(null);
  let loadFailed = $state(false);
  let showAll = $state(false);

  const current = $derived(primaryFamily($fontConfig.fontFamily));
  const options = $derived(pickableFamilies(installed ?? [], showAll, current));
  const currentInstalled = $derived((installed ?? []).some((f) => f.name === current));

  onMount(async () => {
    try {
      const cfg = (await configGet()) as { appearance?: { window_padding?: number } };
      windowPadding = cfg?.appearance?.window_padding ?? 8;
    } catch { /* keep default */ }
    try {
      installed = await fontList();
    } catch {
      loadFailed = true;
    }
  });

  function pickFamily(e: Event) {
    const name = (e.currentTarget as HTMLSelectElement).value;
    updateFontConfig({ fontFamily: stackFor(name) }).catch(() => {});
  }

  function typeFamily(e: Event) {
    updateFontConfig({ fontFamily: (e.currentTarget as HTMLInputElement).value }).catch(() => {});
  }

  function savePadding(n: number) {
    windowPadding = n;
    configUpdate({ appearance: { window_padding: n } }).catch(() => {});
  }
</script>

<div class="appearance-settings">
  <section class="section" aria-labelledby="h-font">
    <h3 id="h-font">Font</h3>

    <div class="field">
      <label class="label" for="font-family">Family</label>
      {#if loadFailed}
        <input
          id="font-family"
          type="text"
          class="text"
          value={$fontConfig.fontFamily}
          oninput={typeFamily}
          placeholder="JetBrains Mono, monospace"
          spellcheck={false}
        />
        <span class="hint">Couldn't list installed fonts; type a font name instead.</span>
      {:else}
        <select id="font-family" class="select" value={current} onchange={pickFamily} disabled={installed === null}>
          {#if installed === null}
            <option value={current}>Loading fonts…</option>
          {:else}
            {#if !currentInstalled}
              <option value={current}>{current || "Default"} (not installed)</option>
            {/if}
            {#each options as f (f.name)}
              <option value={f.name} style:font-family={`"${f.name}", monospace`}>{f.name}</option>
            {/each}
          {/if}
        </select>
        <label class="check">
          <input type="checkbox" bind:checked={showAll} />
          <span>Show all fonts, not just fixed-width ones</span>
        </label>
      {/if}
    </div>

    <div class="field">
      <span class="label" id="lbl-preview">Preview</span>
      <div
        class="preview"
        role="img"
        aria-labelledby="lbl-preview"
        style:font-family={$fontConfig.fontFamily}
        style:font-size="{$fontConfig.fontSize}px"
        style:line-height={$fontConfig.lineHeight}
      >
        <div><span class="ok">~/code/emberterm</span> <span class="info">(main)</span></div>
        <div>$ cargo test</div>
        <div>test result: <span class="ok">ok</span>. 55 passed; 0 failed</div>
        <div class="dim">0O 1lI| {"{}"} [] () =&gt; != === -&gt; … ✓</div>
      </div>
    </div>

    <div class="row">
      <NumberStepper
        label="Size"
        suffix="px"
        value={$fontConfig.fontSize}
        min={8}
        max={48}
        onChange={(n) => updateFontConfig({ fontSize: n }).catch(() => {})}
      />
      <NumberStepper
        label="Line height"
        value={$fontConfig.lineHeight}
        min={1}
        max={2}
        step={0.1}
        decimals={1}
        onChange={(n) => updateFontConfig({ lineHeight: n }).catch(() => {})}
      />
    </div>
  </section>

  <section class="section" aria-labelledby="h-window">
    <h3 id="h-window">Window</h3>
    <NumberStepper label="Padding" suffix="px" value={windowPadding} min={0} max={64} step={2} onChange={savePadding} />
  </section>
</div>

<style>
  .appearance-settings { display: flex; flex-direction: column; gap: 24px; }

  .section { display: flex; flex-direction: column; gap: 14px; }

  h3 {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted);
  }

  .field { display: flex; flex-direction: column; gap: 6px; }
  .row { display: flex; gap: 24px; flex-wrap: wrap; }

  .label { font-size: 12px; color: var(--fg-muted); }
  .hint { font-size: 11px; color: var(--fg-muted); }

  .select, .text {
    width: 100%;
    box-sizing: border-box;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--fg);
    font: inherit;
    font-size: 13px;
    padding: 7px 10px;
    outline: none;
    color-scheme: dark;
  }

  .select option { background: var(--bg-elev-2); color: var(--fg); }

  .select:focus, .text:focus { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-glow); }
  .select:disabled { opacity: 0.6; }

  .check { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--fg-muted); cursor: pointer; }
  .check input { accent-color: var(--accent); color-scheme: dark; margin: 0; }

  .preview {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px 14px;
    color: var(--fg);
    white-space: pre;
    overflow: hidden;
  }

  .preview .ok { color: var(--green); }
  .preview .info { color: var(--blue); }
  .preview .dim { color: var(--fg-muted); }
</style>
