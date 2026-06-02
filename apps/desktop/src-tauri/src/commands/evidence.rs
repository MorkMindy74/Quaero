//! IPC commands for the AI Evidence Assistant (#55 V1A, #58 V1B). Thin mappings
//! onto the pure provider (`quaero_core::evidence`), the local Ollama provider,
//! and the store. V1A: offline Stub (no egress). V1B: local Ollama behind
//! explicit consent + Privacy Guard. Candidates are NEVER persisted here — the
//! lawyer approves a candidate, which becomes a real Estratto through
//! [`accept_evidence_candidate`] (quote verified against the text layer, #52).

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use quaero_core::citation_candidates::{
    CitationCandidateProvider, CitationRequest, ExcerptInput, StubCitationProvider,
    DEFAULT_MAX_CITATION_CANDIDATES,
};
use quaero_core::domain::WorkspaceView;
use quaero_core::evidence::{
    quote_occurs_in_text, EvidenceCandidate, EvidenceCandidateProvider, EvidenceRequest,
    StubEvidenceProvider, DEFAULT_MAX_CANDIDATES,
};
use quaero_core::hash::sha256_hex;
use serde::Serialize;
use tauri::AppHandle;

use crate::commands::workspace::{files_dir, workspaces_dir};
use crate::evidence_consent::ConsentStore;
use crate::evidence_ollama::OllamaEvidenceProvider;
use crate::store::{self, SourceTextStatus};

/// Process-global store of outstanding one-shot consent tokens (#58). In memory
/// only — never persisted.
fn consent_store() -> &'static Mutex<ConsentStore> {
    static STORE: OnceLock<Mutex<ConsentStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(ConsentStore::new()))
}

static CONSENT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique consent token. It is not a secret against a compromised
/// renderer (which could read it); it provides one-shot / replay / cross-source
/// protection. Bound to the source digest so a token can't be reused elsewhere.
fn new_consent_token(sha256: &str) -> String {
    let n = CONSENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    sha256_hex(format!("{}-{}-{}-{}", std::process::id(), n, nanos, sha256).as_bytes())
}

/// Resolve a Documento source's pinned sha256 from the canonical workspace.
fn source_sha256(app: &AppHandle, matter_id: &str, source_id: &str) -> Result<String, String> {
    let ws_dir = workspaces_dir(app)?;
    let view = store::open(&ws_dir, matter_id).map_err(|e| e.to_string())?;
    view.sources
        .iter()
        .find(|s| s.id.0 == source_id)
        .and_then(|s| s.file.as_ref())
        .map(|f| f.sha256.clone())
        .ok_or_else(|| "Fonte senza documento".to_string())
}

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

/// IPC: issue a one-shot consent token (#58) after the lawyer confirms the UI
/// dialog. The token is bound to `(matterId, sourceId, sha256)` and short-lived;
/// it must be consumed by `propose_evidence_local`. Requires the opt-in. No text
/// is read or sent here.
#[tauri::command]
pub fn request_evidence_consent(
    app: AppHandle,
    matter_id: String,
    source_id: String,
) -> Result<String, String> {
    if !evidence_ollama_enabled() {
        return Err("provider Evidence locale non abilitato".to_string());
    }
    let sha = source_sha256(&app, &matter_id, &source_id)?;
    let token = new_consent_token(&sha);
    consent_store()
        .lock()
        .expect("consent store poisoned")
        .issue(&token, &matter_id, &source_id, &sha, SystemTime::now());
    Ok(token)
}

/// IPC: propose Evidence candidates via the LOCAL Ollama model (#58, V1B).
///
/// Requires BOTH a valid one-shot `consent_token` (issued by
/// `request_evidence_consent` after the UI dialog, bound to this Fonte's digest)
/// AND the opt-in env (`QUAERO_EVIDENCE_PROVIDER=ollama`) — otherwise it refuses
/// without contacting any model. The token is consumed (one-shot) BEFORE any
/// send. The document text layer is sent only to a loopback model, through the
/// Privacy Guard (`ClientConfidential → LocalModel`). Candidates are NOT
/// persisted; each is scored against the text layer (`valid`).
#[tauri::command]
pub async fn propose_evidence_local(
    app: AppHandle,
    matter_id: String,
    source_id: String,
    consent_token: String,
) -> Result<LocalEvidenceResult, String> {
    if !evidence_ollama_enabled() {
        return Err("provider Evidence locale non abilitato".to_string());
    }
    // Consume the one-shot, source-bound consent token under the lock, BEFORE any
    // network await (the guard is dropped before the send).
    let sha = source_sha256(&app, &matter_id, &source_id)?;
    consent_store()
        .lock()
        .expect("consent store poisoned")
        .consume(
            &consent_token,
            &matter_id,
            &source_id,
            &sha,
            SystemTime::now(),
        )
        .map_err(|e| e.to_string())?;

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

/// A citation candidate scored against the workspace: `valid` = its `excerptId`
/// references an existing real Estratto. Invalid candidates are shown but not
/// approvable (a Citazione must always cite a real Estratto, ADR-0007).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoredCitationCandidate {
    pub excerpt_id: String,
    pub claim: String,
    pub reason: String,
    pub valid: bool,
}

/// IPC: propose candidate Citazioni (#60, V1C) for the real Estratti of a Pratica
/// that do NOT already have a Citazione. Offline Stub only — no LLM, no network,
/// no auto-save. Candidates are NOT persisted; each carries the `excerptId` of a
/// real Estratto (`valid`). Approval goes through the canonical `add_citation`.
#[tauri::command]
pub fn propose_citations(
    app: AppHandle,
    matter_id: String,
) -> Result<Vec<ScoredCitationCandidate>, String> {
    let ws_dir = workspaces_dir(&app)?;
    let view = store::open(&ws_dir, &matter_id).map_err(|e| e.to_string())?;

    // Real excerpt ids, and those that already have at least one citation.
    let real: HashSet<&str> = view.excerpts.iter().map(|e| e.id().0.as_str()).collect();
    let cited: HashSet<&str> = view
        .citations
        .iter()
        .map(|c| c.excerpt_id().0.as_str())
        .collect();

    // Propose only for real excerpts without a citation yet.
    let inputs: Vec<ExcerptInput> = view
        .excerpts
        .iter()
        .filter(|e| !cited.contains(e.id().0.as_str()))
        .map(|e| ExcerptInput {
            excerpt_id: e.id().0.clone(),
            quote: e.quote().to_string(),
            anchor_kind: e.anchor().kind.clone(),
            anchor_value: e.anchor().value.clone(),
        })
        .collect();
    if inputs.is_empty() {
        return Ok(Vec::new());
    }

    let request = CitationRequest {
        excerpts: inputs,
        max_candidates: DEFAULT_MAX_CITATION_CANDIDATES,
    };
    let out = StubCitationProvider
        .propose(&request)
        .map_err(|e| e.to_string())?;

    let scored = out
        .candidates
        .into_iter()
        .map(|c| ScoredCitationCandidate {
            valid: real.contains(c.excerpt_id.as_str()),
            excerpt_id: c.excerpt_id,
            claim: c.claim,
            reason: c.reason,
        })
        .collect();
    Ok(scored)
}
