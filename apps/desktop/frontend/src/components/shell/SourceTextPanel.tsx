import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge } from "../ui";
import type { SourceRef } from "../../domain/types";
import type { SourceText } from "../../lib/ipc";
import { classifyFormat, sha256Hex } from "../../lib/textLayer";
import { extractDocumentText } from "../../lib/extractText";

/** UI state of a source's text layer (#52): the three persisted store states
 *  plus the two renderer-only ones. `loading` is the transient initial read. */
type UiStatus = "loading" | "available" | "empty" | "absent" | "failed" | "unsupported";

const BADGE_TONE: Record<Exclude<UiStatus, "loading">, "default" | "verified" | "warning"> = {
  available: "verified",
  empty: "default",
  absent: "default",
  failed: "warning",
  unsupported: "default",
};

/** Max characters rendered in the read-only preview (DOM-size guard). */
const PREVIEW_CAP = 20_000;

/** Per-Documento text-layer control: shows the state, lets the lawyer extract
 *  the text locally (re-picking the same file, verified by sha256 against the
 *  imported blob), and previews it read-only. Extraction runs entirely in the
 *  renderer; the store only persists the derived text. No egress, no LLM. */
export function SourceTextPanel({
  matterId,
  source,
  onGet,
  onSet,
}: {
  matterId: string;
  source: SourceRef;
  onGet: (matterId: string, sourceId: string) => Promise<SourceText>;
  onSet: (
    matterId: string,
    sourceId: string,
    expectedSha256: string,
    text: string,
  ) => Promise<SourceText>;
}) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<UiStatus>("loading");
  const [text, setText] = useState("");
  const [previewOpen, setPreviewOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const fmt = source.file ? classifyFormat(source.file.originalName) : null;

  // Identity of the currently-displayed target. An async extraction captures
  // this at start and re-checks it before every post-await `setState`, so a slow
  // extraction that resolves AFTER the user selected another Fonte/Pratica can
  // never repaint the now-visible panel with stale text (cross-context leak).
  // The parent also remounts this component by key per matter:source, so this is
  // belt-and-suspenders that is also unit-testable on a same-instance re-render.
  const targetKey = `${matterId}|${source.id}|${source.file?.sha256 ?? ""}`;
  const targetRef = useRef(targetKey);
  targetRef.current = targetKey;

  // Load the persisted state when the selected source changes. Unsupported
  // formats never hit the store (status derived from the filename).
  useEffect(() => {
    let alive = true;
    setError(null);
    setPreviewOpen(false);
    // Clear any stale busy/error left by an extraction started on a previous
    // source (whose async completion is now guarded out by targetRef).
    setBusy(false);
    if (!source.file) {
      setStatus("absent");
      return;
    }
    if (fmt === null) {
      setStatus("unsupported");
      return;
    }
    setStatus("loading");
    onGet(matterId, source.id)
      .then((res) => {
        if (!alive) return;
        setStatus(res.status);
        setText(res.text ?? "");
      })
      .catch(() => {
        // The store fails closed on a corrupt/hostile/oversize sidecar; surface
        // that as "failed" (not "absent", which means "not extracted yet").
        if (alive) setStatus("failed");
      });
    return () => {
      alive = false;
    };
  }, [matterId, source.id, source.file, fmt, onGet]);

  const onPick = async (file: File) => {
    if (!source.file) return;
    // Capture the target (matter, source, expected digest, name) at extraction
    // START — never re-read live state at commit time, so switching Pratica/Fonte
    // mid-extraction cannot misroute the write.
    const mid = matterId;
    const sid = source.id;
    const expectedSha = source.file.sha256;
    const name = source.file.originalName;
    const myKey = targetKey;
    // Only mutate the visible panel if it still shows the target we started on.
    const fresh = () => targetRef.current === myKey;
    setError(null);
    setBusy(true);
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      // Coherence (renderer-side fast check): the re-picked file must be the
      // imported one (same digest). The backend re-verifies expectedSha too.
      const digest = await sha256Hex(bytes);
      if (digest !== expectedSha) {
        if (fresh()) setError(t("textLayer.shaMismatch"));
        return;
      }
      const outcome = await extractDocumentText(name, bytes);
      if (outcome.kind === "unsupported") {
        if (fresh()) setStatus("unsupported");
      } else if (outcome.kind === "failed") {
        if (fresh()) setStatus("failed");
      } else {
        // Always persist to the captured target (correct Fonte), but only repaint
        // THIS panel if it still shows that same target.
        const res = await onSet(mid, sid, expectedSha, outcome.text);
        if (fresh()) {
          setStatus(res.status);
          setText(res.text ?? "");
          setPreviewOpen(res.status === "available");
        }
      }
    } catch {
      if (fresh()) setStatus("failed");
    } finally {
      if (fresh()) setBusy(false);
    }
  };

  const canExtract = fmt !== null && !busy;
  const hint =
    status === "empty"
      ? t("textLayer.emptyHint")
      : status === "unsupported"
        ? t("textLayer.unsupportedHint")
        : null;

  return (
    <div data-testid="source-text-panel" className="rounded border border-hairline bg-panel p-2">
      <div className="flex items-center justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wide text-muted">
          {t("textLayer.title")}
        </span>
        {status !== "loading" && (
          <Badge tone={BADGE_TONE[status]}>
            <span data-testid="text-layer-status">{t(`textLayer.status.${status}`)}</span>
          </Badge>
        )}
      </div>

      {hint && <p className="mt-1 text-[11px] text-muted">{hint}</p>}

      <div className="mt-2 flex flex-wrap items-center gap-2">
        {canExtract && (
          <label className="inline-flex cursor-pointer items-center gap-1 rounded border border-hairline bg-panel-2 px-2 py-1 text-[11px] hover:bg-panel">
            <span>{status === "available" ? t("textLayer.reextract") : t("textLayer.extract")}</span>
            <input
              type="file"
              aria-label={t("textLayer.extract")}
              className="sr-only"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) void onPick(f);
                e.target.value = "";
              }}
            />
          </label>
        )}
        {status === "available" && text && (
          <button
            type="button"
            className="text-[11px] text-muted underline-offset-2 hover:underline"
            onClick={() => setPreviewOpen((v) => !v)}
          >
            {previewOpen ? t("textLayer.previewHide") : t("textLayer.previewShow")}
          </button>
        )}
        {busy && <span className="text-[11px] text-muted">{t("textLayer.extracting")}</span>}
      </div>

      <p className="mt-1 text-[10px] text-muted">{t("textLayer.hint")}</p>

      {error && (
        <p role="alert" className="mt-1 text-[11px] text-accent-warning">
          {error}
        </p>
      )}

      {previewOpen && status === "available" && (
        <pre
          data-testid="text-layer-preview"
          className="mt-2 max-h-64 overflow-auto whitespace-pre-wrap break-words rounded border border-hairline bg-panel-2 p-2 text-[11px] text-ink"
        >
          {text.slice(0, PREVIEW_CAP)}
          {text.length > PREVIEW_CAP ? `\n… (+${text.length - PREVIEW_CAP})` : ""}
        </pre>
      )}
    </div>
  );
}
