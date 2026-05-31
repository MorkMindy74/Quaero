import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, expect, test } from "vitest";
import AppShell from "./AppShell";
import "../../i18n";
import type { Client, Matter } from "../../domain/types";

afterEach(() => clearMocks());

const viewOf = (client: string, title: string) => ({
  client: { id: "c", name: client },
  matter: { id: "m", client: "c", title, subject: "s" },
  sources: [],
  dossiers: [],
});

test("#5C lists saved matters from searchWorkspaces", async () => {
  mockIPC((cmd) => {
    if (cmd === "search_workspaces")
      return [{ id: "rossi-1", client: "Alfa S.r.l.", title: "Rossi c. Bianchi" }];
  });
  render(<AppShell />);
  const sidebar = screen.getByTestId("region-sidebar");
  await waitFor(() => expect(within(sidebar).getByText("Rossi c. Bianchi")).toBeInTheDocument());
});

test("#5C empty list shows the empty state", async () => {
  mockIPC((cmd) => {
    if (cmd === "search_workspaces") return [];
  });
  render(<AppShell />);
  const sidebar = screen.getByTestId("region-sidebar");
  await waitFor(() => expect(within(sidebar).getByText("Nessuna Pratica. Creane una.")).toBeInTheDocument());
});

test("#5C clicking a matter opens it into the context panel", async () => {
  mockIPC((cmd, args) => {
    if (cmd === "search_workspaces")
      return [{ id: "aperto-1", client: "Beta SRL", title: "Caso Aperto Test" }];
    if (cmd === "open_workspace") {
      expect((args as { id: string }).id).toBe("aperto-1");
      return viewOf("Beta SRL", "Caso Aperto Test");
    }
  });
  render(<AppShell />);
  const sidebar = screen.getByTestId("region-sidebar");
  const row = await within(sidebar).findByText("Caso Aperto Test");
  fireEvent.click(row);
  const context = screen.getByTestId("region-context");
  await waitFor(() => expect(within(context).getByText(/Caso Aperto Test/)).toBeInTheDocument());
});

test("#5C '+ Nuova Pratica' creates a matter via createWorkspace then opens it", async () => {
  let created: { client: Client; matter: Matter } | null = null;
  mockIPC((cmd, args) => {
    if (cmd === "search_workspaces") return [];
    if (cmd === "create_workspace") {
      created = args as { client: Client; matter: Matter };
      return { id: created.matter.id, client: created.client.name, title: created.matter.title };
    }
    if (cmd === "open_workspace") return viewOf("Studio Gamma", "Nuova Causa");
  });

  render(<AppShell />);
  fireEvent.click(screen.getByText("+ Nuova Pratica"));

  const dialog = await screen.findByRole("dialog", { name: "Nuova Pratica" });
  fireEvent.change(within(dialog).getByLabelText("Cliente"), { target: { value: "Studio Gamma" } });
  fireEvent.change(within(dialog).getByLabelText("Titolo pratica"), { target: { value: "Nuova Causa" } });
  fireEvent.click(within(dialog).getByRole("button", { name: "Crea" }));

  await waitFor(() => expect(created).not.toBeNull());
  // ids derived safely; matter.client must equal client.id
  expect(created!.client.id).toBe("studio-gamma");
  expect(created!.matter.client).toBe("studio-gamma");
  expect(created!.matter.id.startsWith("nuova-causa-")).toBe(true);
  // after create it opens → context panel shows the new matter
  const context = screen.getByTestId("region-context");
  await waitFor(() => expect(within(context).getByText(/Nuova Causa/)).toBeInTheDocument());
});

test("#5C search errors surface as an inline message (no crash)", async () => {
  mockIPC((cmd) => {
    if (cmd === "search_workspaces") throw new Error("backend down");
  });
  render(<AppShell />);
  const sidebar = screen.getByTestId("region-sidebar");
  await waitFor(() =>
    expect(within(sidebar).getByText("Errore nel caricamento delle Pratiche.")).toBeInTheDocument(),
  );
});
