import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ModeSwitcher, type ModeId } from "./ModeSwitcher";
import { Panel } from "../ui";
import { ReasoningStep, GenealogyPreview } from "../cards";
import { DraftDocument } from "../workspace/DraftDocument";
import { DraftMetaRail } from "../workspace/DraftMetaRail";
import { ChatPanel } from "./ChatPanel";
import { WorkflowGuide, type GuideTab } from "./WorkflowGuide";
import { Badge } from "../ui";
import { reviewRows, type ReviewEsito } from "../../lib/review";
import { reasoningSteps, genealogyNodes, type MockMatter } from "../../mock/data";
import type { WorkspaceView } from "../../domain/types";

interface MainWorkspaceProps {
  matter: MockMatter | null;
  /** The REAL open Pratica (sidebar). When present the centre shows the guided
   *  workflow for it (#62), instead of the #3 mock. */
  workspace?: WorkspaceView;
  onGoToTab?: (tab: GuideTab) => void;
  onExport?: () => Promise<boolean>;
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
          <div className="mt-3">
            <ModeSwitcher active={mode} onChange={setMode} />
          </div>
        </div>
        <div className="min-h-0 flex-1 space-y-6 overflow-auto p-6">
          {/* The guided "next action" stays as the orientation hero (#62). */}
          <WorkflowGuide
            sources={workspace.sources?.length ?? 0}
            excerpts={workspace.excerpts?.length ?? 0}
            citations={workspace.citations?.length ?? 0}
            verificationWarnings={workspace.verification?.summary?.warnings ?? null}
            onGoToTab={(tab: GuideTab) => onGoToTab?.(tab)}
            onExport={() => onExport?.() ?? Promise.resolve(false)}
          />
          {/* The workbench: each mode bound to the REAL Pratica (#64). */}
          <RealModeSurface mode={mode} workspace={workspace} />
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

// #64 Cantiere reale: the operational modes for the REAL open Pratica. Surfaces
// are bound to real data where it exists (Conversazione, Revisione) and show an
// HONEST empty-state where it does not yet (Ragionamento, Genealogia, Redazione).
// No mock data, no invented data.
function RealModeSurface({ mode, workspace }: { mode: ModeId; workspace: WorkspaceView }) {
  const { t } = useTranslation();

  if (mode === "review") {
    return <ReviewSurface workspace={workspace} />;
  }
  if (mode === "reasoning") {
    return (
      <div data-testid="surface-reasoning">
        <EmptyMode title={t("modes.reasoning")} body={t("workspace.reasoningEmpty")} />
      </div>
    );
  }
  if (mode === "genealogy") {
    return (
      <div data-testid="surface-genealogy">
        <EmptyMode title={t("modes.genealogy")} body={t("workspace.genealogyEmpty")} />
      </div>
    );
  }
  if (mode === "drafting") {
    return (
      <div data-testid="surface-drafting">
        <EmptyMode title={t("modes.drafting")} body={t("workspace.draftingEmpty")} />
      </div>
    );
  }
  // conversation (default): the real, matter-scoped exploratory chat (#7).
  return (
    <div className="h-[420px]">
      <ChatPanel key={workspace.matter.id} />
    </div>
  );
}

// Honest empty-state for a mode whose real data does not exist yet (no mock).
function EmptyMode({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-lg border border-dashed border-hairline bg-panel p-6 text-center">
      <div className="font-serif text-lg text-ink">{title}</div>
      <p className="mx-auto mt-2 max-w-md text-sm text-muted">{body}</p>
    </div>
  );
}

// Read-only review of the REAL Estratto→Citazione chain (#64), with the per-row
// outcome derived from the #9 verification. Honest empty-state when there are no
// Estratti. Pure projection in `lib/review` — no domain logic here.
function ReviewSurface({ workspace }: { workspace: WorkspaceView }) {
  const { t } = useTranslation();
  const rows = reviewRows({
    sources: workspace.sources ?? [],
    excerpts: workspace.excerpts ?? [],
    citations: workspace.citations ?? [],
    verification: workspace.verification,
  });

  if (rows.length === 0) {
    return (
      <div data-testid="surface-review">
        <p className="text-sm text-muted">{t("workspace.review.empty")}</p>
      </div>
    );
  }

  const esitoLabel = (e: ReviewEsito) =>
    e === "coherent"
      ? t("workspace.review.coherent")
      : e === "warning"
        ? t("workspace.review.warning")
        : t("workspace.review.uncited");
  const esitoTone = (e: ReviewEsito): "default" | "verified" | "warning" =>
    e === "coherent" ? "verified" : e === "warning" ? "warning" : "default";

  return (
    <div data-testid="surface-review" className="space-y-1">
      <div className="grid grid-cols-[1fr_1.4fr_auto] gap-3 border-b border-hairline pb-1 font-mono text-[10px] uppercase tracking-wide text-muted">
        <span>{t("review.colFonte")}</span>
        <span>{t("review.colEstratto")}</span>
        <span>{t("review.colEsito")}</span>
      </div>
      {rows.map((r) => (
        <div
          key={`${r.kind}-${r.excerptId}`}
          className="grid grid-cols-[1fr_1.4fr_auto] items-start gap-3 border-b border-hairline py-2 text-sm"
        >
          <div>
            <div className="text-ink">{r.fonte}</div>
            <div className="font-mono text-[11px] text-muted">{r.anchor}</div>
          </div>
          <div>
            <blockquote className="border-l-2 border-hairline pl-2 text-[12px] text-muted">
              “{r.estratto}”
            </blockquote>
            {r.claim && <div className="mt-1 text-[13px] text-ink">↳ {r.claim}</div>}
          </div>
          <Badge tone={esitoTone(r.esito)}>{esitoLabel(r.esito)}</Badge>
        </div>
      ))}
    </div>
  );
}
