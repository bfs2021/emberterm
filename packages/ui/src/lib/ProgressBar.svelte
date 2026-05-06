<script lang="ts">
  interface Props {
    /** Progress percentage, 0–100. */
    pct: number;
  }

  let { pct }: Props = $props();
</script>

<div
  class="progress-bar"
  role="progressbar"
  aria-valuenow={pct}
  aria-valuemin={0}
  aria-valuemax={100}
>
  <div class="progress-fill" style="width: {pct}%"></div>
</div>

<style>
  .progress-bar {
    height: 3px;
    background: var(--border, #30363d);
  }

  .progress-fill {
    height: 100%;
    background: var(--accent, #FF6B35);
    transition: width 0.3s ease;
  }
</style>
