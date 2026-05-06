import { ChildProcess, spawn } from "node:child_process";
import { join } from "node:path";
import type { Options } from "@wdio/types";

// Path to the compiled Tauri app — adjust for macOS/Linux.
const APP_PATH =
  process.env.TAURI_APP_PATH ??
  join("..", "src-tauri", "target", "debug", "emberterm.exe");

// tauri-driver binary (install via: cargo install tauri-driver)
const DRIVER_BIN =
  process.env.TAURI_DRIVER_BINARY ?? "tauri-driver";

let tauriDriver: ChildProcess | undefined;

export const config: Options.Testrunner = {
  // ── Runner ────────────────────────────────────────────────────────────────
  runner: "local",
  specs: ["./tests/**/*.spec.ts"],
  exclude: [],
  maxInstances: 1,

  // ── Capabilities ──────────────────────────────────────────────────────────
  capabilities: [
    {
      maxInstances: 1,
      // tauri-driver speaks the W3C WebDriver protocol and forwards to WebView2.
      "tauri:options": { application: APP_PATH },
      browserName: "wry",
    },
  ],

  // ── Framework ─────────────────────────────────────────────────────────────
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 30_000,
  },

  // ── Hooks — start / stop tauri-driver ────────────────────────────────────
  onPrepare() {
    tauriDriver = spawn(DRIVER_BIN, [], { stdio: "pipe" });
    tauriDriver.stdout?.on("data", (d) => process.stdout.write(d));
    tauriDriver.stderr?.on("data", (d) => process.stderr.write(d));
    // Give the driver a moment to bind its port.
    return new Promise<void>((resolve) => setTimeout(resolve, 1000));
  },

  onComplete() {
    tauriDriver?.kill();
  },

  // ── Hostname / port tauri-driver listens on ───────────────────────────────
  hostname: "localhost",
  port: 4444,
  path: "/",
};
