import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge, Button } from "../ui";
import type { SourceRef } from "../../domain/types";
import type { EvidenceCandidate } from "../../lib/ipc";

/** A candidate row in the review list. Candidates are NEVER persisted until the
 *  lawyer approves; `created` flips once a real Estratto has been made. */
type Row = EvidenceCandidate & { key: string; created: boolean; error: string | null };

/** AI Evidence Assistant V1A (#55). For a selected Documento with a local text
 *  layer (#52), proposes candidate Estratti (offline Stub — no LLM, no egress).
 *  The lawyer approves/edits/discards; only an approved candidate becomes a real
 *  Estratto, and only if its quote is verified against the text layer server-side.
 *  Nothing here is auto-saved. */
export function EvidenceCandidatesPanel({
  matterId,
  source,
  onPropose,
  onAccept,
}: {
  matterId: string;
  source: SourceRef;
  onPropose: (matterId: string, sourceId: string) => Promise<EvidenceCandidate[]>;
  onAccept: (
    matterId: string,
    sourceId: string,
    anchorKind: string,
    anchorValue: string,
    quote: string,
    note?: string,
  ) => Promise<boolean>;
}) {
  const { t } = useTranslation();
  const [rows, setRows] = useState<Row[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);

  const patch = (key: string, p: Partial<Row>) =>
    setRows((rs) => rs?.map((r) => (r.key === key ? { ...r, ...p } : r)) ?? rs);

  const propose = async () => {
    setError(null);
    setEditing(null);
    setBusy(true);
    try {
      const cands = await onPropose(matterId, source.id);
      setRows(cands.map((c, i) => ({ ...c, key: `cand-${i}`, created: false, error: null })));
    } catch {
      setError(t("evidenceAI.proposeError"));
    } finally {
      setBusy(false);
    }
  };

  const approve = async (row: Row) => {
    patch(row.key, { error: null });
    const ok = await onAccept(
      matterId,
      source.id,
      row.anchorKind,
      row.anchorValue,
      row.quote,
      row.reason,
    );
    if (ok) {
      patch(row.key, { created: true });
      setEditing(null);
    } else {
      patch(row.key, { error: t("evidenceAI.acceptError") });
    }
  };

  const discard = (key: string) => setRows((rs) => rs?.filter((r) => r.key !== key) ?? rs);

  const linkBtn = "text-[11px] text-muted underline-offset-2 hover:underline";
  const pending = rows?.filter((r) => !r.created).length ?? 0;

  return (
    <div data-testid="evidence-panel" className="rounded border border-hairline bg-panel p-2">
      <div className="flex items-center justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wide text-muted">
          {t("evidenceAI.title")}
        </span>
        <Button type="button" disabled={busy} onClick={() => void propose()}>
          {busy ? t("evidenceAI.proposing") : t("evidenceAI.propose")}
        </Button>
      </div>

      <p className="mt-1 text-[10px] text-muted">{t("evidenceAI.disclaimer")}</p>

      {error && (
        <p role="alert" className="mt-1 text-[11px] text-accent-warning">
          {error}
        </p>
      )}

      {rows !== null && rows.length === 0 && (
        <p className="mt-2 text-[11px] text-muted">{t("evidenceAI.empty")}</p>
      )}

      {pending > 0 && (
        <p className="mt-2 text-[11px] text-muted">{t("evidenceAI.unsavedHint")}</p>
      )}

      {rows?.map((row) => (
        <div
          key={row.key}
          data-testid="evidence-candidate"
          className="mt-2 rounded border border-hairline bg-panel-2 p-2"
        >
          <div className="flex items-center justify-between gap-2">
            <span className="font-mono text-[10px] uppercase tracking-wide text-muted">
              {row.anchorKind} {row.anchorValue}
            </span>
            <Badge tone={row.created ? "verified" : "default"}>
              <span data-testid="evidence-candidate-status">
                {row.created ? t("evidenceAI.created") : t("evidenceAI.unsaved")}
              </span>
            </Badge>
          </div>

          {editing === row.key && !row.created ? (
            <div className="mt-1 space-y-1">
              <textarea
                aria-label={t("evidenceAI.quote")}
                className="w-full rounded border border-hairline bg-panel p-1 text-sm"
                rows={3}
                value={row.quote}
                onChange={(e) => patch(row.key, { quote: e.target.value })}
              />
              <div className="flex gap-1">
                <input
                  aria-label={t("evidenceAI.anchorKind")}
                  className="w-1/2 rounded border border-hairline bg-panel p-1 text-xs"
                  value={row.anchorKind}
                  onChange={(e) => patch(row.key, { anchorKind: e.target.value })}
                />
                <input
                  aria-label={t("evidenceAI.anchorValue")}
                  className="w-1/2 rounded border border-hairline bg-panel p-1 text-xs"
                  value={row.anchorValue}
                  onChange={(e) => patch(row.key, { anchorValue: e.target.value })}
                />
              </div>
            </div>
          ) : (
            <blockquote className="mt-1 border-l-2 border-hairline pl-2 text-sm">
              “{row.quote}”
            </blockquote>
          )}

          <div className="mt-1 text-[11px] text-muted">— {row.reason}</div>

          {row.error && (
            <p role="alert" className="mt-1 text-[11px] text-accent-warning">
              {row.error}
            </p>
          )}

          {!row.created && (
            <div className="mt-2 flex flex-wrap items-center gap-2 border-t border-hairline pt-2">
              <button type="button" className={linkBtn} onClick={() => void approve(row)}>
                {t("evidenceAI.approve")}
              </button>
              <button
                type="button"
                className={linkBtn}
                onClick={() => setEditing(editing === row.key ? null : row.key)}
              >
                {editing === row.key ? t("evidenceAI.editDone") : t("evidenceAI.edit")}
              </button>
              <button type="button" className={linkBtn} onClick={() => discard(row.key)}>
                {t("evidenceAI.discard")}
              </button>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
