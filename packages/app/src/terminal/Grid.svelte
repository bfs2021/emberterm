<script lang="ts">
  import { untrack } from "svelte";
  import { get } from "svelte/store";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import { sessionWrite, sessionResize } from "../ipc/client.js";
  import { onSessionExit } from "../ipc/events.js";
  import { activeTheme } from "../theme/store.js";
  import { fontConfig } from "../theme/fontStore.js";
  import { getOutputSink } from "./outputSink.js";
  import { terminalConfig } from "../stores/terminalConfigStore.js";
  import { parseOsc7, parseOsc52 } from "../utils/osc.js";
  import { isCopyChord } from "./copyKey.js";
  import { shortcutFor } from "../utils/shortcuts.js";

  // ── Props ────────────────────────────────────────────────────────────────

  interface Props {
    sessionId: string;
    active: boolean;
    onExit?: (code: number) => void;
    onResize?: (cols: number, rows: number) => void;
    onCwd?: (cwd: string) => void;
    onTitle?: (title: string) => void;
  }

  let { sessionId, active, onExit, onResize, onCwd, onTitle }: Props = $props();

  // ── Terminal instance ────────────────────────────────────────────────────

  let container: HTMLDivElement;
  // Props resolve through the $tabs each-item, so reading them directly in an effect re-runs it on
  // every tab update. $derived memoises primitives: effects below re-run only on a real change.
  const paneActive = $derived(active);
  const paneSession = $derived(sessionId);
  let term = $state<Terminal | undefined>(undefined);
  /** Exposed so the font-change effect can call fitAddon.fit() after resize. */
  let fa = $state<FitAddon | undefined>(undefined);

  $effect(() => {
    // Read once, untracked: a tracked read here re-ran this effect (tearing the
    // terminal down) on every $tabs update, because props resolve through the store.
    const sid = untrack(() => sessionId);
    let mounted = true;
    let unlistenExit: (() => void) | undefined;
    let resizeObserver: ResizeObserver | undefined;

    (async () => {
      const cfg = get(fontConfig);
      const tc = get(terminalConfig);
      const baseTheme = get(activeTheme).xterm;
      const t = new Terminal({
        allowProposedApi: true,
        allowTransparency: true,
        fontFamily: cfg.fontFamily,
        fontSize: cfg.fontSize,
        lineHeight: cfg.lineHeight,
        cursorBlink: tc.cursorBlink,
        cursorStyle: tc.cursorStyle as "block" | "bar" | "underline",
        scrollback: tc.scrollbackLines,
        theme: { ...baseTheme },
      });

      const fitAddon = new FitAddon();
      fa = fitAddon;
      t.loadAddon(fitAddon);
      t.loadAddon(new WebLinksAddon());

      t.open(container);
      t.parser.registerOscHandler(7, (data) => {
        const cwd = parseOsc7(data);
        if (cwd) onCwd?.(cwd);
        return true;
      });
      t.parser.registerOscHandler(52, (data) => {
        const text = parseOsc52(data);
        if (text !== null) navigator.clipboard.writeText(text).catch(console.error);
        return true;
      });
      t.onTitleChange((title) => onTitle?.(title));
      t.attachCustomKeyEventHandler((e) => {
        // App shortcuts must reach the window handler; xterm would otherwise consume Tab
        // (so Ctrl+Tab never switched tabs) by sending it to the shell.
        if (shortcutFor(e)) return false;
        if (!isCopyChord(e, t.hasSelection())) return true;
        if (t.hasSelection()) {
          navigator.clipboard.writeText(t.getSelection()).catch(console.error);
          t.clearSelection();
        }
        return false; // never forward a Ctrl+Shift+C to the shell as input
      });
      fitAddon.fit();

      t.onData((data) => {
        const bytes = new TextEncoder().encode(data);
        sessionWrite(sid, bytes).catch(console.error);
      });

      t.onBinary((data) => {
        const bytes = Uint8Array.from(data, (c) => c.charCodeAt(0));
        sessionWrite(sid, bytes).catch(console.error);
      });

      // Replay bytes buffered before this Grid mounted, then stream live.
      getOutputSink(sid)?.attach((bytes) => t.write(bytes));

      unlistenExit = await onSessionExit((payload) => {
        if (payload.session_id !== sid) return;
        t.writeln("\r\n\x1b[2m[process exited with code " + payload.code + "]\x1b[0m");
        onExit?.(payload.code);
      });

      resizeObserver = new ResizeObserver(() => {
        fitAddon!.fit();
        sessionResize(sid, t.cols, t.rows).catch(console.error);
        onResize?.(t.cols, t.rows);
      });
      resizeObserver.observe(container);

      await sessionResize(sid, t.cols, t.rows);
      onResize?.(t.cols, t.rows);

      if (!mounted) {
        getOutputSink(sid)?.detach();
        unlistenExit?.();
        resizeObserver.disconnect();
        t.dispose();
        return;
      }

      term = t;
    })();

    return () => {
      mounted = false;
      fa = undefined;
      getOutputSink(sid)?.detach();
      unlistenExit?.();
      resizeObserver?.disconnect();
      term?.dispose();
      term = undefined;
    };
  });

  // Update xterm theme when the active theme changes.
  $effect(() => {
    const base = $activeTheme.xterm;
    if (term) {
      term.options.theme = { ...base };
    }
  });

  // Live font preview: when fontConfig changes, update the terminal and re-fit.
  $effect(() => {
    const { fontFamily, fontSize, lineHeight } = $fontConfig;
    if (!term || !fa) return;
    term.options.fontFamily = fontFamily;
    term.options.fontSize = fontSize;
    term.options.lineHeight = lineHeight;
    fa.fit();
    // Font changes affect cell dimensions, so update the PTY size.
    sessionResize(paneSession, term.cols, term.rows).catch(console.error);
    untrack(() => onResize)?.(term.cols, term.rows);
  });

  // Live terminal config: apply cursor style/blink changes without restart.
  $effect(() => {
    const { cursorStyle, cursorBlink } = $terminalConfig;
    if (!term) return;
    term.options.cursorStyle = cursorStyle as "block" | "bar" | "underline";
    term.options.cursorBlink = cursorBlink;
    // scrollback can only be set at construction time in xterm.js — new sessions pick it up.
  });

  // Focus the terminal when this tab becomes active.
  // Also call fit() in case the window was resized while this tab was hidden.
  $effect(() => {
    if (paneActive && term) {
      term.focus();
      fa?.fit();
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
