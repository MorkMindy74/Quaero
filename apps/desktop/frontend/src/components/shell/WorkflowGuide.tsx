import { useTranslation } from "react-i18next";
import { Button } from "../ui";
import {
  WORKFLOW_STEPS,
  currentWorkflowStep,
  workflowStepIndex,
  type WorkflowStepId,
} from "../../lib/pilot";

/** Tabs the guide can jump to in the right panel. */
export type GuideTab = "sources" | "excerpts" | "verify";

/** Lawyer Workflow UX V1 (#62). The big, central "next safe action" for the open
 *  Pratica: a compact stepper + one prominent card with a single primary action.
 *  Presentational only — it drives the EXISTING actions (jump to the right panel
 *  tab, or export); it never persists anything itself. Derived purely from counts
 *  (+ the #9 verdict), so it always reflects the real open Pratica. */
export function WorkflowGuide({
  sources,
  excerpts,
  citations,
  verificationWarnings,
  onGoToTab,
  onExport,
}: {
  sources: number;
  excerpts: number;
  citations: number;
  /** #9 warnings count of the open Pratica, or null if unknown. */
  verificationWarnings: number | null;
  onGoToTab: (tab: GuideTab) => void;
  onExport: () => void;
}) {
  const { t } = useTranslation();
  const step = currentWorkflowStep({ sources, excerpts, citations });
  const currentIdx = workflowStepIndex(step);

  // Primary action per step (label key + handler). All actions already exist.
  const primary: Record<WorkflowStepId, { ctaKey: string; run: () => void }> = {
    load: { ctaKey: "workflow.card.load.cta", run: () => onGoToTab("sources") },
    find: { ctaKey: "workflow.card.find.cta", run: () => onGoToTab("sources") },
    claims: { ctaKey: "workflow.card.claims.cta", run: () => onGoToTab("excerpts") },
    exportReview: { ctaKey: "workflow.card.exportReview.cta", run: () => onExport() },
  };

  return (
    <section data-testid="workflow-guide" className="mx-auto max-w-2xl">
      {/* Compact stepper: the path, with the current step highlighted. */}
      <ol className="flex flex-wrap items-center gap-x-2 gap-y-1 text-[11px]" data-testid="workflow-stepper">
        {WORKFLOW_STEPS.map((s, i) => {
          const state = i < currentIdx ? "done" : i === currentIdx ? "current" : "todo";
          return (
            <li key={s.id} className="flex items-center gap-2">
              {i > 0 && <span className="text-hairline">→</span>}
              <span
                data-step={s.id}
                data-state={state}
                className={
                  state === "current"
                    ? "rounded-full border border-l-2 border-accent-source bg-panel-2 px-2 py-0.5 font-medium text-ink"
                    : state === "done"
                      ? "text-accent-verified"
                      : "text-muted"
                }
              >
                {state === "done" ? "✓ " : ""}
                {t(s.labelKey)}
              </span>
            </li>
          );
        })}
      </ol>

      {/* The big "next action" card. */}
      <div
        data-testid="workflow-card"
        className="mt-4 rounded-lg border border-hairline border-l-4 border-l-accent-source bg-panel p-6"
      >
        <div className="font-mono text-[11px] uppercase tracking-wide text-muted">
          {t("workflow.nextLabel")}
        </div>
        <h2 className="mt-1 font-serif text-2xl text-ink" data-testid="workflow-card-title">
          {t(`workflow.card.${step}.title`)}
        </h2>
        <p className="mt-2 text-sm text-muted">{t(`workflow.card.${step}.body`)}</p>

        <div className="mt-4 flex flex-wrap items-center gap-3">
          <Button type="button" variant="primary" onClick={() => primary[step].run()}>
            {t(primary[step].ctaKey)}
          </Button>
          {step === "exportReview" && (
            <button
              type="button"
              className="text-sm text-muted underline-offset-2 hover:underline"
              onClick={() => onGoToTab("verify")}
            >
              {t("workflow.card.exportReview.secondary")}
            </button>
          )}
        </div>

        <div className="mt-5 border-t border-hairline pt-3 text-[11px] text-muted">
          {t("workflow.counts", { sources, excerpts, citations })}
          {verificationWarnings !== null && (
            <>
              {" · "}
              <span className={verificationWarnings > 0 ? "text-accent-warning" : "text-accent-verified"}>
                {verificationWarnings > 0
                  ? t("workflow.review.withWarnings", { count: verificationWarnings })
                  : t("workflow.review.coherent")}
              </span>
            </>
          )}
        </div>
      </div>
    </section>
  );
}
