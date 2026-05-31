import { invoke } from "@tauri-apps/api/core";
import type { Client, Matter, WorkspaceView } from "../domain/types";

// Typed IPC contract mirroring `quaero-core` (ADR-0011). The frontend talks to
// the Rust backend only through `@tauri-apps/api/core`, never `window.__TAURI__`.
export interface PingRequest {
  message: string;
}

export interface PingResponse {
  reply: string;
}

export function ping(request: PingRequest): Promise<PingResponse> {
  return invoke<PingResponse>("ping", { request });
}

// --- #5B local persistence: create / open / search -------------------------

/** Lightweight listing entry; mirrors the desktop store's `WorkspaceSummary`. */
export interface WorkspaceSummary {
  id: string;
  client: string;
  title: string;
}

/** Create a new Pratica (Cliente + Pratica, no sources yet) and persist it. */
export function createWorkspace(
  client: Client,
  matter: Matter,
): Promise<WorkspaceSummary> {
  return invoke<WorkspaceSummary>("create_workspace", { client, matter });
}

/** Open a saved workspace, returning its derived view (dynamic + manual). */
export function openWorkspace(id: string): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("open_workspace", { id });
}

/** List/search saved workspaces by case-insensitive substring (empty = all). */
export function searchWorkspaces(query: string): Promise<WorkspaceSummary[]> {
  return invoke<WorkspaceSummary[]>("search_workspaces", { query });
}

/** Import a local file as a Documento Fonte into a Pratica (#6). The caller
 *  reads the file with `<input type="file">` + `arrayBuffer` and passes the
 *  bytes; the backend stores them and returns the updated view. */
export function importDocument(
  matterId: string,
  originalName: string,
  bytes: Uint8Array,
): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("import_document", { matterId, originalName, bytes });
}
