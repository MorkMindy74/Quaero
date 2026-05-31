import { useState } from "react";
import { useTranslation } from "react-i18next";
import { SearchInput, ListRow } from "../ui";
import { SettingsBlock } from "./SettingsBlock";
import { recentMatters, pinnedMatters, type MockMatter } from "../../mock/data";

interface LeftSidebarProps {
  matter: MockMatter | null;
  onSelectMatter: (matter: MockMatter) => void;
}

const NAV_ITEMS = ["workspace", "matters", "knowledge"] as const;

// Spec §3 comp 03: primary navigation + matter access; settings at the foot.
export function LeftSidebar({ matter, onSelectMatter }: LeftSidebarProps) {
  const { t } = useTranslation();
  const [nav, setNav] = useState<(typeof NAV_ITEMS)[number]>("workspace");

  return (
    <nav data-testid="region-sidebar" aria-label={t("nav.aria")} className="flex min-h-0 flex-col border-r border-hairline bg-panel">
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

      <div className="px-2 pb-2">
        <SearchInput placeholder={t("sidebar.search")} aria-label={t("sidebar.search")} />
      </div>

      <div className="min-h-0 flex-1 overflow-auto px-2">
        <SidebarList label={t("sidebar.recenti")} items={recentMatters} active={matter} onSelect={onSelectMatter} />
        <SidebarList label={t("sidebar.pinned")} items={pinnedMatters} active={matter} onSelect={onSelectMatter} />
      </div>

      <SettingsBlock />
    </nav>
  );
}

function SidebarList({
  label,
  items,
  active,
  onSelect,
}: {
  label: string;
  items: MockMatter[];
  active: MockMatter | null;
  onSelect: (matter: MockMatter) => void;
}) {
  return (
    <div className="mb-3">
      <div className="px-2 py-1 font-mono text-[11px] uppercase tracking-wide text-muted">{label}</div>
      {items.map((matter) => (
        <ListRow
          key={matter.id}
          title={matter.title}
          meta={matter.meta}
          active={active?.id === matter.id}
          onClick={() => onSelect(matter)}
        />
      ))}
    </div>
  );
}
