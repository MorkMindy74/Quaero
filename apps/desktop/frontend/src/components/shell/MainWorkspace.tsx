import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ModeSwitcher, type ModeId } from "./ModeSwitcher";
import { Panel } from "../ui";
import { ReasoningStep, GenealogyPreview } from "../cards";
import { reasoningSteps, genealogyNodes, type MockMatter } from "../../mock/data";

interface MainWorkspaceProps {
  matter: MockMatter | null;
}

// Spec §3 comp 04: matter header + ModeSwitcher + active mode surface (region 3).
export function MainWorkspace({ matter }: MainWorkspaceProps) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<ModeId>("conversation");

  if (!matter) {
    return (
      <main data-testid="region-workspace" className="grid min-h-0 place-items-center overflow-auto">
        <p data-testid="workspace-empty" className="font-serif text-lg text-muted">
          {t("workspace.noMatter")}
        </p>
      </main>
    );
  }

  return (
    <main data-testid="region-workspace" className="flex min-h-0 flex-col overflow-hidden">
      <div className="border-b border-hairline px-6 py-4">
        <h1 className="font-serif text-xl">{matter.title}</h1>
        <p className="font-mono text-xs text-muted">{matter.meta}</p>
        <div className="mt-3">
          <ModeSwitcher active={mode} onChange={setMode} />
        </div>
      </div>
      <div className="min-h-0 flex-1 overflow-auto p-6">
        <ModeSurface mode={mode} />
      </div>
    </main>
  );
}

function ModeSurface({ mode }: { mode: ModeId }) {
  const { t } = useTranslation();

  if (mode === "reasoning") {
    return (
      <div data-testid="surface-reasoning" className="space-y-2">
        {reasoningSteps.map((step) => (
          <ReasoningStep key={step.id} step={step} />
        ))}
      </div>
    );
  }
  if (mode === "genealogy") {
    return (
      <div data-testid="surface-genealogy">
        <GenealogyPreview nodes={genealogyNodes} />
      </div>
    );
  }
  if (mode === "review") {
    return (
      <Panel parchment title={t("modes.review")}>
        <div data-testid="surface-review" className="grid grid-cols-3 gap-2 font-mono text-xs text-muted">
          <span>{t("review.colFonte")}</span>
          <span>{t("review.colEstratto")}</span>
          <span>{t("review.colEsito")}</span>
        </div>
      </Panel>
    );
  }
  if (mode === "drafting") {
    return (
      <Panel parchment title={t("modes.drafting")}>
        <div data-testid="surface-drafting" className="text-sm text-muted">
          — bozza (placeholder) · sigillo AI / umano —
        </div>
      </Panel>
    );
  }
  return (
    <div data-testid="surface-conversation" className="grid h-full place-items-center text-center text-muted">
      <p className="text-sm">{t("conversation.grounded", { count: 3 })}</p>
    </div>
  );
}
