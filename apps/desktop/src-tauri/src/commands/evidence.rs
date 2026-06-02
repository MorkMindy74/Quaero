//! IPC commands for the AI Evidence Assistant (#55, V1A). Thin mappings onto the
//! pure provider (`quaero_core::evidence`) and the store. No egress: V1A uses the
//! offline `StubEvidenceProvider` only. Candidates are NEVER persisted here — the
//! lawyer approves a candidate, which then becomes a real Estratto through
//! [`accept_evidence_candidate`] (quote verified against the text layer, #52).

use quaero_core::domain::WorkspaceView;
use quaero_core::evidence::{
    EvidenceCandidate, EvidenceCandidateProvider, EvidenceRequest, StubEvidenceProvider,
    DEFAULT_MAX_CANDIDATES,
};
use tauri::AppHandle;

use crate::commands::workspace::{files_dir, workspaces_dir};
use crate::store::{self, SourceTextStatus};

/// IPC: propose Evidence candidates for a Documento source from its local text
/// layer (#52). Reads the text layer, runs the **offline** Stub provider, and
/// returns candidates that are **not persisted**. If the source has no available
/// text layer, returns an empty list (the UI prompts to extract the text first).
/// No LLM, no network, no auto-save.
#[tauri::command]
pub fn propose_evidence(
    app: AppHandle,
    matter_id: String,
    source_id: String,
) -> Result<Vec<EvidenceCandidate>, String> {
    let ws_dir = workspaces_dir(&app)?;
    let blob_dir = files_dir(&app)?;
    let layer = store::get_source_text(&ws_dir, &blob_dir, &matter_id, &source_id)
        .map_err(|e| e.to_string())?;
    let text = match layer.status {
        SourceTextStatus::Available => layer.text.unwrap_or_default(),
        // Empty / Absent → nothing to propose (UI guides the lawyer to extract).
        _ => return Ok(Vec::new()),
    };
    let request = EvidenceRequest {
        text,
        max_candidates: DEFAULT_MAX_CANDIDATES,
    };
    let out = StubEvidenceProvider
        .propose(&request)
        .map_err(|e| e.to_string())?;
    Ok(out.candidates)
}

/// IPC: turn an APPROVED candidate into a real Estratto (#55). The store verifies,
/// under the per-matter lock, that the `quote` occurs in the source's current text
/// layer (anti-hallucination); otherwise it refuses with no write. Returns the
/// updated view so the UI refreshes the Estratti list.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn accept_evidence_candidate(
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
    store::accept_evidence_candidate(
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
