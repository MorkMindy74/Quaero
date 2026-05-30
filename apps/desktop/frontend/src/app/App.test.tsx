import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, expect, test } from "vitest";
import App from "./App";
import i18n from "../i18n";

beforeEach(() => {
  mockIPC((cmd, args) => {
    if (cmd === "ping") {
      const message = (args as { request: { message: string } }).request.message;
      return { reply: `pong: ${message}` };
    }
  });
});

afterEach(async () => {
  clearMocks();
  await i18n.changeLanguage("it");
});

test("renders the five cockpit regions", () => {
  render(<App />);
  expect(screen.getByTestId("region-topbar")).toBeInTheDocument();
  expect(screen.getByTestId("region-sidebar")).toBeInTheDocument();
  expect(screen.getByTestId("region-workspace")).toBeInTheDocument();
  expect(screen.getByTestId("region-context")).toBeInTheDocument();
  expect(screen.getByTestId("region-status")).toBeInTheDocument();
});

test("right context panel is permanent with Sources active by default", () => {
  render(<App />);
  const context = screen.getByTestId("region-context");
  expect(within(context).getByText("Contratto Rossi-Bianchi.pdf")).toBeInTheDocument();
});

test("switching the context tab to Excerpts changes the card list", () => {
  render(<App />);
  const context = screen.getByTestId("region-context");
  fireEvent.click(within(context).getByRole("tab", { name: "Estratti" }));
  expect(within(context).getByText(/Il Fornitore potrà recedere/)).toBeInTheDocument();
});

test("the mode switcher swaps the workspace surface", () => {
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Ragionamento" }));
  expect(screen.getByTestId("surface-reasoning")).toBeInTheDocument();
});

test("the command palette stub opens from the ⌘K trigger", () => {
  render(<App />);
  fireEvent.click(screen.getByText("⌘K"));
  expect(screen.getByRole("dialog")).toBeInTheDocument();
});

test("the status strip shows the local/privacy signal", () => {
  render(<App />);
  expect(within(screen.getByTestId("region-status")).getByText("local & private")).toBeInTheDocument();
});

test("language toggle switches Italian to English (regression from #2)", async () => {
  render(<App />);
  expect(screen.getByText("Pratiche")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "EN" }));
  await waitFor(() => expect(screen.getByText("Matters")).toBeInTheDocument());
});

test("ping round-trip still works, surfaced as connectivity (regression from #2)", async () => {
  render(<App />);
  await waitFor(() =>
    expect(screen.getByTestId("status-connectivity")).toHaveTextContent("core: ok"),
  );
});
