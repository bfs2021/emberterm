<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import { WebglAddon } from "@xterm/addon-webgl";
  import { sessionWrite, sessionResize } from "../ipc/client";
  import { onTerminalOutput, onSessionExit } from "../ipc/events";
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
    term = new Terminal({
      allowProposedApi: true,
      fontFamily: '"JetBrains Mono", "Cascadia Code", "Fira Code", monospace',
      fontSize: 13,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: "block",
      scrollback: 10000,
      // Ember Dark palette — will be driven by theme system in Phase 1.
      theme: {
        background: "#0d1117",
        foreground: "#e6edf3",
        cursor: "#e6edf3",
        cursorAccent: "#0d1117",
        black: "#484f58",
        red: "#ff7b72",
        green: "#7ee787",
        yellow: "#d29922",
        blue: "#58a6ff",
        magenta: "#bc8cff",
        cyan: "#39c5cf",
        white: "#b1bac4",
        brightBlack: "#6e7681",
        brightRed: "#ffa198",
        brightGreen: "#56d364",
        brightYellow: "#e3b341",
        brightBlue: "#79c0ff",
        brightMagenta: "#d2a8ff",
        brightCyan: "#56d4dd",
        brightWhite: "#f0f6fc",
      },
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());

    term.open(container);

    // Try WebGL renderer; fall back to canvas silently.
    try {
      const webgl = new WebglAddon();
      webgl.onContextLoss(() => webgl.dispose());
      term.loadAddon(webgl);
    } catch {
      // Canvas fallback is fine for Phase 0.
    }

    fitAddon.fit();

    // Forward keystrokes to the PTY.
    term.onData((data) => {
      const bytes = new TextEncoder().encode(data);
      sessionWrite(sessionId, bytes).catch(console.error);
    });

    // Binary paste passthrough (e.g. ctrl sequences from clipboard).
    term.onBinary((data) => {
      const bytes = Uint8Array.from(data, (c) => c.charCodeAt(0));
      sessionWrite(sessionId, bytes).catch(console.error);
    });

    // Subscribe to PTY output events.
    unlistenOutput = await onTerminalOutput((payload) => {
      if (payload.session_id !== sessionId) return;
      const bytes = new Uint8Array(payload.data);
      term.write(bytes);
    });

    // Subscribe to session exit.
    unlistenExit = await onSessionExit((payload) => {
      if (payload.session_id !== sessionId) return;
      term.writeln("\r\n\x1b[2m[process exited with code " + payload.code + "]\x1b[0m");
      onExit?.(payload.code);
    });

    // Resize PTY when the container changes size.
    resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      sessionResize(sessionId, term.cols, term.rows).catch(console.error);
    });
    resizeObserver.observe(container);

    // Initial resize handshake.
    await sessionResize(sessionId, term.cols, term.rows);
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
  class:hidden={!active}
></div>

<style>
  .grid-container {
    width: 100%;
    height: 100%;
    overflow: hidden;
    /* xterm.js injects its own styles into this element. */
  }
  .grid-container.hidden {
    display: none;
  }

  /* Override xterm.js viewport to fill the container cleanly. */
  :global(.xterm) {
    height: 100%;
    padding: 4px 8px;
  }
  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
