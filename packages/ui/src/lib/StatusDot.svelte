<script lang="ts">
  export type DotStatus = "pending" | "running" | "ok" | "error" | "cancelled";

  interface Props {
    status?: DotStatus;
  }

  let { status = "pending" }: Props = $props();
</script>

<span class="status-dot" data-status={status} aria-hidden="true"></span>

<style>
  .status-dot {
    display: inline-block;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--fg-muted, #7d8590);
  }

  .status-dot[data-status="running"] {
    background: var(--accent, #FF6B35);
    animation: status-pulse 1.2s ease-in-out infinite;
  }

  .status-dot[data-status="ok"] {
    background: var(--green, #3fb950);
  }

  .status-dot[data-status="error"] {
    background: var(--red, #f85149);
  }

  .status-dot[data-status="cancelled"] {
    background: var(--fg-muted, #7d8590);
    opacity: 0.5;
  }

  @keyframes status-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.6; transform: scale(0.85); }
  }
</style>
