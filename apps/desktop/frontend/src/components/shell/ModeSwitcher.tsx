import { useTranslation } from "react-i18next";

export type ModeId = "conversation" | "review" | "drafting" | "reasoning" | "genealogy";
export const MODES: ModeId[] = ["conversation", "review", "drafting", "reasoning", "genealogy"];

interface ModeSwitcherProps {
  active: ModeId;
  onChange: (mode: ModeId) => void;
}

// Spec §3 comp 09: switches the workspace among the 5 modes. Must work in #3.
export function ModeSwitcher({ active, onChange }: ModeSwitcherProps) {
  const { t } = useTranslation();
  return (
    <div role="group" aria-label="Modalità" className="inline-flex gap-1 rounded border border-hairline bg-panel-2 p-1">
      {MODES.map((mode) => (
        <button
          key={mode}
          aria-pressed={active === mode}
          onClick={() => onChange(mode)}
          className={`rounded px-3 py-1 text-xs transition-colors ${
            active === mode ? "bg-panel text-ink" : "text-muted hover:text-ink"
          }`}
        >
          {t(`modes.${mode}`)}
        </button>
      ))}
    </div>
  );
}
