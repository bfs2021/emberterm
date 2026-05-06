<script lang="ts">
  import { tick } from "svelte";
  import { Badge, Kbd, Btn } from "@emberterm/ui";
  import type { BadgeVariant } from "@emberterm/ui";
  import { formatAction, resolveKey } from "./approval-utils.js";

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

  const RISK_VARIANT: Record<Risk, BadgeVariant> = {
    safe: "ok",
    caution: "warning",
    destructive: "error",
  };

  function btnVariant(action: string, idx: number): "primary" | "danger" | "ghost" {
    if (idx === 0) return "primary";
    if (action === "deny" || action === "cancel") return "danger";
    return "ghost";
  }

  let focusIdx = $state(0);
  let sheetEl = $state<HTMLDivElement | undefined>(undefined);

  $effect(() => {
    if (sheetEl) {
      tick().then(() => {
        const btns = sheetEl?.querySelectorAll<HTMLButtonElement>(".approval-footer button");
        btns?.[0]?.focus();
      });
    }
  });

  function act(result: string) {
    onAction(id, result);
  }

  function handleKeydown(e: KeyboardEvent) {
    const key = e.key;
    if (key === "ArrowLeft" || key === "ArrowRight") {
      e.preventDefault();
      const delta = key === "ArrowRight" ? 1 : -1;
      focusIdx = Math.max(0, Math.min(actions.length - 1, focusIdx + delta));
      const btns = sheetEl?.querySelectorAll<HTMLButtonElement>(".approval-footer button");
      btns?.[focusIdx]?.focus();
      return;
    }
    const resolved = resolveKey(key, actions);
    if (resolved !== null) {
      e.preventDefault();
      act(resolved);
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
      <Badge variant={RISK_VARIANT[risk]} label>{RISK_LABEL[risk]}</Badge>
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
        <Kbd>Y</Kbd> allow once
        · <Kbd>A</Kbd> always
        · <Kbd>N</Kbd> deny
      </span>
      <span class="footer-spacer"></span>
      {#each actions as action, i}
        <Btn
          variant={btnVariant(action, i)}
          onclick={() => act(action)}
          onfocus={() => { focusIdx = i; }}
        >
          {formatAction(action)}
        </Btn>
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


</style>
