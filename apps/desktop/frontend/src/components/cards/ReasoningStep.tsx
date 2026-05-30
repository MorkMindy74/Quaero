import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge } from "../ui";
import type { MockReasoningStep } from "../../mock/data";

interface ReasoningStepProps {
  step: MockReasoningStep;
}

// Leaf (Spec §3 comp 12): one step of the AI reasoning trace. Mock flags.
export function ReasoningStep({ step }: ReasoningStepProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  return (
    <div className="rounded border border-hairline bg-panel p-2">
      <button onClick={() => setOpen(!open)} className="flex w-full items-center justify-between gap-2 text-left">
        <span className="text-sm">
          <span className="font-mono text-muted">{step.index}.</span> {step.claim}
        </span>
        <Badge tone={step.verified ? "verified" : "warning"}>
          {step.verified ? t("status.verified") : t("status.unverified")}
        </Badge>
      </button>
      {open && (
        <p className="mt-1 font-mono text-xs text-muted">{t("reasoning.linkedSources", { count: step.sources })}</p>
      )}
    </div>
  );
}
