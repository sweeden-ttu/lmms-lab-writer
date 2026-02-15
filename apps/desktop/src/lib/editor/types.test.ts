import { describe, it, expect } from "vitest";
import { DEFAULT_MINIMAP_SETTINGS, DEFAULT_EDITOR_SETTINGS } from "./types";

describe("DEFAULT_MINIMAP_SETTINGS", () => {
  it("is disabled by default", () => {
    expect(DEFAULT_MINIMAP_SETTINGS.enabled).toBe(false);
  });

  it("has side set to right", () => {
    expect(DEFAULT_MINIMAP_SETTINGS.side).toBe("right");
  });
});

describe("DEFAULT_EDITOR_SETTINGS", () => {
  it("has vim mode off by default", () => {
    expect(DEFAULT_EDITOR_SETTINGS.vimMode).toBe(false);
  });

  it("has font size 14", () => {
    expect(DEFAULT_EDITOR_SETTINGS.fontSize).toBe(14);
  });

  it("has tab size 2", () => {
    expect(DEFAULT_EDITOR_SETTINGS.tabSize).toBe(2);
  });

  it("has git auto-fetch interval of 120 seconds", () => {
    expect(DEFAULT_EDITOR_SETTINGS.gitAutoFetchIntervalSeconds).toBe(120);
  });

  it("has terminal shell mode set to auto", () => {
    expect(DEFAULT_EDITOR_SETTINGS.terminalShellMode).toBe("auto");
  });
});
