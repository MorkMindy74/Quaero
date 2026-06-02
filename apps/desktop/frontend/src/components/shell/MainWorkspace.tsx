import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ModeSwitcher, type ModeId } from "./ModeSwitcher";
import { Panel } from "../ui";
import { ReasoningStep, GenealogyPreview } from "../cards";
import { DraftDocument } from "../workspace/DraftDocument";
import { DraftMetaRail } from "../workspace/DraftMetaRail";
import { ChatPanel } from "./ChatPanel";
import { WorkflowGuide, type GuideTab } from "./WorkflowGuide";
import { reasoningSteps, genealogyNodes, type MockMatter } from "../../mock/data";
import type { WorkspaceView } from "../../domain/types";

interface MainWorkspaceProps {
  matter: MockMatter | null;
  /** The REAL open Pratica (sidebar). When present the centre shows the guided
   *  workflow for it (#62), instead of the #3 mock. */
  workspace?: WorkspaceView;
  onGoToTab?: (tab: GuideTab) => void;
  onExport?: () => void;
}

// Spec §3 comp 04. #62: when a real Pratica is open the centre is the guided
// "next action"; otherwise it falls back to the #3 mock mode surfaces.
export function MainWorkspace({ matter, workspace, onGoToTab, onExport }: MainWorkspaceProps) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<ModeId>("conversation");

  if (workspace) {
    return (
      <main data-testid="region-workspace" className="flex min-h-0 flex-col overflow-hidden">
        <div className="border-b border-hairline px-6 py-4">
          <h1 className="font-serif text-xl">{workspace.matter.title}</h1>
          <p className="font-mono text-xs text-muted">
            {workspace.client.name}
            {workspace.matter.subject ? ` · ${workspace.matter.subject}` : ""}
          </p>
        </div>
        <div className="min-h-0 flex-1 overflow-auto p-6">
          <WorkflowGuide
            sources={workspace.sources?.length ?? 0}
            excerpts={workspace.excerpts?.length ?? 0}
            citations={workspace.citations?.length ?? 0}
            verificationWarnings={workspace.verification?.summary?.warnings ?? null}
            onGoToTab={(tab: GuideTab) => onGoToTab?.(tab)}
            onExport={() => onExport?.()}
          />
        </div>
      </main>
    );
  }

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
        <ModeSurface mode={mode} matterId={matter.id} />
      </div>
    </main>
  );
}

function ModeSurface({ mode, matterId }: { mode: ModeId; matterId: string }) {
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
      <div data-testid="surface-drafting" className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_260px]">
        <DraftDocument />
        <DraftMetaRail />
      </div>
    );
  }
  // `key` scopes the chat to the active matter: switching Pratica remounts the
  // panel and clears its in-memory history → no cross-matter/client bleed (#7).
  return <ChatPanel key={matterId} />;
}
