<script lang="ts">
  import { onMount } from "svelte";
  import { Btn } from "@emberterm/ui";

  interface Props {
    title: string;
    message: string;
    confirmLabel: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let { title, message, confirmLabel, onConfirm, onCancel }: Props = $props();

  let cancelBtn: HTMLElement | undefined = $state();

  // Default to the safe choice: Enter on a freshly opened dialog must not close anything.
  onMount(() => cancelBtn?.querySelector("button")?.focus());

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); onCancel(); }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onkeydown={onKeydown}>
  <div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="cd-title" aria-describedby="cd-msg">
    <h2 id="cd-title">{title}</h2>
    <p id="cd-msg">{message}</p>
    <div class="actions">
      <span bind:this={cancelBtn}><Btn variant="ghost" onclick={onCancel}>Cancel</Btn></span>
      <Btn variant="danger" onclick={onConfirm}>{confirmLabel}</Btn>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 900;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
  }

  .dialog {
    width: min(380px, calc(100vw - 32px));
    padding: 18px 20px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--fg);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  }

  h2 { margin: 0 0 8px; font-size: 14px; font-weight: 600; }
  p { margin: 0 0 16px; font-size: 13px; line-height: 1.5; color: var(--fg-muted); }
  .actions { display: flex; justify-content: flex-end; gap: 8px; }
</style>
