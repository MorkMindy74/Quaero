import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui";
import { chatSend } from "../../lib/ipc";

interface ChatMessage {
  role: "user" | "assistant";
  text: string;
}

// #7 Conversation surface: a controlled, in-memory chat backed by the stub
// provider (offline, deterministic). Every assistant reply is explicitly marked
// as exploratory / unverified / uncited — it is NOT a legal opinion. No
// persistence, no documents, no network.
export function ChatPanel() {
  const { t } = useTranslation();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const send = async () => {
    const prompt = input.trim();
    if (!prompt || loading) return;
    setError(null);
    setMessages((m) => [...m, { role: "user", text: prompt }]);
    setInput("");
    setLoading(true);
    try {
      const reply = await chatSend(prompt);
      setMessages((m) => [...m, { role: "assistant", text: reply.reply }]);
    } catch {
      setError(t("chat.error"));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div data-testid="surface-conversation" className="flex h-full flex-col">
      <div role="note" className="mb-3 rounded border border-hairline bg-panel-2 px-3 py-2 text-xs text-muted">
        {t("chat.disclaimer")}
      </div>

      <div className="min-h-0 flex-1 space-y-2 overflow-auto">
        {messages.length === 0 && <p className="text-sm text-muted">{t("chat.empty")}</p>}
        {messages.map((m, i) => (
          <div key={i} className={m.role === "user" ? "text-right" : ""}>
            <div
              className={`inline-block max-w-[80%] rounded px-3 py-2 text-left text-sm ${
                m.role === "user" ? "bg-ink text-background" : "border border-hairline bg-panel"
              }`}
            >
              {m.text}
            </div>
            {m.role === "assistant" && (
              <div className="mt-0.5 font-mono text-[10px] uppercase tracking-wide text-accent-warning">
                {t("chat.unverified")}
              </div>
            )}
          </div>
        ))}
        {loading && <p className="text-sm text-muted">{t("chat.loading")}</p>}
        {error && (
          <p role="alert" className="text-sm text-accent-warning">
            {error}
          </p>
        )}
      </div>

      <form
        className="mt-3 flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          void send();
        }}
      >
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("chat.placeholder")}
          aria-label={t("chat.placeholder")}
          className="flex-1 rounded border border-hairline bg-panel-2 px-2 py-1 text-sm outline-none"
        />
        <Button type="submit" variant="primary" disabled={loading}>
          {t("chat.send")}
        </Button>
      </form>
    </div>
  );
}
