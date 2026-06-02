import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ping, chatProviderKind } from "../../lib/ipc";

// Spec §3 comp 14 (v0.3): the intentional home of operational status. Sober,
// mono, readable. Also surfaces the #2 ping round-trip as core connectivity.
export function StatusStrip() {
  const { t } = useTranslation();
  const [core, setCore] = useState<"checking" | "ok" | "err">("checking");
  // #37: privacy posture is DERIVED from the active chat provider. Default
  // "stub" → offline, nothing leaves the device; "ollamaLocal" → a local model
  // is active, still on-device. Both statements are literally true.
  const [provider, setProvider] = useState<"stub" | "ollamaLocal">("stub");

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
    chatProviderKind()
      .then((kind) => {
        if (active && kind === "ollamaLocal") setProvider("ollamaLocal");
      })
      .catch(() => {
        /* default stays "stub" */
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
      {/* #62: human-first headline. The technical posture below is secondary. */}
      <span className="inline-flex items-center gap-1.5 text-accent-verified">
        <span className="h-2 w-2 rounded-full bg-accent-verified" /> {t("status.privacyHeadline")}
      </span>
      {sep}
      {/* #10/#37 Privacy Guard posture, DERIVED from the active provider —
          secondary detail. stub → offline; ollamaLocal → local model active. */}
      <span data-testid="status-privacy" className="opacity-70">
        {provider === "ollamaLocal" ? t("status.privacyLocalModel") : t("status.privacyStub")}
      </span>
      {sep}
      <span data-testid="status-connectivity" className="opacity-70">
        {core === "ok" ? t("status.coreActive") : core === "err" ? t("status.coreErr") : "…"}
      </span>
    </footer>
  );
}
