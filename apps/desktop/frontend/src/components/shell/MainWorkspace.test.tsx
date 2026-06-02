import { render, screen, fireEvent } from "@testing-library/react";
import { expect, test } from "vitest";
import { MainWorkspace } from "./MainWorkspace";
import { matters } from "../../mock/data";
import type { WorkspaceView } from "../../domain/types";
import "../../i18n";

function makeWorkspace(over: Partial<WorkspaceView> = {}): WorkspaceView {
  return {
    client: { id: "alfa", name: "Alfa S.r.l." },
    matter: { id: "m1", client: "alfa", title: "Rossi c. Bianchi", subject: "Inadempimento" },
    sources: [{ id: "s1", kind: "Documento", title: "Contratto.pdf", meta: "" }],
    dossiers: [],
    excerpts: [
      { id: "e1", sourceId: "s1", anchor: { kind: "pagina", value: "7" }, quote: "Il Fornitore può recedere…" },
    ],
    citations: [{ id: "c1", excerptId: "e1", claim: "Il recesso è ammesso." }],
    ...over,
  };
}

test("open Pratica: real header + guided card + the mode switcher (real workbench)", () => {
  render(<MainWorkspace matter={null} workspace={makeWorkspace()} />);
  expect(screen.getByRole("heading", { name: "Rossi c. Bianchi" })).toBeInTheDocument();
  expect(screen.getByText(/Alfa S\.r\.l\./)).toBeInTheDocument();
  // The shipped #62 guided card stays.
  expect(screen.getByTestId("workflow-guide")).toBeInTheDocument();
  // The operational modes are now available for the REAL Pratica.
  expect(screen.getByRole("group", { name: "Modalità" })).toBeInTheDocument();
});

test("Revisione mode shows the REAL chain (Estratto + Affermazione) of the open Pratica", () => {
  render(<MainWorkspace matter={null} workspace={makeWorkspace()} />);
  fireEvent.click(screen.getByRole("button", { name: "Revisione" }));
  const surface = screen.getByTestId("surface-review");
  expect(surface).toHaveTextContent("Il recesso è ammesso.");
  expect(surface).toHaveTextContent("Il Fornitore può recedere…");
  expect(surface).toHaveTextContent("Contratto.pdf");
});

test("Revisione mode is honestly empty when the Pratica has no Estratti", () => {
  render(<MainWorkspace matter={null} workspace={makeWorkspace({ excerpts: [], citations: [] })} />);
  fireEvent.click(screen.getByRole("button", { name: "Revisione" }));
  expect(screen.getByTestId("surface-review")).toHaveTextContent(/Nessun Estratto/i);
});

test("Genealogia mode is an honest empty-state — NO mock genealogy data", () => {
  render(<MainWorkspace matter={null} workspace={makeWorkspace()} />);
  fireEvent.click(screen.getByRole("button", { name: "Genealogia" }));
  // Honest copy about no generated Document yet.
  expect(screen.getByTestId("surface-genealogy")).toHaveTextContent(/Nessun Documento generato/i);
  // The mock graph labels must NOT appear for a real Pratica.
  expect(screen.queryByText("Bozza v1")).not.toBeInTheDocument();
});

test("Ragionamento mode is an honest empty-state — NO mock reasoning data", () => {
  render(<MainWorkspace matter={null} workspace={makeWorkspace()} />);
  fireEvent.click(screen.getByRole("button", { name: "Ragionamento" }));
  expect(screen.getByTestId("surface-reasoning")).toBeInTheDocument();
  // The mock reasoning claim must NOT appear for a real Pratica.
  expect(screen.queryByText(/La clausola 7\.2 consente il recesso/)).not.toBeInTheDocument();
});

test("Conversazione mode renders the real chat surface", () => {
  render(<MainWorkspace matter={null} workspace={makeWorkspace()} />);
  fireEvent.click(screen.getByRole("button", { name: "Conversazione" }));
  expect(screen.getByTestId("surface-conversation")).toBeInTheDocument();
});

test("fallback (no real Pratica) keeps the mock surfaces unchanged", () => {
  render(<MainWorkspace matter={matters[0]} />);
  fireEvent.click(screen.getByRole("button", { name: "Ragionamento" }));
  // Mock reasoning data is still shown in the demo fallback.
  expect(screen.getByText(/La clausola 7\.2 consente il recesso/)).toBeInTheDocument();
});
