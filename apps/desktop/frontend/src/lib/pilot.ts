// Pilot-readiness UX helpers (frontend-only). Pure derivation from the open
// Pratica's counts — no backend, no data model change.

export interface PraticaCounts {
  sources: number;
  excerpts: number;
  citations: number;
}

/**
 * The i18n key of the "next suggested action" for the operational path
 * Fonti → Estratti → Citazioni → Export. Pure function of the counts:
 * - no sources            → import a document
 * - sources, no excerpts  → create the first Estratto
 * - excerpts, no citations→ add a Citazione
 * - has citations         → export the grounded Markdown report
 */
export function nextActionKey(counts: PraticaCounts): string {
  if (counts.sources === 0) return "pilot.next.importSource";
  if (counts.excerpts === 0) return "pilot.next.createExcerpt";
  if (counts.citations === 0) return "pilot.next.addCitation";
  return "pilot.next.export";
}
