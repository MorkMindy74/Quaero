import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge, Button } from "../ui";
import type { SourceRef } from "../../domain/types";
import type { EvidenceCandidate, LocalEvidenceResult } from "../../lib/ipc";

/** A candidate row. `valid` = the quote was verified present in the text layer
 *  (Stub candidates are valid by construction; local-model ones are scored).
 *  Candidates are NEVER persisted until approved; `created` flips after that. */
type Row = EvidenceCandidate & {
  key: string;
  created: boolean;
  error: string | null;
  valid: boolean;
};

/** AI Evidence Assistant (#55 V1A + #58 V1B). For a selected Documento with a
 *  text layer (#52): proposes candidate Estratti either with the offline Stub or,
 *  if enabled, with the LOCAL Ollama model (after explicit consent). The model is
 *  never trusted: invalid candidates (quote not in the text) are shown but not
 *  approvable, and approval re-verifies server-side. Nothing is auto-saved. */
export function EvidenceCandidatesPanel({
  matterId,
  source,
  localEnabled,
  onPropose,
  onProposeLocal,
  onAccept,
}: {
  matterId: string;
  source: SourceRef;
  localEnabled?: boolean;
  onPropose: (matterId: string, sourceId: string) => Promise<EvidenceCandidate[]>;
  onProposeLocal?: (
    matterId: string,
    sourceId: string,
    consent: boolean,
  ) => Promise<LocalEvidenceResult>;
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
  const [consentOpen, setConsentOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  // Per-row "approval in flight" guard (synchronous ref blocks a same-tick
  // double-click; state mirror disables the button).
  const acceptingRef = useRef<Set<string>>(new Set());
  const [accepting, setAccepting] = useState<readonly string[]>([]);
  const isAccepting = (key: string) => accepting.includes(key);

  const patch = (key: string, p: Partial<Row>) =>
    setRows((rs) => rs?.map((r) => (r.key === key ? { ...r, ...p } : r)) ?? rs);

  const reset = () => {
    setError(null);
    setEditing(null);
    setNotice(null);
  };

  const proposeStub = async () => {
    reset();
    setConsentOpen(false);
    setBusy(true);
    try {
      const cands = await onPropose(matterId, source.id);
      setRows(
        cands.map((c, i) => ({ ...c, key: `stub-${i}`, created: false, error: null, valid: true })),
      );
    } catch {
      setError(t("evidenceAI.proposeError"));
    } finally {
      setBusy(false);
    }
  };

  const proposeLocal = async () => {
    if (!onProposeLocal) return;
    reset();
    setConsentOpen(false);
    setBusy(true);
    try {
      const res = await onProposeLocal(matterId, source.id, true);
      setRows(
        res.candidates.map((c, i) => ({
          quote: c.quote,
          anchorKind: c.anchorKind,
          anchorValue: c.anchorValue,
          reason: c.reason,
          key: `local-${i}`,
          created: false,
          error: null,
          valid: c.valid,
        })),
      );
      if (res.truncated) setNotice(t("evidenceAI.truncated", { chars: res.analyzedChars }));
    } catch {
      setError(t("evidenceAI.proposeLocalError"));
    } finally {
      setBusy(false);
    }
  };

  const approve = async (row: Row) => {
    // Never approve an invalid (model-hallucinated) candidate; guard re-entrancy.
    if (row.created || !row.valid || acceptingRef.current.has(row.key)) return;
    acceptingRef.current.add(row.key);
    setAccepting([...acceptingRef.current]);
    patch(row.key, { error: null });
    try {
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
    } finally {
      acceptingRef.current.delete(row.key);
      setAccepting([...acceptingRef.current]);
    }
  };

  const discard = (key: string) => setRows((rs) => rs?.filter((r) => r.key !== key) ?? rs);

  const linkBtn = "text-[11px] text-muted underline-offset-2 hover:underline";
  const pending = rows?.filter((r) => !r.created).length ?? 0;

  const statusLabel = (row: Row) =>
    row.created
      ? t("evidenceAI.created")
      : row.valid
        ? t("evidenceAI.unsaved")
        : t("evidenceAI.invalid");
  const statusTone = (row: Row): "default" | "verified" | "warning" =>
    row.created ? "verified" : row.valid ? "default" : "warning";

  return (
    <div data-testid="evidence-panel" className="rounded border border-hairline bg-panel p-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wide text-muted">
          {t("evidenceAI.title")}
        </span>
        <div className="flex flex-wrap items-center gap-2">
          <Button type="button" disabled={busy} onClick={() => void proposeStub()}>
            {busy ? t("evidenceAI.proposing") : t("evidenceAI.propose")}
          </Button>
          {localEnabled && onProposeLocal && (
            <Button type="button" disabled={busy} onClick={() => setConsentOpen(true)}>
              {t("evidenceAI.proposeLocal")}
            </Button>
          )}
        </div>
      </div>

      <p className="mt-1 text-[10px] text-muted">{t("evidenceAI.disclaimer")}</p>

      {consentOpen && (
        <div
          role="dialog"
          aria-label={t("evidenceAI.consentTitle")}
          className="mt-2 rounded border border-l-2 border-accent-source bg-panel-2 p-2"
        >
          <div className="text-sm font-medium text-ink">{t("evidenceAI.consentTitle")}</div>
          <p className="mt-1 text-[11px] text-muted">{t("evidenceAI.consentBody")}</p>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <Button type="button" onClick={() => void proposeLocal()}>
              {t("evidenceAI.consentConfirm")}
            </Button>
            <button type="button" className={linkBtn} onClick={() => setConsentOpen(false)}>
              {t("evidenceAI.consentCancel")}
            </button>
          </div>
        </div>
      )}

      {notice && (
        <p data-testid="evidence-truncation" className="mt-2 text-[11px] text-accent-warning">
          {notice}
        </p>
      )}

      {error && (
        <p role="alert" className="mt-1 text-[11px] text-accent-warning">
          {error}
        </p>
      )}

      {rows !== null && rows.length === 0 && (
        <p className="mt-2 text-[11px] text-muted">{t("evidenceAI.empty")}</p>
      )}

      {pending > 0 && <p className="mt-2 text-[11px] text-muted">{t("evidenceAI.unsavedHint")}</p>}

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
            <Badge tone={statusTone(row)}>
              <span data-testid="evidence-candidate-status">{statusLabel(row)}</span>
            </Badge>
          </div>

          {editing === row.key && !row.created && row.valid ? (
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
              {row.valid ? (
                <>
                  <button
                    type="button"
                    className={linkBtn}
                    disabled={isAccepting(row.key)}
                    onClick={() => void approve(row)}
                  >
                    {isAccepting(row.key) ? t("evidenceAI.approving") : t("evidenceAI.approve")}
                  </button>
                  <button
                    type="button"
                    className={linkBtn}
                    onClick={() => setEditing(editing === row.key ? null : row.key)}
                  >
                    {editing === row.key ? t("evidenceAI.editDone") : t("evidenceAI.edit")}
                  </button>
                </>
              ) : (
                <span className="text-[11px] text-muted">{t("evidenceAI.invalidHint")}</span>
              )}
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
