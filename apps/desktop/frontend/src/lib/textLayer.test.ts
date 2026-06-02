import { describe, expect, test } from "vitest";
import { classifyFormat, decodeUtf8Strict, sha256Hex } from "./textLayer";

describe("classifyFormat", () => {
  test("maps text and pdf extensions, case-insensitively", () => {
    expect(classifyFormat("note.txt")).toBe("text");
    expect(classifyFormat("README.MD")).toBe("text");
    expect(classifyFormat("a.markdown")).toBe("text");
    expect(classifyFormat("Contratto.pdf")).toBe("pdf");
    expect(classifyFormat("SCAN.PDF")).toBe("pdf");
  });

  test("returns null for unsupported formats", () => {
    expect(classifyFormat("atto.docx")).toBeNull();
    expect(classifyFormat("foto.png")).toBeNull();
    expect(classifyFormat("senza-estensione")).toBeNull();
    expect(classifyFormat("")).toBeNull();
  });
});

describe("decodeUtf8Strict", () => {
  test("decodes valid UTF-8", () => {
    const bytes = new TextEncoder().encode("Articolo 1. È vietato.");
    expect(decodeUtf8Strict(bytes)).toBe("Articolo 1. È vietato.");
  });

  test("decodes empty input to empty string", () => {
    expect(decodeUtf8Strict(new Uint8Array([]))).toBe("");
  });

  test("returns null for invalid UTF-8", () => {
    // 0xff is never valid in UTF-8.
    expect(decodeUtf8Strict(new Uint8Array([0xff, 0x28, 0x80]))).toBeNull();
  });
});

describe("sha256Hex", () => {
  test("computes the known digest of 'abc'", async () => {
    const bytes = new TextEncoder().encode("abc");
    expect(await sha256Hex(bytes)).toBe(
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
  });
});
