import { useState } from "react";
import { useTranslation } from "react-i18next";
import { TabButton } from "../ui";
import { SourceCard, ExcerptCard, ReasoningStep, GenealogyPreview, NormativeGenealogyCard } from "../cards";
import {
  sources,
  excerpts,
  reasoningSteps,
  genealogyNodes,
  memoryItems,
  agentActivity,
} from "../../mock/data";

type TabId = "sources" | "excerpts" | "reasoning" | "memory" | "genealogy" | "agent";
const TABS: TabId[] = ["sources", "excerpts", "reasoning", "memory", "genealogy", "agent"];

// Spec §3 comp 05 + §5: permanent evidence surface, never a drawer. Default = Sources.
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
      <div role="tablist" aria-label={t("context.aria")} className="flex flex-wrap gap-1 border-b border-hairline px-2 py-1">
        {TABS.map((id) => (
          <TabButton key={id} active={tab === id} onClick={() => setTab(id)}>
            {t(`tabs.${id}`)}
          </TabButton>
        ))}
      </div>

      <div role="tabpanel" className="min-h-0 flex-1 space-y-2 overflow-auto p-2">
        {tab === "sources" &&
          sources.map((source) => (
            <SourceCard
              key={source.id}
              source={source}
              selected={selected === source.id}
              onSelect={() => setSelected(source.id)}
            />
          ))}
        {tab === "excerpts" && excerpts.map((excerpt) => <ExcerptCard key={excerpt.id} excerpt={excerpt} />)}
        {tab === "reasoning" && reasoningSteps.map((step) => <ReasoningStep key={step.id} step={step} />)}
        {tab === "memory" &&
          memoryItems.map((item) => (
            <div key={item.id} className="rounded border border-hairline bg-panel p-2 text-sm">
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
            <div key={row.id} className="flex items-center justify-between rounded border border-hairline bg-panel p-2 text-sm">
              <span>{row.label}</span>
              <span className="font-mono text-xs text-muted">{t(`agent.${row.status}`)}</span>
            </div>
          ))}
      </div>
    </aside>
  );
}
