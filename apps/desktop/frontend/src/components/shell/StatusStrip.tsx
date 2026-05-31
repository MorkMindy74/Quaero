import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ping } from "../../lib/ipc";

// Spec §3 comp 14 (v0.3): the intentional home of operational status. Sober,
// mono, readable. Also surfaces the #2 ping round-trip as core connectivity.
export function StatusStrip() {
  const { t } = useTranslation();
  const [core, setCore] = useState<"checking" | "ok" | "err">("checking");

  useEffect(() => {
    let active = true;
    Promise.resolve()
      .then(() => ping({ message: "status" }))
      .then(() => {
        if (active) setCore("ok");
      })
      .catch(() => {
        if (active) setCore("err");
      });
    return () => {
      active = false;
    };
  }, []);

  const sep = <span className="text-hairline">·</span>;

  return (
    <footer
      data-testid="region-status"
      role="contentinfo"
      className="flex items-center gap-3 border-t border-hairline bg-panel px-4 py-1.5 font-mono text-[11px] text-muted"
    >
      <span className="inline-flex items-center gap-1.5 text-accent-verified">
        <span className="h-2 w-2 rounded-full bg-accent-verified" /> {t("status.localPrivate")}
      </span>
      {sep}
      <span data-testid="status-connectivity">
        {core === "ok" ? t("status.coreActive") : core === "err" ? t("status.coreErr") : "…"}
      </span>
      {sep}
      <span>{t("status.sourcesVerified")}</span>
      {sep}
      <span className="text-accent-warning">{t("status.timeCheck")}</span>
      <span className="ml-auto">{t("status.indexing")}</span>
    </footer>
  );
}
