import { describe, expect, test, vi, beforeEach } from "vitest";

// Mock the pdf.js wrapper so these tests never load pdf.js / its worker.
vi.mock("./pdfText", () => ({ extractPdfText: vi.fn() }));

import { extractDocumentText } from "./extractText";
import { extractPdfText } from "./pdfText";

const utf8 = (s: string) => new TextEncoder().encode(s);

beforeEach(() => {
  vi.mocked(extractPdfText).mockReset();
});

describe("extractDocumentText", () => {
  test("unsupported format short-circuits without touching pdf.js", async () => {
    const out = await extractDocumentText("atto.docx", utf8("x"));
    expect(out).toEqual({ kind: "unsupported" });
    expect(extractPdfText).not.toHaveBeenCalled();
  });

  test("decodes .txt/.md as UTF-8 text", async () => {
    expect(await extractDocumentText("n.txt", utf8("ciao"))).toEqual({
      kind: "text",
      text: "ciao",
    });
  });

  test("invalid UTF-8 text file → failed", async () => {
    expect(await extractDocumentText("n.txt", new Uint8Array([0xff, 0x28]))).toEqual({
      kind: "failed",
    });
  });

  test("PDF text is extracted via pdf.js", async () => {
    vi.mocked(extractPdfText).mockResolvedValue("contenuto pdf");
    expect(await extractDocumentText("c.pdf", utf8("%PDF"))).toEqual({
      kind: "text",
      text: "contenuto pdf",
    });
  });

  test("a corrupt PDF (pdf.js throws) → failed, never throws", async () => {
    vi.mocked(extractPdfText).mockRejectedValue(new Error("bad pdf"));
    expect(await extractDocumentText("c.pdf", utf8("garbage"))).toEqual({ kind: "failed" });
  });

  test("a scanned PDF (empty text) is still text (store will mark it empty)", async () => {
    vi.mocked(extractPdfText).mockResolvedValue("   ");
    expect(await extractDocumentText("scan.pdf", utf8("%PDF"))).toEqual({
      kind: "text",
      text: "   ",
    });
  });
});
