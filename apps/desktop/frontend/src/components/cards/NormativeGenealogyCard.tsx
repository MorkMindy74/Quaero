import { useTranslation } from "react-i18next";
import { Panel, Badge } from "../ui";
import { normativeGenealogy } from "../../mock/data";

// Presentational mock only (functional idea: a norm's history over time).
// No real normative data, no scraping, no backend. Candidate for a future slice
// "Genealogia normativa / Normative Timeline".
export function NormativeGenealogyCard() {
  const { t } = useTranslation();
  const g = normativeGenealogy;
  return (
    <Panel parchment title={t("genealogy.normTitle")}>
      <p className="mb-3 text-xs text-muted">{t("genealogy.normSubtitle")}</p>

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
        <ol className="mt-1 flex flex-wrap items-center gap-1 font-mono text-xs">
          {g.timeline.map((v, i) => (
            <li key={v.id} className="flex items-center gap-1">
              <span className={v.current ? "text-accent-verified" : "text-accent-source"}>
                {v.date} · {v.label}
              </span>
              {i < g.timeline.length - 1 && <span className="text-muted">→</span>}
            </li>
          ))}
        </ol>
      </div>

      <div className="mt-3 flex items-center gap-2">
        <span className="font-mono text-[11px] uppercase tracking-wide text-muted">{t("genealogy.alert")}</span>
        <Badge tone="warning">{g.alert}</Badge>
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
