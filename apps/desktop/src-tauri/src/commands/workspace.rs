//! IPC commands for local workspace persistence (#5B). Thin mappings onto the
//! pure `crate::store` functions; they only resolve the per-app data directory
//! and translate `StoreError` into a `String` for the IPC boundary (no panics).

use std::path::PathBuf;

use quaero_core::domain::{Client, Matter, WorkspaceView};
use tauri::{AppHandle, Manager};

use crate::store::{self, SourceText, WorkspaceSummary};

/// Resolve the per-app `workspaces/` directory under the OS app-data location.
fn workspaces_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(base.join("workspaces"))
}

/// Resolve the per-app `files/` blob directory (sibling of `workspaces/`).
fn files_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(base.join("files"))
}

/// IPC: create a new Pratica (Cliente + Pratica, no sources yet) and persist it.
#[tauri::command]
pub fn create_workspace(
    app: AppHandle,
    client: Client,
    matter: Matter,
) -> Result<WorkspaceSummary, String> {
    let dir = workspaces_dir(&app)?;
    store::create(&dir, client, matter).map_err(|e| e.to_string())
}

/// IPC: open a saved workspace, returning its derived view.
#[tauri::command]
pub fn open_workspace(app: AppHandle, id: String) -> Result<WorkspaceView, String> {
    let dir = workspaces_dir(&app)?;
    store::open(&dir, &id).map_err(|e| e.to_string())
}

/// IPC: list/search saved workspaces by case-insensitive substring (empty = all).
#[tauri::command]
pub fn search_workspaces(app: AppHandle, query: String) -> Result<Vec<WorkspaceSummary>, String> {
    let dir = workspaces_dir(&app)?;
    store::search(&dir, &query).map_err(|e| e.to_string())
}

/// IPC: import a local file as a Documento Fonte into an existing Pratica.
/// The frontend reads the file (`<input type="file">` + `arrayBuffer`) and sends
/// the bytes here — no `tauri-plugin-fs`/`-dialog`, no extra capability.
#[tauri::command]
pub fn import_document(
    app: AppHandle,
    matter_id: String,
    original_name: String,
    bytes: Vec<u8>,
) -> Result<WorkspaceView, String> {
    let ws_dir = workspaces_dir(&app)?;
    let blob_dir = files_dir(&app)?;
    store::import_document(&ws_dir, &blob_dir, &matter_id, &original_name, &bytes)
        .map_err(|e| e.to_string())
}

/// IPC: create a manual Estratto (#8B) linked to a Fonte of an existing Pratica.
/// The excerpt id and `createdAt` timestamp are generated server-side; if the
/// Fonte has a stored file the excerpt is auto-pinned to its sha256. No extra
/// capability, no filesystem dialog: only the canonical JSON is updated.
#[tauri::command]
pub fn add_excerpt(
    app: AppHandle,
    matter_id: String,
    source_id: String,
    anchor_kind: String,
    anchor_value: String,
    quote: String,
    note: Option<String>,
) -> Result<WorkspaceView, String> {
    let ws_dir = workspaces_dir(&app)?;
    let blob_dir = files_dir(&app)?;
    store::add_excerpt(
        &ws_dir,
        &blob_dir,
        &matter_id,
        &source_id,
        &anchor_kind,
        &anchor_value,
        &quote,
        note.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// IPC: create a manual Citazione (citations-from-UI) linking a `claim` to an
/// existing Estratto of a Pratica. The citation id is generated server-side;
/// referential integrity (citation → excerpt) is enforced by the core. No
/// filesystem/blob access, no extra capability: only the canonical JSON updates.
#[tauri::command]
pub fn add_citation(
    app: AppHandle,
    matter_id: String,
    excerpt_id: String,
    claim: String,
) -> Result<WorkspaceView, String> {
    let ws_dir = workspaces_dir(&app)?;
    store::add_citation(&ws_dir, &matter_id, &excerpt_id, &claim).map_err(|e| e.to_string())
}

/// IPC: render a Pratica as a grounded Markdown report (#12 decomposition). The
/// backend returns the Markdown string; the frontend downloads it via a Blob.
/// No file is written by Rust, no save dialog, no extra capability.
#[tauri::command]
pub fn export_markdown(app: AppHandle, matter_id: String) -> Result<String, String> {
    let ws_dir = workspaces_dir(&app)?;
    store::workspace_markdown(&ws_dir, &matter_id).map_err(|e| e.to_string())
}

/// IPC: persist a derived text layer (#52) for a Documento source. The text is
/// produced in the renderer (UTF-8 for `.txt/.md`, pdf.js for PDF); Rust does NOT
/// parse the document — it validates and writes a local sidecar. No egress, no
/// new capability.
#[tauri::command]
pub fn set_source_text(
    app: AppHandle,
    matter_id: String,
    source_id: String,
    expected_sha256: String,
    text: String,
) -> Result<SourceText, String> {
    let ws_dir = workspaces_dir(&app)?;
    let blob_dir = files_dir(&app)?;
    store::set_source_text(
        &ws_dir,
        &blob_dir,
        &matter_id,
        &source_id,
        &expected_sha256,
        &text,
    )
    .map_err(|e| e.to_string())
}

/// IPC: read the derived text layer of a Documento source (#52). Read-only.
#[tauri::command]
pub fn get_source_text(
    app: AppHandle,
    matter_id: String,
    source_id: String,
) -> Result<SourceText, String> {
    let ws_dir = workspaces_dir(&app)?;
    let blob_dir = files_dir(&app)?;
    store::get_source_text(&ws_dir, &blob_dir, &matter_id, &source_id).map_err(|e| e.to_string())
}

/// IPC: edit an existing Estratto (quote + anchor + note); the Fonte link, the
/// sha256 pin and createdAt are preserved by the store. No new capability.
#[tauri::command]
pub fn update_excerpt(
    app: AppHandle,
    matter_id: String,
    excerpt_id: String,
    anchor_kind: String,
    anchor_value: String,
    quote: String,
    note: Option<String>,
) -> Result<WorkspaceView, String> {
    let ws_dir = workspaces_dir(&app)?;
    store::update_excerpt(
        &ws_dir,
        &matter_id,
        &excerpt_id,
        &anchor_kind,
        &anchor_value,
        &quote,
        note.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// IPC: delete an Estratto. Refused (error) if it is still cited.
#[tauri::command]
pub fn delete_excerpt(
    app: AppHandle,
    matter_id: String,
    excerpt_id: String,
) -> Result<WorkspaceView, String> {
    let ws_dir = workspaces_dir(&app)?;
    store::delete_excerpt(&ws_dir, &matter_id, &excerpt_id).map_err(|e| e.to_string())
}

/// IPC: edit a Citazione's claim (linked Estratto unchanged).
#[tauri::command]
pub fn update_citation(
    app: AppHandle,
    matter_id: String,
    citation_id: String,
    claim: String,
) -> Result<WorkspaceView, String> {
    let ws_dir = workspaces_dir(&app)?;
    store::update_citation(&ws_dir, &matter_id, &citation_id, &claim).map_err(|e| e.to_string())
}

/// IPC: delete a Citazione (always safe).
#[tauri::command]
pub fn delete_citation(
    app: AppHandle,
    matter_id: String,
    citation_id: String,
) -> Result<WorkspaceView, String> {
    let ws_dir = workspaces_dir(&app)?;
    store::delete_citation(&ws_dir, &matter_id, &citation_id).map_err(|e| e.to_string())
}
