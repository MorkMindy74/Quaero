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
    // #5C: the shell loads the saved-matters list on mount. Keep it empty here
    // so the context panel falls back to the #3 mock view (regression intact).
    if (cmd === "search_workspaces") return [];
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
  // client · matter header from the typed domain workspace (slice #5A)
  expect(within(context).getByText(/Alfa S\.r\.l\./)).toBeInTheDocument();
  // a dynamic Fascicolo (grouped by source type) is shown
  expect(within(context).getByText(/^Documenti/)).toBeInTheDocument();
});

test("sources are grouped by typed dossiers; a source can appear in many (domain model)", () => {
  render(<App />);
  const context = screen.getByTestId("region-context");
  // manual Fascicolo present alongside the dynamic ones
  expect(within(context).getByText(/Produzione avversaria/)).toBeInTheDocument();
  // s1 (Contratto…) appears both in its dynamic "Documenti" and in the manual dossier
  expect(within(context).getAllByText(/Contratto Rossi-Bianchi\.pdf/).length).toBeGreaterThanOrEqual(2);
});

test("the Excerpts tab shows an empty state when no workspace is open (#8, no mock fallback)", () => {
  render(<App />);
  const context = screen.getByTestId("region-context");
  fireEvent.click(within(context).getByRole("tab", { name: "Estratti" }));
  // #8: real Estratti come only from an opened workspace; otherwise a contextual
  // empty state guides the user (pilot UX).
  expect(
    within(context).getByText("Importa prima un documento (tab Fonti)."),
  ).toBeInTheDocument();
});

test("the genealogy tab shows the normative genealogy mock", () => {
  render(<App />);
  const context = screen.getByTestId("region-context");
  fireEvent.click(within(context).getByRole("tab", { name: "Genealogia" }));
  expect(within(context).getByText("Genealogia normativa")).toBeInTheDocument();
  expect(within(context).getByText("Art. 1375 c.c.")).toBeInTheDocument();
});

test("the mode switcher swaps the workspace surface", () => {
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Ragionamento" }));
  expect(screen.getByTestId("surface-reasoning")).toBeInTheDocument();
});

test("the drafting mode shows the unvalidated draft surface", () => {
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Redazione" }));
  expect(screen.getByTestId("surface-drafting")).toBeInTheDocument();
  expect(screen.getByText("Comparsa di costituzione e risposta")).toBeInTheDocument();
  expect(screen.getByText("BOZZA NON VALIDATA")).toBeInTheDocument();
});

test("the command palette stub opens from the ⌘K trigger", () => {
  render(<App />);
  fireEvent.click(screen.getByText("⌘K"));
  expect(screen.getByRole("dialog")).toBeInTheDocument();
});

test("the status strip shows the local/privacy signal", () => {
  render(<App />);
  expect(within(screen.getByTestId("region-status")).getByText("Locale e privato")).toBeInTheDocument();
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
    expect(screen.getByTestId("status-connectivity")).toHaveTextContent("Core attivo"),
  );
});
