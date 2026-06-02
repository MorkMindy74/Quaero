//! Evidence-candidate contract (#55, AI Evidence Assistant V1A). Pure and
//! Tauri-free (ADR-0011).
//!
//! An [`EvidenceCandidateProvider`] turns an [`EvidenceRequest`] (the document's
//! local text layer, #52/ADR-0012) into [`EvidenceCandidates`] — *proposed*
//! Estratti the lawyer reviews and approves. The only V1A implementation is
//! [`StubEvidenceProvider`]: deterministic, **offline**, no network, no LLM, no
//! state. A real local-model provider (Ollama) comes later (V1B), behind explicit
//! consent + the Privacy Guard.
//!
//! Anti-hallucination (ADR-0007): a candidate is only meaningful if its `quote`
//! is actually present in the source text. [`quote_occurs_in_text`] is the pure
//! check used both to flag candidates in the UI and — crucially — to **enforce**,
//! server-side under lock, that a real Estratto can never be created from text
//! that is not in the Fonte. Candidates are never persisted until approved.

use serde::{Deserialize, Serialize};

/// Maximum accepted text-layer length, in characters, for a proposal request.
pub const MAX_EVIDENCE_TEXT_CHARS: usize = 200_000;

/// Default number of candidates a provider is asked to propose.
pub const DEFAULT_MAX_CANDIDATES: usize = 8;

/// A request to propose Evidence candidates from a document's text layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceRequest {
    /// The full local text layer of the Fonte (already extracted, #52).
    pub text: String,
    /// Upper bound on how many candidates to return.
    pub max_candidates: usize,
}

/// A single proposed Estratto. NOT persisted: the lawyer approves/edits/discards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceCandidate {
    /// The proposed verbatim quote (must be verifiable in the text layer).
    pub quote: String,
    /// Declarative anchor kind (e.g. "paragrafo").
    pub anchor_kind: String,
    /// Declarative anchor value (e.g. "3").
    pub anchor_value: String,
    /// Short human-readable reason the passage may be relevant. Never a citation.
    pub reason: String,
}

/// The set of proposed candidates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceCandidates {
    pub candidates: Vec<EvidenceCandidate>,
}

/// Why a proposal could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    /// The text layer was empty (after trimming).
    EmptyText,
    /// The text layer exceeded [`MAX_EVIDENCE_TEXT_CHARS`].
    TextTooLong { limit: usize, actual: usize },
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvidenceError::EmptyText => write!(f, "empty text layer"),
            EvidenceError::TextTooLong { limit, actual } => {
                write!(f, "text layer too long: {actual} chars (limit {limit})")
            }
        }
    }
}

impl std::error::Error for EvidenceError {}

/// Proposes Evidence candidates from a text layer. Pure: no I/O, no network, no
/// secrets. A real local-model provider implements this in the desktop crate.
pub trait EvidenceCandidateProvider {
    fn propose(&self, request: &EvidenceRequest) -> Result<EvidenceCandidates, EvidenceError>;
}

/// Collapse every run of whitespace to a single space and trim — so a quote that
/// differs only in spacing/newlines (typical of PDF extraction) still matches.
/// Content (letters, accents, case) is compared exactly: stricter is safer for
/// the anti-hallucination guarantee (no false "present").
fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// True iff `quote` occurs in `text` under whitespace normalization. An empty
/// (or whitespace-only) quote never "occurs" — it could not anchor anything.
pub fn quote_occurs_in_text(text: &str, quote: &str) -> bool {
    let needle = normalize_ws(quote);
    if needle.is_empty() {
        return false;
    }
    normalize_ws(text).contains(&needle)
}

/// Deterministic, offline stub. Same input → same output. No network, no state,
/// no secrets, no LLM. It selects the first non-empty lines of the text layer as
/// verbatim candidates (so each candidate is, by construction, present in the
/// text), purely to exercise the review/approval pipeline end to end.
#[derive(Debug, Default, Clone, Copy)]
pub struct StubEvidenceProvider;

impl EvidenceCandidateProvider for StubEvidenceProvider {
    fn propose(&self, request: &EvidenceRequest) -> Result<EvidenceCandidates, EvidenceError> {
        if request.text.trim().is_empty() {
            return Err(EvidenceError::EmptyText);
        }
        let len = request.text.chars().count();
        if len > MAX_EVIDENCE_TEXT_CHARS {
            return Err(EvidenceError::TextTooLong {
                limit: MAX_EVIDENCE_TEXT_CHARS,
                actual: len,
            });
        }
        let candidates = request
            .text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(request.max_candidates)
            .enumerate()
            .map(|(i, line)| EvidenceCandidate {
                quote: line.to_string(),
                anchor_kind: "paragrafo".to_string(),
                anchor_value: (i + 1).to_string(),
                reason:
                    "Passaggio selezionato automaticamente (proposta esplorativa, non verificata)."
                        .to_string(),
            })
            .collect();
        Ok(EvidenceCandidates { candidates })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(text: &str, max: usize) -> EvidenceRequest {
        EvidenceRequest {
            text: text.to_string(),
            max_candidates: max,
        }
    }

    #[test]
    fn quote_present_verbatim_is_found() {
        let text = "Articolo 1. Il conduttore è tenuto.\nArticolo 2. Recesso.";
        assert!(quote_occurs_in_text(text, "Il conduttore è tenuto."));
    }

    #[test]
    fn quote_matches_under_whitespace_normalization() {
        // PDF-style extraction often varies spacing/newlines.
        let text = "Articolo 1.\nIl   conduttore\nè tenuto.";
        assert!(quote_occurs_in_text(text, "Il conduttore è tenuto."));
    }

    #[test]
    fn quote_absent_is_not_found() {
        let text = "Articolo 1. Il conduttore è tenuto.";
        assert!(!quote_occurs_in_text(
            text,
            "Il locatore rinuncia a ogni pretesa."
        ));
    }

    #[test]
    fn empty_or_whitespace_quote_never_occurs() {
        assert!(!quote_occurs_in_text("qualcosa", ""));
        assert!(!quote_occurs_in_text("qualcosa", "   \n\t"));
    }

    #[test]
    fn content_difference_is_not_matched_case_and_accents_exact() {
        let text = "Il conduttore è tenuto.";
        assert!(!quote_occurs_in_text(text, "il conduttore e tenuto.")); // case + accent
    }

    #[test]
    fn stub_is_deterministic() {
        let r = req("Riga uno.\nRiga due.\nRiga tre.", 8);
        let a = StubEvidenceProvider.propose(&r).unwrap();
        let b = StubEvidenceProvider.propose(&r).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn stub_proposes_verbatim_lines_that_occur_in_text() {
        let text = "  Prima riga.  \n\n  Seconda riga.  \nTerza riga.";
        let out = StubEvidenceProvider.propose(&req(text, 2)).unwrap();
        assert_eq!(out.candidates.len(), 2);
        assert_eq!(out.candidates[0].quote, "Prima riga.");
        assert_eq!(out.candidates[0].anchor_kind, "paragrafo");
        assert_eq!(out.candidates[0].anchor_value, "1");
        assert_eq!(out.candidates[1].quote, "Seconda riga.");
        assert_eq!(out.candidates[1].anchor_value, "2");
        // Every proposed quote must be verifiable in the source text.
        for c in &out.candidates {
            assert!(quote_occurs_in_text(text, &c.quote));
        }
    }

    #[test]
    fn stub_rejects_empty_text() {
        assert_eq!(
            StubEvidenceProvider.propose(&req("   \n\t", 8)),
            Err(EvidenceError::EmptyText)
        );
    }

    #[test]
    fn stub_rejects_text_over_the_cap() {
        let big = "a\n".repeat(MAX_EVIDENCE_TEXT_CHARS); // 2 chars per line → over cap
        match StubEvidenceProvider.propose(&req(&big, 8)) {
            Err(EvidenceError::TextTooLong { limit, actual }) => {
                assert_eq!(limit, MAX_EVIDENCE_TEXT_CHARS);
                assert!(actual > MAX_EVIDENCE_TEXT_CHARS);
            }
            other => panic!("expected TextTooLong, got {other:?}"),
        }
    }

    #[test]
    fn candidate_survives_json_round_trip_camel_case() {
        let c = EvidenceCandidate {
            quote: "x".into(),
            anchor_kind: "paragrafo".into(),
            anchor_value: "1".into(),
            reason: "r".into(),
        };
        let encoded = serde_json::to_string(&c).unwrap();
        assert!(encoded.contains("\"anchorKind\""));
        assert!(encoded.contains("\"anchorValue\""));
        let decoded: EvidenceCandidate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(c, decoded);
    }
}
