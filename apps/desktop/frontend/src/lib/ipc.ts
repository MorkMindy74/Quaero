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

// --- #8B manual Evidence: create an Estratto linked to a Fonte --------------

/** Create a manual Estratto (#8B) linked to a Fonte of an existing Pratica.
 *  The excerpt id and `createdAt` timestamp are generated server-side; if the
 *  Fonte has a stored file the excerpt is auto-pinned to its sha256. Returns the
 *  updated view so the caller can refresh the Estratti list. */
export function addExcerpt(args: {
  matterId: string;
  sourceId: string;
  anchorKind: string;
  anchorValue: string;
  quote: string;
  note?: string;
}): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("add_excerpt", {
    matterId: args.matterId,
    sourceId: args.sourceId,
    anchorKind: args.anchorKind,
    anchorValue: args.anchorValue,
    quote: args.quote,
    note: args.note ?? null,
  });
}

// --- #7 chat (stub provider) -----------------------------------------------

/** A chat reply. `grounded` is always false in #7 (no citations). */
export interface ChatReply {
  reply: string;
  grounded: boolean;
}

/** Send a chat turn to the active provider (default offline stub; opt-in local Ollama). */
export function chatSend(prompt: string): Promise<ChatReply> {
  return invoke<ChatReply>("chat_send", { request: { prompt } });
}

/** Which chat provider is active: "stub" (offline) | "ollamaLocal". Used by the
 *  StatusStrip to show an honest privacy posture. Returns a config flag, no data. */
export function chatProviderKind(): Promise<string> {
  return invoke<string>("chat_provider_kind");
}
