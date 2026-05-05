<script lang="ts">
  interface Props {
    shell?: string;
    cwd?: string;
    gitBranch?: string;
    agentName?: string;
  }

  let { shell = "", cwd = "", gitBranch = "", agentName = "" }: Props = $props();

  function shortenCwd(path: string): string {
    if (!path) return "~";
    const home = path.match(/^[A-Za-z]:\\Users\\[^\\]+/) ?? path.match(/^\/Users\/[^\/]+/) ?? path.match(/^\/home\/[^\/]+/);
    if (home) return "~" + path.slice(home[0].length);
    return path;
  }

  let clock = $state("");

  $effect(() => {
    function tick() {
      const now = new Date();
      const h = now.getHours().toString().padStart(2, "0");
      const m = now.getMinutes().toString().padStart(2, "0");
      clock = `${h}:${m}`;
    }
    tick();
    const id = setInterval(tick, 60_000);
    return () => clearInterval(id);
  });

  // Track whether any segment has been rendered (for separator logic)
  const hasShell = $derived(!!shell);
  const hasCwd = $derived(!!cwd);
  const hasGit = $derived(!!gitBranch);
  const hasAgent = $derived(!!agentName);
</script>

<footer class="status-bar">
  {#if hasShell}
    <span class="status-item status-shell">
      <svg class="status-icon" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <polyline points="2,4 5,6 2,8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
        <line x1="6" y1="8" x2="10" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
      </svg>
      {shell}
    </span>
  {/if}

  {#if hasShell && hasCwd}
    <span class="status-sep" aria-hidden="true">·</span>
  {/if}

  {#if hasCwd}
    <span class="status-item status-cwd">
      <svg class="status-icon" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="M6 1L11 5v6H8V8H4v3H1V5L6 1z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>
      </svg>
      {shortenCwd(cwd)}
    </span>
  {/if}

  {#if (hasShell || hasCwd) && hasGit}
    <span class="status-sep" aria-hidden="true">·</span>
  {/if}

  {#if hasGit}
    <span class="status-item status-git">
      <svg class="status-icon" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <circle cx="3" cy="3" r="1.5" stroke="currentColor" stroke-width="1.2"/>
        <circle cx="3" cy="9" r="1.5" stroke="currentColor" stroke-width="1.2"/>
        <circle cx="9" cy="3" r="1.5" stroke="currentColor" stroke-width="1.2"/>
        <path d="M3 4.5V7.5M3 4.5C3 6.5 9 5 9 4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
      {gitBranch}
    </span>
  {/if}

  {#if (hasShell || hasCwd || hasGit) && hasAgent}
    <span class="status-sep" aria-hidden="true">·</span>
  {/if}

  {#if hasAgent}
    <span class="status-item status-agent">
      <span class="agent-dot" aria-hidden="true"></span>
      {agentName}
    </span>
  {/if}

  <span class="status-spacer"></span>

  {#if clock}
    <span class="status-item status-clock" aria-label="Current time">{clock}</span>
  {/if}
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 26px;
    padding: 0 12px;
    background: var(--status-bg, #010409);
    border-top: 1px solid var(--border, #30363d);
    font-size: 11px;
    color: var(--status-fg, #7d8590);
    flex-shrink: 0;
    overflow: hidden;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .status-icon {
    width: 10px;
    height: 10px;
    opacity: 0.7;
    flex-shrink: 0;
  }

  .status-sep { opacity: 0.4; }

  .status-spacer { flex: 1; }

  .status-agent { color: var(--accent, #FF6B35); }

  .status-clock {
    font-variant-numeric: tabular-nums;
    opacity: 0.7;
  }

  .agent-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent, #FF6B35);
    animation: pulse 2s ease-in-out infinite;
    flex-shrink: 0;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>
