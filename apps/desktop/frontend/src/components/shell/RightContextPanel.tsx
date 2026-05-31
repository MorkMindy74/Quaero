import { useState } from "react";
import { useTranslation } from "react-i18next";
import { TabButton } from "../ui";
import { SourceCard, ExcerptCard, ReasoningStep, GenealogyPreview, NormativeGenealogyCard } from "../cards";
import {
  workspaceView,
  excerpts,
  reasoningSteps,
  genealogyNodes,
  memoryItems,
  agentActivity,
} from "../../mock/data";
import { SOURCE_TYPE_LABEL } from "../../domain/types";

type TabId = "sources" | "excerpts" | "reasoning" | "memory" | "genealogy" | "agent";

const GROUPS: { label: string; tabs: TabId[] }[] = [
  { label: "tabs.groupEvidence", tabs: ["sources", "excerpts", "reasoning"] },
  { label: "tabs.groupContext", tabs: ["memory", "genealogy", "agent"] },
];

const COUNTS: Partial<Record<TabId, number>> = {
  sources: workspaceView.sources.length,
  excerpts: excerpts.length,
  reasoning: reasoningSteps.length,
  memory: memoryItems.length,
  agent: agentActivity.length,
};

// Sources tab (slice #5A): the matter's Fonti grouped by typed Fascicolo views —
// dynamic dossiers (by SourceType) + a manual one. Demonstrates the domain model
// (Cliente → Pratica → Fascicolo/vista → Fonte) with many-to-many membership.
function SourcesTab({ selected, onSelect }: { selected: string | null; onSelect: (id: string) => void }) {
  const { client, matter, sources, dossiers } = workspaceView;
  const findSource = (id: string) => sources.find((s) => s.id === id);
  const dynamic = dossiers.filter((d) => d.kind === "Dynamic");
  const manual = dossiers.filter((d) => d.kind === "Manual");

  return (
    <div className="space-y-3">
      <div className="font-mono text-[11px] text-muted">
        {client.name} · {matter.title}
      </div>

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
export function RightContextPanel() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<TabId>("sources");
  const [selected, setSelected] = useState<string | null>(null);

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
                  count={COUNTS[id]}
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
        {tab === "sources" && <SourcesTab selected={selected} onSelect={setSelected} />}
        {tab === "excerpts" && excerpts.map((excerpt) => <ExcerptCard key={excerpt.id} excerpt={excerpt} />)}
        {tab === "reasoning" && reasoningSteps.map((step) => <ReasoningStep key={step.id} step={step} />)}
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
