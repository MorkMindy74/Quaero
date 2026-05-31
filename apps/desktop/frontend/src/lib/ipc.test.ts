import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, expect, test } from "vitest";
import { createWorkspace, openWorkspace, searchWorkspaces } from "./ipc";
import type { Client, Matter } from "../domain/types";

afterEach(() => clearMocks());

const client: Client = { id: "alfa", name: "Alfa S.r.l." };
const matter: Matter = {
  id: "rossi-bianchi",
  client: "alfa",
  title: "Rossi c. Bianchi",
  subject: "Inadempimento contrattuale",
};

test("createWorkspace forwards client+matter and returns the summary", async () => {
  mockIPC((cmd, args) => {
    expect(cmd).toBe("create_workspace");
    const a = args as { client: Client; matter: Matter };
    expect(a.client.id).toBe("alfa");
    expect(a.matter.id).toBe("rossi-bianchi");
    return { id: a.matter.id, client: a.client.name, title: a.matter.title };
  });

  const summary = await createWorkspace(client, matter);
  expect(summary).toEqual({
    id: "rossi-bianchi",
    client: "Alfa S.r.l.",
    title: "Rossi c. Bianchi",
  });
});

test("openWorkspace sends the id and returns a WorkspaceView", async () => {
  mockIPC((cmd, args) => {
    expect(cmd).toBe("open_workspace");
    expect((args as { id: string }).id).toBe("rossi-bianchi");
    return { client, matter, sources: [], dossiers: [] };
  });

  const view = await openWorkspace("rossi-bianchi");
  expect(view.matter.title).toBe("Rossi c. Bianchi");
  expect(view.dossiers).toEqual([]);
});

test("searchWorkspaces forwards the query and returns summaries", async () => {
  mockIPC((cmd, args) => {
    expect(cmd).toBe("search_workspaces");
    expect((args as { query: string }).query).toBe("alfa");
    return [{ id: "rossi-bianchi", client: "Alfa S.r.l.", title: "Rossi c. Bianchi" }];
  });

  const results = await searchWorkspaces("alfa");
  expect(results).toHaveLength(1);
  expect(results[0].id).toBe("rossi-bianchi");
});

test("searchWorkspaces with an empty query returns all saved summaries", async () => {
  mockIPC((cmd, args) => {
    expect(cmd).toBe("search_workspaces");
    expect((args as { query: string }).query).toBe("");
    return [
      { id: "rossi", client: "Alfa S.r.l.", title: "Rossi c. Bianchi" },
      { id: "utp", client: "Banca Beta", title: "Operazione UTP" },
    ];
  });

  const results = await searchWorkspaces("");
  expect(results.map((r) => r.id)).toEqual(["rossi", "utp"]);
});
