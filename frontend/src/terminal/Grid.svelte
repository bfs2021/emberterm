<script lang="ts">
  import { get } from "svelte/store";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import { WebglAddon } from "@xterm/addon-webgl";
  import { ImageAddon } from "@xterm/addon-image";
  import { sessionWrite, sessionResize } from "../ipc/client.js";
  import { onTerminalOutput, onSessionExit, onKittyKeyboard } from "../ipc/events.js";
  import { activeTheme } from "../theme/store.js";
  import { drainOutputBuffer } from "../stores/app.js";
  import { shouldInterceptKey, encodeKittyKey, FLAG_REPORT_EVENTS } from "../utils/kittyKeyboard.js";

  // ── Props ────────────────────────────────────────────────────────────────

  interface Props {
    sessionId: string;
    active: boolean;
    onExit?: (code: number) => void;
    onResize?: (cols: number, rows: number) => void;
  }

  let { sessionId, active, onExit, onResize }: Props = $props();

  // ── Terminal instance ────────────────────────────────────────────────────

  let container: HTMLDivElement;
  let term = $state<Terminal | undefined>(undefined);
  /** Active Kitty keyboard enhancement flags (0 = disabled). */
  let kittyFlags = $state(0);

  $effect(() => {
    let mounted = true;
    let fitAddon: FitAddon | undefined;
    let unlistenOutput: (() => void) | undefined;
    let unlistenExit: (() => void) | undefined;
    let unlistenKitty: (() => void) | undefined;
    let resizeObserver: ResizeObserver | undefined;

    (async () => {
      // Buffer for the tiny gap between listener registration and terminal open.
      const earlyBuf: Uint8Array[] = [];
      let liveTerminal: Terminal | undefined;

      // Register BEFORE creating the terminal so no bytes are missed.
      // Bytes arriving before liveTerminal is set go into earlyBuf.
      unlistenOutput = await onTerminalOutput((payload) => {
        if (payload.session_id !== sessionId) return;
        const bytes = new Uint8Array(payload.data);
        if (liveTerminal) {
          liveTerminal.write(bytes);
        } else {
          earlyBuf.push(bytes);
        }
      });

      const t = new Terminal({
        allowProposedApi: true,
        fontFamily: '"JetBrains Mono", "Cascadia Code", "Fira Code", monospace',
        fontSize: 13,
        lineHeight: 1.2,
        cursorBlink: true,
        cursorStyle: "block",
        scrollback: 10000,
        theme: get(activeTheme).xterm,
      });

      fitAddon = new FitAddon();
      t.loadAddon(fitAddon);
      t.loadAddon(new WebLinksAddon());
      // Kitty graphics protocol + Sixel rendering.
      t.loadAddon(new ImageAddon());

      t.open(container);

      try {
        const webgl = new WebglAddon();
        webgl.onContextLoss(() => webgl.dispose());
        t.loadAddon(webgl);
      } catch {
        // canvas fallback
      }

      fitAddon.fit();

      t.onData((data) => {
        const bytes = new TextEncoder().encode(data);
        sessionWrite(sessionId, bytes).catch(console.error);
      });

      t.onBinary((data) => {
        const bytes = Uint8Array.from(data, (c) => c.charCodeAt(0));
        sessionWrite(sessionId, bytes).catch(console.error);
      });

      // Kitty keyboard protocol: intercept key events when an application has
      // pushed Kitty mode, encode them in Kitty format, and suppress xterm.js
      // default encoding for those events.
      t.attachCustomKeyEventHandler((e: KeyboardEvent) => {
        if (!shouldInterceptKey(e, kittyFlags)) return true;
        const eventType = e.type === "keyup" ? 3 : e.repeat ? 2 : 1;
        const encoded = encodeKittyKey(e, kittyFlags, eventType as 1 | 2 | 3);
        if (encoded) {
          sessionWrite(sessionId, encoded).catch(console.error);
          return false; // Prevent xterm.js from also sending the key
        }
        return true;
      });

      // Drain App.svelte's pre-mount buffer (bytes emitted before Grid mounted).
      for (const bytes of drainOutputBuffer(sessionId)) {
        t.write(bytes);
      }
      // Replay the tiny early buffer from the listener-registration gap.
      for (const bytes of earlyBuf) {
        t.write(bytes);
      }
      earlyBuf.length = 0;

      // Go live — all future terminal:output events write directly.
      liveTerminal = t;

      unlistenExit = await onSessionExit((payload) => {
        if (payload.session_id !== sessionId) return;
        t.writeln("\r\n\x1b[2m[process exited with code " + payload.code + "]\x1b[0m");
        onExit?.(payload.code);
      });

      // Track Kitty keyboard mode changes for this session.
      unlistenKitty = await onKittyKeyboard((payload) => {
        if (payload.session_id !== sessionId) return;
        kittyFlags = payload.flags;
      });

      resizeObserver = new ResizeObserver(() => {
        fitAddon!.fit();
        sessionResize(sessionId, t.cols, t.rows).catch(console.error);
        onResize?.(t.cols, t.rows);
      });
      resizeObserver.observe(container);

      await sessionResize(sessionId, t.cols, t.rows);
      onResize?.(t.cols, t.rows);

      if (!mounted) {
        unlistenOutput?.();
        unlistenExit?.();
        resizeObserver.disconnect();
        t.dispose();
        return;
      }

      term = t;
    })();

    return () => {
      mounted = false;
      unlistenOutput?.();
      unlistenExit?.();
      unlistenKitty?.();
      resizeObserver?.disconnect();
      term?.dispose();
      term = undefined;
    };
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
