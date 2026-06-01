import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui";
import type { Excerpt } from "../../domain/types";

interface NewCitationDialogProps {
  /** The Estratto being cited (preselected, read-only; fixed also in edit). */
  excerpt: Excerpt;
  /** EDIT mode: prefilled claim. When present the dialog edits an existing
   *  Citazione (the linked Estratto stays fixed). */
  initialClaim?: string;
  onClose: () => void;
  /** Returns true on success (dialog closes), false on error (stays open). */
  onSubmit: (claim: string) => Promise<boolean>;
  error: string | null;
}

// Citations-from-UI create + edit. The excerptId is implicit (the Estratto is
// fixed, also when editing); only the claim (Affermazione) is editable.
export function NewCitationDialog({ excerpt, initialClaim, onClose, onSubmit, error }: NewCitationDialogProps) {
  const { t } = useTranslation();
  const editing = initialClaim !== undefined;
  const [claim, setClaim] = useState(initialClaim ?? "");
  const [busy, setBusy] = useState(false);

  const canSubmit = claim.trim() !== "" && !busy;

  const submit = async () => {
    if (!canSubmit) return;
    setBusy(true);
    const ok = await onSubmit(claim.trim());
    setBusy(false);
    if (ok) onClose();
  };

  const field = "w-full rounded border border-hairline bg-panel-2 px-2 py-1 text-sm outline-none";
  const title = editing ? t("citations.editTitle") : t("citations.newTitle");

  return (
    <div
      role="dialog"
      aria-label={title}
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
        <h2 className="mb-3 font-serif text-base font-semibold">{title}</h2>

        <div className="mb-3">
          <div className="mb-1 text-muted">{t("citations.citing")}</div>
          <blockquote className="border-l-2 border-hairline pl-2 text-sm text-muted">
            “{excerpt.quote}”
          </blockquote>
        </div>

        <label className="mb-3 block text-sm">
          <span className="mb-1 block text-muted">{t("citations.claim")}</span>
          <textarea
            autoFocus
            rows={3}
            value={claim}
            onChange={(e) => setClaim(e.target.value)}
            className={field}
          />
        </label>

        {error && (
          <p role="alert" className="mb-2 text-xs text-accent-warning">
            {error}
          </p>
        )}

        <div className="flex justify-end gap-2">
          <Button type="button" onClick={onClose}>
            {t("matters.cancel")}
          </Button>
          <Button type="submit" variant="primary" disabled={!canSubmit}>
            {t("citations.save")}
          </Button>
        </div>
      </form>
    </div>
  );
}
