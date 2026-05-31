import { renderHook, act, waitFor } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, expect, test } from "vitest";
import { useWorkspaces } from "./useWorkspaces";
import type { Client, Matter } from "../domain/types";

afterEach(() => clearMocks());

test("loads all workspaces on mount (empty query)", async () => {
  mockIPC((cmd, args) => {
    if (cmd === "search_workspaces") {
      expect((args as { query: string }).query).toBe("");
      return [{ id: "rossi-1", client: "Alfa S.r.l.", title: "Rossi c. Bianchi" }];
    }
  });
  const { result } = renderHook(() => useWorkspaces());
  await waitFor(() => expect(result.current.items).toHaveLength(1));
  expect(result.current.items[0].id).toBe("rossi-1");
  expect(result.current.error).toBeNull();
});

test("setQuery re-runs search with the query", async () => {
  const seen: string[] = [];
  mockIPC((cmd, args) => {
    if (cmd === "search_workspaces") {
      const q = (args as { query: string }).query;
      seen.push(q);
      return q === "beta" ? [{ id: "utp-1", client: "Banca Beta", title: "UTP" }] : [];
    }
  });
  const { result } = renderHook(() => useWorkspaces());
  await waitFor(() => expect(seen).toContain(""));
  act(() => result.current.setQuery("beta"));
  await waitFor(() => expect(result.current.items).toHaveLength(1));
  expect(result.current.items[0].id).toBe("utp-1");
});

test("search error sets error and empties the list (no crash)", async () => {
  mockIPC((cmd) => {
    if (cmd === "search_workspaces") throw new Error("backend down");
  });
  const { result } = renderHook(() => useWorkspaces());
  await waitFor(() => expect(result.current.error).not.toBeNull());
  expect(result.current.items).toEqual([]);
});

test("createMatter derives safe ids (client.id == matter.client) and refreshes", async () => {
  let createdMatter: Matter | null = null;
  let createdClient: Client | null = null;
  let searchCalls = 0;
  mockIPC((cmd, args) => {
    if (cmd === "create_workspace") {
      const a = args as { client: Client; matter: Matter };
      createdClient = a.client;
      createdMatter = a.matter;
      return { id: a.matter.id, client: a.client.name, title: a.matter.title };
    }
    if (cmd === "search_workspaces") {
      searchCalls += 1;
      return createdMatter
        ? [{ id: createdMatter.id, client: createdClient!.name, title: createdMatter.title }]
        : [];
    }
  });

  const { result } = renderHook(() => useWorkspaces(() => 0));
  await waitFor(() => expect(searchCalls).toBeGreaterThan(0));

  let summary: { id: string } | null = null;
  await act(async () => {
    summary = await result.current.createMatter("Alfa S.r.l.", "Rossi c. Bianchi", "Inadempimento");
  });

  expect(summary).not.toBeNull();
  expect(createdClient!.id).toBe("alfa-s-r-l");
  expect(createdMatter!.client).toBe("alfa-s-r-l"); // matter.client == client.id
  expect(createdMatter!.id).toBe("rossi-c-bianchi-000000"); // slug(title)-suffix
  expect(createdMatter!.subject).toBe("Inadempimento");
  await waitFor(() => expect(result.current.items.some((i) => i.id === "rossi-c-bianchi-000000")).toBe(true));
});

test("createMatter surfaces backend AlreadyExists as error, returns null", async () => {
  mockIPC((cmd) => {
    if (cmd === "search_workspaces") return [];
    if (cmd === "create_workspace") throw new Error("workspace already exists: dup");
  });
  const { result } = renderHook(() => useWorkspaces(() => 0));
  let summary: unknown = "x";
  await act(async () => {
    summary = await result.current.createMatter("Alfa", "Dup");
  });
  expect(summary).toBeNull();
  await waitFor(() => expect(result.current.error).toMatch(/already exists/));
});
