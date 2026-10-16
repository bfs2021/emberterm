import { describe, it, expect } from "vitest";
import { pickProfile } from "./profiles.js";
import type { Profile } from "../ipc/client.js";

const p = (id: string): Profile => ({ id, name: id, icon: "terminal", shell: id, args: [], cwd: "~", env: {} });
const list = [p("cmd"), p("gitbash"), p("pwsh")];

describe("pickProfile", () => {
  it("uses the requested profile when it exists", () => {
    expect(pickProfile(list, "pwsh", "gitbash")?.id).toBe("pwsh");
  });
  it("falls back to the configured default when the requested profile is gone", () => {
    expect(pickProfile(list, "deleted", "gitbash")?.id).toBe("gitbash");
  });
  it("uses the default when nothing is requested", () => {
    expect(pickProfile(list, undefined, "gitbash")?.id).toBe("gitbash");
  });
  it("falls back to the first profile when the default is also unknown", () => {
    expect(pickProfile(list, "deleted", "also-gone")?.id).toBe("cmd");
    expect(pickProfile(list, undefined, "")?.id).toBe("cmd");
  });
  it("returns undefined for an empty list", () => {
    expect(pickProfile([], "x", "y")).toBeUndefined();
  });
});
