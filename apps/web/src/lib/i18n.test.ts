import { describe, it, expect } from "vitest";
import {
  isLocale,
  normalizeLocale,
  stripLocalePrefix,
  withLocalePrefix,
  interpolate,
  localeFromAcceptLanguage,
} from "./i18n";

describe("isLocale", () => {
  it("returns true for supported locales", () => {
    expect(isLocale("en")).toBe(true);
    expect(isLocale("zh")).toBe(true);
    expect(isLocale("ja")).toBe(true);
  });

  it("returns false for unsupported values", () => {
    expect(isLocale("fr")).toBe(false);
    expect(isLocale("EN")).toBe(false);
    expect(isLocale("")).toBe(false);
  });
});

describe("normalizeLocale", () => {
  it("returns valid locale as-is", () => {
    expect(normalizeLocale("zh")).toBe("zh");
    expect(normalizeLocale("ja")).toBe("ja");
  });

  it("returns default locale for null/undefined/invalid", () => {
    expect(normalizeLocale(null)).toBe("en");
    expect(normalizeLocale(undefined)).toBe("en");
    expect(normalizeLocale("fr")).toBe("en");
  });
});

describe("stripLocalePrefix", () => {
  it("strips locale from path", () => {
    expect(stripLocalePrefix("/zh/about")).toBe("/about");
  });

  it("strips locale from root locale path", () => {
    expect(stripLocalePrefix("/ja")).toBe("/");
  });

  it("leaves non-locale paths unchanged", () => {
    expect(stripLocalePrefix("/about")).toBe("/about");
  });

  it("trims trailing slashes", () => {
    expect(stripLocalePrefix("/zh/about/")).toBe("/about");
  });
});

describe("withLocalePrefix", () => {
  it("adds prefix for non-default locale", () => {
    expect(withLocalePrefix("/about", "zh")).toBe("/zh/about");
  });

  it("does not add prefix for default locale (en)", () => {
    expect(withLocalePrefix("/about", "en")).toBe("/about");
  });

  it("handles root path", () => {
    expect(withLocalePrefix("/", "zh")).toBe("/zh");
    expect(withLocalePrefix("/", "en")).toBe("/");
  });
});

describe("interpolate", () => {
  it("replaces a single placeholder", () => {
    expect(interpolate("Hello {name}!", { name: "World" })).toBe(
      "Hello World!",
    );
  });

  it("replaces multiple placeholders", () => {
    expect(
      interpolate("{greeting}, {name}!", { greeting: "Hi", name: "User" }),
    ).toBe("Hi, User!");
  });

  it("replaces all occurrences of the same placeholder", () => {
    expect(interpolate("{x} and {x}", { x: "A" })).toBe("A and A");
  });

  it("preserves unmatched placeholders", () => {
    expect(interpolate("Hello {name}!", {})).toBe("Hello {name}!");
  });
});

describe("localeFromAcceptLanguage", () => {
  it("returns default locale for null header", () => {
    expect(localeFromAcceptLanguage(null)).toBe("en");
  });

  it("matches exact locale", () => {
    expect(localeFromAcceptLanguage("zh")).toBe("zh");
    expect(localeFromAcceptLanguage("ja")).toBe("ja");
  });

  it("matches prefix (zh-CN → zh)", () => {
    expect(localeFromAcceptLanguage("zh-CN,en;q=0.9")).toBe("zh");
  });

  it("respects quality values", () => {
    expect(localeFromAcceptLanguage("fr;q=0.9,ja;q=1.0")).toBe("ja");
  });

  it("returns default for unsupported languages", () => {
    expect(localeFromAcceptLanguage("fr,de;q=0.9")).toBe("en");
  });
});
