import { useTranslation } from "react-i18next";
import { Badge, Button } from "../ui";
import { sources } from "../../mock/data";

// Drafting mode — meta rail (v0.3). All actions are mock / non-functional.
export function DraftMetaRail() {
  const { t } = useTranslation();
  return (
    <aside className="space-y-4 rounded border border-hairline bg-panel p-4 text-sm">
      <div>
        <div className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("genealogy.stato")}</div>
        <div className="mt-1">
          <Badge tone="warning">{t("draft.metaState")}</Badge>
        </div>
      </div>

      <div>
        <div className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("draft.flow")}</div>
        <ol className="mt-1 flex items-center gap-1 font-mono text-xs">
          <li className="text-muted">{t("draft.stepOutput")}</li>
          <li className="text-muted">→</li>
          <li className="rounded bg-accent-source px-1.5 py-0.5 text-background">{t("draft.stepBozza")}</li>
          <li className="text-muted">→</li>
          <li className="text-muted">{t("draft.stepDocumento")}</li>
        </ol>
      </div>

      <div>
        <div className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("draft.origin")}</div>
        <p className="mt-1 text-xs text-muted">{t("draft.originCount")}</p>
        <div className="mt-1 flex flex-wrap gap-1">
          {sources.map((s) => (
            <Badge key={s.id} tone={s.verified ? "verified" : "source"}>
              {s.title}
            </Badge>
          ))}
        </div>
      </div>

      <p className="rounded border border-accent-warning bg-parchment p-2 text-xs">{t("draft.warning")}</p>

      <div className="flex flex-wrap gap-2">
        <Button>{t("draft.actionOpen")}</Button>
        <Button>{t("draft.actionVerify")}</Button>
        <Button variant="primary">{t("draft.actionValidate")}</Button>
      </div>
    </aside>
  );
}
