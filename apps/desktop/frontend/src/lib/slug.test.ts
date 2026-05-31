import { expect, test } from "vitest";
import { slug, shortSuffix, makeMatterId } from "./slug";

const SAFE = /^[A-Za-z0-9_-]+$/;

test("slug lowercases and replaces unsafe runs with single hyphens", () => {
  expect(slug("Rossi c. Bianchi")).toBe("rossi-c-bianchi");
  expect(slug("Alfa S.r.l.")).toBe("alfa-s-r-l");
});

test("slug strips diacritics", () => {
  expect(slug("Società àèìòù")).toBe("societa-aeiou");
});

test("slug never returns empty (fallback)", () => {
  expect(slug("***")).toBe("p");
  expect(slug("")).toBe("p");
});

test("shortSuffix is 6 safe chars and deterministic under injected rng", () => {
  expect(shortSuffix(() => 0)).toBe("000000");
  expect(shortSuffix(() => 0)).toMatch(SAFE);
});

test("makeMatterId is slug(title)-suffix and always backend-safe", () => {
  expect(makeMatterId("Rossi c. Bianchi", () => 0)).toBe("rossi-c-bianchi-000000");
  const id = makeMatterId("Operazione UTP", () => 0.5);
  expect(id.startsWith("operazione-utp-")).toBe(true);
  expect(id).toMatch(SAFE);
});
