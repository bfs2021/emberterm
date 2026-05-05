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

  let focusIdx = $state(0);
  let sheetEl = $state<HTMLDivElement | undefined>(undefined);

  $effect(() => {
    if (sheetEl) {
      // Focus first button after mount
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
    } else if (key === "n" || key === "N") {
      const deny = actions.find(a => a === "deny");
      if (deny) { e.preventDefault(); act(deny); }
    }
  }

  function formatAction(a: string): string {
    return a.replace(/_/g, " ").replace(/\b\w/g, c => c.toUpperCase());
  }
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
    data-risk={risk}
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
  >
    <div class="approval-header">
      <span class="approval-title">{title}</span>
      <span class="risk-badge" data-risk={risk}>{RISK_LABEL[risk]}</span>
    </div>

    <div class="approval-body">
      <pre class="approval-detail">{detail}</pre>
    </div>

    <div class="approval-footer">
      {#each actions as action, i}
        <button
          class="approval-action"
          class:primary={i === 0}
          class:danger={action === "deny" || risk === "destructive" && i === 0}
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
  .approval-backdrop {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: flex-end;
    pointer-events: none;
    z-index: 100;
  }

  .approval-sheet {
    width: 100%;
    background: var(--bg-elev, #010409);
    border-top: 1px solid var(--border, #30363d);
    box-shadow: 0 -8px 32px rgba(0,0,0,0.4);
    pointer-events: all;
    animation: slide-up 240ms cubic-bezier(.2,.8,.2,1);
    outline: none;
  }

  @keyframes slide-up {
    from { transform: translateY(100%); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .approval-sheet[data-risk="destructive"] {
    border-top-color: color-mix(in srgb, var(--red, #f85149) 40%, var(--border, #30363d));
  }

  .approval-sheet[data-risk="caution"] {
    border-top-color: color-mix(in srgb, var(--yellow, #d29922) 40%, var(--border, #30363d));
  }

  .approval-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px 0;
  }

  .approval-title {
    flex: 1;
    font-size: 13px;
    font-weight: 500;
    color: var(--fg, #e6edf3);
  }

  .risk-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  .risk-badge[data-risk="safe"] {
    background: color-mix(in srgb, var(--green, #3fb950) 15%, transparent);
    color: var(--green, #3fb950);
  }

  .risk-badge[data-risk="caution"] {
    background: color-mix(in srgb, var(--yellow, #d29922) 15%, transparent);
    color: var(--yellow, #d29922);
  }

  .risk-badge[data-risk="destructive"] {
    background: color-mix(in srgb, var(--red, #f85149) 15%, transparent);
    color: var(--red, #f85149);
  }

  .approval-body {
    padding: 8px 14px;
  }

  .approval-detail {
    margin: 0;
    font-family: monospace;
    font-size: 12px;
    color: var(--fg-muted, #7d8590);
    background: var(--bg-block, rgba(255,255,255,0.03));
    border: 1px solid var(--border, #30363d);
    border-radius: 4px;
    padding: 8px 10px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 80px;
    overflow-y: auto;
  }

  .approval-footer {
    display: flex;
    gap: 6px;
    padding: 8px 14px 12px;
    justify-content: flex-end;
  }

  .approval-action {
    padding: 5px 14px;
    border-radius: 5px;
    border: 1px solid var(--border, #30363d);
    background: transparent;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    font-size: 12px;
    transition: background 0.1s, color 0.1s;
  }

  .approval-action:hover { color: var(--fg, #e6edf3); background: rgba(255,255,255,0.06); }
  .approval-action:focus { outline: 2px solid var(--accent, #FF6B35); outline-offset: 1px; }

  .approval-action.primary:not(.danger) {
    background: color-mix(in srgb, var(--accent, #FF6B35) 15%, transparent);
    border-color: color-mix(in srgb, var(--accent, #FF6B35) 40%, transparent);
    color: var(--accent, #FF6B35);
  }

  .approval-action.danger {
    background: color-mix(in srgb, var(--red, #f85149) 12%, transparent);
    border-color: color-mix(in srgb, var(--red, #f85149) 35%, transparent);
    color: var(--red, #f85149);
  }
</style>
