//! Local workspace store (#5B): one canonical JSON file per Pratica under a
//! base directory. Filesystem I/O only via `std::fs` — no `tauri-plugin-fs`,
//! no extra capabilities (ADR-0011). The base dir is injected, so the store is
//! fully testable without a Tauri runtime.
//!
//! Boundaries enforced here:
//! - only a canonical [`Workspace`] is ever written (the writer takes
//!   `&Workspace`);
//! - loading goes through [`quaero_core::persistence::from_json`] → `RawWorkspace`
//!   + `TryFrom` validation;
//! - the on-disk file stem and the embedded `matter` id must agree and be
//!   path-safe ([`load_consistent`]), so a hostile but otherwise-canonical file
//!   cannot misroute `open` or surface a misleading/duplicate entry in `search`;
//! - ids are path-safe (no `..`, `/`, `\`, no separators) so they cannot escape
//!   the store directory;
//! - `create` is **exclusive and atomic**: content is written to a unique temp
//!   file, then published with `hard_link`, which fails atomically if the
//!   destination already exists — so concurrent creates can never overwrite or
//!   corrupt each other (no `exists()` TOCTOU, no shared temp path);
//! - `open` is strict (corrupt/inconsistent files error); `search` is tolerant
//!   (skips them).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use quaero_core::domain::{Client, Matter, Workspace, WorkspaceView};
use quaero_core::persistence;
use serde::Serialize;

/// Per-process counter giving each in-flight write its own temp file, so two
/// concurrent writers never collide on a shared temp path.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    /// A stored file could not be parsed, or its embedded id does not match its
    /// file stem (hostile / mislabeled file).
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

/// A per-call unique temp path in the store directory, so concurrent writers
/// never share a temp file.
fn unique_tmp(base: &Path, stem: &str) -> PathBuf {
    let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    base.join(format!("{stem}.{}.{}.tmp", std::process::id(), n))
}

fn summary(ws: &Workspace) -> WorkspaceSummary {
    WorkspaceSummary {
        id: ws.matter().id.0.clone(),
        client: ws.client().name.clone(),
        title: ws.matter().title.clone(),
    }
}

/// Load a canonical workspace from a store path and require that its embedded
/// `matter` id is filesystem-safe AND equal to the file stem. This rejects a
/// hostile-but-canonical file whose embedded id is unsafe (e.g. `../evil`) or
/// disagrees with its filename, so neither `open` nor `search` can be tricked
/// into misrouting, surfacing an unopenable entry, or advertising a duplicate
/// id. Loading itself flows through the validating `from_json` path.
fn load_consistent(path: &Path) -> Result<Workspace, StoreError> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let json = fs::read_to_string(path)?;
    let ws = persistence::from_json(&json).map_err(|e| StoreError::Corrupt {
        id: stem.to_string(),
        reason: e.to_string(),
    })?;
    let embedded = &ws.matter().id.0;
    let consistent = safe_file_stem(embedded).map(|s| s == stem).unwrap_or(false);
    if !consistent {
        return Err(StoreError::Corrupt {
            id: stem.to_string(),
            reason: format!("embedded matter id {embedded:?} does not match file stem {stem:?}"),
        });
    }
    Ok(ws)
}

/// Create a new Pratica (Cliente + Pratica, no sources yet) and persist it.
///
/// Exclusive + atomic: the complete JSON is written to a unique temp file and
/// then published with `hard_link`, which fails atomically if the destination
/// already exists. Concurrent creates for the same id therefore resolve to
/// exactly one winner; the rest get [`StoreError::AlreadyExists`]. The
/// destination only ever appears with complete content (no partial/overwrite).
pub fn create(base: &Path, client: Client, matter: Matter) -> Result<WorkspaceSummary, StoreError> {
    let id = matter.id.0.clone();
    let stem = safe_file_stem(&id)?.to_string();
    let path = base.join(format!("{stem}.json"));

    let workspace = Workspace::new(client, matter, Vec::new(), Vec::new())
        .map_err(|e| StoreError::Domain(e.to_string()))?;
    let json = persistence::to_json(&workspace).map_err(|e| StoreError::Domain(e.to_string()))?;

    fs::create_dir_all(base)?;
    let tmp = unique_tmp(base, &stem);
    fs::write(&tmp, json.as_bytes())?;

    // Atomic, exclusive publish: link fails if `path` already exists.
    let publish = fs::hard_link(&tmp, &path);
    // The temp link is no longer needed in either outcome.
    let _ = fs::remove_file(&tmp);

    match publish {
        Ok(()) => Ok(summary(&workspace)),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(StoreError::AlreadyExists(id))
        }
        Err(e) => Err(e.into()),
    }
}

/// Load a saved workspace and return its derived view for the UI. A corrupt,
/// inconsistent, or hostile file is an error (open is strict).
pub fn open(base: &Path, id: &str) -> Result<WorkspaceView, StoreError> {
    let path = workspace_path(base, id)?;
    if !path.exists() {
        return Err(StoreError::NotFound(id.to_string()));
    }
    let ws = load_consistent(&path)?;
    Ok(ws.view())
}

/// List saved workspaces, filtered by a case-insensitive substring over client
/// name and matter title (empty query = all). Files that are unreadable,
/// corrupt, or whose embedded id disagrees with their file stem are skipped, so
/// one bad/hostile file can't break or bias listing. Results are sorted by id.
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
        // Only canonical `.json` files; skip `.tmp` and anything else.
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // search tolerates bad/hostile/mislabeled files: skip what fails the
        // consistent-load check (id must be safe AND equal the file stem).
        let ws = match load_consistent(&path) {
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

    /// Build a canonical workspace directly (bypassing `create`) so tests can
    /// plant hostile / mislabeled files on disk.
    fn canonical(client_id: &str, matter_id: &str, title: &str) -> Workspace {
        Workspace::new(
            client(client_id, &client_id.to_uppercase()),
            matter(matter_id, client_id, title),
            vec![],
            vec![],
        )
        .expect("valid canonical workspace")
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
    fn create_leaves_no_temp_file_behind() {
        let dir = tempdir().unwrap();
        create(dir.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let tmp_left = fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("tmp"));
        assert!(!tmp_left, "no .tmp file should remain after create");
    }

    #[test]
    fn create_refuses_to_overwrite() {
        let dir = tempdir().unwrap();
        create(dir.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let again = create(dir.path(), client("alfa", "Alfa"), matter("m", "alfa", "T"));
        assert!(matches!(again, Err(StoreError::AlreadyExists(_))));
    }

    #[test]
    fn concurrent_create_same_id_exactly_one_wins() {
        use std::sync::Arc;
        use std::thread;

        let dir = tempdir().unwrap();
        let base: Arc<PathBuf> = Arc::new(dir.path().to_path_buf());

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let b = Arc::clone(&base);
                thread::spawn(move || {
                    create(
                        b.as_path(),
                        client("alfa", "Alfa"),
                        matter("dup", "alfa", "T"),
                    )
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let won = results.iter().filter(|r| r.is_ok()).count();
        let lost = results
            .iter()
            .filter(|r| matches!(r, Err(StoreError::AlreadyExists(_))))
            .count();

        assert_eq!(won, 1, "exactly one concurrent create must win");
        assert_eq!(lost, 7, "all other creates must fail with AlreadyExists");
        // Exactly one canonical file, openable.
        assert!(open(base.as_path(), "dup").is_ok());
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
    fn open_rejects_file_whose_internal_id_mismatches_stem() {
        let dir = tempdir().unwrap();
        // canonical file named mislabeled.json but embedding matter id "other"
        fs::write(
            dir.path().join("mislabeled.json"),
            persistence::to_json(&canonical("a", "other", "T")).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            open(dir.path(), "mislabeled"),
            Err(StoreError::Corrupt { .. })
        ));
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
    fn search_skips_file_whose_internal_id_mismatches_stem() {
        let dir = tempdir().unwrap();
        create(
            dir.path(),
            client("alfa", "Alfa"),
            matter("ok", "alfa", "T"),
        )
        .unwrap();
        // canonical-but-mislabeled: filename evil.json, embedded id "ok2"
        fs::write(
            dir.path().join("evil.json"),
            persistence::to_json(&canonical("beta", "ok2", "X")).unwrap(),
        )
        .unwrap();
        let r = search(dir.path(), "").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "ok");
    }

    #[test]
    fn search_skips_file_with_unsafe_internal_id() {
        let dir = tempdir().unwrap();
        // canonical file (safe filename) whose embedded matter id is traversal.
        fs::write(
            dir.path().join("eviltrap.json"),
            persistence::to_json(&canonical("a", "../evil", "T")).unwrap(),
        )
        .unwrap();
        assert!(search(dir.path(), "").unwrap().is_empty());
    }

    #[test]
    fn search_skips_file_advertising_a_duplicate_id() {
        let dir = tempdir().unwrap();
        create(
            dir.path(),
            client("alfa", "Alfa"),
            matter("ok", "alfa", "T"),
        )
        .unwrap();
        // a second file (ok2.json) that lies, advertising the existing id "ok"
        fs::write(
            dir.path().join("ok2.json"),
            persistence::to_json(&canonical("beta", "ok", "DUPE")).unwrap(),
        )
        .unwrap();
        let r = search(dir.path(), "").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "ok");
        assert_eq!(
            r[0].title, "T",
            "the real file wins, not the duplicate-id liar"
        );
    }

    #[test]
    fn search_on_missing_dir_is_empty() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("does-not-exist");
        assert!(search(&sub, "").unwrap().is_empty());
    }
}
