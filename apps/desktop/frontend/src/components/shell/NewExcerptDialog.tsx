import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui";
import type { SourceRef } from "../../domain/types";

interface NewExcerptDialogProps {
  /** Fonti the excerpt can be linked to (Documento sources of the open Pratica). */
  sources: SourceRef[];
  onClose: () => void;
  /** Returns true on success (dialog closes), false on error (stays open). */
  onCreate: (args: {
    sourceId: string;
    anchorKind: string;
    anchorValue: string;
    quote: string;
    note?: string;
  }) => Promise<boolean>;
  /** Backend error to surface inline. */
  error: string | null;
}

// #8B: manual Evidence capture. The user pastes/types the quote and a manual
// Ancora (kind + value, e.g. "pagina"/"3"); the excerpt id, the createdAt
// timestamp and any sha-pin are produced by the backend. No parsing/highlight.
export function NewExcerptDialog({ sources, onClose, onCreate, error }: NewExcerptDialogProps) {
  const { t } = useTranslation();
  const [sourceId, setSourceId] = useState(sources[0]?.id ?? "");
  const [anchorKind, setAnchorKind] = useState("pagina");
  const [anchorValue, setAnchorValue] = useState("");
  const [quote, setQuote] = useState("");
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);

  const canSubmit =
    sourceId !== "" &&
    quote.trim() !== "" &&
    anchorKind.trim() !== "" &&
    anchorValue.trim() !== "" &&
    !busy;

  const submit = async () => {
    if (!canSubmit) return;
    setBusy(true);
    const ok = await onCreate({
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

  return (
    <div
      role="dialog"
      aria-label={t("excerpts.newTitle")}
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/20 pt-24"
      onClick={onClose}
    >
      <form
        className="w-[440px] rounded-lg border border-hairline bg-panel p-4 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        <h2 className="mb-3 font-serif text-base font-semibold">{t("excerpts.newTitle")}</h2>

        {sources.length === 0 ? (
          <p role="alert" className="mb-3 text-sm text-muted">
            {t("excerpts.noDocuments")}
          </p>
        ) : (
          <>
            <label className="mb-2 block text-sm">
              <span className="mb-1 block text-muted">{t("excerpts.source")}</span>
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
        {sources.length > 0 && !busy && !canSubmit && (
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
