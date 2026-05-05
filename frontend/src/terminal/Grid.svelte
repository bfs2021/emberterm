<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import { WebglAddon } from "@xterm/addon-webgl";
  import { sessionWrite, sessionResize } from "../ipc/client.js";
  import { onTerminalOutput, onSessionExit } from "../ipc/events.js";
  import { activeTheme } from "../theme/store.js";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  // ── Props ────────────────────────────────────────────────────────────────

  interface Props {
    sessionId: string;
    active: boolean;
    onExit?: (code: number) => void;
  }

  let { sessionId, active, onExit }: Props = $props();

  // ── Terminal instance ────────────────────────────────────────────────────

  let container: HTMLDivElement;
  let term: Terminal;
  let fitAddon: FitAddon;
  let unlistenOutput: UnlistenFn;
  let unlistenExit: UnlistenFn;
  let resizeObserver: ResizeObserver;

  onMount(async () => {
    const theme = $activeTheme;

    term = new Terminal({
      allowProposedApi: true,
      fontFamily: '"JetBrains Mono", "Cascadia Code", "Fira Code", monospace',
      fontSize: 13,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: "block",
      scrollback: 10000,
      theme: theme.xterm,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());

    term.open(container);

    try {
      const webgl = new WebglAddon();
      webgl.onContextLoss(() => webgl.dispose());
      term.loadAddon(webgl);
    } catch {
      // canvas fallback
    }

    fitAddon.fit();

    term.onData((data) => {
      const bytes = new TextEncoder().encode(data);
      sessionWrite(sessionId, bytes).catch(console.error);
    });

    term.onBinary((data) => {
      const bytes = Uint8Array.from(data, (c) => c.charCodeAt(0));
      sessionWrite(sessionId, bytes).catch(console.error);
    });

    unlistenOutput = await onTerminalOutput((payload) => {
      if (payload.session_id !== sessionId) return;
      const bytes = new Uint8Array(payload.data);
      term.write(bytes);
    });

    unlistenExit = await onSessionExit((payload) => {
      if (payload.session_id !== sessionId) return;
      term.writeln("\r\n\x1b[2m[process exited with code " + payload.code + "]\x1b[0m");
      onExit?.(payload.code);
    });

    resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      sessionResize(sessionId, term.cols, term.rows).catch(console.error);
    });
    resizeObserver.observe(container);

    await sessionResize(sessionId, term.cols, term.rows);
  });

  // Update xterm theme when the active theme changes.
  $effect(() => {
    if (term) {
      term.options.theme = $activeTheme.xterm;
    }
  });

  // Focus the terminal when this tab becomes active.
  $effect(() => {
    if (active && term) {
      term.focus();
    }
  });

  onDestroy(() => {
    unlistenOutput?.();
    unlistenExit?.();
    resizeObserver?.disconnect();
    term?.dispose();
  });
</script>

<div
  bind:this={container}
  class="grid-container"
></div>

<style>
  .grid-container {
    width: 100%;
    height: 100%;
    overflow: hidden;
    flex: 1;
    min-height: 0;
  }

  :global(.xterm) {
    height: 100%;
    padding: 4px 8px;
  }

  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
