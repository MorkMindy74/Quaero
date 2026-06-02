// Thin, isolated pdf.js wrapper (#52): extract text from PDF bytes in the
// renderer. The document is untrusted; pdf.js runs in the webview sandbox, never
// in the Rust/native process.
//
// Hardening:
// - `isEvalSupported: false` → mitigates CVE-2024-4367 (font-driven eval);
// - the worker is bundled LOCALLY via Vite `?url` (never a CDN) → no egress;
// - no cMapUrl / standardFontDataUrl set → no remote resource fetch.
//
// Not unit-tested in jsdom (pdf.js needs a worker/real PDFs) — covered by the
// human smoke test. Kept tiny so the trusted surface is easy to audit.
import * as pdfjs from "pdfjs-dist";
// Vite resolves this to a local bundled asset URL — NOT a network URL.
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

/** Defensive upper bound on pages scanned for text. */
const MAX_PAGES = 2000;

/** Extract concatenated text from a PDF. Throws on a parse error (caller maps to
 *  "extraction failed"). May return an empty/whitespace string for a scanned or
 *  image-only PDF (caller maps to "empty"). */
export async function extractPdfText(bytes: Uint8Array): Promise<string> {
  const doc = await pdfjs.getDocument({
    data: bytes,
    isEvalSupported: false,
    disableFontFace: true,
    useSystemFonts: false,
  }).promise;
  try {
    const pages = Math.min(doc.numPages, MAX_PAGES);
    const out: string[] = [];
    for (let i = 1; i <= pages; i++) {
      const page = await doc.getPage(i);
      const content = await page.getTextContent();
      out.push(content.items.map((it) => ("str" in it ? it.str : "")).join(" "));
      page.cleanup();
    }
    return out.join("\n");
  } finally {
    await doc.destroy();
  }
}
