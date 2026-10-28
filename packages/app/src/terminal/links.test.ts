import { describe, it, expect, vi, beforeEach } from "vitest";

const openUrl = vi.fn().mockResolvedValue(undefined);

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl }));

const { openExternalLink } = await import("./links.js");

describe("openExternalLink", () => {
  beforeEach(() => openUrl.mockClear());

  it("opens http and https links through the opener plugin", () => {
    openExternalLink(undefined, "http://example.com");
    openExternalLink(undefined, "https://example.com/docs?q=1");
    expect(openUrl).toHaveBeenCalledTimes(2);
    expect(openUrl).toHaveBeenNthCalledWith(1, "http://example.com");
    expect(openUrl).toHaveBeenNthCalledWith(2, "https://example.com/docs?q=1");
  });

  it("is case-insensitive on the scheme", () => {
    openExternalLink(undefined, "HTTPS://EXAMPLE.COM");
    expect(openUrl).toHaveBeenCalledTimes(1);
    expect(openUrl).toHaveBeenCalledWith("HTTPS://EXAMPLE.COM");
  });

  it("ignores other schemes so the webview never navigates", () => {
    openExternalLink(undefined, "file:///etc/passwd");
    openExternalLink(undefined, "javascript:alert(1)");
    openExternalLink(undefined, "mailto:a@b.example");
    openExternalLink(undefined, "not a url");
    expect(openUrl).not.toHaveBeenCalled();
  });
});
