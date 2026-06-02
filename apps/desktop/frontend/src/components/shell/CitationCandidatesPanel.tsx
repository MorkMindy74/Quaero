import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge, Button } from "../ui";
import type { Excerpt } from "../../domain/types";
import type { CitationCandidate } from "../../lib/ipc";

/** A candidate row. `valid` = its excerptId references an existing real Estratto.
 *  Candidates are NEVER persisted until approved; `created` flips after that. */
type Row = CitationCandidate & {
  key: string;
  created: boolean;
  error: string | null;
  edited: boolean;
};

/** AI Evidence Assistant V1C (#60). From the real Estratti of a Pratica without a
 *  Citazione yet, proposes candidate Citazioni (offline Stub — no LLM, no egress).
 *  Each candidate is structurally tied to a real Estratto (ADR-0007). The lawyer
 *  approves/edits/discards; a real Citazione is created ONLY via the canonical
 *  `add_citation` path. Invalid candidates (Estratto gone) are not approvable.
 *  Nothing is auto-saved. */
export function CitationCandidatesPanel({
  excerpts,
  onPropose,
  onAccept,
}: {
  excerpts: Excerpt[];
  onPropose: () => Promise<CitationCandidate[]>;
  onAccept: (excerptId: string, claim: string) => Promise<boolean>;
}) {
  const { t } = useTranslation();
  const [rows, setRows] = useState<Row[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  // Per-row "approval in flight" guard (synchronous ref blocks same-tick
  // double-click; state mirror disables the button).
  const acceptingRef = useRef<Set<string>>(new Set());
  const [accepting, setAccepting] = useState<readonly string[]>([]);
  const isAccepting = (key: string) => accepting.includes(key);

  const patch = (key: string, p: Partial<Row>) =>
    setRows((rs) => rs?.map((r) => (r.key === key ? { ...r, ...p } : r)) ?? rs);

  const propose = async () => {
    setError(null);
    setEditing(null);
    setBusy(true);
    try {
      const cands = await onPropose();
      setRows(
        cands.map((c, i) => ({ ...c, key: `cit-${i}`, created: false, error: null, edited: false })),
      );
    } catch {
      setError(t("citationAI.proposeError"));
    } finally {
      setBusy(false);
    }
  };

  const approve = async (row: Row) => {
    // Never approve an invalid candidate (Estratto gone); guard re-entrancy.
    if (row.created || !row.valid || acceptingRef.current.has(row.key)) return;
    // Defence: re-verify the linked Estratto still exists in the CURRENT excerpts
    // (it may have been deleted, or the workspace changed) — never approve a stale
    // candidate. The `excerpts` prop is always the open Pratica's current set.
    if (!excerpts.some((e) => e.id === row.excerptId)) {
      patch(row.key, { valid: false, error: t("citationAI.invalidHint") });
      return;
    }
    acceptingRef.current.add(row.key);
    setAccepting([...acceptingRef.current]);
    patch(row.key, { error: null });
    try {
      const ok = await onAccept(row.excerptId, row.claim);
      if (ok) {
        patch(row.key, { created: true });
        setEditing(null);
      } else {
        patch(row.key, { error: t("citationAI.acceptError") });
      }
    } finally {
      acceptingRef.current.delete(row.key);
      setAccepting([...acceptingRef.current]);
    }
  };

  const discard = (key: string) => setRows((rs) => rs?.filter((r) => r.key !== key) ?? rs);

  const linkBtn = "text-[11px] text-muted underline-offset-2 hover:underline";
  const pending = rows?.filter((r) => !r.created).length ?? 0;
  const excerptOf = (id: string) => excerpts.find((e) => e.id === id);

  const statusLabel = (row: Row) =>
    row.created
      ? t("citationAI.created")
      : !row.valid
        ? t("citationAI.invalid")
        : row.edited
          ? t("citationAI.draft")
          : t("citationAI.unsaved");
  const statusTone = (row: Row): "default" | "verified" | "warning" =>
    row.created ? "verified" : row.valid ? "default" : "warning";

  return (
    <div data-testid="citation-panel" className="rounded border border-hairline bg-panel p-2">
      <div className="flex items-center justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wide text-muted">
          {t("citationAI.title")}
        </span>
        <Button type="button" disabled={busy} onClick={() => void propose()}>
          {busy ? t("citationAI.proposing") : t("citationAI.propose")}
        </Button>
      </div>

      <p className="mt-1 text-[10px] text-muted">{t("citationAI.disclaimer")}</p>

      {error && (
        <p role="alert" className="mt-1 text-[11px] text-accent-warning">
          {error}
        </p>
      )}

      {rows !== null && rows.length === 0 && (
        <p className="mt-2 text-[11px] text-muted">{t("citationAI.empty")}</p>
      )}

      {pending > 0 && <p className="mt-2 text-[11px] text-muted">{t("citationAI.unsavedHint")}</p>}

      {rows?.map((row) => {
        const ex = excerptOf(row.excerptId);
        return (
          <div
            key={row.key}
            data-testid="citation-candidate"
            className="mt-2 rounded border border-hairline bg-panel-2 p-2"
          >
            <div className="flex items-center justify-between gap-2">
              <span className="font-mono text-[10px] uppercase tracking-wide text-muted">
                {t("citationAI.linkedExcerpt")}
                {ex ? ` · ${ex.anchor.kind} ${ex.anchor.value}` : ""}
              </span>
              <Badge tone={statusTone(row)}>
                <span data-testid="citation-candidate-status">{statusLabel(row)}</span>
              </Badge>
            </div>

            {/* The linked Estratto (read-only context: the claim must cite it). */}
            {ex ? (
              <blockquote className="mt-1 border-l-2 border-hairline pl-2 text-[11px] text-muted">
                “{ex.quote}”
              </blockquote>
            ) : null}

            {editing === row.key && !row.created && row.valid ? (
              <textarea
                aria-label={t("citationAI.claim")}
                className="mt-1 w-full rounded border border-hairline bg-panel p-1 text-sm"
                rows={2}
                value={row.claim}
                onChange={(e) => patch(row.key, { claim: e.target.value, edited: true })}
              />
            ) : (
              <div className="mt-1 text-sm">↳ {row.claim}</div>
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
                      {isAccepting(row.key) ? t("citationAI.approving") : t("citationAI.approve")}
                    </button>
                    <button
                      type="button"
                      className={linkBtn}
                      onClick={() => setEditing(editing === row.key ? null : row.key)}
                    >
                      {editing === row.key ? t("citationAI.editDone") : t("citationAI.edit")}
                    </button>
                  </>
                ) : (
                  <span className="text-[11px] text-muted">{t("citationAI.invalidHint")}</span>
                )}
                <button type="button" className={linkBtn} onClick={() => discard(row.key)}>
                  {t("citationAI.discard")}
                </button>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
