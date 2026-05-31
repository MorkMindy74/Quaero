import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui";

interface NewMatterDialogProps {
  onClose: () => void;
  /** Returns the created summary on success, or null on error. */
  onCreate: (
    clientName: string,
    matterTitle: string,
    subject: string,
  ) => Promise<{ id: string } | null>;
  /** Backend error to surface inline (AlreadyExists / UnsafeId / …). */
  error: string | null;
}

// #5C: minimal create form. The user enters display fields only — ids are
// derived safely upstream (slug). The backend stays the source of truth: on
// error we keep the dialog open and show the message inline.
export function NewMatterDialog({ onClose, onCreate, error }: NewMatterDialogProps) {
  const { t } = useTranslation();
  const [clientName, setClientName] = useState("");
  const [title, setTitle] = useState("");
  const [subject, setSubject] = useState("");
  const [busy, setBusy] = useState(false);

  const canSubmit = clientName.trim() !== "" && title.trim() !== "" && !busy;

  const submit = async () => {
    if (!canSubmit) return;
    setBusy(true);
    const res = await onCreate(clientName.trim(), title.trim(), subject.trim());
    setBusy(false);
    if (res) onClose();
  };

  const field =
    "w-full rounded border border-hairline bg-panel-2 px-2 py-1 text-sm outline-none";

  return (
    <div
      role="dialog"
      aria-label={t("matters.newTitle")}
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/20 pt-24"
      onClick={onClose}
    >
      <form
        className="w-[420px] rounded-lg border border-hairline bg-panel p-4 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        <h2 className="mb-3 font-serif text-base font-semibold">{t("matters.newTitle")}</h2>

        <label className="mb-2 block text-sm">
          <span className="mb-1 block text-muted">{t("matters.client")}</span>
          <input
            autoFocus
            value={clientName}
            onChange={(e) => setClientName(e.target.value)}
            className={field}
          />
        </label>

        <label className="mb-2 block text-sm">
          <span className="mb-1 block text-muted">{t("matters.matterTitle")}</span>
          <input value={title} onChange={(e) => setTitle(e.target.value)} className={field} />
        </label>

        <label className="mb-3 block text-sm">
          <span className="mb-1 block text-muted">{t("matters.subject")}</span>
          <input value={subject} onChange={(e) => setSubject(e.target.value)} className={field} />
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
            {t("matters.create")}
          </Button>
        </div>
      </form>
    </div>
  );
}
