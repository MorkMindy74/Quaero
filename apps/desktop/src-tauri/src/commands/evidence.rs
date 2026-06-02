//! IPC commands for the AI Evidence Assistant (#55 V1A, #58 V1B). Thin mappings
//! onto the pure provider (`quaero_core::evidence`), the local Ollama provider,
//! and the store. V1A: offline Stub (no egress). V1B: local Ollama behind
//! explicit consent + Privacy Guard. Candidates are NEVER persisted here — the
//! lawyer approves a candidate, which becomes a real Estratto through
//! [`accept_evidence_candidate`] (quote verified against the text layer, #52).

use quaero_core::domain::WorkspaceView;
use quaero_core::evidence::{
    quote_occurs_in_text, EvidenceCandidate, EvidenceCandidateProvider, EvidenceRequest,
    StubEvidenceProvider, DEFAULT_MAX_CANDIDATES,
};
use serde::Serialize;
use tauri::AppHandle;

use crate::commands::workspace::{files_dir, workspaces_dir};
use crate::evidence_ollama::OllamaEvidenceProvider;
use crate::store::{self, SourceTextStatus};

/// Opt-in for the LOCAL Ollama Evidence provider — deliberately separate from the
/// chat opt-in (`QUAERO_CHAT_PROVIDER`): chat and Evidence have different data,
/// prompts and risk. Default is the offline Stub.
fn evidence_ollama_enabled() -> bool {
    std::env::var("QUAERO_EVIDENCE_PROVIDER")
        .map(|v| v.eq_ignore_ascii_case("ollama"))
        .unwrap_or(false)
}

/// IPC: which Evidence provider is active. Returns "ollamaLocal" ONLY when the
/// opt-in is set AND the endpoint is genuinely loopback; otherwise "stub". A
/// config flag only — carries no client data.
#[tauri::command]
pub fn evidence_provider_kind() -> String {
    if evidence_ollama_enabled() && OllamaEvidenceProvider::from_env().endpoint_is_local() {
        "ollamaLocal".to_string()
    } else {
        "stub".to_string()
    }
}

/// A candidate scored against the text layer: `valid` = the quote occurs in it.
/// Invalid candidates are shown but not approvable (the model is not trusted).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoredCandidate {
    pub quote: String,
    pub anchor_kind: String,
    pub anchor_value: String,
    pub reason: String,
    pub valid: bool,
}

/// Result of a local-model proposal, with the explicit truncation notice.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalEvidenceResult {
    pub candidates: Vec<ScoredCandidate>,
    pub truncated: bool,
    pub analyzed_chars: usize,
}

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

/// IPC: propose Evidence candidates via the LOCAL Ollama model (#58, V1B).
///
/// Requires BOTH explicit user consent (the `consent` flag, set by the UI after
/// the confirmation dialog) AND the opt-in env (`QUAERO_EVIDENCE_PROVIDER=ollama`)
/// — otherwise it refuses without contacting any model. The document text layer
/// is sent only to a loopback model, through the Privacy Guard
/// (`ClientConfidential → LocalModel`). Candidates are NOT persisted; each is
/// scored against the text layer (`valid`), and only valid ones are approvable.
#[tauri::command]
pub async fn propose_evidence_local(
    app: AppHandle,
    matter_id: String,
    source_id: String,
    consent: bool,
) -> Result<LocalEvidenceResult, String> {
    if !consent {
        return Err("consenso richiesto prima di inviare il testo al modello locale".to_string());
    }
    if !evidence_ollama_enabled() {
        return Err("provider Evidence locale non abilitato".to_string());
    }
    let ws_dir = workspaces_dir(&app)?;
    let blob_dir = files_dir(&app)?;
    let layer = store::get_source_text(&ws_dir, &blob_dir, &matter_id, &source_id)
        .map_err(|e| e.to_string())?;
    let text = match layer.status {
        SourceTextStatus::Available => layer.text.unwrap_or_default(),
        // No text layer → nothing to analyze (UI guides to extract first).
        _ => {
            return Ok(LocalEvidenceResult {
                candidates: Vec::new(),
                truncated: false,
                analyzed_chars: 0,
            })
        }
    };
    let proposal = OllamaEvidenceProvider::from_env().propose(&text).await?;
    let candidates = proposal
        .candidates
        .into_iter()
        .map(|c| ScoredCandidate {
            valid: quote_occurs_in_text(&text, &c.quote),
            quote: c.quote,
            anchor_kind: c.anchor_kind,
            anchor_value: c.anchor_value,
            reason: c.reason,
        })
        .collect();
    Ok(LocalEvidenceResult {
        candidates,
        truncated: proposal.truncated,
        analyzed_chars: proposal.analyzed_chars,
    })
}
