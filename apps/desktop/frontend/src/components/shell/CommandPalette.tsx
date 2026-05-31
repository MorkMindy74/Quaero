import { useTranslation } from "react-i18next";

interface CommandPaletteProps {
  onClose: () => void;
}

// The single floating layer (Spec §7). Stub overlay: opens/closes, no real commands.
export function CommandPalette({ onClose }: CommandPaletteProps) {
  const { t } = useTranslation();
  return (
    <div
      role="dialog"
      aria-label={t("palette.title")}
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/20 pt-24"
      onClick={onClose}
    >
      <div
        className="w-[480px] overflow-hidden rounded-lg border border-hairline bg-panel shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <input
          autoFocus
          placeholder={t("palette.placeholder")}
          aria-label={t("palette.title")}
          className="w-full border-b border-hairline bg-transparent px-3 py-2 text-sm outline-none"
        />
        <div className="p-3 font-mono text-xs text-muted">{t("palette.empty")}</div>
      </div>
    </div>
  );
}
