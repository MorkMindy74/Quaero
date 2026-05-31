//! Local workspace store (#5B): one canonical JSON file per Pratica under a
//! base directory. Filesystem I/O only via `std::fs` — no `tauri-plugin-fs`,
//! no extra capabilities (ADR-0011). The base dir is injected, so the store is
//! fully testable without a Tauri runtime.
//!
//! Boundaries enforced here:
//! - only a canonical [`Workspace`] is ever written ([`save`] takes `&Workspace`);
//! - loading goes through [`quaero_core::persistence::from_json`] → `RawWorkspace`
//!   + `TryFrom` validation;
//! - ids are path-safe (no `..`, `/`, `\`, no separators) so they cannot escape
//!   the store directory;
//! - writes are atomic (temp file + rename);
//! - `open` is strict (corrupt files error); `search` is tolerant (skips them).

use std::fs;
use std::path::{Path, PathBuf};

use quaero_core::domain::{Client, Matter, Workspace, WorkspaceView};
use quaero_core::persistence;
use serde::Serialize;

/// Lightweight listing entry for create/search results (never the full workspace).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    /// The matter id — also the on-disk file stem.
    pub id: String,
    /// Client display name (searchable).
    pub client: String,
    /// Matter title (searchable).
    pub title: String,
}

/// Why a store operation failed. Commands map this to a `String` for IPC.
#[derive(Debug)]
pub enum StoreError {
    /// The id is empty or contains path-traversal / unsafe characters.
    UnsafeId(String),
    /// No saved workspace with that id.
    NotFound(String),
    /// A workspace with that id already exists (create won't overwrite).
    AlreadyExists(String),
    /// The canonical model rejected the data (referential integrity, etc.).
    Domain(String),
    /// A stored file could not be parsed as a canonical workspace.
    Corrupt { id: String, reason: String },
    /// Filesystem error.
    Io(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::UnsafeId(id) => write!(f, "unsafe workspace id: {id:?}"),
            StoreError::NotFound(id) => write!(f, "workspace not found: {id}"),
            StoreError::AlreadyExists(id) => write!(f, "workspace already exists: {id}"),
            StoreError::Domain(msg) => write!(f, "invalid workspace: {msg}"),
            StoreError::Corrupt { id, reason } => write!(f, "corrupt workspace {id}: {reason}"),
            StoreError::Io(msg) => write!(f, "filesystem error: {msg}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e.to_string())
    }
}

/// Reject empty ids, path separators, `..`, and anything outside a strict
/// `[A-Za-z0-9_-]` set, so an id can never escape the store directory.
fn safe_file_stem(id: &str) -> Result<&str, StoreError> {
    let ok = !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(id)
    } else {
        Err(StoreError::UnsafeId(id.to_string()))
    }
}

/// Resolve the on-disk path for a workspace id, rejecting unsafe ids first.
fn workspace_path(base: &Path, id: &str) -> Result<PathBuf, StoreError> {
    let stem = safe_file_stem(id)?;
    Ok(base.join(format!("{stem}.json")))
}

fn summary(ws: &Workspace) -> WorkspaceSummary {
    WorkspaceSummary {
        id: ws.matter().id.0.clone(),
        client: ws.client().name.clone(),
        title: ws.matter().title.clone(),
    }
}

/// Persist a **canonical** workspace (never a view) atomically: write a temp
/// file, then rename it into place.
pub fn save(base: &Path, workspace: &Workspace) -> Result<(), StoreError> {
    let id = workspace.matter().id.0.clone();
    let path = workspace_path(base, &id)?;
    fs::create_dir_all(base)?;
    let json = persistence::to_json(workspace).map_err(|e| StoreError::Domain(e.to_string()))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json.as_bytes())?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Create a new Pratica (Cliente + Pratica, no sources yet) and persist it.
/// Refuses to overwrite an existing workspace.
pub fn create(base: &Path, client: Client, matter: Matter) -> Result<WorkspaceSummary, StoreError> {
    let id = matter.id.0.clone();
    let path = workspace_path(base, &id)?;
    if path.exists() {
        return Err(StoreError::AlreadyExists(id));
    }
    let workspace = Workspace::new(client, matter, Vec::new(), Vec::new())
        .map_err(|e| StoreError::Domain(e.to_string()))?;
    save(base, &workspace)?;
    Ok(summary(&workspace))
}

/// Load a saved workspace and return its derived view for the UI. A corrupt or
/// invalid file is an error (open is strict).
pub fn open(base: &Path, id: &str) -> Result<WorkspaceView, StoreError> {
    let path = workspace_path(base, id)?;
    if !path.exists() {
        return Err(StoreError::NotFound(id.to_string()));
    }
    let json = fs::read_to_string(&path)?;
    let ws = persistence::from_json(&json).map_err(|e| StoreError::Corrupt {
        id: id.to_string(),
        reason: e.to_string(),
    })?;
    Ok(ws.view())
}

/// List saved workspaces, filtered by a case-insensitive substring over client
/// name and matter title (empty query = all). Unreadable / corrupt files are
/// skipped so one bad file can't break listing. Results are sorted by id.
pub fn search(base: &Path, query: &str) -> Result<Vec<WorkspaceSummary>, StoreError> {
    let needle = query.trim().to_lowercase();
    let mut out: Vec<WorkspaceSummary> = Vec::new();

    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        // No store directory yet → nothing saved.
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(StoreError::Io(e.to_string())),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        // Only canonical `.json` files; skip `.json.tmp` and anything else.
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let json = match fs::read_to_string(&path) {
            Ok(json) => json,
            Err(_) => continue,
        };
        // search tolerates bad files: skip what won't parse as canonical.
        let ws = match persistence::from_json(&json) {
            Ok(ws) => ws,
            Err(_) => continue,
        };
        let s = summary(&ws);
        let hay = format!("{} {}", s.client.to_lowercase(), s.title.to_lowercase());
        if needle.is_empty() || hay.contains(&needle) {
            out.push(s);
        }
    }

    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quaero_core::domain::{ClientId, MatterId};
    use tempfile::tempdir;

    fn client(id: &str, name: &str) -> Client {
        Client {
            id: ClientId::new(id),
            name: name.to_string(),
        }
    }

    fn matter(id: &str, client: &str, title: &str) -> Matter {
        Matter {
            id: MatterId::new(id),
            client: ClientId::new(client),
            title: title.to_string(),
            subject: "s".to_string(),
        }
    }

    #[test]
    fn create_then_open_round_trips() {
        let dir = tempdir().unwrap();
        let s = create(
            dir.path(),
            client("alfa", "Alfa S.r.l."),
            matter("rossi-bianchi", "alfa", "Rossi c. Bianchi"),
        )
        .unwrap();
        assert_eq!(s.id, "rossi-bianchi");
        assert_eq!(s.client, "Alfa S.r.l.");

        let view = open(dir.path(), "rossi-bianchi").unwrap();
        assert_eq!(view.client.name, "Alfa S.r.l.");
        assert_eq!(view.matter.title, "Rossi c. Bianchi");
        // A freshly created Pratica has no sources → no dynamic dossiers.
        assert!(view.dossiers.is_empty());
    }

    #[test]
    fn saved_file_is_canonical_camelcase_no_view_state() {
        let dir = tempdir().unwrap();
        create(dir.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let raw = fs::read_to_string(dir.path().join("m.json")).unwrap();
        assert!(raw.contains("manualDossiers"));
        assert!(!raw.contains("\"dossiers\""));
        assert!(!raw.contains("dyn-"));
    }

    #[test]
    fn create_refuses_to_overwrite() {
        let dir = tempdir().unwrap();
        create(dir.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let again = create(dir.path(), client("alfa", "Alfa"), matter("m", "alfa", "T"));
        assert!(matches!(again, Err(StoreError::AlreadyExists(_))));
    }

    #[test]
    fn create_rejects_client_matter_mismatch() {
        let dir = tempdir().unwrap();
        let r = create(dir.path(), client("alfa", "Alfa"), matter("m", "beta", "T"));
        assert!(matches!(r, Err(StoreError::Domain(_))));
    }

    #[test]
    fn create_rejects_unsafe_matter_id_before_touching_disk() {
        let dir = tempdir().unwrap();
        let r = create(dir.path(), client("a", "A"), matter("../evil", "a", "T"));
        assert!(matches!(r, Err(StoreError::UnsafeId(_))));
        // nothing was written
        assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
    }

    #[test]
    fn open_missing_is_not_found() {
        let dir = tempdir().unwrap();
        assert!(matches!(
            open(dir.path(), "nope"),
            Err(StoreError::NotFound(_))
        ));
    }

    #[test]
    fn open_corrupt_file_errors() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        fs::write(dir.path().join("bad.json"), b"{ not valid json").unwrap();
        assert!(matches!(
            open(dir.path(), "bad"),
            Err(StoreError::Corrupt { .. })
        ));
    }

    #[test]
    fn open_rejects_unsafe_ids_no_escape() {
        let dir = tempdir().unwrap();
        for bad in [
            "..",
            "../evil",
            "a/b",
            "a\\b",
            "",
            "a.b",
            "with space",
            "a/../b",
        ] {
            assert!(
                matches!(open(dir.path(), bad), Err(StoreError::UnsafeId(_))),
                "must reject unsafe id {bad:?}"
            );
        }
    }

    #[test]
    fn search_filters_case_insensitive_and_empty_returns_all_sorted() {
        let dir = tempdir().unwrap();
        create(
            dir.path(),
            client("alfa", "Alfa S.r.l."),
            matter("rossi", "alfa", "Rossi c. Bianchi"),
        )
        .unwrap();
        create(
            dir.path(),
            client("beta", "Banca Beta"),
            matter("utp", "beta", "Operazione UTP"),
        )
        .unwrap();

        // empty query → all, sorted by id
        let all = search(dir.path(), "").unwrap();
        let ids: Vec<String> = all.iter().map(|s| s.id.clone()).collect();
        assert_eq!(ids, vec!["rossi".to_string(), "utp".to_string()]);

        // by client name, case-insensitive
        let r = search(dir.path(), "BETA").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "utp");

        // by matter title
        let r2 = search(dir.path(), "rossi c.").unwrap();
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0].id, "rossi");

        // no match
        assert!(search(dir.path(), "zzz").unwrap().is_empty());
    }

    #[test]
    fn search_skips_corrupt_files_and_continues() {
        let dir = tempdir().unwrap();
        create(
            dir.path(),
            client("alfa", "Alfa"),
            matter("ok", "alfa", "T"),
        )
        .unwrap();
        fs::write(dir.path().join("bad.json"), b"garbage").unwrap();
        let r = search(dir.path(), "").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "ok");
    }

    #[test]
    fn search_on_missing_dir_is_empty() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("does-not-exist");
        assert!(search(&sub, "").unwrap().is_empty());
    }
}
