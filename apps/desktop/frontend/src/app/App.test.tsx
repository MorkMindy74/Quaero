import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, expect, test } from "vitest";
import App from "./App";
import i18n from "../i18n";

afterEach(async () => {
  clearMocks();
  await i18n.changeLanguage("it");
});

test("renders the Italian workspace shell by default", () => {
  render(<App />);
  expect(
    screen.getByText("Cosa vuoi cercare, analizzare o costruire oggi?"),
  ).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Test connessione" })).toBeInTheDocument();
});

test("clicking the ping button shows the pong reply from the backend (mocked IPC)", async () => {
  mockIPC((cmd, args) => {
    if (cmd === "ping") {
      const message = (args as { request: { message: string } }).request.message;
      return { reply: `pong: ${message}` };
    }
  });

  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Test connessione" }));

  await waitFor(() =>
    expect(screen.getByTestId("ping-reply")).toHaveTextContent("pong: Quaero"),
  );
});

test("can switch the interface language from Italian to English", async () => {
  render(<App />);
  expect(
    screen.getByText("Cosa vuoi cercare, analizzare o costruire oggi?"),
  ).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "EN" }));

  await waitFor(() =>
    expect(
      screen.getByText("What would you like to search, analyse or build today?"),
    ).toBeInTheDocument(),
  );
});
