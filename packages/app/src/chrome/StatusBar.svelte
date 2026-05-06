<script lang="ts">
  interface Props {
    shell?: string;
    cwd?: string;
    gitBranch?: string;
    agentName?: string;
    termCols?: number;
    termRows?: number;
  }

  let { shell = "", cwd = "", gitBranch = "", agentName = "", termCols = 0, termRows = 0 }: Props = $props();

  function shortenCwd(path: string): string {
    if (!path) return "~";
    const home = path.match(/^[A-Za-z]:\\Users\\[^\\]+/) ?? path.match(/^\/Users\/[^/]+/) ?? path.match(/^\/home\/[^/]+/);
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

  const hasShell = $derived(!!shell);
  const hasCwd = $derived(!!cwd);
  const hasGit = $derived(!!gitBranch);
  const hasAgent = $derived(!!agentName);
  const hasSize = $derived(termCols > 0 && termRows > 0);
</script>

<footer class="status-bar">
  <!-- Left items: shell · cwd · git · agent -->
  {#if hasShell}
    <span class="status-item status-shell">
      <!-- shell/terminal icon -->
      <svg class="status-icon" width="11" height="11" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <polyline points="4 17 10 11 4 5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <line x1="12" y1="19" x2="20" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
      {shell}
    </span>
  {/if}

  {#if hasCwd}
    <span class="status-item status-cwd" class:first-item={!hasShell}>
      <!-- folder/cwd icon -->
      <svg class="status-icon" width="11" height="11" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M3 7l9-4 9 4M3 7v10l9 4 9-4V7M3 7l9 4 9-4M12 11v10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      {shortenCwd(cwd)}
    </span>
  {/if}

  {#if hasGit}
    <span class="status-item status-git" class:first-item={!hasShell && !hasCwd}>
      <!-- git branch icon -->
      <svg class="status-icon" width="11" height="11" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx="6" cy="6" r="3" stroke="currentColor" stroke-width="2"/>
        <circle cx="6" cy="18" r="3" stroke="currentColor" stroke-width="2"/>
        <circle cx="18" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
        <path d="M6 9v6M18 9c-3 0-6-1-6-3M18 15c-3 0-6 1-6 3" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
      {gitBranch}
    </span>
  {/if}

  {#if hasAgent}
    <span class="status-item status-agent" class:first-item={!hasShell && !hasCwd && !hasGit}>
      <span class="agent-dot" aria-hidden="true"></span>
      {agentName}
    </span>
  {/if}

  <span class="status-spacer"></span>

  <!-- Right items: size · encoding · hint · clock -->
  {#if hasSize}
    <span class="status-item status-size">{termCols}×{termRows}</span>
  {/if}

  <span class="status-item status-enc">
    <span class="c-dim">UTF-8 · LF</span>
  </span>

  <span class="status-item status-hint clickable">
    <svg class="status-icon" width="11" height="11" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
      <line x1="21" y1="21" x2="16.65" y2="16.65" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>
    <span>Ctrl+Shift+P</span>
  </span>

  {#if clock}
    <span class="status-item status-clock" aria-label="Current time">{clock}</span>
  {/if}
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    height: 26px;
    padding: 0 12px;
    background: var(--bg-elev, #010409);
    border-top: 1px solid var(--border, #21262d);
    font-size: 11.5px;
    color: var(--fg-muted, #7d8590);
    flex-shrink: 0;
    overflow: hidden;
    font-family: var(--sans, system-ui, sans-serif);
  }

  /* Each item: flex row, vertically centered, with left border as separator */
  .status-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px;
    height: 100%;
    white-space: nowrap;
    border-left: 1px solid var(--border-subtle, #1a1f27);
  }

  /* First item on each side has no left border */
  .status-item:first-child,
  .status-item.first-item {
    border-left: none;
  }

  /* Spacer pushes right items to the right; items after it reset border */
  .status-spacer { flex: 1; min-width: 8px; }

  /* Items after spacer (right side) — first right item should have no extra border */
  .status-spacer + .status-item {
    border-left: none;
  }

  /* Clickable items */
  .clickable {
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .clickable:hover {
    background: rgba(255, 255, 255, 0.04);
    color: var(--fg, #e6edf3);
  }

  .status-icon {
    opacity: 0.7;
    flex-shrink: 0;
  }

  /* Agent item */
  .status-agent {
    color: var(--accent, #FF6B35);
  }

  .agent-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent, #FF6B35);
    box-shadow: 0 0 6px var(--accent-glow, rgba(255, 107, 53, 0.35));
    flex-shrink: 0;
  }

  /* Dim helper */
  .c-dim {
    color: var(--fg-dim, #484f58);
  }

  /* Clock */
  .status-clock {
    font-variant-numeric: tabular-nums;
  }
</style>
