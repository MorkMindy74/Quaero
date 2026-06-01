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

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use quaero_core::domain::{
    Anchor, Citation, CitationId, Client, Excerpt, ExcerptId, Matter, SourceId, SourceRef,
    SourceType, StoredFile, Workspace, WorkspaceView,
};
use quaero_core::hash::sha256_hex;
use quaero_core::persistence;
use serde::Serialize;

/// Maximum size of a single imported document (#6 cap; no streaming yet).
pub const MAX_IMPORT_BYTES: u64 = 25 * 1024 * 1024;

/// Per-process counter giving each in-flight write its own temp file, so two
/// concurrent writers never collide on a shared temp path.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Per-process counter feeding generated, filesystem-safe source ids.
static SOURCE_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    /// The imported file exceeds the size cap.
    TooLarge { limit: u64, actual: u64 },
    /// A referenced stored file is missing, or its bytes no longer match the
    /// recorded digest/length — so an Excerpt cannot be pinned to it (#8B).
    EvidenceIntegrity { source: String, reason: String },
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
            StoreError::TooLarge { limit, actual } => {
                write!(f, "file too large: {actual} bytes (limit {limit})")
            }
            StoreError::EvidenceIntegrity { source, reason } => {
                write!(
                    f,
                    "evidence integrity failure for source {source}: {reason}"
                )
            }
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

/// A persisted `StoredFile.stored_name` is generated safely at import
/// (`doc-<pid>-<n>.<ext>`), but it is a plain string in the canonical model — a
/// hostile/corrupt JSON could set it to a traversal/absolute path or a Windows
/// device. Anywhere we turn it into a filesystem path we accept ONLY a name of
/// the shape Quaero itself generates — a **positive allowlist**, not a denylist:
/// - only ASCII `[A-Za-z0-9._-]` → closes whole classes at once (`:` drive/ADS,
///   `$` of `CONIN$`/`CONOUT$`, non-ASCII superscripts like `COM¹`, separators,
///   NUL, spaces);
/// - structural: non-empty, no `..`, not absolute, exactly one component, no
///   trailing `.`;
/// - plus reserved DOS device basenames (`CON`/`PRN`/`AUX`/`NUL`/`COM1..9`/
///   `LPT1..9`), case-insensitive, incl. with an extension (e.g. `CON.txt`) —
///   these use only allowed characters, so the allowlist alone wouldn't catch
///   them.
fn is_safe_stored_name(name: &str) -> bool {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        || name.contains("..")
        || name.ends_with('.')
        || Path::new(name).is_absolute()
        || Path::new(name).components().count() != 1
    {
        return false;
    }
    // The device name is the part before the first `.` (so `NUL.txt` is unsafe).
    let base = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    !matches!(
        base.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

/// A real sha256 digest: exactly 64 ASCII hex chars. Persisted digests come from
/// JSON and are otherwise free text — validating them at load keeps "a sha256 is
/// a sha256" an invariant (no free text masquerading as a digest, e.g. active
/// Markdown in a later export).
fn is_valid_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
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

/// Generate a filesystem-safe source id. Never derived from user input, so it
/// can't carry path separators or traversal.
fn new_source_id() -> String {
    let n = SOURCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("doc-{}-{}", std::process::id(), n)
}

/// Derive a safe file extension from the original name: lowercase ASCII
/// alphanumerics only, capped; never includes separators or dots. Empty → "bin".
fn safe_extension(original_name: &str) -> String {
    let raw = original_name.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .take(12)
        .collect();
    if cleaned.is_empty() {
        "bin".to_string()
    } else {
        cleaned
    }
}

/// Sanitise the original name for display only (strip control chars). Never used
/// as a path. Empty → a generic label.
fn display_title(original_name: &str) -> String {
    let t: String = original_name.chars().filter(|c| !c.is_control()).collect();
    let t = t.trim();
    if t.is_empty() {
        "Documento".to_string()
    } else {
        t.to_string()
    }
}

/// In-process, per-matter lock so concurrent read-modify-write imports into the
/// same Pratica are serialised (no last-write-wins source loss). Single-process
/// only by design — no cross-process locking, no database.
fn matter_lock(base: &Path, matter_id: &str) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();
    let registry = LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let key = format!("{}|{}", base.display(), matter_id);
    let mut guard = registry.lock().expect("matter-lock registry poisoned");
    Arc::clone(guard.entry(key).or_insert_with(|| Arc::new(Mutex::new(()))))
}

/// Write `bytes` to `tmp`, then publish to `dest` atomically and exclusively
/// via `hard_link` (which fails if `dest` already exists). The caller removes
/// `tmp` afterwards on EVERY outcome, so no partial temp file is ever left.
fn write_and_publish(tmp: &Path, dest: &Path, bytes: &[u8], id: &str) -> Result<(), StoreError> {
    fs::write(tmp, bytes)?;
    match fs::hard_link(tmp, dest) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(StoreError::AlreadyExists(id.to_string()))
        }
        Err(e) => Err(e.into()),
    }
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
    // Harden the persisted `storedName` (a plain canonical string): reject any
    // file whose name is not a bare filename, so a hostile/corrupt JSON cannot
    // later steer a filesystem read (e.g. the #8B Evidence pin) out of the blob
    // store via traversal/absolute paths. Caught here, at load, for every reader.
    for source in ws.sources() {
        if let Some(file) = source.file.as_ref() {
            if !is_safe_stored_name(&file.stored_name) {
                return Err(StoreError::Corrupt {
                    id: stem.to_string(),
                    reason: format!("unsafe stored file name {:?}", file.stored_name),
                });
            }
            // A hostile JSON must not be able to inflate the recorded size beyond
            // the import cap (would otherwise drive an oversized Evidence read).
            if file.byte_len > MAX_IMPORT_BYTES {
                return Err(StoreError::Corrupt {
                    id: stem.to_string(),
                    reason: format!(
                        "stored file byteLen {} exceeds the limit {MAX_IMPORT_BYTES}",
                        file.byte_len
                    ),
                });
            }
            // A persisted digest must be a real sha256 (64 hex), not free text —
            // otherwise a corrupted JSON could carry e.g. active Markdown that a
            // later export would render.
            if !is_valid_sha256(&file.sha256) {
                return Err(StoreError::Corrupt {
                    id: stem.to_string(),
                    reason: format!("invalid stored file sha256 {:?}", file.sha256),
                });
            }
        }
    }
    // Same invariant for any Excerpt sha-pin loaded from the JSON.
    for excerpt in ws.excerpts() {
        if let Some(sha) = excerpt.source_sha256() {
            if !is_valid_sha256(sha) {
                return Err(StoreError::Corrupt {
                    id: stem.to_string(),
                    reason: format!("invalid excerpt sourceSha256 {sha:?}"),
                });
            }
        }
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

    // Write the complete content then publish exclusively. The temp file is
    // removed on EVERY outcome — write failure, publish failure (AlreadyExists
    // or other), or success — so create never leaves a partial `.tmp` behind.
    let outcome = write_and_publish(&tmp, &path, json.as_bytes(), &id);
    let _ = fs::remove_file(&tmp);

    outcome.map(|()| summary(&workspace))
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

/// Atomically overwrite an existing workspace's canonical JSON (unique temp +
/// rename). Used by import to persist a workspace that gained a source.
pub fn update(base: &Path, workspace: &Workspace) -> Result<(), StoreError> {
    let id = workspace.matter().id.0.clone();
    let path = workspace_path(base, &id)?;
    fs::create_dir_all(base)?;
    let json = persistence::to_json(workspace).map_err(|e| StoreError::Domain(e.to_string()))?;
    let tmp = unique_tmp(base, safe_file_stem(&id)?);
    fs::write(&tmp, json.as_bytes())?;
    if let Err(e) = fs::rename(&tmp, &path) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}

/// Import a local file as a Documento Fonte into an existing Pratica (#6).
///
/// Contract:
/// - the file content is written **first** (atomically), then the canonical
///   JSON is updated — so the JSON never references a missing blob; the only
///   failure residue is a harmless orphan blob;
/// - the on-disk names are **generated** (`source id` + sanitised extension);
///   `original_name` is kept as display metadata only and never becomes a path;
/// - a per-matter in-process lock serialises the read-modify-write so concurrent
///   imports into the same Pratica cannot drop a source via last-write-wins;
/// - files above [`MAX_IMPORT_BYTES`] are rejected (no streaming yet).
///
/// Returns the updated [`WorkspaceView`] for the UI.
pub fn import_document(
    workspaces_dir: &Path,
    files_dir: &Path,
    matter_id: &str,
    original_name: &str,
    bytes: &[u8],
) -> Result<WorkspaceView, StoreError> {
    import_document_with(
        workspaces_dir,
        files_dir,
        matter_id,
        original_name,
        bytes,
        &mut || new_source_id(),
    )
}

/// Core of [`import_document`] with an injectable id generator (for tests that
/// force id collisions). Blob publication is **exclusive** (`hard_link`, never
/// overwrite): a candidate id that collides with an existing source in the
/// loaded workspace, or with an existing blob on disk, triggers regeneration —
/// so an existing client document's bytes are never replaced, keeping the
/// persisted `sha256`/`byteLen` consistent with the physical file.
fn import_document_with(
    workspaces_dir: &Path,
    files_dir: &Path,
    matter_id: &str,
    original_name: &str,
    bytes: &[u8],
    gen_id: &mut dyn FnMut() -> String,
) -> Result<WorkspaceView, StoreError> {
    if bytes.len() as u64 > MAX_IMPORT_BYTES {
        return Err(StoreError::TooLarge {
            limit: MAX_IMPORT_BYTES,
            actual: bytes.len() as u64,
        });
    }

    // Validate the matter id (also the files subdir name) before anything else.
    let matter_stem = safe_file_stem(matter_id)?.to_string();

    // Serialise read-modify-write per matter.
    let lock = matter_lock(workspaces_dir, &matter_stem);
    let _guard = lock.lock().expect("per-matter lock poisoned");

    // The Pratica must already exist.
    let ws_path = workspace_path(workspaces_dir, matter_id)?;
    if !ws_path.exists() {
        return Err(StoreError::NotFound(matter_id.to_string()));
    }
    let workspace = load_consistent(&ws_path)?;

    let ext = safe_extension(original_name); // safe; never a path
    let sha256 = sha256_hex(bytes);
    let matter_dir = files_dir.join(&matter_stem);
    fs::create_dir_all(&matter_dir)?;

    // Ids already taken by this workspace — a candidate matching one would be
    // rejected by `with_source`, so we skip it before writing any bytes.
    let taken: HashSet<String> = workspace.sources().iter().map(|s| s.id.0.clone()).collect();

    // Allocate a unique id and publish the blob EXCLUSIVELY (no overwrite).
    let mut attempts = 0u32;
    let (source_id, stored_name) = loop {
        attempts += 1;
        if attempts > 1000 {
            return Err(StoreError::Io(
                "could not allocate a unique source id".to_string(),
            ));
        }
        let candidate = gen_id();
        // never accept an unsafe generated id, and never reuse a workspace id
        if safe_file_stem(&candidate).is_err() || taken.contains(&candidate) {
            continue;
        }
        let stored_name = format!("{candidate}.{ext}");
        let blob_path = matter_dir.join(&stored_name);
        let tmp = unique_tmp(&matter_dir, &candidate);
        match write_and_publish(&tmp, &blob_path, bytes, &candidate) {
            Ok(()) => {
                let _ = fs::remove_file(&tmp);
                break (candidate, stored_name);
            }
            // blob path already exists (e.g. an orphan): regenerate, never overwrite
            Err(StoreError::AlreadyExists(_)) => {
                let _ = fs::remove_file(&tmp);
                continue;
            }
            Err(e) => {
                let _ = fs::remove_file(&tmp);
                return Err(e);
            }
        }
    };

    // Register the canonical SourceRef and update the JSON (blob already on disk).
    let source = SourceRef {
        id: SourceId::new(source_id),
        kind: SourceType::Documento,
        title: display_title(original_name),
        meta: format!("{} byte", bytes.len()),
        file: Some(StoredFile {
            stored_name,
            original_name: original_name.to_string(),
            byte_len: bytes.len() as u64,
            sha256,
        }),
    };
    let updated = workspace
        .with_source(source)
        .map_err(|e| StoreError::Domain(e.to_string()))?;
    update(workspaces_dir, &updated)?;
    Ok(updated.view())
}

/// Per-process counter feeding generated, filesystem-safe excerpt ids.
static EXCERPT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a safe excerpt id. Never derived from user input (no separators /
/// traversal). Collisions with an existing excerpt are handled by the caller.
fn new_excerpt_id() -> String {
    let n = EXCERPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("exc-{}-{}", std::process::id(), n)
}

/// Current UTC time as RFC3339 (`YYYY-MM-DDTHH:MM:SSZ`), std-only (no `chrono`
/// dependency). Generated in the desktop layer so the pure core stays
/// clock-free.
fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    rfc3339_from_unix(secs)
}

/// Format a Unix timestamp (seconds, UTC) as RFC3339. Pure → unit-testable.
fn rfc3339_from_unix(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (hh, mm, ss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 → (year, month,
/// day). Valid for the full proleptic Gregorian range; std-only.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Add a manually-captured [`Excerpt`] to an existing Pratica (#8B).
///
/// Contract:
/// - read-modify-write is serialised **per matter** (no last-write-wins loss);
/// - the excerpt id is **generated server-side** (path-safe, never from input)
///   and regenerated on the rare collision with an existing excerpt id;
/// - `created_at` (RFC3339 UTC) is stamped **here** — the core stays clock-free;
/// - if the referenced Fonte carries a [`StoredFile`], the actual blob bytes are
///   **re-hashed under the lock** and the excerpt is pinned to that freshly
///   computed digest; a missing blob, or bytes disagreeing with the recorded
///   digest/length, fail with [`StoreError::EvidenceIntegrity`] (no tautological
///   pin). A fileless Fonte gets no pin;
/// - persisted via the atomic [`update`] (unique temp + rename);
/// - empty quote/anchor and referential integrity are enforced by the core.
#[allow(clippy::too_many_arguments)]
pub fn add_excerpt(
    base: &Path,
    files_dir: &Path,
    matter_id: &str,
    source_id: &str,
    anchor_kind: &str,
    anchor_value: &str,
    quote: &str,
    note: Option<&str>,
) -> Result<WorkspaceView, StoreError> {
    add_excerpt_with(
        base,
        files_dir,
        matter_id,
        source_id,
        anchor_kind,
        anchor_value,
        quote,
        note,
        &mut new_excerpt_id,
        &now_rfc3339,
    )
}

/// Core of [`add_excerpt`] with injectable id generator and clock (for tests
/// that force an id collision or assert a deterministic timestamp).
#[allow(clippy::too_many_arguments)]
fn add_excerpt_with(
    base: &Path,
    files_dir: &Path,
    matter_id: &str,
    source_id: &str,
    anchor_kind: &str,
    anchor_value: &str,
    quote: &str,
    note: Option<&str>,
    gen_id: &mut dyn FnMut() -> String,
    now: &dyn Fn() -> String,
) -> Result<WorkspaceView, StoreError> {
    // Validate the matter id before touching disk.
    let matter_stem = safe_file_stem(matter_id)?.to_string();

    // Serialise read-modify-write per matter.
    let lock = matter_lock(base, &matter_stem);
    let _guard = lock.lock().expect("per-matter lock poisoned");

    let ws_path = workspace_path(base, matter_id)?;
    if !ws_path.exists() {
        return Err(StoreError::NotFound(matter_id.to_string()));
    }
    let workspace = load_consistent(&ws_path)?;

    // The referenced Fonte must exist; if it carries a file, the excerpt is
    // pinned to a FRESHLY COMPUTED digest of the actual stored bytes (Evidence).
    // We read + hash the blob under the per-matter lock and fail if it is missing
    // or disagrees with the recorded digest/length — so a tampered/stale blob can
    // never receive a tautological pin. A fileless Fonte gets no pin; a missing
    // source yields no pin and the canonical validation below rejects it as a
    // dangling excerpt.
    let pin = match workspace.sources().iter().find(|s| s.id.0 == source_id) {
        Some(source) => match source.file.as_ref() {
            Some(file) => {
                // Defence-in-depth: `load_consistent` already rejects unsafe
                // stored names, but never turn one into a path without checking.
                if !is_safe_stored_name(&file.stored_name) {
                    return Err(StoreError::EvidenceIntegrity {
                        source: source_id.to_string(),
                        reason: format!("unsafe stored file name {:?}", file.stored_name),
                    });
                }
                let blob = files_dir.join(&matter_stem).join(&file.stored_name);

                // No-follow metadata on the final path: the evidence blob must be
                // a REGULAR file (reject symlinks/reparse points, directories,
                // devices/special files) — never read THROUGH a symlink.
                let meta =
                    fs::symlink_metadata(&blob).map_err(|_| StoreError::EvidenceIntegrity {
                        source: source_id.to_string(),
                        reason: "stored file missing or unreadable".to_string(),
                    })?;
                if !meta.file_type().is_file() {
                    return Err(StoreError::EvidenceIntegrity {
                        source: source_id.to_string(),
                        reason: "stored file is not a regular file (symlink/device/dir rejected)"
                            .to_string(),
                    });
                }
                // Independent read cap: never read more than the import limit,
                // regardless of the (untrusted) recorded byteLen.
                if meta.len() > MAX_IMPORT_BYTES {
                    return Err(StoreError::EvidenceIntegrity {
                        source: source_id.to_string(),
                        reason: format!(
                            "stored file exceeds the size limit ({MAX_IMPORT_BYTES} bytes)"
                        ),
                    });
                }
                // Check the on-disk length against the recorded byteLen BEFORE
                // reading any bytes: bounds the read and catches tampering early.
                if meta.len() != file.byte_len {
                    return Err(StoreError::EvidenceIntegrity {
                        source: source_id.to_string(),
                        reason: "stored file length does not match the recorded byteLen"
                            .to_string(),
                    });
                }
                // Only now read the bytes and verify the digest.
                let bytes = fs::read(&blob).map_err(|_| StoreError::EvidenceIntegrity {
                    source: source_id.to_string(),
                    reason: "stored file missing or unreadable".to_string(),
                })?;
                let digest = sha256_hex(&bytes);
                if digest != file.sha256 {
                    return Err(StoreError::EvidenceIntegrity {
                        source: source_id.to_string(),
                        reason: "stored file bytes do not match the recorded digest".to_string(),
                    });
                }
                Some(digest)
            }
            None => None,
        },
        None => None,
    };

    // Allocate an excerpt id not already used by this workspace.
    let taken: HashSet<String> = workspace
        .excerpts()
        .iter()
        .map(|e| e.id().0.clone())
        .collect();
    let mut attempts = 0u32;
    let excerpt_id = loop {
        attempts += 1;
        if attempts > 1000 {
            return Err(StoreError::Io(
                "could not allocate a unique excerpt id".to_string(),
            ));
        }
        let candidate = gen_id();
        if safe_file_stem(&candidate).is_err() || taken.contains(&candidate) {
            continue;
        }
        break candidate;
    };

    let excerpt = Excerpt::new_with_meta(
        excerpt_id,
        SourceId::new(source_id),
        Anchor {
            kind: anchor_kind.to_string(),
            value: anchor_value.to_string(),
        },
        quote,
        pin,
        note.map(|n| n.to_string()),
        Some(now()),
    )
    .map_err(|e| StoreError::Domain(e.to_string()))?;

    let updated = workspace
        .with_excerpt(excerpt)
        .map_err(|e| StoreError::Domain(e.to_string()))?;
    update(base, &updated)?;
    Ok(updated.view())
}

/// Per-process counter feeding generated, filesystem-safe citation ids.
static CITATION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a safe citation id. Never derived from user input; collisions with
/// an existing citation are handled by the caller.
fn new_citation_id() -> String {
    let n = CITATION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("cit-{}-{}", std::process::id(), n)
}

/// Add a manual Citation (#8 / citations-from-UI) linking a `claim` to an
/// existing Excerpt of a Pratica.
///
/// Contract:
/// - read-modify-write serialised **per matter** (no last-write-wins loss);
/// - the citation id is **generated server-side** (path-safe shape; never from
///   input) and regenerated on the rare collision with an existing citation;
/// - referential integrity (citation → excerpt exists; id unique) and an empty
///   `claim` are rejected by the core (mapped to `StoreError::Domain`); nothing
///   is persisted on failure;
/// - persisted via the atomic [`update`]. No filesystem/blob access: a Citation
///   references an `excerptId`, never a file.
pub fn add_citation(
    base: &Path,
    matter_id: &str,
    excerpt_id: &str,
    claim: &str,
) -> Result<WorkspaceView, StoreError> {
    add_citation_with(base, matter_id, excerpt_id, claim, &mut new_citation_id)
}

/// Core of [`add_citation`] with an injectable id generator (for tests that
/// force a collision).
fn add_citation_with(
    base: &Path,
    matter_id: &str,
    excerpt_id: &str,
    claim: &str,
    gen_id: &mut dyn FnMut() -> String,
) -> Result<WorkspaceView, StoreError> {
    let matter_stem = safe_file_stem(matter_id)?.to_string();

    let lock = matter_lock(base, &matter_stem);
    let _guard = lock.lock().expect("per-matter lock poisoned");

    let ws_path = workspace_path(base, matter_id)?;
    if !ws_path.exists() {
        return Err(StoreError::NotFound(matter_id.to_string()));
    }
    let workspace = load_consistent(&ws_path)?;

    // Allocate a citation id not already used by this workspace.
    let taken: HashSet<String> = workspace
        .citations()
        .iter()
        .map(|c| c.id().0.clone())
        .collect();
    let mut attempts = 0u32;
    let citation_id = loop {
        attempts += 1;
        if attempts > 1000 {
            return Err(StoreError::Io(
                "could not allocate a unique citation id".to_string(),
            ));
        }
        let candidate = gen_id();
        if safe_file_stem(&candidate).is_err() || taken.contains(&candidate) {
            continue;
        }
        break candidate;
    };

    let citation = Citation::new(citation_id, claim, ExcerptId::new(excerpt_id))
        .map_err(|e| StoreError::Domain(e.to_string()))?;

    let updated = workspace
        .with_citation(citation)
        .map_err(|e| StoreError::Domain(e.to_string()))?;
    update(base, &updated)?;
    Ok(updated.view())
}

/// Render an existing Pratica as a grounded Markdown report (#12 decomposition).
/// Read-only: loads the canonical workspace and delegates to the pure core
/// exporter. **No file is written** — the caller (frontend) downloads the string.
pub fn workspace_markdown(base: &Path, matter_id: &str) -> Result<String, StoreError> {
    let ws_path = workspace_path(base, matter_id)?;
    if !ws_path.exists() {
        return Err(StoreError::NotFound(matter_id.to_string()));
    }
    let workspace = load_consistent(&ws_path)?;
    Ok(quaero_core::export::workspace_to_markdown(&workspace))
}

/// Per-matter locked load → run `op` on the canonical workspace → atomic update,
/// returning the derived view. Shared by the edit/delete commands.
fn mutate_workspace(
    base: &Path,
    matter_id: &str,
    op: impl FnOnce(Workspace) -> Result<Workspace, StoreError>,
) -> Result<WorkspaceView, StoreError> {
    let matter_stem = safe_file_stem(matter_id)?.to_string();
    let lock = matter_lock(base, &matter_stem);
    let _guard = lock.lock().expect("per-matter lock poisoned");

    let ws_path = workspace_path(base, matter_id)?;
    if !ws_path.exists() {
        return Err(StoreError::NotFound(matter_id.to_string()));
    }
    let workspace = load_consistent(&ws_path)?;
    let updated = op(workspace)?;
    update(base, &updated)?;
    Ok(updated.view())
}

/// Edit an existing Estratto (#edit/delete): change quote + anchor + note. The
/// core's `edit_excerpt` preserves the immutable evidence (`sourceId`, the
/// `sourceSha256` pin and `createdAt`) — the store no longer has to remember to.
/// Unknown id / empty quote/anchor → `Domain`, nothing persisted.
#[allow(clippy::too_many_arguments)]
pub fn update_excerpt(
    base: &Path,
    matter_id: &str,
    excerpt_id: &str,
    anchor_kind: &str,
    anchor_value: &str,
    quote: &str,
    note: Option<&str>,
) -> Result<WorkspaceView, StoreError> {
    mutate_workspace(base, matter_id, |workspace| {
        workspace
            .edit_excerpt(
                &ExcerptId::new(excerpt_id),
                Anchor {
                    kind: anchor_kind.to_string(),
                    value: anchor_value.to_string(),
                },
                quote,
                note.map(|n| n.to_string()),
            )
            .map_err(|e| StoreError::Domain(e.to_string()))
    })
}

/// Delete an Estratto. Refused (`Domain`, nothing persisted) if it is still cited.
pub fn delete_excerpt(
    base: &Path,
    matter_id: &str,
    excerpt_id: &str,
) -> Result<WorkspaceView, StoreError> {
    mutate_workspace(base, matter_id, |workspace| {
        workspace
            .without_excerpt(&ExcerptId::new(excerpt_id))
            .map_err(|e| StoreError::Domain(e.to_string()))
    })
}

/// Edit an existing Citazione: change the `claim`, keeping the linked Estratto.
pub fn update_citation(
    base: &Path,
    matter_id: &str,
    citation_id: &str,
    claim: &str,
) -> Result<WorkspaceView, StoreError> {
    mutate_workspace(base, matter_id, |workspace| {
        workspace
            .edit_citation(&CitationId::new(citation_id), claim)
            .map_err(|e| StoreError::Domain(e.to_string()))
    })
}

/// Delete a Citazione (always safe — citations are leaves).
pub fn delete_citation(
    base: &Path,
    matter_id: &str,
    citation_id: &str,
) -> Result<WorkspaceView, StoreError> {
    mutate_workspace(base, matter_id, |workspace| {
        workspace
            .without_citation(&CitationId::new(citation_id))
            .map_err(|e| StoreError::Domain(e.to_string()))
    })
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
    fn failed_create_leaves_no_temp_file_behind() {
        let dir = tempdir().unwrap();
        // First create wins and writes dup.json.
        create(
            dir.path(),
            client("alfa", "Alfa"),
            matter("dup", "alfa", "T"),
        )
        .unwrap();
        // Second create for the same id fails at publish (AlreadyExists) AFTER a
        // unique temp file has been written — exercising the failure cleanup path.
        let again = create(
            dir.path(),
            client("alfa", "Alfa"),
            matter("dup", "alfa", "T"),
        );
        assert!(matches!(again, Err(StoreError::AlreadyExists(_))));

        let names: Vec<String> = fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            !names.iter().any(|n| n.ends_with(".tmp")),
            "failed create must leave no .tmp residue, found: {names:?}"
        );
        // Exactly one canonical file survives (the first create's).
        assert_eq!(names.iter().filter(|n| n.ends_with(".json")).count(), 1);
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

    // --- #6 document ingestion --------------------------------------------

    fn dirs(root: &Path) -> (PathBuf, PathBuf) {
        (root.join("workspaces"), root.join("files"))
    }

    #[test]
    fn import_writes_blob_registers_source_and_persists() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(
            &ws,
            client("alfa", "Alfa"),
            matter("rossi", "alfa", "Rossi"),
        )
        .unwrap();

        let view = import_document(&ws, &files, "rossi", "Contratto.PDF", b"hello pdf").unwrap();

        let docs: Vec<_> = view
            .sources
            .iter()
            .filter(|s| matches!(s.kind, SourceType::Documento))
            .collect();
        assert_eq!(docs.len(), 1);
        let f = docs[0].file.as_ref().unwrap();
        assert_eq!(f.original_name, "Contratto.PDF");
        assert_eq!(f.byte_len, 9);
        assert_eq!(f.sha256, sha256_hex(b"hello pdf"));
        assert!(f.stored_name.ends_with(".pdf")); // extension sanitised+lowercased

        // the bytes live on disk under files/rossi/<stored_name>
        let blob = files.join("rossi").join(&f.stored_name);
        assert!(blob.exists());
        assert_eq!(fs::read(&blob).unwrap(), b"hello pdf");

        // re-open shows the imported Documento
        let reopened = open(&ws, "rossi").unwrap();
        assert!(reopened
            .sources
            .iter()
            .any(|s| matches!(s.kind, SourceType::Documento)));
    }

    #[test]
    fn imported_workspace_json_has_metadata_not_bytes() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(
            &ws,
            client("alfa", "Alfa"),
            matter("rossi", "alfa", "Rossi"),
        )
        .unwrap();
        import_document(&ws, &files, "rossi", "Contratto.pdf", b"hello pdf").unwrap();

        let raw = fs::read_to_string(ws.join("rossi.json")).unwrap();
        assert!(raw.contains("sha256"));
        assert!(raw.contains("storedName"));
        assert!(raw.contains("byteLen"));
        // the file content (bytes) is NEVER embedded in the canonical JSON
        assert!(!raw.contains("hello pdf"));
    }

    #[test]
    fn hostile_original_name_cannot_escape_the_store() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(
            &ws,
            client("alfa", "Alfa"),
            matter("rossi", "alfa", "Rossi"),
        )
        .unwrap();

        let view = import_document(&ws, &files, "rossi", "../../etc/passwd", b"x").unwrap();
        let f = view.sources.iter().find_map(|s| s.file.as_ref()).unwrap();
        // stored name is generated: no separators, no traversal
        assert!(!f.stored_name.contains('/'));
        assert!(!f.stored_name.contains('\\'));
        assert!(!f.stored_name.contains(".."));
        // original name preserved as display metadata only
        assert_eq!(f.original_name, "../../etc/passwd");
        // exactly one blob, and it lives under files/rossi/
        let entries: Vec<_> = fs::read_dir(files.join("rossi"))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(entries.len(), 1);
        // nothing escaped above the files dir
        assert!(!tmp.path().join("etc").exists());
    }

    #[test]
    fn import_rejects_unsafe_matter_id() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        let r = import_document(&ws, &files, "../evil", "a.txt", b"x");
        assert!(matches!(r, Err(StoreError::UnsafeId(_))));
    }

    #[test]
    fn import_enforces_size_cap() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(
            &ws,
            client("alfa", "Alfa"),
            matter("rossi", "alfa", "Rossi"),
        )
        .unwrap();
        let big = vec![0u8; (MAX_IMPORT_BYTES + 1) as usize];
        let r = import_document(&ws, &files, "rossi", "big.bin", &big);
        assert!(matches!(r, Err(StoreError::TooLarge { .. })));
        // nothing was written
        assert!(!files.join("rossi").exists());
    }

    #[test]
    fn import_into_missing_matter_is_not_found() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        let r = import_document(&ws, &files, "ghost", "a.txt", b"x");
        assert!(matches!(r, Err(StoreError::NotFound(_))));
    }

    #[test]
    fn concurrent_imports_into_same_matter_keep_every_source() {
        use std::thread;

        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(
            &ws,
            client("alfa", "Alfa"),
            matter("rossi", "alfa", "Rossi"),
        )
        .unwrap();
        let ws = Arc::new(ws);
        let files = Arc::new(files);

        let handles: Vec<_> = (0..8)
            .map(|i| {
                let ws = Arc::clone(&ws);
                let files = Arc::clone(&files);
                thread::spawn(move || {
                    import_document(
                        ws.as_path(),
                        files.as_path(),
                        "rossi",
                        &format!("f{i}.txt"),
                        format!("content-{i}").as_bytes(),
                    )
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap().unwrap();
        }

        // every concurrent import survived the read-modify-write (no lost source)
        let view = open(ws.as_path(), "rossi").unwrap();
        let docs = view
            .sources
            .iter()
            .filter(|s| matches!(s.kind, SourceType::Documento))
            .count();
        assert_eq!(docs, 8);
    }

    /// Read a blob and assert the SourceRef metadata matches the physical bytes.
    fn assert_blob_integrity(files: &Path, matter: &str, f: &StoredFile, expected: &[u8]) {
        let blob = files.join(matter).join(&f.stored_name);
        let raw = fs::read(&blob).unwrap();
        assert_eq!(raw, expected);
        assert_eq!(f.byte_len as usize, raw.len());
        assert_eq!(f.sha256, sha256_hex(&raw));
    }

    #[test]
    fn id_collision_with_existing_source_regenerates_without_touching_old_blob() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(&ws, client("alfa", "Alfa"), matter("rossi", "alfa", "R")).unwrap();

        // first import pinned to a fixed id
        import_document_with(&ws, &files, "rossi", "a.txt", b"first", &mut || {
            "dup-id".to_string()
        })
        .unwrap();
        let old_blob = files.join("rossi").join("dup-id.txt");
        assert_eq!(fs::read(&old_blob).unwrap(), b"first");

        // second import: gen yields the colliding id first, then a free one
        let mut seq = vec!["fresh-id".to_string(), "dup-id".to_string()];
        let view = import_document_with(&ws, &files, "rossi", "b.txt", b"second", &mut move || {
            seq.pop().unwrap()
        })
        .unwrap();

        // the old blob is untouched; the new one used the regenerated id
        assert_eq!(fs::read(&old_blob).unwrap(), b"first");
        assert_eq!(
            fs::read(files.join("rossi").join("fresh-id.txt")).unwrap(),
            b"second"
        );

        // both sources persisted, each consistent with its physical bytes
        let docs: Vec<_> = view
            .sources
            .iter()
            .filter_map(|s| s.file.as_ref())
            .collect();
        assert_eq!(docs.len(), 2);
        let by_name = |n: &str| docs.iter().find(|f| f.stored_name == n).unwrap();
        assert_blob_integrity(&files, "rossi", by_name("dup-id.txt"), b"first");
        assert_blob_integrity(&files, "rossi", by_name("fresh-id.txt"), b"second");
    }

    #[test]
    fn blob_path_collision_with_orphan_regenerates_without_overwrite() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(&ws, client("alfa", "Alfa"), matter("rossi", "alfa", "R")).unwrap();

        // an orphan blob on disk, not referenced by the workspace
        fs::create_dir_all(files.join("rossi")).unwrap();
        let orphan = files.join("rossi").join("orphan-id.bin");
        fs::write(&orphan, b"orphan").unwrap();

        // gen yields the orphan-colliding id first (no dot in name → ext "bin"),
        // then a free id
        let mut seq = vec!["good-id".to_string(), "orphan-id".to_string()];
        let view = import_document_with(&ws, &files, "rossi", "data", b"new", &mut move || {
            seq.pop().unwrap()
        })
        .unwrap();

        // exclusive publish never overwrote the orphan
        assert_eq!(fs::read(&orphan).unwrap(), b"orphan");
        let f = view.sources.iter().find_map(|s| s.file.as_ref()).unwrap();
        assert_eq!(f.stored_name, "good-id.bin");
        assert_blob_integrity(&files, "rossi", f, b"new");
    }

    #[test]
    fn unresolvable_id_collision_fails_controlled_and_keeps_old_blob() {
        let tmp = tempdir().unwrap();
        let (ws, files) = dirs(tmp.path());
        create(&ws, client("alfa", "Alfa"), matter("rossi", "alfa", "R")).unwrap();

        import_document_with(&ws, &files, "rossi", "a.txt", b"first", &mut || {
            "stuck-id".to_string()
        })
        .unwrap();
        let blob = files.join("rossi").join("stuck-id.txt");

        // a generator that always collides → controlled error, never overwrite
        let r = import_document_with(&ws, &files, "rossi", "b.txt", b"second", &mut || {
            "stuck-id".to_string()
        });
        assert!(matches!(r, Err(StoreError::Io(_))));
        assert_eq!(fs::read(&blob).unwrap(), b"first");
        // the workspace still has exactly one imported source
        let n = open(&ws, "rossi")
            .unwrap()
            .sources
            .iter()
            .filter(|s| s.file.is_some())
            .count();
        assert_eq!(n, 1);
    }

    // ----- #8B: manual Excerpt capture -----

    #[test]
    fn rfc3339_formats_known_epochs() {
        assert_eq!(rfc3339_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339_from_unix(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn add_excerpt_persists_with_autopin_note_and_timestamp() {
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view =
            import_document(ws.path(), files.path(), "m", "contratto.pdf", b"hello pdf").unwrap();
        let sid = view.sources[0].id.0.clone();
        let sha = view.sources[0].file.as_ref().unwrap().sha256.clone();

        let after = add_excerpt(
            ws.path(),
            files.path(),
            "m",
            &sid,
            "clausola",
            "7.2",
            "il conduttore...",
            Some("rilevante"),
        )
        .unwrap();
        assert_eq!(after.excerpts.len(), 1);

        // Reload from disk → persistence after close/reopen.
        let reopened = open(ws.path(), "m").unwrap();
        assert_eq!(reopened.excerpts.len(), 1);
        let ex = &reopened.excerpts[0];
        assert_eq!(ex.quote(), "il conduttore...");
        assert_eq!(ex.source_id().0, sid);
        assert_eq!(ex.anchor().kind, "clausola");
        assert_eq!(ex.anchor().value, "7.2");
        assert_eq!(ex.note(), Some("rilevante"));
        assert_eq!(ex.source_sha256(), Some(sha.as_str())); // auto-pinned
        assert!(ex.created_at().is_some());
    }

    #[test]
    fn add_excerpt_to_missing_matter_is_not_found() {
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        let r = add_excerpt(ws.path(), files.path(), "nope", "s1", "k", "v", "q", None);
        assert!(matches!(r, Err(StoreError::NotFound(_))));
    }

    #[test]
    fn add_excerpt_to_unknown_source_is_rejected() {
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let r = add_excerpt(ws.path(), files.path(), "m", "ghost", "k", "v", "q", None);
        assert!(matches!(r, Err(StoreError::Domain(_))));
    }

    #[test]
    fn add_excerpt_empty_quote_is_rejected_and_not_persisted() {
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view = import_document(ws.path(), files.path(), "m", "c.pdf", b"x").unwrap();
        let sid = view.sources[0].id.0.clone();
        let r = add_excerpt(ws.path(), files.path(), "m", &sid, "k", "v", "   ", None);
        assert!(matches!(r, Err(StoreError::Domain(_))));
        assert!(open(ws.path(), "m").unwrap().excerpts.is_empty());
    }

    #[test]
    fn add_excerpt_regenerates_on_id_collision() {
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view = import_document(ws.path(), files.path(), "m", "c.pdf", b"x").unwrap();
        let sid = view.sources[0].id.0.clone();

        // gen yields: exc-1 (1st add), exc-1 again (collision on 2nd add), exc-2.
        let mut seq = vec![
            "exc-1".to_string(),
            "exc-1".to_string(),
            "exc-2".to_string(),
        ]
        .into_iter();
        let mut gen = move || seq.next().unwrap();
        let now = || "2026-06-01T10:00:00Z".to_string();

        add_excerpt_with(
            ws.path(),
            files.path(),
            "m",
            &sid,
            "k",
            "v",
            "q1",
            None,
            &mut gen,
            &now,
        )
        .unwrap();
        let after = add_excerpt_with(
            ws.path(),
            files.path(),
            "m",
            &sid,
            "k",
            "v",
            "q2",
            None,
            &mut gen,
            &now,
        )
        .unwrap();

        let ids: Vec<String> = after.excerpts.iter().map(|e| e.id().0.clone()).collect();
        assert_eq!(after.excerpts.len(), 2);
        assert!(ids.contains(&"exc-1".to_string()));
        assert!(ids.contains(&"exc-2".to_string()));
        assert_eq!(after.excerpts[0].created_at(), Some("2026-06-01T10:00:00Z"));
    }

    #[test]
    fn add_excerpt_rejects_a_tampered_blob_and_persists_nothing() {
        // Regression for the Codex BLOCKER: the auto-pin must verify the actual
        // stored bytes, not trust the recorded metadata.
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view =
            import_document(ws.path(), files.path(), "m", "contratto.pdf", b"hello pdf").unwrap();
        let sid = view.sources[0].id.0.clone();
        let stored_name = view.sources[0].file.as_ref().unwrap().stored_name.clone();

        // Tamper the blob on disk AFTER import (bytes no longer match metadata).
        let blob = files.path().join("m").join(&stored_name);
        fs::write(&blob, b"TAMPERED BYTES").unwrap();

        let r = add_excerpt(
            ws.path(),
            files.path(),
            "m",
            &sid,
            "clausola",
            "7.2",
            "q",
            None,
        );
        assert!(matches!(r, Err(StoreError::EvidenceIntegrity { .. })));
        // Nothing persisted — no stale pin slipped through.
        assert!(open(ws.path(), "m").unwrap().excerpts.is_empty());
    }

    #[test]
    fn add_excerpt_rejects_a_missing_blob() {
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view = import_document(ws.path(), files.path(), "m", "c.pdf", b"bytes").unwrap();
        let sid = view.sources[0].id.0.clone();
        let stored_name = view.sources[0].file.as_ref().unwrap().stored_name.clone();

        // Delete the blob; the recorded metadata is now dangling.
        fs::remove_file(files.path().join("m").join(&stored_name)).unwrap();

        let r = add_excerpt(ws.path(), files.path(), "m", &sid, "k", "v", "q", None);
        assert!(matches!(r, Err(StoreError::EvidenceIntegrity { .. })));
        assert!(open(ws.path(), "m").unwrap().excerpts.is_empty());
    }

    #[test]
    fn add_excerpt_rejects_blob_replaced_by_directory() {
        // A non-regular file (here a directory at the blob path) must be rejected
        // before any read — covers symlink/reparse/device/dir uniformly.
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view = import_document(ws.path(), files.path(), "m", "c.pdf", b"bytes").unwrap();
        let sid = view.sources[0].id.0.clone();
        let stored_name = view.sources[0].file.as_ref().unwrap().stored_name.clone();

        let blob = files.path().join("m").join(&stored_name);
        fs::remove_file(&blob).unwrap();
        fs::create_dir(&blob).unwrap(); // same name, but a directory now

        let r = add_excerpt(ws.path(), files.path(), "m", &sid, "k", "v", "q", None);
        assert!(matches!(r, Err(StoreError::EvidenceIntegrity { .. })));
        assert!(open(ws.path(), "m").unwrap().excerpts.is_empty());
    }

    #[test]
    fn add_excerpt_rejects_same_length_tamper_via_digest() {
        // Bytes changed but length preserved → caught by the post-read sha check.
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view = import_document(ws.path(), files.path(), "m", "c.pdf", b"hello pdf").unwrap();
        let sid = view.sources[0].id.0.clone();
        let stored_name = view.sources[0].file.as_ref().unwrap().stored_name.clone();

        // Overwrite with the SAME length (9 bytes) but different content.
        let blob = files.path().join("m").join(&stored_name);
        assert_eq!(b"hello pdf".len(), b"HELLO pdf".len());
        fs::write(&blob, b"HELLO pdf").unwrap();

        let r = add_excerpt(ws.path(), files.path(), "m", &sid, "k", "v", "q", None);
        assert!(matches!(r, Err(StoreError::EvidenceIntegrity { .. })));
        assert!(open(ws.path(), "m").unwrap().excerpts.is_empty());
    }

    #[test]
    fn load_consistent_rejects_inflated_byte_len() {
        let dir = tempdir().unwrap();
        let ws = Workspace::new_with_evidence(
            client("alfa", "Alfa"),
            matter("m", "alfa", "T"),
            vec![SourceRef {
                id: SourceId::new("s1"),
                kind: SourceType::Documento,
                title: "x".to_string(),
                meta: String::new(),
                file: Some(StoredFile {
                    stored_name: "doc-1-1.pdf".to_string(),
                    original_name: "x".to_string(),
                    byte_len: MAX_IMPORT_BYTES + 1, // inflated beyond the cap
                    sha256: "ab".repeat(32),
                }),
            }],
            vec![],
            vec![],
            vec![],
        )
        .unwrap();
        update(dir.path(), &ws).unwrap();
        assert!(matches!(
            open(dir.path(), "m"),
            Err(StoreError::Corrupt { .. })
        ));
    }

    #[test]
    fn add_excerpt_rejects_blob_larger_than_cap_before_reading() {
        let dir = tempdir().unwrap();
        let files = tempdir().unwrap();
        // Safe name + small byteLen → workspace loads fine…
        plant_workspace_with_stored_name(dir.path(), "doc-1-1.pdf");
        // …but the on-disk blob is sparse-huge (> cap); set_len avoids writing 25MB.
        fs::create_dir_all(files.path().join("m")).unwrap();
        let blob = files.path().join("m").join("doc-1-1.pdf");
        let f = fs::File::create(&blob).unwrap();
        f.set_len(MAX_IMPORT_BYTES + 1).unwrap();
        drop(f);

        let r = add_excerpt(dir.path(), files.path(), "m", "s1", "k", "v", "q", None);
        assert!(matches!(r, Err(StoreError::EvidenceIntegrity { .. })));
        assert!(open(dir.path(), "m").unwrap().excerpts.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn add_excerpt_rejects_symlink_blob() {
        use std::os::unix::fs::symlink;
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        let outside = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view = import_document(ws.path(), files.path(), "m", "c.pdf", b"hello pdf").unwrap();
        let sid = view.sources[0].id.0.clone();
        let stored_name = view.sources[0].file.as_ref().unwrap().stored_name.clone();

        // Replace the blob with a symlink to an outside file (same content/len).
        let target = outside.path().join("secret");
        fs::write(&target, b"hello pdf").unwrap();
        let blob = files.path().join("m").join(&stored_name);
        fs::remove_file(&blob).unwrap();
        symlink(&target, &blob).unwrap();

        // Even though the target content matches, a symlink is not a regular file.
        let r = add_excerpt(ws.path(), files.path(), "m", &sid, "k", "v", "q", None);
        assert!(matches!(r, Err(StoreError::EvidenceIntegrity { .. })));
        assert!(open(ws.path(), "m").unwrap().excerpts.is_empty());
    }

    /// Plant a workspace JSON whose Documento source carries a hostile
    /// `storedName` (bypassing the safe import path), to exercise the read-path
    /// hardening. The canonical model does not validate `storedName`, so this
    /// builds + persists directly.
    fn plant_workspace_with_stored_name(dir: &Path, stored_name: &str) {
        let ws = Workspace::new_with_evidence(
            client("alfa", "Alfa"),
            matter("m", "alfa", "T"),
            vec![SourceRef {
                id: SourceId::new("s1"),
                kind: SourceType::Documento,
                title: "x".to_string(),
                meta: String::new(),
                file: Some(StoredFile {
                    stored_name: stored_name.to_string(),
                    original_name: "x".to_string(),
                    byte_len: 3,
                    sha256: "ab".repeat(32),
                }),
            }],
            vec![],
            vec![],
            vec![],
        )
        .unwrap();
        update(dir, &ws).unwrap();
    }

    #[test]
    fn add_excerpt_rejects_traversal_stored_name_and_persists_nothing() {
        let dir = tempdir().unwrap();
        let files = tempdir().unwrap();
        plant_workspace_with_stored_name(dir.path(), "../../evil.bin");
        let before = fs::read_to_string(dir.path().join("m.json")).unwrap();

        let r = add_excerpt(
            dir.path(),
            files.path(),
            "m",
            "s1",
            "clausola",
            "7.2",
            "q",
            None,
        );
        assert!(r.is_err(), "a traversal storedName must be rejected");
        // Fail-closed: the JSON on disk is unchanged (no excerpt persisted).
        assert_eq!(
            fs::read_to_string(dir.path().join("m.json")).unwrap(),
            before
        );
    }

    #[test]
    fn add_excerpt_rejects_absolute_stored_name() {
        let dir = tempdir().unwrap();
        let files = tempdir().unwrap();
        plant_workspace_with_stored_name(dir.path(), "/etc/passwd");
        let r = add_excerpt(dir.path(), files.path(), "m", "s1", "k", "v", "q", None);
        assert!(r.is_err(), "an absolute storedName must be rejected");
    }

    #[test]
    fn load_consistent_rejects_unsafe_stored_name() {
        // A hostile/corrupt workspace is intercepted at load, for every reader.
        let dir = tempdir().unwrap();
        plant_workspace_with_stored_name(dir.path(), "..\\..\\evil.bin");
        assert!(matches!(
            open(dir.path(), "m"),
            Err(StoreError::Corrupt { .. })
        ));
    }

    #[test]
    fn is_safe_stored_name_accepts_generated_rejects_hostile() {
        // import-generated names (doc-<pid>-<n>.<ext>) must keep passing
        assert!(is_safe_stored_name("doc-123-0.pdf"));
        assert!(is_safe_stored_name("doc-123-1.pdf"));
        assert!(is_safe_stored_name(&format!(
            "doc-{}-7.docx",
            std::process::id()
        )));
        assert!(is_safe_stored_name("exc.bin"));
        for bad in [
            "",
            "..",
            "../x",
            "..\\x",
            "a/b",
            "a\\b",
            "/abs",
            "/etc/passwd",
            "with\0nul",
            // outside the ASCII allowlist
            "C:",
            "C:foo",
            "file.txt:stream",
            "nome con spazio.pdf", // space
            "CONIN$",              // '$' not allowed
            "CONOUT$",
            "COM\u{00B9}.txt", // superscript ¹ → non-ASCII
            "LPT\u{00B2}.doc", // superscript ²
            "café.pdf",        // non-ASCII letter
            // trailing dot
            "file.",
            // reserved DOS device names, with and without extension, any case
            "NUL",
            "CON",
            "con",
            "Aux",
            "PRN",
            "COM1",
            "COM1.txt",
            "lpt1",
            "LPT1.pdf",
            "NUL.docx",
        ] {
            assert!(!is_safe_stored_name(bad), "must reject {bad:?}");
        }
    }

    #[test]
    fn add_excerpt_rejects_reserved_device_stored_name() {
        let dir = tempdir().unwrap();
        let files = tempdir().unwrap();
        // A hostile JSON pointing the blob at a Windows console device.
        plant_workspace_with_stored_name(dir.path(), "CON");
        let before = fs::read_to_string(dir.path().join("m.json")).unwrap();
        let r = add_excerpt(dir.path(), files.path(), "m", "s1", "k", "v", "q", None);
        assert!(r.is_err(), "a reserved device storedName must be rejected");
        assert_eq!(
            fs::read_to_string(dir.path().join("m.json")).unwrap(),
            before
        );
        // and it is intercepted at load too
        assert!(matches!(
            open(dir.path(), "m"),
            Err(StoreError::Corrupt { .. })
        ));
    }

    // ----- citations-from-UI -----

    /// Seed a Pratica "m" with one Documento source + one Excerpt, returning the
    /// (ws dir, files dir, excerpt id) for citation tests.
    fn seed_with_excerpt() -> (tempfile::TempDir, tempfile::TempDir, String) {
        let ws = tempdir().unwrap();
        let files = tempdir().unwrap();
        create(ws.path(), client("alfa", "Alfa"), matter("m", "alfa", "T")).unwrap();
        let view = import_document(ws.path(), files.path(), "m", "c.pdf", b"hello pdf").unwrap();
        let sid = view.sources[0].id.0.clone();
        let view = add_excerpt(
            ws.path(),
            files.path(),
            "m",
            &sid,
            "clausola",
            "7.2",
            "Il conduttore è tenuto.",
            None,
        )
        .unwrap();
        let eid = view.excerpts[0].id().0.clone();
        (ws, files, eid)
    }

    #[test]
    fn add_citation_persists_and_reloads_linked_to_excerpt() {
        let (ws, _files, eid) = seed_with_excerpt();
        let after =
            add_citation(ws.path(), "m", &eid, "Recesso con preavviso di 15 giorni.").unwrap();
        assert_eq!(after.citations.len(), 1);

        let reopened = open(ws.path(), "m").unwrap();
        assert_eq!(reopened.citations.len(), 1);
        let c = &reopened.citations[0];
        assert_eq!(c.claim(), "Recesso con preavviso di 15 giorni.");
        assert_eq!(c.excerpt_id().0, eid);
    }

    #[test]
    fn add_citation_to_missing_matter_is_not_found() {
        let ws = tempdir().unwrap();
        let r = add_citation(ws.path(), "nope", "e1", "x");
        assert!(matches!(r, Err(StoreError::NotFound(_))));
    }

    #[test]
    fn add_citation_to_unknown_excerpt_is_rejected() {
        let (ws, _files, _eid) = seed_with_excerpt();
        let r = add_citation(ws.path(), "m", "ghost", "x");
        assert!(matches!(r, Err(StoreError::Domain(_))));
        // exactly one citation never appeared
        assert!(open(ws.path(), "m").unwrap().citations.is_empty());
    }

    #[test]
    fn add_citation_empty_claim_is_rejected_and_not_persisted() {
        let (ws, _files, eid) = seed_with_excerpt();
        let r = add_citation(ws.path(), "m", &eid, "   ");
        assert!(matches!(r, Err(StoreError::Domain(_))));
        assert!(open(ws.path(), "m").unwrap().citations.is_empty());
    }

    #[test]
    fn add_citation_regenerates_on_id_collision() {
        let (ws, _files, eid) = seed_with_excerpt();
        let mut seq = vec![
            "cit-1".to_string(),
            "cit-1".to_string(),
            "cit-2".to_string(),
        ]
        .into_iter();
        let mut gen = move || seq.next().unwrap();

        add_citation_with(ws.path(), "m", &eid, "prima", &mut gen).unwrap();
        let after = add_citation_with(ws.path(), "m", &eid, "seconda", &mut gen).unwrap();

        let ids: Vec<String> = after.citations.iter().map(|c| c.id().0.clone()).collect();
        assert_eq!(after.citations.len(), 2);
        assert!(ids.contains(&"cit-1".to_string()));
        assert!(ids.contains(&"cit-2".to_string()));
    }

    // ----- export grounded Markdown -----

    #[test]
    fn workspace_markdown_renders_the_chain() {
        let (ws, _files, eid) = seed_with_excerpt();
        add_citation(ws.path(), "m", &eid, "Recesso con preavviso.").unwrap();
        let md = workspace_markdown(ws.path(), "m").unwrap();
        assert!(md.contains("# Quaero — Report Evidence"));
        assert!(md.contains("**Cliente:** Alfa"));
        assert!(md.contains("## Verifica della catena"));
        assert!(md.contains("### «Recesso con preavviso.»"));
        assert!(md.contains("> Il conduttore è tenuto."));
        assert!(md.contains("## Fonti"));
    }

    #[test]
    fn workspace_markdown_missing_matter_is_not_found() {
        let ws = tempdir().unwrap();
        assert!(matches!(
            workspace_markdown(ws.path(), "nope"),
            Err(StoreError::NotFound(_))
        ));
    }

    #[test]
    fn load_consistent_rejects_invalid_sha256() {
        let dir = tempdir().unwrap();
        let ws = Workspace::new_with_evidence(
            client("alfa", "Alfa"),
            matter("m", "alfa", "T"),
            vec![SourceRef {
                id: SourceId::new("s1"),
                kind: SourceType::Documento,
                title: "x".to_string(),
                meta: String::new(),
                file: Some(StoredFile {
                    stored_name: "doc-1-1.pdf".to_string(),
                    original_name: "x".to_string(),
                    byte_len: 3,
                    sha256: "![](https:a)".to_string(), // free text, not 64 hex
                }),
            }],
            vec![],
            vec![],
            vec![],
        )
        .unwrap();
        update(dir.path(), &ws).unwrap();
        assert!(matches!(
            open(dir.path(), "m"),
            Err(StoreError::Corrupt { .. })
        ));
    }

    // ----- edit/delete Estratti e Citazioni -----

    #[test]
    fn update_excerpt_persists_preserving_pin_and_created_at() {
        let (ws, _files, eid) = seed_with_excerpt();
        let before = open(ws.path(), "m").unwrap().excerpts[0].clone();
        let pin = before.source_sha256().map(|s| s.to_string());
        let created = before.created_at().map(|s| s.to_string());

        update_excerpt(
            ws.path(),
            "m",
            &eid,
            "pagina",
            "12",
            "testo corretto",
            Some("nota nuova"),
        )
        .unwrap();

        let e = open(ws.path(), "m").unwrap().excerpts[0].clone();
        assert_eq!(e.quote(), "testo corretto");
        assert_eq!(e.anchor().kind, "pagina");
        assert_eq!(e.anchor().value, "12");
        assert_eq!(e.note(), Some("nota nuova"));
        // invariants preserved
        assert_eq!(e.source_sha256().map(|s| s.to_string()), pin);
        assert_eq!(e.created_at().map(|s| s.to_string()), created);
        assert_eq!(e.source_id().0, before.source_id().0);
    }

    #[test]
    fn delete_excerpt_is_blocked_while_cited_then_succeeds() {
        let (ws, _files, eid) = seed_with_excerpt();
        let view = add_citation(ws.path(), "m", &eid, "Affermazione.").unwrap();
        let cid = view.citations[0].id().0.clone();

        // cited → blocked, nothing persisted
        assert!(matches!(
            delete_excerpt(ws.path(), "m", &eid),
            Err(StoreError::Domain(_))
        ));
        assert_eq!(open(ws.path(), "m").unwrap().excerpts.len(), 1);

        // delete the citation first, then the excerpt
        delete_citation(ws.path(), "m", &cid).unwrap();
        delete_excerpt(ws.path(), "m", &eid).unwrap();
        let v = open(ws.path(), "m").unwrap();
        assert!(v.excerpts.is_empty());
        assert!(v.citations.is_empty());
    }

    #[test]
    fn update_citation_persists_new_claim() {
        let (ws, _files, eid) = seed_with_excerpt();
        let view = add_citation(ws.path(), "m", &eid, "Vecchia.").unwrap();
        let cid = view.citations[0].id().0.clone();

        update_citation(ws.path(), "m", &cid, "Nuova affermazione.").unwrap();
        let v = open(ws.path(), "m").unwrap();
        assert_eq!(v.citations[0].claim(), "Nuova affermazione.");
        assert_eq!(v.citations[0].excerpt_id().0, eid); // link unchanged
    }

    #[test]
    fn edit_delete_unknown_id_and_missing_matter_error() {
        let (ws, _files, _eid) = seed_with_excerpt();
        assert!(matches!(
            update_excerpt(ws.path(), "m", "ghost", "k", "v", "q", None),
            Err(StoreError::Domain(_))
        ));
        assert!(matches!(
            delete_citation(ws.path(), "m", "ghost"),
            Err(StoreError::Domain(_))
        ));
        let empty = tempdir().unwrap();
        assert!(matches!(
            delete_excerpt(empty.path(), "nope", "e1"),
            Err(StoreError::NotFound(_))
        ));
    }
}
