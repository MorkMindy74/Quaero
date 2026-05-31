import { useTranslation } from "react-i18next";

interface CommandPaletteTriggerProps {
  onOpen: () => void;
}

// Spec §3 comp 07: entry to the ⌘K palette; signals command-first design.
export function CommandPaletteTrigger({ onOpen }: CommandPaletteTriggerProps) {
  const { t } = useTranslation();
  return (
    <button
      onClick={onOpen}
      aria-keyshortcuts="Meta+K"
      className="inline-flex items-center gap-2 rounded border border-hairline bg-panel px-2 py-1 font-mono text-xs text-muted hover:text-ink"
    >
      <span>⌘K</span>
      <span className="hidden sm:inline">{t("topbar.commandHint")}</span>
    </button>
  );
}
