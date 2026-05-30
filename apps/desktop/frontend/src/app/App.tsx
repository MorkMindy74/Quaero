import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ping } from "../lib/ipc";

// Minimal workspace shell (slice #2). No domain logic: just proves the
// frontend ↔ Rust IPC round-trip and i18n wiring.
export default function App() {
  const { t, i18n } = useTranslation();
  const [reply, setReply] = useState<string | null>(null);

  async function handlePing() {
    const response = await ping({ message: t("app.name") });
    setReply(response.reply);
  }

  function switchLanguage(lng: "it" | "en") {
    void i18n.changeLanguage(lng);
  }

  return (
    <div className="min-h-screen bg-stone-100 text-stone-900">
      <header className="flex items-center gap-4 border-b border-stone-300 px-4 py-3">
        <strong className="text-base font-semibold">{t("app.name")}</strong>
        <nav className="flex gap-3 text-sm text-stone-600">
          <span>{t("nav.workspace")}</span>
          <span>{t("nav.matters")}</span>
          <span>{t("nav.knowledge")}</span>
        </nav>
        <div className="ml-auto flex gap-1 text-xs">
          <button type="button" onClick={() => switchLanguage("it")} className="rounded px-2 py-1 hover:bg-stone-200">
            IT
          </button>
          <button type="button" onClick={() => switchLanguage("en")} className="rounded px-2 py-1 hover:bg-stone-200">
            EN
          </button>
        </div>
      </header>

      <main className="mx-auto max-w-2xl px-4 py-16 text-center">
        <p className="mb-8 text-lg">{t("workspace.welcome")}</p>
        <button
          type="button"
          onClick={handlePing}
          className="rounded-md bg-stone-800 px-4 py-2 text-stone-50"
        >
          {t("action.ping")}
        </button>
        {reply !== null && (
          <p data-testid="ping-reply" className="mt-6 text-stone-700">
            {reply}
          </p>
        )}
      </main>
    </div>
  );
}
