<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "danger" | "ghost";
    size?: "sm" | "md";
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
  }

  let {
    variant = "ghost",
    size = "md",
    disabled = false,
    type = "button",
    onclick,
    children,
  }: Props = $props();
</script>

<button
  {type}
  {disabled}
  class="btn btn-{variant} btn-{size}"
  onclick={onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
    font-weight: 400;
    line-height: 1;
    white-space: nowrap;
    transition: opacity 0.1s, background 0.1s, color 0.1s, border-color 0.1s;
  }

  .btn:disabled { opacity: 0.5; cursor: not-allowed; }

  /* Sizes */
  .btn-md { padding: 6px 16px; }
  .btn-sm { padding: 3px 10px; font-size: 11px; }

  /* Variants */
  .btn-primary {
    background: var(--accent, #FF6B35);
    border: none;
    color: white;
  }

  .btn-primary:hover:not(:disabled) { opacity: 0.88; }

  .btn-ghost {
    background: transparent;
    border: 1px solid var(--border, #30363d);
    color: var(--fg-muted, #7d8590);
  }

  .btn-ghost:hover:not(:disabled) {
    color: var(--fg, #e6edf3);
    background: rgba(255, 255, 255, 0.04);
  }

  .btn-danger {
    background: color-mix(in srgb, var(--red, #f85149) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--red, #f85149) 35%, transparent);
    color: var(--red, #f85149);
  }

  .btn-danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--red, #f85149) 22%, transparent);
  }
</style>
