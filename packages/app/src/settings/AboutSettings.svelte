<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Btn } from "@emberterm/ui";
  import { runUpdate, tauriUpdateDeps, type UpdateState } from "../utils/updater.js";

  const REPO = "https://github.com/emberterm/emberterm";

  let version = $state("");
  let update = $state<UpdateState>({ kind: "idle" });
  const busy = $derived(["checking", "downloading", "installing"].includes(update.kind));

  const status = $derived.by(() => {
    switch (update.kind) {
      case "checking": return "Checking for updates…";
      case "current": return "You're on the latest version.";
      case "downloading": return `Downloading v${update.version}${update.pct === null ? "…" : ` — ${update.pct}%`}`;
      case "installing": return `Installing v${update.version}; EmberTerm will restart.`;
      case "error": return `Update failed: ${update.message}`;
      default: return "";
    }
  });

  onMount(async () => {
    try { version = await getVersion(); } catch { /* shown without a version */ }
  });

  // The webview would navigate away from the app; hand the link to the system browser instead.
  function open(e: MouseEvent, url: string) {
    e.preventDefault();
    openUrl(url).catch(console.error);
  }
</script>

<div class="about">
  <header class="hero">
    <svg class="mark" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2.5 C 13 6, 16 7, 16.5 11 C 17 14.5, 14.8 17, 12 17 C 9.2 17, 7 14.5, 7.5 11 C 8 7.5, 12 7, 12 2.5 Z" fill="currentColor" opacity="0.95"/>
      <path d="M12 8.5 C 12.5 10.5, 14 11.2, 14 13.2 C 14 14.8, 13 16, 12 16 C 11 16, 10 14.8, 10 13.2 C 10 11.2, 11.5 11, 12 8.5 Z" fill="#fff" opacity="0.55"/>
    </svg>
    <div>
      <h3>EmberTerm</h3>
      <p class="sub">A fast, simple terminal{#if version}{" "}· v{version}{/if}</p>
    </div>
  </header>

  <p class="body">
    EmberTerm is a lightweight terminal for Windows. Run many tabs, each with its own shell
    (Git Bash, WSL, PowerShell), split a tab into panes, and get your last session back
    — tabs, shells and folders — when you reopen it. No account, no telemetry.
  </p>

  <section class="section">
    <h4>Updates</h4>
    <div class="update-row">
      <Btn variant="primary" disabled={busy} onclick={() => runUpdate(tauriUpdateDeps, (s) => (update = s))}>Check for updates</Btn>
      <span class="status" class:error={update.kind === "error"} role="status" aria-live="polite">{status}</span>
    </div>
  </section>

  <section class="section">
    <h4>Links</h4>
    <ul class="links">
      <li><a href={REPO} onclick={(e) => open(e, REPO)}>Source code on GitHub</a></li>
      <li><a href="{REPO}/issues" onclick={(e) => open(e, `${REPO}/issues`)}>Report a problem or request a feature</a></li>
    </ul>
  </section>

  <section class="section">
    <h4>Credits</h4>
    <p class="body">
      Built with Tauri, Svelte, xterm.js and portable-pty. Released under the MIT license.
    </p>
  </section>
</div>

<style>
  .about { display: flex; flex-direction: column; gap: 20px; max-width: 520px; }

  .hero { display: flex; align-items: center; gap: 16px; }

  .mark { width: 56px; height: 56px; color: var(--accent); flex-shrink: 0; }

  h3 { margin: 0; font-size: 20px; font-weight: 600; color: var(--fg); }
  .sub { margin: 2px 0 0; font-size: 13px; color: var(--fg-muted); }

  .section { display: flex; flex-direction: column; gap: 8px; }

  h4 {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted);
  }

  .body { margin: 0; font-size: 13px; line-height: 1.6; color: var(--fg); }

  .update-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  .status { font-size: 12px; color: var(--fg-muted); }
  .status.error { color: var(--red); }

  .links { margin: 0; padding: 0; list-style: none; display: flex; flex-direction: column; gap: 6px; }

  a { color: var(--accent); text-decoration: none; font-size: 13px; }
  a:hover { text-decoration: underline; }
  a:focus-visible { outline: 2px solid var(--accent-glow); outline-offset: 2px; border-radius: 2px; }
</style>
