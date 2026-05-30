import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ping } from "../../lib/ipc";
import type { MockMatter } from "../../mock/data";

interface StatusStripProps {
  matter: MockMatter | null;
}

// Spec §3 comp 14: quiet instrumentation; carries the privacy signal.
// Also the home of the #2 ping round-trip, surfaced as a connectivity check.
export function StatusStrip({ matter }: StatusStripProps) {
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

  return (
    <footer
      data-testid="region-status"
      role="contentinfo"
      className="flex items-center gap-4 border-t border-hairline bg-panel px-4 py-1 font-mono text-[11px] text-muted"
    >
      <span className="inline-flex items-center gap-1 text-accent-verified">
        <span className="h-2 w-2 rounded-full bg-accent-verified" /> {t("status.localPrivate")}
      </span>
      <span>{t("status.index")} 100%</span>
      <span>{matter ? t("status.matterOpen") : t("status.noMatter")}</span>
      <span>{t("status.citations")} 6/6</span>
      <span data-testid="status-connectivity" className="ml-auto">
        {core === "ok" ? t("status.coreOk") : core === "err" ? t("status.coreErr") : "…"}
      </span>
    </footer>
  );
}
