// Pure helpers for the document text layer (#52). No pdf.js here on purpose —
// these stay fast and unit-testable; PDF extraction lives in ./pdfText and the
// orchestration in ./extractText.

/** Formats whose text we can extract locally in the renderer. */
export type TextFormat = "text" | "pdf";

/** Map an original filename to a supported text format, or null if unsupported
 *  (e.g. .docx, images). Extension-based; the file picker also constrains it. */
export function classifyFormat(originalName: string): TextFormat | null {
  const ext = originalName.toLowerCase().split(".").pop() ?? "";
  if (ext === "txt" || ext === "md" || ext === "markdown") return "text";
  if (ext === "pdf") return "pdf";
  return null;
}

/** Strict UTF-8 decode: the text, or null if the bytes are not valid UTF-8.
 *  `.txt/.md` only — we refuse to persist mojibake. */
export function decodeUtf8Strict(bytes: Uint8Array): string | null {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    return null;
  }
}

/** Lowercase hex SHA-256 of bytes (Web Crypto). Used to verify a re-picked file
 *  matches the imported Fonte's recorded digest before deriving/storing text, so
 *  the text layer stays coherent with the pinned blob. */
export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  // Copy into a fresh ArrayBuffer-backed view so the digest input is a concrete
  // `BufferSource` (TS rejects the generic `ArrayBufferLike` form).
  const view = new Uint8Array(bytes);
  const digest = await crypto.subtle.digest("SHA-256", view);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
