// Cantiere reale V1 (#64): presentational derivation for the central "Revisione"
// surface of the OPEN Pratica. Pure grouping of the already-computed real chain
// (Estratti/Citazioni + the #9 verification findings) into read-only rows —
// NO domain logic, NO invented data. The verification verdict itself is computed
// in `quaero-core` (Rust); here we only project it for display.

import type { Citation, Excerpt, SourceRef, VerificationReport } from "../domain/types";

/** Per-row outcome shown in the "Esito" column. */
export type ReviewEsito = "coherent" | "warning" | "uncited";

export interface ReviewRow {
  /** "citation": an Affermazione backed by an Estratto. "uncited": an Estratto
   *  that no Citazione references yet. */
  kind: "citation" | "uncited";
  excerptId: string;
  /** Title of the Fonte the Estratto belongs to (falls back to its id). */
  fonte: string;
  /** Verbatim quote of the Estratto. */
  estratto: string;
  /** Human anchor, e.g. "pagina 7". */
  anchor: string;
  /** The Affermazione (citation rows only). */
  claim: string | null;
  esito: ReviewEsito;
}

/**
 * Project the open Pratica's real chain into read-only review rows, in a stable
 * order: every Citazione first (in `citations` order), then the Estratti that no
 * Citazione references (in `excerpts` order). A citation row is `warning` when
 * its Estratto carries a #9 Warning finding, otherwise `coherent`.
 */
export function reviewRows(input: {
  sources: SourceRef[];
  excerpts: Excerpt[];
  citations: Citation[];
  verification?: VerificationReport;
}): ReviewRow[] {
  const { sources, excerpts, citations, verification } = input;

  const excerptById = new Map(excerpts.map((e) => [e.id, e]));
  const sourceTitle = (id: string) => sources.find((s) => s.id === id)?.title ?? id;
  const anchorOf = (e: Excerpt) => `${e.anchor.kind} ${e.anchor.value}`.trim();

  // Excerpt ids flagged by a #9 Warning finding (e.g. unpinned/orphan).
  const warned = new Set(
    (verification?.findings ?? [])
      .filter((f) => f.severity === "Warning" && f.excerptId)
      .map((f) => f.excerptId as string),
  );

  const rows: ReviewRow[] = [];

  // Citazioni first — each tied to its Estratto (ADR-0007). Skip dangling ones.
  for (const c of citations) {
    const e = excerptById.get(c.excerptId);
    if (!e) continue;
    rows.push({
      kind: "citation",
      excerptId: e.id,
      fonte: sourceTitle(e.sourceId),
      estratto: e.quote,
      anchor: anchorOf(e),
      claim: c.claim,
      esito: warned.has(e.id) ? "warning" : "coherent",
    });
  }

  // Then Estratti that no Citazione references yet.
  const cited = new Set(citations.map((c) => c.excerptId));
  for (const e of excerpts) {
    if (cited.has(e.id)) continue;
    rows.push({
      kind: "uncited",
      excerptId: e.id,
      fonte: sourceTitle(e.sourceId),
      estratto: e.quote,
      anchor: anchorOf(e),
      claim: null,
      esito: "uncited",
    });
  }

  return rows;
}
