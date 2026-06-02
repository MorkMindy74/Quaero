// Renderer-side extraction orchestrator (#52). Dispatches by format and never
// throws — the caller turns the outcome into one of the UI states. PDF parsing
// is delegated to ./pdfText (pdf.js); `.txt/.md` use a strict UTF-8 decode.
import { classifyFormat, decodeUtf8Strict } from "./textLayer";

export type ExtractOutcome =
  | { kind: "text"; text: string }
  | { kind: "unsupported" }
  | { kind: "failed" };

/** Extract text from a document's bytes. Never throws. Empty/whitespace text is
 *  still `{ kind: "text" }` — the store classifies it as "empty". */
export async function extractDocumentText(
  originalName: string,
  bytes: Uint8Array,
): Promise<ExtractOutcome> {
  const fmt = classifyFormat(originalName);
  if (fmt === null) return { kind: "unsupported" };
  if (fmt === "text") {
    const text = decodeUtf8Strict(bytes);
    return text === null ? { kind: "failed" } : { kind: "text", text };
  }
  try {
    // Lazy-load pdf.js only when a PDF is actually extracted, so it stays out of
    // the main bundle (and out of memory until needed).
    const { extractPdfText } = await import("./pdfText");
    return { kind: "text", text: await extractPdfText(bytes) };
  } catch {
    return { kind: "failed" };
  }
}
