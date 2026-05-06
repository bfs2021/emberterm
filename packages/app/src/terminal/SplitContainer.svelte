<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    direction: "horizontal" | "vertical";
    /** Fraction of space given to the primary pane, 0.1–0.9. */
    ratio?: number;
    primary: Snippet;
    secondary: Snippet;
    /** Called when the user closes the secondary pane via the close button. */
    onClose: () => void;
  }

  let { direction, ratio = $bindable(0.5), primary, secondary, onClose }: Props = $props();

  let containerEl: HTMLElement | undefined = $state();
  let dragging = $state(false);

  function clamp(v: number) { return Math.max(0.1, Math.min(0.9, v)); }

  function onDividerMousedown(e: MouseEvent) {
    e.preventDefault();
    dragging = true;

    function onMove(mv: MouseEvent) {
      if (!containerEl) return;
      const rect = containerEl.getBoundingClientRect();
      if (direction === "horizontal") {
        ratio = clamp((mv.clientX - rect.left) / rect.width);
      } else {
        ratio = clamp((mv.clientY - rect.top) / rect.height);
      }
    }

    function onUp() {
      dragging = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  const isHoriz = $derived(direction === "horizontal");
</script>

<div
  class="split-container"
  class:dragging
  class:horizontal={isHoriz}
  class:vertical={!isHoriz}
  bind:this={containerEl}
>
  <div class="pane pane-primary" style:flex="{ratio} 1 0">
    {@render primary()}
  </div>

  <!-- ARIA 1.1: role="separator" with tabindex+aria-valuenow is the interactive splitter pattern. -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="split-divider"
    role="separator"
    aria-orientation={isHoriz ? "vertical" : "horizontal"}
    aria-valuenow={Math.round(ratio * 100)}
    aria-valuemin={10}
    aria-valuemax={90}
    aria-label={isHoriz ? "Resize panes horizontally" : "Resize panes vertically"}
    tabindex="0"
    onmousedown={onDividerMousedown}
    onkeydown={(e) => {
      const step = 0.05;
      if (e.key === "ArrowLeft" || e.key === "ArrowUp") { e.preventDefault(); ratio = clamp(ratio - step); }
      if (e.key === "ArrowRight" || e.key === "ArrowDown") { e.preventDefault(); ratio = clamp(ratio + step); }
    }}
  >
    <div class="divider-track"></div>
  </div>

  <div class="pane pane-secondary" style:flex="{1 - ratio} 1 0">
    {@render secondary()}
    <button class="pane-close" onclick={onClose} aria-label="Close secondary pane" title="Close pane">✕</button>
  </div>
</div>

<style>
  .split-container {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .split-container.horizontal { flex-direction: row; }
  .split-container.vertical   { flex-direction: column; }

  .pane {
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .split-divider {
    flex: 0 0 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: col-resize;
    background: transparent;
    z-index: 5;
  }

  .horizontal .split-divider { cursor: col-resize; }
  .vertical   .split-divider { cursor: row-resize; }

  .divider-track {
    background: var(--border, rgba(255,255,255,0.12));
    border-radius: 2px;
    transition: background 0.15s;
  }

  .horizontal .divider-track { width: 1px; height: 100%; }
  .vertical   .divider-track { height: 1px; width: 100%; }

  .split-divider:hover .divider-track,
  .dragging .divider-track {
    background: var(--accent, #FF6B35);
  }

  /* Close button for the secondary pane */
  .pane-secondary {
    position: relative;
  }

  .pane-close {
    position: absolute;
    top: 4px;
    right: 6px;
    z-index: 10;
    background: var(--bg-elev, #161b22);
    border: 1px solid var(--border, rgba(255,255,255,0.15));
    border-radius: 4px;
    color: var(--fg-muted, #7d8590);
    font-size: 10px;
    line-height: 1;
    padding: 2px 5px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .pane-secondary:hover .pane-close { opacity: 1; }
</style>
