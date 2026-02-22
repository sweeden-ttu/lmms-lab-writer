import { describe, it, expect } from "vitest";
import {
  generatePresenceColor,
  sha256,
  parseLatexLog,
  DEFAULT_CLI_CONFIG,
} from "./index";

describe("generatePresenceColor", () => {
  it("returns a valid HSL color string", () => {
    const color = generatePresenceColor();
    expect(color).toMatch(/^hsl\(\d{1,3}, 70%, 50%\)$/);
  });

  it("returns hue in range 0-359", () => {
    for (let i = 0; i < 50; i++) {
      const color = generatePresenceColor();
      const hue = parseInt(color.match(/hsl\((\d+)/)?.[1] ?? "-1", 10);
      expect(hue).toBeGreaterThanOrEqual(0);
      expect(hue).toBeLessThanOrEqual(359);
    }
  });

  it("returns varied colors across multiple calls", () => {
    const colors = new Set<string>();
    for (let i = 0; i < 20; i++) {
      colors.add(generatePresenceColor());
    }
    expect(colors.size).toBeGreaterThan(1);
  });
});

describe("sha256", () => {
  it("produces known hash for 'hello'", async () => {
    const hash = await sha256("hello");
    expect(hash).toBe(
      "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
    );
  });

  it("produces known hash for empty string", async () => {
    const hash = await sha256("");
    expect(hash).toBe(
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
  });

  it("handles Uint8Array input", async () => {
    const data = new Uint8Array([104, 101, 108, 108, 111]); // "hello"
    const hash = await sha256(data);
    expect(hash).toBe(
      "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
    );
  });

  it("handles unicode content", async () => {
    const hash = await sha256("Hello, 世界!");
    expect(hash).toMatch(/^[a-f0-9]{64}$/);
  });
});

describe("parseLatexLog", () => {
  it("returns empty results for empty log", () => {
    const result = parseLatexLog("");
    expect(result.errors).toEqual([]);
    expect(result.warnings).toEqual([]);
  });

  it("parses error with line number", () => {
    const log = `(./main.tex
! Undefined control sequence.
l.42 \\badcommand
`;
    const result = parseLatexLog(log);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0]).toEqual({
      file: "./main.tex",
      line: 42,
      message: "Undefined control sequence.",
      type: "error",
    });
  });

  it("parses multiple errors", () => {
    const log = `(./main.tex
! Undefined control sequence.
l.10 \\foo
! Missing $ inserted.
l.20 some math
`;
    const result = parseLatexLog(log);
    expect(result.errors).toHaveLength(2);
    expect(result.errors[0]!.line).toBe(10);
    expect(result.errors[1]!.line).toBe(20);
  });

  it("parses warnings", () => {
    const log = `LaTeX Warning: Reference 'fig:missing' on page 1 undefined.
Package hyperref Warning: Token not allowed in a PDF string.`;
    const result = parseLatexLog(log);
    expect(result.warnings).toHaveLength(2);
    expect(result.warnings[0]).toContain("Reference");
    expect(result.warnings[1]).toContain("hyperref");
  });

  it("parses mixed errors and warnings", () => {
    const log = `(./main.tex
! Undefined control sequence.
l.5 \\foo
LaTeX Warning: Citation 'missing' undefined.`;
    const result = parseLatexLog(log);
    expect(result.errors).toHaveLength(1);
    expect(result.warnings).toHaveLength(1);
  });

  it("handles error without line number", () => {
    const log = `! Emergency stop.`;
    const result = parseLatexLog(log);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0]!.line).toBe(0);
    expect(result.errors[0]!.message).toBe("Emergency stop.");
  });

  it("tracks current file from parenthesized paths", () => {
    const log = `(./chapter1.tex
! Undefined control sequence.
l.3 \\badcmd`;
    const result = parseLatexLog(log);
    expect(result.errors[0]!.file).toBe("./chapter1.tex");
  });
});

describe("DEFAULT_CLI_CONFIG", () => {
  it("has xelatex as default engine", () => {
    expect(DEFAULT_CLI_CONFIG.defaultEngine).toBe("xelatex");
  });

  it("has the expected API URL", () => {
    expect(DEFAULT_CLI_CONFIG.apiUrl).toBe(
      "https://lmms-lab-writer.vercel.app",
    );
  });

  it("has watchIgnore patterns including common aux files", () => {
    expect(DEFAULT_CLI_CONFIG.watchIgnore).toContain("*.aux");
    expect(DEFAULT_CLI_CONFIG.watchIgnore).toContain("*.log");
    expect(DEFAULT_CLI_CONFIG.watchIgnore).toContain(".git/**");
    expect(DEFAULT_CLI_CONFIG.watchIgnore).toContain("node_modules/**");
  });
});
