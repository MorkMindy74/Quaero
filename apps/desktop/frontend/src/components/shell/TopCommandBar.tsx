import { useTranslation } from "react-i18next";
import { Button } from "../ui";
import { CommandPaletteTrigger } from "./CommandPaletteTrigger";
import { MatterSelector } from "./MatterSelector";
import type { MockMatter } from "../../mock/data";

interface TopCommandBarProps {
  matter: MockMatter | null;
  onSelectMatter: (matter: MockMatter) => void;
  onOpenPalette: () => void;
}

// Spec §3 comp 02: global command + orientation strip; never scrolls (region 1).
export function TopCommandBar({ matter, onSelectMatter, onOpenPalette }: TopCommandBarProps) {
  const { t, i18n } = useTranslation();
  return (
    <header
      data-testid="region-topbar"
      role="banner"
      className="flex items-center gap-3 border-b border-hairline bg-panel px-3 py-1.5"
    >
      <span className="font-serif text-base font-semibold">{t("app.name")}</span>
      <CommandPaletteTrigger onOpen={onOpenPalette} />
      <MatterSelector selected={matter} onSelect={onSelectMatter} />
      <div className="ml-auto flex items-center gap-3 font-mono text-xs text-muted">
        <span className="inline-flex items-center gap-1 text-accent-verified">
          <span className="h-2 w-2 rounded-full bg-accent-verified" /> local
        </span>
        <span>{t("topbar.model")} ▾</span>
        <span className="inline-flex items-center gap-1">
          <button onClick={() => void i18n.changeLanguage("it")} className="hover:text-ink">
            IT
          </button>
          <span>/</span>
          <button onClick={() => void i18n.changeLanguage("en")} className="hover:text-ink">
            EN
          </button>
        </span>
        <Button aria-label={t("settings.label")} title={t("settings.label")}>
          ⚙
        </Button>
      </div>
    </header>
  );
}
