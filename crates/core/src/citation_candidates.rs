//! Citation-candidate contract (#60, AI Evidence Assistant V1C). Pure and
//! Tauri-free (ADR-0011).
//!
//! A [`CitationCandidateProvider`] turns approved real Estratti into *proposed*
//! Citazioni — short claims, each tied to an existing Excerpt — that the lawyer
//! reviews and approves. The only V1C implementation is [`StubCitationProvider`]:
//! deterministic, **offline**, no network, no LLM, no state.
//!
//! Anti-hallucination is **structural** here (not text-matching): a candidate
//! always carries an `excerpt_id` taken from the provided real Estratti, so
//! ADR-0007 ("cite an Estratto, never a Fonte") holds — there is no free-floating
//! claim. The claim itself is a derived assertion the lawyer must approve; the
//! chain claim → Estratto → `quote ∈ text` is already guaranteed by how the
//! Estratto was created. Candidates are never persisted until approved, and the
//! real Citazione is created only through the canonical `add_citation` path.

use serde::{Deserialize, Serialize};

/// Default number of citation candidates a provider is asked to propose.
pub const DEFAULT_MAX_CITATION_CANDIDATES: usize = 20;

/// One approved Estratto offered to the provider as input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExcerptInput {
    pub excerpt_id: String,
    pub quote: String,
    pub anchor_kind: String,
    pub anchor_value: String,
}

/// A request to propose Citazioni from real Estratti.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CitationRequest {
    pub excerpts: Vec<ExcerptInput>,
    pub max_candidates: usize,
}

/// A single proposed Citazione. NOT persisted: the lawyer approves/edits/discards.
/// Always carries the `excerpt_id` of an existing Estratto (never a free claim).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CitationCandidate {
    pub excerpt_id: String,
    pub claim: String,
    pub reason: String,
}

/// The set of proposed candidates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationCandidates {
    pub candidates: Vec<CitationCandidate>,
}

/// Why a proposal could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationCandidateError {
    /// No Estratti were provided to cite.
    NoExcerpts,
}

impl std::fmt::Display for CitationCandidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CitationCandidateError::NoExcerpts => write!(f, "no excerpts to cite"),
        }
    }
}

impl std::error::Error for CitationCandidateError {}

/// Proposes Citazioni from real Estratti. Pure: no I/O, no network, no secrets.
pub trait CitationCandidateProvider {
    fn propose(
        &self,
        request: &CitationRequest,
    ) -> Result<CitationCandidates, CitationCandidateError>;
}

/// Deterministic, offline stub. Same input → same output. No network, no state,
/// no LLM. For each provided Estratto it emits ONE candidate whose `excerpt_id`
/// is that Estratto's id (so every candidate is structurally tied to a real
/// Estratto) and a clearly-exploratory placeholder claim — purely to exercise the
/// review/approval pipeline end to end.
#[derive(Debug, Default, Clone, Copy)]
pub struct StubCitationProvider;

impl CitationCandidateProvider for StubCitationProvider {
    fn propose(
        &self,
        request: &CitationRequest,
    ) -> Result<CitationCandidates, CitationCandidateError> {
        if request.excerpts.is_empty() {
            return Err(CitationCandidateError::NoExcerpts);
        }
        let candidates = request
            .excerpts
            .iter()
            .take(request.max_candidates)
            .map(|e| CitationCandidate {
                excerpt_id: e.excerpt_id.clone(),
                claim: format!(
                    "Affermazione proposta da verificare, basata sull'Estratto in {} {}.",
                    e.anchor_kind, e.anchor_value
                ),
                reason: "Proposta automatica esplorativa, non verificata.".to_string(),
            })
            .collect();
        Ok(CitationCandidates { candidates })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ex(id: &str, kind: &str, value: &str) -> ExcerptInput {
        ExcerptInput {
            excerpt_id: id.to_string(),
            quote: "Il conduttore è tenuto.".to_string(),
            anchor_kind: kind.to_string(),
            anchor_value: value.to_string(),
        }
    }

    fn req(excerpts: Vec<ExcerptInput>, max: usize) -> CitationRequest {
        CitationRequest {
            excerpts,
            max_candidates: max,
        }
    }

    #[test]
    fn stub_is_deterministic() {
        let r = req(
            vec![ex("e1", "pagina", "8"), ex("e2", "clausola", "7.2")],
            20,
        );
        assert_eq!(
            StubCitationProvider.propose(&r).unwrap(),
            StubCitationProvider.propose(&r).unwrap()
        );
    }

    #[test]
    fn every_candidate_is_tied_to_a_provided_excerpt() {
        let r = req(
            vec![ex("e1", "pagina", "8"), ex("e2", "clausola", "7.2")],
            20,
        );
        let out = StubCitationProvider.propose(&r).unwrap();
        assert_eq!(out.candidates.len(), 2);
        let ids: Vec<&str> = out
            .candidates
            .iter()
            .map(|c| c.excerpt_id.as_str())
            .collect();
        assert_eq!(ids, vec!["e1", "e2"]);
        // Every candidate carries a non-empty claim (a real Citazione needs one).
        for c in &out.candidates {
            assert!(!c.claim.trim().is_empty());
        }
    }

    #[test]
    fn stub_rejects_empty_input() {
        assert_eq!(
            StubCitationProvider.propose(&req(vec![], 20)),
            Err(CitationCandidateError::NoExcerpts)
        );
    }

    #[test]
    fn stub_respects_max_candidates() {
        let r = req(
            vec![ex("e1", "p", "1"), ex("e2", "p", "2"), ex("e3", "p", "3")],
            2,
        );
        assert_eq!(
            StubCitationProvider.propose(&r).unwrap().candidates.len(),
            2
        );
    }

    #[test]
    fn candidate_survives_json_round_trip_camel_case() {
        let c = CitationCandidate {
            excerpt_id: "e1".into(),
            claim: "x".into(),
            reason: "r".into(),
        };
        let encoded = serde_json::to_string(&c).unwrap();
        assert!(encoded.contains("\"excerptId\""));
        let decoded: CitationCandidate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(c, decoded);
    }
}
