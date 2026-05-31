import { useState } from "react";
import { useTranslation } from "react-i18next";
import { TabButton } from "../ui";
import { SourceCard, ReasoningStep, GenealogyPreview, NormativeGenealogyCard } from "../cards";
import {
  workspaceView,
  reasoningSteps,
  genealogyNodes,
  memoryItems,
  agentActivity,
} from "../../mock/data";
import {
  SOURCE_TYPE_LABEL,
  type WorkspaceView,
  type Excerpt,
  type Citation,
  type SourceRef,
  type VerificationReport,
  type Finding,
} from "../../domain/types";

type TabId =
  | "sources"
  | "excerpts"
  | "reasoning"
  | "verify"
  | "memory"
  | "genealogy"
  | "agent";

const GROUPS: { label: string; tabs: TabId[] }[] = [
  { label: "tabs.groupEvidence", tabs: ["sources", "excerpts", "reasoning"] },
  { label: "tabs.groupContext", tabs: ["verify", "memory", "genealogy", "agent"] },
];

// Verifica tab (#9): read-only audit of the Estratto→Citazione chain, separate
// from Evidence (Estratti). Shows a positive verdict + summary counts and the
// findings (Warnings full-weight, Info lower-weight). No mock fallback.
function VerifyTab({ report }: { report?: VerificationReport }) {
  const { t } = useTranslation();
  if (!report) {
    return <p className="text-sm text-muted">{t("verify.empty")}</p>;
  }
  const { summary, findings } = report;
  const warnings = findings.filter((f) => f.severity === "Warning");
  const infos = findings.filter((f) => f.severity === "Info");
  const ref = (f: Finding) => f.excerptId ?? f.sourceId ?? f.citationId;
  return (
    <div className="space-y-3">
      <div
        className={`rounded border px-3 py-2 ${
          summary.warnings === 0 ? "border-hairline bg-panel" : "border-accent-warning bg-panel"
        }`}
      >
        <div className="text-sm font-medium">
          {summary.warnings === 0
            ? t("verify.coherent")
            : t("verify.withWarnings", { count: summary.warnings })}
        </div>
        <div className="mt-1 font-mono text-[11px] text-muted">
          {t("verify.summary", {
            citations: summary.citations,
            excerpts: summary.excerpts,
            documentBacked: summary.documentBackedExcerpts,
            pinned: summary.pinnedExcerpts,
          })}
        </div>
      </div>

      {warnings.map((f, i) => (
        <div key={`w${i}`} className="text-sm text-accent-warning">
          <span className="font-mono text-[10px] uppercase">{t(`verify.severity.${f.severity}`)}</span>{" "}
          {t(`verify.code.${f.code}`)}
          {ref(f) ? <span className="font-mono text-muted"> · {ref(f)}</span> : null}
        </div>
      ))}
      {infos.map((f, i) => (
        <div key={`i${i}`} className="text-xs text-muted opacity-80">
          <span className="font-mono text-[10px] uppercase">{t(`verify.severity.${f.severity}`)}</span>{" "}
          {t(`verify.code.${f.code}`)}
          {ref(f) ? <span> · {ref(f)}</span> : null}
        </div>
      ))}
    </div>
  );
}

// Counts for the static (#3 mock) tabs; sources and excerpts counts are derived
// from the active workspace at render time.
const STATIC_COUNTS: Partial<Record<TabId, number>> = {
  reasoning: reasoningSteps.length,
  memory: memoryItems.length,
  agent: agentActivity.length,
};

// Estratti tab (#8): real Estratti of the OPEN workspace (no mock fallback).
// Shows the verbatim quote, its Ancora, the Fonte it belongs to, and the
// Citazioni that cite it. Empty state when nothing is open or no excerpts.
function ExcerptsTab({
  excerpts,
  citations,
  sources,
}: {
  excerpts: Excerpt[];
  citations: Citation[];
  sources: SourceRef[];
}) {
  const { t } = useTranslation();
  if (excerpts.length === 0) {
    return <p className="text-sm text-muted">{t("empty.excerpts")}</p>;
  }
  const sourceTitle = (id: string) => sources.find((s) => s.id === id)?.title ?? id;
  return (
    <div className="space-y-3">
      {excerpts.map((ex) => (
        <div key={ex.id} className="rounded border border-hairline bg-panel p-2">
          <div className="font-mono text-[10px] uppercase tracking-wide text-muted">
            {sourceTitle(ex.sourceId)} · {ex.anchor.kind} {ex.anchor.value}
          </div>
          <blockquote className="mt-1 border-l-2 border-hairline pl-2 text-sm">“{ex.quote}”</blockquote>
          {citations
            .filter((c) => c.excerptId === ex.id)
            .map((c) => (
              <div key={c.id} className="mt-1 text-xs text-muted">
                ↳ {c.claim}
              </div>
            ))}
        </div>
      ))}
    </div>
  );
}

// Sources tab (slice #5A): the matter's Fonti grouped by typed Fascicolo views —
// dynamic dossiers (by SourceType) + a manual one. Demonstrates the domain model
// (Cliente → Pratica → Fascicolo/vista → Fonte) with many-to-many membership.
function SourcesTab({
  view,
  selected,
  onSelect,
  onImportFile,
  importError,
}: {
  view: WorkspaceView;
  selected: string | null;
  onSelect: (id: string) => void;
  onImportFile?: (file: File) => void;
  importError?: string | null;
}) {
  const { t } = useTranslation();
  const { client, matter, sources, dossiers } = view;
  const findSource = (id: string) => sources.find((s) => s.id === id);
  const dynamic = dossiers.filter((d) => d.kind === "Dynamic");
  const manual = dossiers.filter((d) => d.kind === "Manual");

  return (
    <div className="space-y-3">
      <div className="font-mono text-[11px] text-muted">
        {client.name} · {matter.title}
      </div>

      {onImportFile && (
        <div className="space-y-1">
          <label className="inline-flex cursor-pointer items-center gap-2 rounded border border-hairline bg-panel px-2 py-1 text-sm hover:bg-panel-2">
            <span>{t("documents.import")}</span>
            <input
              type="file"
              aria-label={t("documents.import")}
              className="sr-only"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) onImportFile(file);
                e.target.value = "";
              }}
            />
          </label>
          {importError && (
            <p role="alert" className="text-xs text-accent-warning">
              {importError}
            </p>
          )}
        </div>
      )}

      {dynamic.map((dossier) => (
        <div key={dossier.id} className="space-y-2">
          <div className="font-mono text-[10px] uppercase tracking-wide text-muted">
            {dossier.name} ({dossier.sources.length})
          </div>
          {dossier.sources.map((id) => {
            const s = findSource(id);
            if (!s) return null;
            return (
              <SourceCard
                key={`${dossier.id}:${id}`}
                source={{
                  id: s.id,
                  type: SOURCE_TYPE_LABEL[s.kind],
                  title: s.title,
                  meta: s.meta,
                  verified: s.kind !== "Nota",
                }}
                selected={selected === s.id}
                onSelect={() => onSelect(s.id)}
              />
            );
          })}
        </div>
      ))}

      {manual.map((dossier) => (
        <div key={dossier.id} className="rounded border border-hairline bg-panel p-2">
          <div className="font-mono text-[10px] uppercase tracking-wide text-muted">
            {dossier.name} ({dossier.sources.length}) · manuale
          </div>
          <ul className="mt-1 space-y-0.5 text-xs">
            {dossier.sources.map((id) => {
              const s = findSource(id);
              return s ? (
                <li key={`${dossier.id}:${id}`} className="truncate text-muted">
                  {s.title}
                </li>
              ) : null;
            })}
          </ul>
        </div>
      ))}
    </div>
  );
}

// Spec §3 comp 05 + §5: permanent evidence/control panel. Default = Sources.
// #5C: when a real workspace is open it drives the Sources tab; otherwise the
// panel falls back to the #3 mock view (no regression to the shell demo).
export function RightContextPanel({
  workspace,
  onImportFile,
  importError,
}: {
  workspace?: WorkspaceView;
  onImportFile?: (file: File) => void;
  importError?: string | null;
}) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<TabId>("sources");
  const [selected, setSelected] = useState<string | null>(null);

  const view = workspace ?? workspaceView;
  // #8/#9: Estratti/Citazioni/Verifica come ONLY from a real open workspace.
  const realExcerpts = workspace?.excerpts ?? [];
  const realCitations = workspace?.citations ?? [];
  const realVerification = workspace?.verification;
  const counts: Partial<Record<TabId, number>> = {
    ...STATIC_COUNTS,
    sources: view.sources.length,
    excerpts: realExcerpts.length,
    // badge only when there are warnings (no "0" noise)
    verify:
      realVerification && realVerification.summary.warnings > 0
        ? realVerification.summary.warnings
        : undefined,
  };

  return (
    <aside
      data-testid="region-context"
      aria-label={t("context.aria")}
      className="flex min-h-0 flex-col border-l border-hairline bg-panel-2"
    >
      <div className="space-y-2 border-b border-hairline px-3 py-2">
        {GROUPS.map((group) => (
          <div key={group.label}>
            <div className="mb-1 font-mono text-[10px] uppercase tracking-wide text-muted">{t(group.label)}</div>
            <div role="tablist" aria-label={t(group.label)} className="flex flex-wrap gap-1">
              {group.tabs.map((id) => (
                <TabButton
                  key={id}
                  active={tab === id}
                  onClick={() => setTab(id)}
                  count={counts[id]}
                  alert={id === "genealogy"}
                >
                  {t(`tabs.${id}`)}
                </TabButton>
              ))}
            </div>
          </div>
        ))}
      </div>

      <div role="tabpanel" className="min-h-0 flex-1 space-y-3 overflow-auto p-3 leading-relaxed">
        {tab === "sources" && (
          <SourcesTab
            view={view}
            selected={selected}
            onSelect={setSelected}
            onImportFile={onImportFile}
            importError={importError}
          />
        )}
        {tab === "excerpts" && (
          <ExcerptsTab
            excerpts={realExcerpts}
            citations={realCitations}
            sources={workspace?.sources ?? []}
          />
        )}
        {tab === "reasoning" && reasoningSteps.map((step) => <ReasoningStep key={step.id} step={step} />)}
        {tab === "verify" && <VerifyTab report={realVerification} />}
        {tab === "memory" &&
          memoryItems.map((item) => (
            <div key={item.id} className="rounded border border-hairline bg-panel p-3 text-sm">
              <span className="font-mono text-xs text-muted">{item.key}: </span>
              {item.note}
            </div>
          ))}
        {tab === "genealogy" && (
          <>
            <NormativeGenealogyCard />
            <GenealogyPreview nodes={genealogyNodes} />
          </>
        )}
        {tab === "agent" &&
          agentActivity.map((row) => (
            <div key={row.id} className="flex items-center justify-between rounded border border-hairline bg-panel p-3 text-sm">
              <span>{row.label}</span>
              <span className="font-mono text-xs text-muted">{t(`agent.${row.status}`)}</span>
            </div>
          ))}
      </div>
    </aside>
  );
}
