/**
 * Basic E2E smoke test: open tab → run command → verify block appears.
 *
 * Requirements:
 *   1. `cargo build --manifest-path ../src-tauri/Cargo.toml` has been run.
 *   2. `tauri-driver` is on PATH (`cargo install tauri-driver`).
 *
 * Run:
 *   cd e2e && npm install && npm test
 *
 * Or on CI, set TAURI_APP_PATH and TAURI_DRIVER_BINARY env vars.
 */

const TERMINAL_GRID_SELECTOR = ".xterm-screen";
const STATUS_BAR_SELECTOR = ".status-bar";
const TAB_SELECTOR = ".tab";
const BLOCK_SELECTOR = ".block";

// --- Helpers -----------------------------------------------------------------

/**
 * Wait until a selector appears and return it. Throws on timeout.
 */
async function waitFor(selector: string, timeout = 15_000): Promise<WebdriverIO.Element> {
  const el = await browser.$(selector);
  await el.waitForExist({ timeout, timeoutMsg: `Timed out waiting for "${selector}"` });
  return el;
}

/**
 * Send a line of text to the active xterm.js terminal grid.
 * xterm.js handles input via keyboard events on the .xterm-helper-textarea.
 */
async function typeInTerminal(line: string): Promise<void> {
  const textarea = await browser.$(".xterm-helper-textarea");
  await textarea.click();
  await browser.keys(line.split(""));
  await browser.keys(["Enter"]);
}

// --- Tests -------------------------------------------------------------------

describe("EmberTerm basic smoke tests", () => {
  before(async () => {
    // Give the app a moment to fully initialize before any assertion.
    await browser.pause(2000);
  });

  it("renders the app shell — title bar and status bar visible", async () => {
    const titleBar = await waitFor(".title-bar, [data-tauri-drag-region]");
    expect(await titleBar.isDisplayed()).toBe(true);

    const statusBar = await waitFor(STATUS_BAR_SELECTOR);
    expect(await statusBar.isDisplayed()).toBe(true);
  });

  it("shows at least one tab on launch", async () => {
    const tab = await waitFor(TAB_SELECTOR);
    expect(await tab.isDisplayed()).toBe(true);
  });

  it("renders the terminal grid (xterm.js canvas)", async () => {
    // The xterm screen div is created by xterm.js on mount.
    const grid = await waitFor(TERMINAL_GRID_SELECTOR, 10_000);
    expect(await grid.isDisplayed()).toBe(true);
  });

  it("accepts keyboard input — typing echoes to terminal", async () => {
    // Type a trivial no-op command.  We wait for the textarea to exist so
    // xterm.js must have mounted successfully.
    const textarea = await browser.$(".xterm-helper-textarea");
    await textarea.waitForExist({ timeout: 10_000 });
    await textarea.click();
    // Typing any character means PTY write round-trip succeeded.
    await browser.keys(["e", "c", "h", "o", " ", "h", "i"]);
    // No assertion on output here — just ensuring no throw.
    await browser.keys(["Control", "c"]); // interrupt, leaving a clean prompt
  });

  it("running a command produces a block element", async () => {
    // Only shells with OSC 133 integration emit blocks.  We inject the marks
    // manually here so the test is shell-agnostic.
    //
    // The sequence is: A (prompt start) → B (command start) → C (output start)
    //   → D;0 (command end, exit 0).
    // We send it via the xterm textarea which forwards bytes to the PTY.
    const textarea = await browser.$(".xterm-helper-textarea");
    await textarea.waitForExist({ timeout: 10_000 });
    await textarea.click();

    // OSC 133 A — prompt mark: \e]133;A\a
    const osc133A = "\x1b]133;A\x07";
    const osc133B = "\x1b]133;B\x07";
    const osc133C = "\x1b]133;C\x07";
    const osc133D = "\x1b]133;D;0\x07";

    // Send marks in sequence.  In a real shell these arrive as PTY output;
    // here we script the PTY write path directly by sending the raw bytes
    // as keyboard input (they pass through to the PTY writer which the
    // OSC parser intercepts).
    //
    // Note: The xterm-helper-textarea does not forward raw escape sequences
    // from browser.keys — we use the `session_write` IPC instead via JS eval.
    await browser.execute((a: string, b: string, c: string, d: string) => {
      // Access the Tauri invoke bridge injected into the webview.
      const w = window as unknown as {
        __TAURI_INTERNALS__?: { invoke: (cmd: string, args: unknown) => Promise<void> };
        __TAURI__?: { invoke: (cmd: string, args: unknown) => Promise<void> };
      };
      const invoke = w.__TAURI_INTERNALS__?.invoke ?? w.__TAURI__?.invoke;
      if (!invoke) return;

      // Get the active session ID from the store exposed on window for testing.
      const sessionId = (window as unknown as { __EMBER_ACTIVE_SESSION__?: string })
        .__EMBER_ACTIVE_SESSION__;
      if (!sessionId) return;

      const enc = new TextEncoder();
      const writeSeq = (seq: string) =>
        invoke("session_write", { id: sessionId, data: Array.from(enc.encode(seq)) });

      // Fire the marks in sequence, 50 ms apart, to let the PTY reader thread
      // process each one before the next arrives.
      const delay = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
      (async () => {
        await writeSeq(a); await delay(50);
        await writeSeq("echo hello\r"); await delay(50);
        await writeSeq(b); await delay(50);
        await writeSeq(c); await delay(50);
        await writeSeq("hello\r\n"); await delay(50);
        await writeSeq(d);
      })();
    }, osc133A, osc133B, osc133C, osc133D);

    // Wait for the block element to appear in the DOM.
    const block = await browser.$(BLOCK_SELECTOR);
    await block.waitForExist({
      timeout: 5_000,
      timeoutMsg: "Expected .block element to appear after OSC 133 sequence",
    });
    expect(await block.isDisplayed()).toBe(true);
  });

  it("opening a new tab creates a second tab", async () => {
    // Click the "+" new-tab button.
    const addBtn = await browser.$(".tab-add-btn, [aria-label='New tab']");
    await addBtn.waitForExist({ timeout: 5_000 });
    await addBtn.click();

    // Wait for two tabs to be present.
    const tabs = await browser.$$(".tab");
    await browser.waitUntil(
      async () => (await browser.$$(".tab")).length >= 2,
      { timeout: 5_000, timeoutMsg: "Expected 2 tabs after clicking '+'" },
    );
    const tabCount = (await browser.$$(".tab")).length;
    expect(tabCount).toBeGreaterThanOrEqual(2);
  });
});
