import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui";
import type { SourceRef } from "../../domain/types";

export interface ExcerptDialogValues {
  sourceId: string;
  anchorKind: string;
  anchorValue: string;
  quote: string;
  note?: string;
}

interface NewExcerptDialogProps {
  /** Documento sources selectable in CREATE mode (ignored in edit mode). */
  sources: SourceRef[];
  /** EDIT mode: prefilled values + the (locked, read-only) source title. When
   *  present the dialog edits an existing Estratto and the Fonte is fixed. */
  initial?: ExcerptDialogValues & { sourceTitle: string };
  onClose: () => void;
  /** Returns true on success (dialog closes), false on error (stays open). */
  onSubmit: (args: ExcerptDialogValues) => Promise<boolean>;
  error: string | null;
}

// #8B create + edit/delete edit: same dialog. In edit mode the Fonte is locked
// (the quote correction must not move/re-pin the Fonte) and only quote/anchor/
// note are editable.
export function NewExcerptDialog({ sources, initial, onClose, onSubmit, error }: NewExcerptDialogProps) {
  const { t } = useTranslation();
  const editing = initial !== undefined;
  const [sourceId, setSourceId] = useState(initial?.sourceId ?? sources[0]?.id ?? "");
  const [anchorKind, setAnchorKind] = useState(initial?.anchorKind ?? "pagina");
  const [anchorValue, setAnchorValue] = useState(initial?.anchorValue ?? "");
  const [quote, setQuote] = useState(initial?.quote ?? "");
  const [note, setNote] = useState(initial?.note ?? "");
  const [busy, setBusy] = useState(false);

  const canSubmit =
    sourceId !== "" &&
    quote.trim() !== "" &&
    anchorKind.trim() !== "" &&
    anchorValue.trim() !== "" &&
    !busy;

  // #57: close only on explicit action. Esc closes; clicking outside does NOT.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const submit = async () => {
    if (!canSubmit) return;
    setBusy(true);
    const ok = await onSubmit({
      sourceId,
      anchorKind: anchorKind.trim(),
      anchorValue: anchorValue.trim(),
      quote: quote.trim(),
      note: note.trim() === "" ? undefined : note.trim(),
    });
    setBusy(false);
    if (ok) onClose();
  };

  const field = "w-full rounded border border-hairline bg-panel-2 px-2 py-1 text-sm outline-none";
  const title = editing ? t("excerpts.editTitle") : t("excerpts.newTitle");

  return (
    <div
      role="dialog"
      aria-label={title}
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/20 pt-24"
    >
      <form
        className="w-[440px] rounded-lg border border-hairline bg-panel p-4 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        <h2 className="mb-3 font-serif text-base font-semibold">{title}</h2>

        {!editing && sources.length === 0 ? (
          <p role="alert" className="mb-3 text-sm text-muted">
            {t("excerpts.noDocuments")}
          </p>
        ) : (
          <>
            <label className="mb-2 block text-sm">
              <span className="mb-1 block text-muted">{t("excerpts.source")}</span>
              {editing ? (
                <div className={`${field} text-muted`}>{initial?.sourceTitle}</div>
              ) : (
                <select
                  value={sourceId}
                  onChange={(e) => setSourceId(e.target.value)}
                  className={field}
                >
                  {sources.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.title}
                    </option>
                  ))}
                </select>
              )}
            </label>

            <label className="mb-2 block text-sm">
              <span className="mb-1 block text-muted">{t("excerpts.quote")}</span>
              <textarea
                autoFocus
                rows={3}
                value={quote}
                onChange={(e) => setQuote(e.target.value)}
                className={field}
              />
            </label>

            <div className="mb-2 grid grid-cols-2 gap-2">
              <label className="block text-sm">
                <span className="mb-1 block text-muted">{t("excerpts.anchorKind")}</span>
                <input
                  value={anchorKind}
                  onChange={(e) => setAnchorKind(e.target.value)}
                  className={field}
                />
              </label>
              <label className="block text-sm">
                <span className="mb-1 block text-muted">{t("excerpts.anchorValue")}</span>
                <input
                  value={anchorValue}
                  onChange={(e) => setAnchorValue(e.target.value)}
                  className={field}
                />
              </label>
            </div>

            <label className="mb-3 block text-sm">
              <span className="mb-1 block text-muted">{t("excerpts.note")}</span>
              <input value={note} onChange={(e) => setNote(e.target.value)} className={field} />
            </label>
          </>
        )}

        {error && (
          <p role="alert" className="mb-2 text-xs text-accent-warning">
            {error}
          </p>
        )}

        {/* Explain why "Salva" is disabled, so the button is never silently dead. */}
        {(editing || sources.length > 0) && !busy && !canSubmit && (
          <p className="mb-2 text-xs text-muted">{t("excerpts.required")}</p>
        )}

        <div className="flex justify-end gap-2">
          <Button type="button" onClick={onClose}>
            {t("matters.cancel")}
          </Button>
          <Button type="submit" variant="primary" disabled={!canSubmit}>
            {t("excerpts.save")}
          </Button>
        </div>
      </form>
    </div>
  );
}
