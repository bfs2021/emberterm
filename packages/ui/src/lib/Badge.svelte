<script lang="ts">
  import type { Snippet } from "svelte";

  export type BadgeVariant = "ok" | "error" | "warning" | "agent" | "neutral";

  interface Props {
    variant?: BadgeVariant;
    /** Bold, uppercase, tag-shaped — use for risk/status labels (e.g. "Destructive"). */
    label?: boolean;
    /** Monospace font — use for numeric values (e.g. exit codes). */
    mono?: boolean;
    children: Snippet;
  }

  let { variant = "neutral", label = false, mono = false, children }: Props = $props();
</script>

<span class="badge" class:label class:mono data-variant={variant}>
  {@render children()}
</span>

<style>
  .badge {
    display: inline-block;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
    white-space: nowrap;
    flex-shrink: 0;
    line-height: 1.6;
  }

  /* Label mode: bold uppercase tag (risk badges) */
  .badge.label {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 3px 8px;
    border-radius: 4px;
  }

  .badge.mono {
    font-family: monospace;
  }

  /* Variants */
  .badge[data-variant="ok"] {
    background: color-mix(in srgb, var(--green, #3fb950) 12%, transparent);
    color: var(--green, #3fb950);
    border: 1px solid color-mix(in srgb, var(--green, #3fb950) 28%, transparent);
  }

  .badge[data-variant="error"] {
    background: color-mix(in srgb, var(--red, #f85149) 12%, transparent);
    color: var(--red, #f85149);
    border: 1px solid color-mix(in srgb, var(--red, #f85149) 28%, transparent);
  }

  .badge[data-variant="warning"] {
    background: rgba(210, 153, 34, 0.12);
    color: var(--yellow, #d29922);
    border: 1px solid rgba(210, 153, 34, 0.3);
  }

  .badge[data-variant="agent"] {
    background: color-mix(in srgb, var(--accent, #FF6B35) 15%, transparent);
    color: var(--accent, #FF6B35);
    border: 1px solid color-mix(in srgb, var(--accent, #FF6B35) 30%, transparent);
  }

  .badge[data-variant="neutral"] {
    background: rgba(255, 255, 255, 0.06);
    color: var(--fg-muted, #7d8590);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
</style>
