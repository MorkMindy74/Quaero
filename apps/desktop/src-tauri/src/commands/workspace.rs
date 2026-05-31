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
