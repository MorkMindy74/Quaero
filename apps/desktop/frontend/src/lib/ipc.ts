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

/** Create a manual Citazione linking a `claim` to an existing Estratto. The
 *  citation id is generated server-side; integrity (citation → excerpt) is
 *  enforced by the core. Returns the updated view so the list refreshes. */
export function addCitation(args: {
  matterId: string;
  excerptId: string;
  claim: string;
}): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("add_citation", {
    matterId: args.matterId,
    excerptId: args.excerptId,
    claim: args.claim,
  });
}

/** Render a Pratica as a grounded Markdown report (#12 decomposition). Returns
 *  the Markdown string; the caller downloads it via a Blob (no file is written
 *  by the backend, no save dialog). */
export function exportMarkdown(matterId: string): Promise<string> {
  return invoke<string>("export_markdown", { matterId });
}

// --- edit/delete Estratti e Citazioni --------------------------------------

/** Edit an existing Estratto (quote + anchor + note). The Fonte link, sha256
 *  pin and createdAt are preserved by the backend. */
export function updateExcerpt(args: {
  matterId: string;
  excerptId: string;
  anchorKind: string;
  anchorValue: string;
  quote: string;
  note?: string;
}): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("update_excerpt", {
    matterId: args.matterId,
    excerptId: args.excerptId,
    anchorKind: args.anchorKind,
    anchorValue: args.anchorValue,
    quote: args.quote,
    note: args.note ?? null,
  });
}

/** Delete an Estratto. Rejected by the backend if it is still cited. */
export function deleteExcerpt(matterId: string, excerptId: string): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("delete_excerpt", { matterId, excerptId });
}

/** Edit a Citazione's claim (linked Estratto unchanged). */
export function updateCitation(
  matterId: string,
  citationId: string,
  claim: string,
): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("update_citation", { matterId, citationId, claim });
}

/** Delete a Citazione (always safe). */
export function deleteCitation(matterId: string, citationId: string): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("delete_citation", { matterId, citationId });
}

// --- #52 document text layer -----------------------------------------------

/** Derived text-layer state of a source. Mirrors the desktop store's
 *  `SourceText`: `available` carries text; `empty` = supported file, no useful
 *  text; `absent` = no sidecar yet / file-less Fonte. ("failed"/"unsupported"
 *  are renderer-only states, never returned by the store.) */
export interface SourceText {
  status: "available" | "empty" | "absent";
  text?: string;
}

/** Persist a derived text layer for a Documento source (#52). The text is
 *  produced in the renderer (UTF-8 for .txt/.md, pdf.js for PDF); the backend
 *  never parses the document — it writes a local sidecar. `expectedSha256` is
 *  captured at extraction start and verified by the backend (under the per-matter
 *  lock) against the source's pinned digest, so the text can never be persisted
 *  against the wrong version/Fonte. */
export function setSourceText(
  matterId: string,
  sourceId: string,
  expectedSha256: string,
  text: string,
): Promise<SourceText> {
  return invoke<SourceText>("set_source_text", { matterId, sourceId, expectedSha256, text });
}

/** Read the derived text layer of a Documento source (#52). Read-only. */
export function getSourceText(matterId: string, sourceId: string): Promise<SourceText> {
  return invoke<SourceText>("get_source_text", { matterId, sourceId });
}

// --- #55 AI Evidence Assistant V1A -----------------------------------------

/** A proposed Estratto (not persisted). Mirrors `quaero_core::evidence`'s
 *  `EvidenceCandidate`. The lawyer approves/edits/discards it; only on approval
 *  does it become a real Estratto (with the quote verified against the text
 *  layer, server-side). */
export interface EvidenceCandidate {
  quote: string;
  anchorKind: string;
  anchorValue: string;
  reason: string;
}

/** Propose Evidence candidates for a Documento source from its local text layer
 *  (#52). Offline Stub provider (V1A): no LLM, no network, no auto-save. Returns
 *  an empty list when the source has no available text layer. */
export function proposeEvidence(
  matterId: string,
  sourceId: string,
): Promise<EvidenceCandidate[]> {
  return invoke<EvidenceCandidate[]>("propose_evidence", { matterId, sourceId });
}

/** Turn an approved candidate into a real Estratto (#55). The backend verifies,
 *  under the per-matter lock, that the `quote` occurs in the source's text layer;
 *  otherwise it refuses with no write. Returns the updated view. */
export function acceptEvidenceCandidate(
  matterId: string,
  sourceId: string,
  anchorKind: string,
  anchorValue: string,
  quote: string,
  note?: string,
): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("accept_evidence_candidate", {
    matterId,
    sourceId,
    anchorKind,
    anchorValue,
    quote,
    note: note ?? null,
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
