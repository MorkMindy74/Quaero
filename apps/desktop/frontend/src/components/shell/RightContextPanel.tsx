import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { TabButton, Button, Badge } from "../ui";
import { SourceCard, ReasoningStep, GenealogyPreview, NormativeGenealogyCard } from "../cards";
import { NewExcerptDialog, type ExcerptDialogValues } from "./NewExcerptDialog";
import { NewCitationDialog } from "./NewCitationDialog";
import { SourceTextPanel } from "./SourceTextPanel";
import { EvidenceCandidatesPanel } from "./EvidenceCandidatesPanel";
import { nextActionKey } from "../../lib/pilot";
import type { SourceText, EvidenceCandidate, LocalEvidenceResult } from "../../lib/ipc";
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
  onAddExcerpt,
  addExcerptError,
  onAddCitation,
  addCitationError,
  onExportMarkdown,
  exportError,
  onUpdateExcerpt,
  onDeleteExcerpt,
  onUpdateCitation,
  onDeleteCitation,
}: {
  excerpts: Excerpt[];
  citations: Citation[];
  sources: SourceRef[];
  onAddExcerpt?: (args: ExcerptDialogValues) => Promise<boolean>;
  addExcerptError?: string | null;
  onAddCitation?: (excerptId: string, claim: string) => Promise<boolean>;
  addCitationError?: string | null;
  onExportMarkdown?: () => Promise<boolean>;
  exportError?: string | null;
  onUpdateExcerpt?: (excerptId: string, args: ExcerptDialogValues) => Promise<boolean>;
  onDeleteExcerpt?: (excerptId: string) => Promise<boolean>;
  onUpdateCitation?: (citationId: string, claim: string) => Promise<boolean>;
  onDeleteCitation?: (citationId: string) => Promise<boolean>;
}) {
  const { t } = useTranslation();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [citing, setCiting] = useState<Excerpt | null>(null);
  const [editingExcerpt, setEditingExcerpt] = useState<Excerpt | null>(null);
  const [editingCitation, setEditingCitation] = useState<{ citation: Citation; excerpt: Excerpt } | null>(null);
  // Two-step delete confirm: which item id is awaiting "Confermi?".
  const [confirm, setConfirm] = useState<{ kind: "excerpt" | "citation"; id: string } | null>(null);
  const [deleteBlocked, setDeleteBlocked] = useState<string | null>(null);
  const [exportState, setExportState] = useState<"idle" | "busy" | "done">("idle");
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const sourceTitle = (id: string) => sources.find((s) => s.id === id)?.title ?? id;
  const documentSources = sources.filter((s) => s.kind === "Documento");

  useEffect(() => () => {
    if (hideTimer.current) clearTimeout(hideTimer.current);
  }, []);

  const handleExport = async () => {
    if (!onExportMarkdown) return;
    if (hideTimer.current) {
      clearTimeout(hideTimer.current);
      hideTimer.current = null;
    }
    setExportState("busy");
    const ok = await onExportMarkdown();
    setExportState(ok ? "done" : "idle");
    if (ok) {
      hideTimer.current = setTimeout(() => setExportState("idle"), 4000);
    }
  };

  const linkBtn = "text-[11px] text-muted underline-offset-2 hover:underline";

  const confirmDelete = async (kind: "excerpt" | "citation", id: string) => {
    setConfirm(null);
    setDeleteBlocked(null);
    if (kind === "citation") {
      if (onDeleteCitation) await onDeleteCitation(id);
    } else if (onDeleteExcerpt) {
      const ok = await onDeleteExcerpt(id);
      // The only delete refusal is "still cited" → surface a clear message.
      if (!ok) setDeleteBlocked(t("excerpts.deleteCitedBlocked"));
    }
  };

  // A small "Elimina X → Confermi eliminazione X? Sì/Annulla" inline control.
  // Labels are kind-specific so the Estratto and Citazione actions are never
  // ambiguous even when shown close together.
  const deleteControl = (kind: "excerpt" | "citation", id: string) => {
    const deleteLabel = kind === "excerpt" ? t("evidence.deleteExcerpt") : t("evidence.deleteCitation");
    const confirmLabel =
      kind === "excerpt" ? t("evidence.confirmDeleteExcerpt") : t("evidence.confirmDeleteCitation");
    return confirm && confirm.kind === kind && confirm.id === id ? (
      <span className="text-[11px] text-accent-warning">
        {confirmLabel}{" "}
        <button type="button" className={linkBtn} onClick={() => void confirmDelete(kind, id)}>
          {t("evidence.yes")}
        </button>{" "}
        <button type="button" className={linkBtn} onClick={() => setConfirm(null)}>
          {t("evidence.cancel")}
        </button>
      </span>
    ) : (
      <button
        type="button"
        className={linkBtn}
        onClick={() => {
          setDeleteBlocked(null);
          setConfirm({ kind, id });
        }}
      >
        {deleteLabel}
      </button>
    );
  };

  return (
    <div className="space-y-3">
      {(onAddExcerpt || onExportMarkdown) && (
        <div className="flex flex-wrap items-center gap-2">
          {onAddExcerpt && (
            <Button type="button" onClick={() => setDialogOpen(true)}>
              {t("excerpts.new")}
            </Button>
          )}
          {onExportMarkdown && (
            <Button type="button" disabled={exportState === "busy"} onClick={() => void handleExport()}>
              {exportState === "busy" ? t("export.exporting") : t("export.markdown")}
            </Button>
          )}
        </div>
      )}
      {onExportMarkdown && <p className="text-[11px] text-muted">{t("export.hint")}</p>}
      {exportState === "done" && (
        <p role="status" className="text-xs text-accent-verified">
          {t("export.done")}
        </p>
      )}
      {exportError && (
        <p role="alert" className="text-xs text-accent-warning">
          {exportError}
        </p>
      )}
      {deleteBlocked && (
        <p role="alert" className="text-xs text-accent-warning">
          {deleteBlocked}
        </p>
      )}

      {excerpts.length === 0 ? (
        <p className="text-sm text-muted">
          {sources.length === 0 ? t("pilot.empty.noSources") : t("pilot.empty.noExcerpts")}
        </p>
      ) : (
        excerpts.map((ex) => (
          <div key={ex.id} className="rounded border border-hairline bg-panel p-2">
            <div className="font-mono text-[10px] uppercase tracking-wide text-muted">
              {sourceTitle(ex.sourceId)} · {ex.anchor.kind} {ex.anchor.value}
            </div>
            <blockquote className="mt-1 border-l-2 border-hairline pl-2 text-sm">“{ex.quote}”</blockquote>
            {ex.note && <div className="mt-1 text-xs text-muted">— {ex.note}</div>}
            {ex.createdAt && (
              <div className="mt-1 font-mono text-[10px] text-muted">{ex.createdAt}</div>
            )}
            {citations
              .filter((c) => c.excerptId === ex.id)
              .map((c) => (
                <div key={c.id} className="mt-1 flex flex-wrap items-center gap-2 text-xs text-muted">
                  <span>↳ {c.claim}</span>
                  {onUpdateCitation && (
                    <button
                      type="button"
                      className={linkBtn}
                      onClick={() => setEditingCitation({ citation: c, excerpt: ex })}
                    >
                      {t("evidence.editCitation")}
                    </button>
                  )}
                  {onDeleteCitation && deleteControl("citation", c.id)}
                </div>
              ))}
            {/* Excerpt-level actions, separated from the per-citation actions above. */}
            <div className="mt-2 flex flex-wrap items-center gap-2 border-t border-hairline pt-2">
              {onAddCitation && (
                <button type="button" onClick={() => setCiting(ex)} className={linkBtn}>
                  {t("citations.cite")}
                </button>
              )}
              {onUpdateExcerpt && (
                <button type="button" onClick={() => setEditingExcerpt(ex)} className={linkBtn}>
                  {t("evidence.editExcerpt")}
                </button>
              )}
              {onDeleteExcerpt && deleteControl("excerpt", ex.id)}
            </div>
          </div>
        ))
      )}

      {dialogOpen && onAddExcerpt && (
        <NewExcerptDialog
          sources={documentSources}
          error={addExcerptError ?? null}
          onClose={() => setDialogOpen(false)}
          onSubmit={onAddExcerpt}
        />
      )}

      {editingExcerpt && onUpdateExcerpt && (
        <NewExcerptDialog
          sources={documentSources}
          initial={{
            sourceId: editingExcerpt.sourceId,
            sourceTitle: sourceTitle(editingExcerpt.sourceId),
            anchorKind: editingExcerpt.anchor.kind,
            anchorValue: editingExcerpt.anchor.value,
            quote: editingExcerpt.quote,
            note: editingExcerpt.note,
          }}
          error={addExcerptError ?? null}
          onClose={() => setEditingExcerpt(null)}
          onSubmit={(args) => onUpdateExcerpt(editingExcerpt.id, args)}
        />
      )}

      {citing && onAddCitation && (
        <NewCitationDialog
          excerpt={citing}
          error={addCitationError ?? null}
          onClose={() => setCiting(null)}
          onSubmit={(claim) => onAddCitation(citing.id, claim)}
        />
      )}

      {editingCitation && onUpdateCitation && (
        <NewCitationDialog
          excerpt={editingCitation.excerpt}
          initialClaim={editingCitation.citation.claim}
          error={addCitationError ?? null}
          onClose={() => setEditingCitation(null)}
          onSubmit={(claim) => onUpdateCitation(editingCitation.citation.id, claim)}
        />
      )}
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
  matterId,
  onGetSourceText,
  onSetSourceText,
  onProposeEvidence,
  onAcceptEvidence,
  evidenceLocalEnabled,
  onProposeEvidenceLocal,
}: {
  view: WorkspaceView;
  selected: string | null;
  onSelect: (id: string) => void;
  onImportFile?: (file: File) => void;
  importError?: string | null;
  matterId?: string;
  onGetSourceText?: (matterId: string, sourceId: string) => Promise<SourceText>;
  onSetSourceText?: (
    matterId: string,
    sourceId: string,
    expectedSha256: string,
    text: string,
  ) => Promise<SourceText>;
  onProposeEvidence?: (matterId: string, sourceId: string) => Promise<EvidenceCandidate[]>;
  onAcceptEvidence?: (
    matterId: string,
    sourceId: string,
    anchorKind: string,
    anchorValue: string,
    quote: string,
    note?: string,
  ) => Promise<boolean>;
  evidenceLocalEnabled?: boolean;
  onProposeEvidenceLocal?: (
    matterId: string,
    sourceId: string,
  ) => Promise<LocalEvidenceResult>;
}) {
  const { t } = useTranslation();
  const { client, matter, sources, dossiers } = view;
  const findSource = (id: string) => sources.find((s) => s.id === id);
  const dynamic = dossiers.filter((d) => d.kind === "Dynamic");
  const manual = dossiers.filter((d) => d.kind === "Manual");
  // Text layer (#52): for a real open Pratica, the selected Documento source
  // gets a text-layer panel (extract/preview). Shown once (not per dossier).
  const selectedSource = selected ? findSource(selected) : undefined;
  const isSelectedDocument =
    !!matterId && selectedSource?.kind === "Documento" && !!selectedSource.file;
  const showTextLayer = isSelectedDocument && !!onGetSourceText && !!onSetSourceText;
  // Evidence Assistant (#55 V1A + #58 V1B): same gating; proposes candidate Estratti.
  const showEvidence = isSelectedDocument && !!onProposeEvidence && !!onAcceptEvidence;

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

      {showTextLayer && selectedSource && matterId && onGetSourceText && onSetSourceText && (
        <SourceTextPanel
          // Remount per matter:source so a fresh panel never inherits an
          // in-flight extraction's state from a previously-selected Fonte.
          key={`${matterId}:${selectedSource.id}`}
          matterId={matterId}
          source={selectedSource}
          onGet={onGetSourceText}
          onSet={onSetSourceText}
        />
      )}

      {showEvidence && selectedSource && matterId && onProposeEvidence && onAcceptEvidence && (
        <EvidenceCandidatesPanel
          // Remount per matter:source so candidates never leak across Fonti.
          key={`evidence:${matterId}:${selectedSource.id}`}
          matterId={matterId}
          source={selectedSource}
          localEnabled={evidenceLocalEnabled}
          onPropose={onProposeEvidence}
          onProposeLocal={onProposeEvidenceLocal}
          onAccept={onAcceptEvidence}
        />
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

// Pilot-readiness: a compact "Stato Pratica" shown at the top of the panel when a
// real Pratica is open. Pure derivation from the view (counts + #9 verdict) plus
// the next suggested step along Fonti → Estratti → Citazioni → Export.
function PraticaStatus({ workspace }: { workspace: WorkspaceView }) {
  const { t } = useTranslation();
  // Defensive: the canonical view always carries these arrays, but be lenient
  // with partial fixtures (count what's present).
  const sources = workspace.sources?.length ?? 0;
  const excerpts = workspace.excerpts?.length ?? 0;
  const citations = workspace.citations?.length ?? 0;
  const v = workspace.verification;
  const verdict = v
    ? v.summary.warnings === 0
      ? t("verify.coherent")
      : t("verify.withWarnings", { count: v.summary.warnings })
    : "—";
  const action = t(nextActionKey({ sources, excerpts, citations }));
  return (
    <div
      data-testid="pratica-status"
      className="border-b border-hairline bg-panel px-3 py-2 text-xs"
    >
      <div className="mb-1 font-mono text-[10px] uppercase tracking-wide text-muted">
        {t("pilot.status.title")}
      </div>
      <div className="text-muted">{t("pilot.status.counts", { sources, excerpts, citations })}</div>
      <div className="text-muted">{t("pilot.status.verify", { verdict })}</div>
      {/* Next action as a calm, professional callout (no popup/animation): a
          bordered box with a left accent + "Prossima azione" badge + the action
          text in a clearer hierarchy. Content stays driven by nextActionKey. */}
      <div
        data-testid="next-action"
        role="note"
        aria-label={t("pilot.status.nextBadge")}
        className="mt-2 rounded border border-hairline border-l-2 border-l-accent-source bg-panel-2 px-2 py-1.5"
      >
        <Badge tone="source">{t("pilot.status.nextBadge")}</Badge>
        <div className="mt-1 text-sm font-medium text-ink">{action}</div>
      </div>
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
  onAddExcerpt,
  addExcerptError,
  onAddCitation,
  addCitationError,
  onExportMarkdown,
  exportError,
  onUpdateExcerpt,
  onDeleteExcerpt,
  onUpdateCitation,
  onDeleteCitation,
  onGetSourceText,
  onSetSourceText,
  onProposeEvidence,
  onAcceptEvidence,
  evidenceLocalEnabled,
  onProposeEvidenceLocal,
}: {
  workspace?: WorkspaceView;
  onImportFile?: (file: File) => void;
  importError?: string | null;
  onAddExcerpt?: (args: {
    sourceId: string;
    anchorKind: string;
    anchorValue: string;
    quote: string;
    note?: string;
  }) => Promise<boolean>;
  addExcerptError?: string | null;
  onAddCitation?: (excerptId: string, claim: string) => Promise<boolean>;
  addCitationError?: string | null;
  onExportMarkdown?: () => Promise<boolean>;
  exportError?: string | null;
  onUpdateExcerpt?: (excerptId: string, args: ExcerptDialogValues) => Promise<boolean>;
  onDeleteExcerpt?: (excerptId: string) => Promise<boolean>;
  onUpdateCitation?: (citationId: string, claim: string) => Promise<boolean>;
  onDeleteCitation?: (citationId: string) => Promise<boolean>;
  onGetSourceText?: (matterId: string, sourceId: string) => Promise<SourceText>;
  onSetSourceText?: (
    matterId: string,
    sourceId: string,
    expectedSha256: string,
    text: string,
  ) => Promise<SourceText>;
  onProposeEvidence?: (matterId: string, sourceId: string) => Promise<EvidenceCandidate[]>;
  onAcceptEvidence?: (
    matterId: string,
    sourceId: string,
    anchorKind: string,
    anchorValue: string,
    quote: string,
    note?: string,
  ) => Promise<boolean>;
  evidenceLocalEnabled?: boolean;
  onProposeEvidenceLocal?: (
    matterId: string,
    sourceId: string,
  ) => Promise<LocalEvidenceResult>;
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
      {workspace && <PraticaStatus workspace={workspace} />}
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
            matterId={workspace?.matter.id}
            onGetSourceText={onGetSourceText}
            onSetSourceText={onSetSourceText}
            onProposeEvidence={onProposeEvidence}
            onAcceptEvidence={onAcceptEvidence}
            evidenceLocalEnabled={evidenceLocalEnabled}
            onProposeEvidenceLocal={onProposeEvidenceLocal}
          />
        )}
        {tab === "excerpts" && (
          <ExcerptsTab
            excerpts={realExcerpts}
            citations={realCitations}
            sources={workspace?.sources ?? []}
            onAddExcerpt={onAddExcerpt}
            addExcerptError={addExcerptError}
            onAddCitation={onAddCitation}
            addCitationError={addCitationError}
            onExportMarkdown={onExportMarkdown}
            exportError={exportError}
            onUpdateExcerpt={onUpdateExcerpt}
            onDeleteExcerpt={onDeleteExcerpt}
            onUpdateCitation={onUpdateCitation}
            onDeleteCitation={onDeleteCitation}
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
