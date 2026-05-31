import { useTranslation } from "react-i18next";
import { Badge } from "../ui";

// Drafting mode — document surface (v0.3). Presentational mock; not editable.
export function DraftDocument() {
  const { t } = useTranslation();
  return (
    <article className="rounded border border-hairline bg-parchment p-6 shadow-sm">
      <div className="mb-4 flex items-start justify-between gap-3">
        <h2 className="font-serif text-lg leading-snug">{t("draft.title")}</h2>
        <Badge tone="warning">{t("draft.unvalidated")}</Badge>
      </div>
      <p className="mb-3 font-serif text-sm leading-relaxed">{t("draft.body1")}</p>
      <p className="mb-3 font-serif text-sm leading-relaxed">{t("draft.body2")}</p>
      <p className="font-serif text-sm leading-relaxed">{t("draft.body3")}</p>
      <p className="mt-5 border-t border-hairline pt-3 font-mono text-xs text-muted">{t("draft.morePages")}</p>
    </article>
  );
}
