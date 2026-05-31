import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, expect, test } from "vitest";
import { ChatPanel } from "./ChatPanel";
import "../../i18n";

afterEach(() => clearMocks());

test("shows the ungrounded disclaimer up front", () => {
  render(<ChatPanel />);
  expect(
    screen.getByText(/non sono verificate.*non costituiscono un parere legale/i),
  ).toBeInTheDocument();
});

test("sending a message calls chat_send and shows user + assistant turns", async () => {
  let sentPrompt: string | null = null;
  mockIPC((cmd, args) => {
    if (cmd === "chat_send") {
      sentPrompt = (args as { request: { prompt: string } }).request.prompt;
      return { reply: `[bozza esplorativa] eco: ${sentPrompt}`, grounded: false };
    }
  });

  render(<ChatPanel />);
  fireEvent.change(screen.getByLabelText("Scrivi un messaggio…"), {
    target: { value: "la clausola 7.2 è valida?" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Invia" }));

  // the user's message appears
  await waitFor(() =>
    expect(screen.getByText("la clausola 7.2 è valida?")).toBeInTheDocument(),
  );
  expect(sentPrompt).toBe("la clausola 7.2 è valida?");
  // the assistant reply appears, marked as unverified / no citations
  await waitFor(() => expect(screen.getByText(/eco: la clausola 7\.2/)).toBeInTheDocument());
  expect(screen.getAllByText("Non verificata · senza citazioni").length).toBeGreaterThanOrEqual(1);
});

test("a failing chat_send surfaces an inline error (no crash)", async () => {
  mockIPC((cmd) => {
    if (cmd === "chat_send") throw new Error("provider down");
  });

  render(<ChatPanel />);
  fireEvent.change(screen.getByLabelText("Scrivi un messaggio…"), {
    target: { value: "ciao" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Invia" }));

  await waitFor(() =>
    expect(screen.getByText("Risposta non riuscita. Riprova.")).toBeInTheDocument(),
  );
});
