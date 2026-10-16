<script lang="ts">
  import { toasts, dismissToast } from "../stores/app.js";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each $toasts as toast (toast.id)}
    <div class="toast">
      <span class="toast-msg">{toast.message}</span>
      <button class="toast-close" aria-label="Dismiss notification" onclick={() => dismissToast(toast.id)}>×</button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 12px;
    bottom: 36px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 440px;
    z-index: 1000;
  }
  .toast {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 10px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-left: 3px solid var(--yellow);
    border-radius: 6px;
    color: var(--fg);
    font-size: 12px;
  }
  .toast-msg { flex: 1; word-break: break-word; }
  .toast-close {
    background: none;
    border: none;
    color: var(--fg-muted);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
  }
  .toast-close:hover { color: var(--fg); }
</style>
