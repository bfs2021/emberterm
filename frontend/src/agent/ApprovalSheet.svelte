<script lang="ts">
  import { tick } from "svelte";

  type Risk = "safe" | "caution" | "destructive";

  interface Props {
    id: string;
    title: string;
    detail: string;
    risk: Risk;
    actions: string[];
    onAction: (id: string, result: string) => void;
  }

  let { id, title, detail, risk, actions, onAction }: Props = $props();

  const RISK_LABEL: Record<Risk, string> = {
    safe: "Safe",
    caution: "Caution",
    destructive: "Destructive",
  };

  // Map action keys to display labels
  function formatAction(a: string): string {
    return a.replace(/_/g, " ").replace(/\b\w/g, c => c.toUpperCase());
  }

  // Button style class per action
  function btnClass(a: string, i: number): string {
    if (a === "deny") return "btn btn-danger";
    if (i === 0 || a.startsWith("allow_session") || a === "allow_always") return "btn btn-primary";
    if (a.startsWith("allow")) return "btn btn-ghost";
    return "btn";
  }

  let focusIdx = $state(0);
  let sheetEl = $state<HTMLDivElement | undefined>(undefined);

  $effect(() => {
    if (sheetEl) {
      tick().then(() => {
        const btns = sheetEl?.querySelectorAll<HTMLButtonElement>(".approval-action");
        btns?.[0]?.focus();
      });
    }
  });

  function act(result: string) {
    onAction(id, result);
  }

  function handleKeydown(e: KeyboardEvent) {
    const key = e.key;
    if (key === "Escape") {
      e.preventDefault();
      const deny = actions.find(a => a === "deny") ?? actions.at(-1);
      if (deny) act(deny);
    } else if (key === "ArrowLeft" || key === "ArrowRight") {
      e.preventDefault();
      const delta = key === "ArrowRight" ? 1 : -1;
      focusIdx = Math.max(0, Math.min(actions.length - 1, focusIdx + delta));
      const btns = sheetEl?.querySelectorAll<HTMLButtonElement>(".approval-action");
      btns?.[focusIdx]?.focus();
    } else if (key === "y" || key === "Y") {
      const allow = actions.find(a => a.startsWith("allow"));
      if (allow) { e.preventDefault(); act(allow); }
    } else if (key === "a" || key === "A") {
      const always = actions.find(a => a === "allow_always" || a === "allow_session");
      if (always) { e.preventDefault(); act(always); }
    } else if (key === "n" || key === "N") {
      const deny = actions.find(a => a === "deny");
      if (deny) { e.preventDefault(); act(deny); }
    }
  }

  // Check if detail looks like a command (no spaces at start, looks like a shell command)
  const isCommand = $derived(detail.trim().split("\n").every(l => l.length < 120));
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="approval-backdrop"
  onkeydown={handleKeydown}
  role="presentation"
>
  <div
    bind:this={sheetEl}
    class="approval-sheet"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
  >
    <!-- Header: risk badge + title -->
    <div class="approval-header">
      <span class="risk-badge" data-risk={risk}>{RISK_LABEL[risk]}</span>
      <span class="approval-title">{title}</span>
    </div>

    <!-- Body: detail -->
    <div class="approval-body">
      <p class="approval-detail-text">What would you like to do?</p>
      <pre class="approval-detail">{detail}</pre>
    </div>

    <!-- Footer: hint + actions -->
    <div class="approval-footer">
      <span class="approval-hint">
        <span class="kbd">Y</span> allow once
        · <span class="kbd">A</span> always
        · <span class="kbd">N</span> deny
      </span>
      <span class="footer-spacer"></span>
      {#each actions as action, i}
        <button
          class="approval-action {btnClass(action, i)}"
          onclick={() => act(action)}
          onfocus={() => { focusIdx = i; }}
        >
          {formatAction(action)}
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  /* ── Backdrop: positions sheet at bottom-center ── */
  .approval-backdrop {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding: 24px;
    pointer-events: none;
    z-index: 100;
  }

  /* ── Floating card ────────────────────────────── */
  .approval-sheet {
    pointer-events: auto;
    width: 100%;
    max-width: 540px;
    background: var(--bg-elev-2, #161b22);
    border: 1px solid var(--border, #21262d);
    border-radius: var(--radius-lg, 14px);
    box-shadow: var(--shadow-lg, 0 24px 60px -12px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.04));
    overflow: hidden;
    animation: slide-up 220ms cubic-bezier(.2,.8,.2,1);
    outline: none;
  }

  @keyframes slide-up {
    from { transform: translateY(20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  /* ── Header ───────────────────────────────────── */
  .approval-header {
    padding: 14px 16px 10px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .risk-badge {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 3px 8px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .risk-badge[data-risk="safe"] {
    background: rgba(126, 231, 135, 0.12);
    color: var(--green, #7ee787);
    border: 1px solid rgba(126, 231, 135, 0.3);
  }

  .risk-badge[data-risk="caution"] {
    background: rgba(210, 153, 34, 0.12);
    color: var(--yellow, #d29922);
    border: 1px solid rgba(210, 153, 34, 0.3);
  }

  .risk-badge[data-risk="destructive"] {
    background: rgba(255, 123, 114, 0.12);
    color: var(--red, #ff7b72);
    border: 1px solid rgba(255, 123, 114, 0.3);
  }

  .approval-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--fg, #e6edf3);
    flex: 1;
    min-width: 0;
  }

  /* ── Body ─────────────────────────────────────── */
  .approval-body {
    padding: 4px 16px 14px;
  }

  .approval-detail-text {
    color: var(--fg-muted, #7d8590);
    font-size: 12.5px;
    margin: 0 0 12px;
  }

  .approval-detail {
    margin: 0;
    font-family: var(--mono, monospace);
    font-size: 12px;
    background: var(--bg, #0d1117);
    border: 1px solid var(--border, #21262d);
    border-radius: 6px;
    padding: 9px 12px;
    color: var(--fg-muted, #7d8590);
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 100px;
    overflow-y: auto;
  }

  /* ── Footer ───────────────────────────────────── */
  .approval-footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--bg-elev, #010409);
    border-top: 1px solid var(--border, #21262d);
  }

  .approval-hint {
    font-size: 11.5px;
    color: var(--fg-muted, #7d8590);
    white-space: nowrap;
  }

  .footer-spacer { flex: 1; min-width: 4px; }

  /* ── kbd spans ────────────────────────────────── */
  .kbd {
    display: inline-block;
    padding: 1px 5px;
    background: var(--bg-elev-2, #161b22);
    border: 1px solid var(--border, #21262d);
    border-radius: 4px;
    font-family: var(--mono, monospace);
    font-size: 10.5px;
    color: var(--fg-muted, #7d8590);
  }

  /* ── Buttons ──────────────────────────────────── */
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 6px;
    border: 1px solid var(--border, #21262d);
    background: var(--bg-elev-2, #161b22);
    color: var(--fg, #e6edf3);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 120ms, border-color 120ms, color 120ms;
    white-space: nowrap;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--fg-dim, #484f58);
  }

  .btn:focus {
    outline: 2px solid var(--accent, #FF6B35);
    outline-offset: 1px;
  }

  .btn.btn-primary {
    background: var(--accent, #FF6B35);
    border-color: var(--accent, #FF6B35);
    color: #1a0a04;
    font-weight: 600;
  }

  .btn.btn-primary:hover {
    background: color-mix(in srgb, var(--accent, #FF6B35) 85%, #fff);
    border-color: color-mix(in srgb, var(--accent, #FF6B35) 85%, #fff);
  }

  .btn.btn-danger {
    background: transparent;
    color: var(--red, #ff7b72);
    border-color: rgba(255, 123, 114, 0.3);
  }

  .btn.btn-danger:hover {
    background: rgba(255, 123, 114, 0.08);
    border-color: rgba(255, 123, 114, 0.5);
  }

  .btn.btn-ghost {
    background: transparent;
  }

  .btn.btn-ghost:hover {
    background: rgba(255, 255, 255, 0.06);
  }
</style>
