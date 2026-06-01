import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui";
import type { Excerpt } from "../../domain/types";

interface NewCitationDialogProps {
  /** The Estratto being cited (preselected — the user only writes the claim). */
  excerpt: Excerpt;
  onClose: () => void;
  /** Returns true on success (dialog closes), false on error (stays open). */
  onCreate: (claim: string) => Promise<boolean>;
  /** Backend error to surface inline. */
  error: string | null;
}

// Citations-from-UI: the user is looking at an Excerpt and cites it. The
// excerptId is implicit (preselected); only the claim (Affermazione) is typed.
// A Citation references an excerpt, never a Fonte (ADR-0007).
export function NewCitationDialog({ excerpt, onClose, onCreate, error }: NewCitationDialogProps) {
  const { t } = useTranslation();
  const [claim, setClaim] = useState("");
  const [busy, setBusy] = useState(false);

  const canSubmit = claim.trim() !== "" && !busy;

  const submit = async () => {
    if (!canSubmit) return;
    setBusy(true);
    const ok = await onCreate(claim.trim());
    setBusy(false);
    if (ok) onClose();
  };

  const field = "w-full rounded border border-hairline bg-panel-2 px-2 py-1 text-sm outline-none";

  return (
    <div
      role="dialog"
      aria-label={t("citations.newTitle")}
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
        <h2 className="mb-3 font-serif text-base font-semibold">{t("citations.newTitle")}</h2>

        {/* The cited Excerpt, read-only, so the link is unambiguous. */}
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
