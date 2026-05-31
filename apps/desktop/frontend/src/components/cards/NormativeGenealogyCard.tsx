import { useTranslation } from "react-i18next";
import { Panel, Badge } from "../ui";
import { normativeGenealogy } from "../../mock/data";

// Presentational mock only (a norm's history over time). No real normative data,
// no scraping, no backend. Candidate for a future slice "Normative Timeline".
export function NormativeGenealogyCard() {
  const { t } = useTranslation();
  const g = normativeGenealogy;
  return (
    <Panel parchment title={t("genealogy.normTitle")}>
      <p className="mb-3 text-xs leading-relaxed text-muted">{t("genealogy.normSubtitle")}</p>

      <dl className="space-y-2 text-sm">
        <div className="flex items-center justify-between gap-2">
          <dt className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("genealogy.norma")}</dt>
          <dd className="font-serif">{g.norma}</dd>
        </div>
        <div className="flex items-center justify-between gap-2">
          <dt className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("genealogy.stato")}</dt>
          <dd>
            <Badge tone="verified">{g.status}</Badge>
          </dd>
        </div>
      </dl>

      <div className="mt-3">
        <div className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("genealogy.timeline")}</div>
        <ol className="mt-2 space-y-1.5 border-l border-hairline pl-3 font-mono text-xs">
          {g.timeline.map((v) => (
            <li key={v.id} className="relative">
              <span
                className={`absolute -left-[15px] top-1 h-2 w-2 rounded-full ${
                  v.current ? "bg-accent-verified" : "bg-accent-source"
                }`}
              />
              <span className={v.current ? "text-accent-verified" : ""}>
                {v.date} · {v.label}
              </span>
            </li>
          ))}
        </ol>
      </div>

      <div className="mt-3 rounded border border-accent-warning bg-parchment p-2 text-xs">
        <div className="font-mono text-[11px] uppercase tracking-wide text-accent-warning">{t("genealogy.alert")}</div>
        <p className="mt-1">{t("genealogy.alert1")}</p>
        <p className="mt-1 text-muted">{t("genealogy.alert2")}</p>
      </div>

      <div className="mt-3">
        <div className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("genealogy.linkedSources")}</div>
        <div className="mt-1 flex flex-wrap gap-1">
          {g.linkedSources.map((s) => (
            <Badge key={s} tone="source">
              {s}
            </Badge>
          ))}
        </div>
      </div>
    </Panel>
  );
}
