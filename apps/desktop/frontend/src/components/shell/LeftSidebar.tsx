import { useState } from "react";
import { useTranslation } from "react-i18next";
import { SearchInput, ListRow, Button } from "../ui";
import { SettingsBlock } from "./SettingsBlock";
import type { WorkspaceSummary } from "../../lib/ipc";

interface LeftSidebarProps {
  items: WorkspaceSummary[];
  loading: boolean;
  error: string | null;
  query: string;
  onQueryChange: (q: string) => void;
  onOpen: (id: string) => void;
  onNew: () => void;
  activeId: string | null;
}

const NAV_ITEMS = ["workspace", "matters", "knowledge"] as const;

// Spec §3 comp 03: primary navigation + matter access; settings at the foot.
// #5C: the matter list is now driven by the real persistence layer
// (searchWorkspaces); the search box filters it; "+ Nuova Pratica" creates one.
export function LeftSidebar({
  items,
  loading,
  error,
  query,
  onQueryChange,
  onOpen,
  onNew,
  activeId,
}: LeftSidebarProps) {
  const { t } = useTranslation();
  const [nav, setNav] = useState<(typeof NAV_ITEMS)[number]>("workspace");

  return (
    <nav
      data-testid="region-sidebar"
      aria-label={t("nav.aria")}
      className="flex min-h-0 flex-col border-r border-hairline bg-panel"
    >
      <div className="space-y-1 p-2">
        {NAV_ITEMS.map((item) => (
          <button
            key={item}
            aria-current={nav === item ? "page" : undefined}
            onClick={() => setNav(item)}
            className={`block w-full rounded px-2 py-1.5 text-left text-sm transition-colors ${
              nav === item ? "bg-panel-2 text-ink" : "text-muted hover:bg-panel-2 hover:text-ink"
            }`}
          >
            {t(`nav.${item}`)}
          </button>
        ))}
      </div>

      <div className="space-y-2 px-2 pb-2">
        <SearchInput
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          placeholder={t("sidebar.search")}
          aria-label={t("sidebar.search")}
        />
        <Button variant="primary" className="w-full justify-center" onClick={onNew}>
          {t("matters.new")}
        </Button>
      </div>

      <div className="min-h-0 flex-1 overflow-auto px-2" aria-label={t("matters.list")}>
        <div className="px-2 py-1 font-mono text-[11px] uppercase tracking-wide text-muted">
          {t("matters.list")}
        </div>

        {loading && <div className="px-2 py-1 text-xs text-muted">{t("matters.loading")}</div>}

        {error && (
          <div role="alert" className="px-2 py-1 text-xs text-accent-warning">
            {t("matters.errorLoad")}
          </div>
        )}

        {!loading && !error && items.length === 0 && (
          <div className="px-2 py-1 text-xs text-muted">{t("matters.empty")}</div>
        )}

        {items.map((m) => (
          <ListRow
            key={m.id}
            title={m.title}
            meta={m.client}
            active={activeId === m.id}
            onClick={() => onOpen(m.id)}
          />
        ))}
      </div>

      <SettingsBlock />
    </nav>
  );
}
