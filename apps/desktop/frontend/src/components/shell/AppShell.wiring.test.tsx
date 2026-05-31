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

test("#6 importing a document calls import_document and shows the new Fonte", async () => {
  let imported: { matterId: string; originalName: string } | null = null;
  mockIPC((cmd, args) => {
    if (cmd === "search_workspaces") return [{ id: "rossi-1", client: "Alfa", title: "Rossi" }];
    if (cmd === "open_workspace") return viewOf("Alfa", "Rossi");
    if (cmd === "import_document") {
      const a = args as { matterId: string; originalName: string };
      imported = { matterId: a.matterId, originalName: a.originalName };
      return {
        client: { id: "c", name: "Alfa" },
        matter: { id: "m", client: "c", title: "Rossi", subject: "s" },
        sources: [
          {
            id: "doc-1",
            kind: "Documento",
            title: "contract.pdf",
            meta: "3 byte",
            file: { storedName: "doc-1.pdf", originalName: "contract.pdf", byteLen: 3, sha256: "ab" },
          },
        ],
        dossiers: [{ id: "dyn-documento", name: "Documenti", kind: "Dynamic", sources: ["doc-1"] }],
      };
    }
  });

  render(<AppShell />);
  const sidebar = screen.getByTestId("region-sidebar");
  fireEvent.click(await within(sidebar).findByText("Rossi"));

  const context = screen.getByTestId("region-context");
  const input = await within(context).findByLabelText("Importa documento");
  const file = new File([new Uint8Array([1, 2, 3])], "contract.pdf", { type: "application/pdf" });
  // jsdom does not implement Blob.arrayBuffer(); stub it for this instance.
  Object.defineProperty(file, "arrayBuffer", {
    value: async () => new Uint8Array([1, 2, 3]).buffer,
  });
  fireEvent.change(input, { target: { files: [file] } });

  await waitFor(() => expect(imported).not.toBeNull());
  expect(imported!.originalName).toBe("contract.pdf");
  await waitFor(() => expect(within(context).getByText("contract.pdf")).toBeInTheDocument());
});

test("#6 a file over 25 MB is rejected client-side (error shown, no IPC call)", async () => {
  let importCalled = false;
  mockIPC((cmd) => {
    if (cmd === "search_workspaces") return [{ id: "rossi-1", client: "Alfa", title: "Rossi" }];
    if (cmd === "open_workspace") return viewOf("Alfa", "Rossi");
    if (cmd === "import_document") {
      importCalled = true;
      return viewOf("Alfa", "Rossi");
    }
  });

  render(<AppShell />);
  const sidebar = screen.getByTestId("region-sidebar");
  fireEvent.click(await within(sidebar).findByText("Rossi"));

  const context = screen.getByTestId("region-context");
  const input = await within(context).findByLabelText("Importa documento");
  const big = new File([new Uint8Array([1])], "big.bin");
  Object.defineProperty(big, "size", { value: 26 * 1024 * 1024 });
  fireEvent.change(input, { target: { files: [big] } });

  await waitFor(() =>
    expect(within(context).getByText("File troppo grande (limite 25 MB).")).toBeInTheDocument(),
  );
  expect(importCalled).toBe(false);
});
