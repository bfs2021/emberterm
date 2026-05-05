<script lang="ts">
  interface Props {
    cwd?: string;
    gitBranch?: string;
    agentName?: string;
  }

  let { cwd = "", gitBranch = "", agentName = "" }: Props = $props();

  function shortenCwd(path: string): string {
    if (!path) return "~";
    // Replace home dir prefix with ~
    const home = path.match(/^[A-Za-z]:\\Users\\[^\\]+/) ?? path.match(/^\/Users\/[^\/]+/) ?? path.match(/^\/home\/[^\/]+/);
    if (home) {
      return "~" + path.slice(home[0].length);
    }
    return path;
  }
</script>

<footer class="status-bar">
  {#if cwd}
    <span class="status-item status-cwd">
      <span class="icon">⌂</span>
      {shortenCwd(cwd)}
    </span>
  {/if}

  {#if gitBranch}
    <span class="status-sep">·</span>
    <span class="status-item status-git">
      <span class="icon">⎇</span>
      {gitBranch}
    </span>
  {/if}

  {#if agentName}
    <span class="status-sep">·</span>
    <span class="status-item status-agent">
      <span class="agent-dot"></span>
      {agentName}
    </span>
  {/if}

  <span class="status-spacer"></span>

  <span class="status-item status-label">EmberTerm</span>
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 12px;
    background: var(--status-bg, #010409);
    border-top: 1px solid var(--border, #21262d);
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

  .icon {
    font-size: 10px;
    opacity: 0.7;
  }

  .status-sep {
    opacity: 0.4;
  }

  .status-spacer {
    flex: 1;
  }

  .status-label {
    opacity: 0.4;
    font-size: 10px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .status-agent {
    color: var(--accent, #FF6B35);
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
