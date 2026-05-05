<script lang="ts">
  interface Props {
    name: string;
    version: string;
    onDismiss: () => void;
  }

  let { name, version, onDismiss }: Props = $props();

  let visible = $state(true);

  $effect(() => {
    const id = setTimeout(() => {
      visible = false;
      setTimeout(onDismiss, 300);
    }, 3000);
    return () => clearTimeout(id);
  });
</script>

{#if visible}
  <div class="hello-banner" role="status" aria-live="polite">
    <span class="agent-dot" aria-hidden="true"></span>
    <span class="hello-text">
      <strong>{name}</strong>
      <span class="hello-version">v{version}</span>
      <span class="hello-label">connected</span>
    </span>
    <button class="dismiss-btn" aria-label="Dismiss" onclick={() => { visible = false; setTimeout(onDismiss, 300); }}>
      ×
    </button>
  </div>
{/if}

<style>
  .hello-banner {
    position: absolute;
    bottom: 34px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px 6px 10px;
    background: var(--bg-elev, #010409);
    border: 1px solid color-mix(in srgb, var(--accent, #FF6B35) 35%, var(--border, #30363d));
    border-radius: 20px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.4);
    font-size: 12px;
    white-space: nowrap;
    z-index: 50;
    animation: pop-in 220ms cubic-bezier(.2,.8,.2,1);
  }

  @keyframes pop-in {
    from { opacity: 0; transform: translateX(-50%) translateY(6px) scale(0.95); }
    to { opacity: 1; transform: translateX(-50%) translateY(0) scale(1); }
  }

  .agent-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent, #FF6B35);
    flex-shrink: 0;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .hello-text {
    display: flex;
    align-items: center;
    gap: 5px;
    color: var(--fg, #e6edf3);
  }

  strong { font-weight: 500; }

  .hello-version {
    color: var(--fg-muted, #7d8590);
    font-size: 11px;
  }

  .hello-label {
    color: var(--accent, #FF6B35);
    font-size: 11px;
  }

  .dismiss-btn {
    background: transparent;
    border: none;
    color: var(--fg-muted, #7d8590);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0 0 0 2px;
    flex-shrink: 0;
  }

  .dismiss-btn:hover { color: var(--fg, #e6edf3); }
</style>
