//! IPC commands for local workspace persistence (#5B). Thin mappings onto the
//! pure `crate::store` functions; they only resolve the per-app data directory
//! and translate `StoreError` into a `String` for the IPC boundary (no panics).

use std::path::PathBuf;

use quaero_core::domain::{Client, Matter, WorkspaceView};
use tauri::{AppHandle, Manager};

use crate::store::{self, WorkspaceSummary};

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
