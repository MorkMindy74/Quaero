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

// --- Lawyer Workflow UX V1 (#62): a guided, human path over the same counts ---

/** The id of a guided step. Derivable from the Pratica counts (no per-document
 *  text-layer tracking in V1: "prepare the text" is guidance inside `find`). */
export type WorkflowStepId = "load" | "find" | "claims" | "exportReview";

/** The guided path, in order. `labelKey` is the short stepper label. */
export const WORKFLOW_STEPS: { id: WorkflowStepId; labelKey: string }[] = [
  { id: "load", labelKey: "workflow.step.load" },
  { id: "find", labelKey: "workflow.step.find" },
  { id: "claims", labelKey: "workflow.step.claims" },
  { id: "exportReview", labelKey: "workflow.step.exportReview" },
];

/** The current step of the open Pratica, a pure function of its counts. */
export function currentWorkflowStep(counts: PraticaCounts): WorkflowStepId {
  if (counts.sources === 0) return "load";
  if (counts.excerpts === 0) return "find";
  if (counts.citations === 0) return "claims";
  return "exportReview";
}

/** Zero-based index of a step in [`WORKFLOW_STEPS`] (for the stepper). */
export function workflowStepIndex(step: WorkflowStepId): number {
  return WORKFLOW_STEPS.findIndex((s) => s.id === step);
}
