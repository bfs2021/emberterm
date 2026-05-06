<script lang="ts">
  import { agentConfigStore, updateAgentConfig } from "../stores/app.js";

  function toggleOsc() {
    updateAgentConfig({ oscEnabled: !$agentConfigStore.oscEnabled }).catch(() => {});
  }

  function toggleAutoAccept() {
    updateAgentConfig({ autoAccept: !$agentConfigStore.autoAccept }).catch(() => {});
  }
</script>

<div class="agent-settings">

  <section class="section">
    <h3>OSC 7777 Integration</h3>

    <div class="toggle-row" role="button" tabindex="0"
      onclick={toggleOsc}
      onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleOsc(); } }}
    >
      <span class="toggle-label">
        <span class="toggle-title">Enable agent integration</span>
        <span class="toggle-hint">Inline tool cards, diff viewer, and approval sheets via OSC 7777</span>
      </span>
      <span class="toggle-wrap">
        <input type="checkbox" class="toggle-input" checked={$agentConfigStore.oscEnabled} readonly />
        <span class="toggle-track" aria-hidden="true"></span>
      </span>
    </div>

    <div class="toggle-row" role="button" tabindex="0"
      onclick={toggleAutoAccept}
      onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleAutoAccept(); } }}
    >
      <span class="toggle-label">
        <span class="toggle-title">Auto-accept safe tool calls</span>
        <span class="toggle-hint">Automatically approve tool calls marked as safe-risk</span>
      </span>
      <span class="toggle-wrap">
        <input type="checkbox" class="toggle-input" checked={$agentConfigStore.autoAccept} readonly />
        <span class="toggle-track" aria-hidden="true"></span>
      </span>
    </div>
  </section>

  <section class="section">
    <h3>Protocol</h3>
    <dl class="info-list">
      <div class="info-row">
        <dt>Spec version</dt>
        <dd>0.1</dd>
      </div>
      <div class="info-row">
        <dt>Environment</dt>
        <dd><code>TERM_OSC_7777=1</code> set in all sessions</dd>
      </div>
      <div class="info-row">
        <dt>Wire format</dt>
        <dd><code>ESC ] 7777 ; verb ; json ESC \</code></dd>
      </div>
    </dl>
  </section>

</div>

<style>
  .agent-settings { display: flex; flex-direction: column; gap: 20px; }

  .section { display: flex; flex-direction: column; gap: 10px; }

  h3 {
    margin: 0 0 2px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted, #7d8590);
  }

  /* Toggle rows */
  .toggle-row {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    outline: none;
    padding: 2px 0;
  }

  .toggle-row:focus-visible { outline: 2px solid var(--accent-glow, rgba(255,107,53,0.5)); border-radius: 4px; }

  .toggle-label { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .toggle-title { font-size: 13px; color: var(--fg, #e6edf3); }
  .toggle-hint { font-size: 11px; color: var(--fg-muted, #7d8590); }

  .toggle-wrap { position: relative; flex-shrink: 0; }
  .toggle-input { position: absolute; opacity: 0; width: 0; height: 0; }

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

  /* Info list */
  .info-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
  }

  .info-row {
    display: flex;
    align-items: baseline;
    gap: 10px;
    font-size: 12px;
  }

  dt {
    color: var(--fg-muted, #7d8590);
    flex-shrink: 0;
    min-width: 100px;
  }

  dd {
    color: var(--fg, #e6edf3);
    margin: 0;
  }

  code {
    font-family: var(--mono, monospace);
    font-size: 11px;
    background: var(--bg-block, rgba(255,255,255,0.04));
    border: 1px solid var(--border, #21262d);
    border-radius: 3px;
    padding: 1px 5px;
    color: var(--fg-muted, #7d8590);
  }
</style>
