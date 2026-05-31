import { useTranslation } from "react-i18next";
import { Button } from "../ui";

// Spec §3 comp 06: identity + settings entry, anchored to the sidebar foot.
export function SettingsBlock() {
  const { t } = useTranslation();
  return (
    <div
      data-testid="settings-block"
      className="mt-auto flex items-center gap-2 border-t border-hairline px-3 py-2"
    >
      <span className="grid h-7 w-7 place-items-center rounded-full bg-panel-2 font-mono text-xs">MR</span>
      <span className="min-w-0 flex-1 truncate text-sm">Avv. M. Rossi</span>
      <Button aria-label={t("settings.label")} title={t("settings.label")}>
        ⚙
      </Button>
    </div>
  );
}
