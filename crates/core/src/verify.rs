//! Citation Verifier (#9). Pure and Tauri-free (ADR-0011).
//!
//! #8 already makes structurally-invalid evidence **unrepresentable** (refs
//! resolve, ids unique, `sourceSha256` matches the file digest, no empty
//! quote/anchor/claim, "cite an Estratto not a Fonte" by type). So this module
//! does NOT re-check structural validity — a workspace that reaches `verify`
//! is already valid. Instead it audits **quality, coverage and explainability**
//! of the Estratto→Citazione chain and produces a human-readable report.
//!
//! Pure: no I/O, no filesystem, no byte access, no network. The report is a
//! DERIVED view (carried in `WorkspaceView`), never persisted.

use crate::domain::{CitationId, ExcerptId, SourceId, Workspace};
use serde::Serialize;
use std::collections::HashSet;

/// Finding severity. There is no `Error`: structural errors are rejected at
/// construction/load by #8 and cannot reach the verifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Info,
    Warning,
}

/// What kind of audit finding this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VerificationCode {
    /// An Estratto that no Citazione references.
    OrphanExcerpt,
    /// An Estratto whose Fonte has a stored file but no `sourceSha256` pin.
    UnpinnedDocumentExcerpt,
    /// A Fonte that has no Estratti (normal for not-yet-used material → Info).
    UncitedSource,
}

/// A single audit finding, with typed references to the items involved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub severity: Severity,
    pub code: VerificationCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt_id: Option<ExcerptId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<SourceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_id: Option<CitationId>,
}

/// Positive attestation: counts that let the UI state what was verified, not
/// only what is wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationSummary {
    pub citations: usize,
    pub excerpts: usize,
    pub document_backed_excerpts: usize,
    pub pinned_excerpts: usize,
    pub warnings: usize,
    pub infos: usize,
}

/// The full audit report over a (valid) workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub summary: VerificationSummary,
    pub findings: Vec<Finding>,
}

/// Audit the citation chain of a (valid) workspace. Pure and deterministic:
/// findings are emitted in a stable order (excerpt order for orphan/unpinned,
/// then source order for uncited). `UncitedSource` is `Info` and never degrades
/// the verdict (the UI treats `warnings == 0` as "coherent").
pub fn verify(workspace: &Workspace) -> VerificationReport {
    let sources = workspace.sources();
    let excerpts = workspace.excerpts();
    let citations = workspace.citations();

    // Excerpt ids referenced by at least one citation.
    let cited: HashSet<&ExcerptId> = citations.iter().map(|c| c.excerpt_id()).collect();
    // Source ids that back at least one excerpt.
    let sources_with_excerpt: HashSet<&SourceId> = excerpts.iter().map(|e| e.source_id()).collect();
    // Does this source have stored file bytes? (drives the "document-backed" rule)
    let has_file = |sid: &SourceId| sources.iter().any(|s| &s.id == sid && s.file.is_some());

    let mut findings: Vec<Finding> = Vec::new();
    let mut document_backed_excerpts = 0usize;
    let mut pinned_excerpts = 0usize;

    for excerpt in excerpts {
        let backed = has_file(excerpt.source_id());
        if backed {
            document_backed_excerpts += 1;
        }
        if excerpt.source_sha256().is_some() {
            pinned_excerpts += 1;
        }

        if !cited.contains(excerpt.id()) {
            findings.push(Finding {
                severity: Severity::Warning,
                code: VerificationCode::OrphanExcerpt,
                excerpt_id: Some(excerpt.id().clone()),
                source_id: None,
                citation_id: None,
            });
        }

        // Only Documento Fonti that actually carry stored bytes can be pinned;
        // a missing pin there is a real verifiability gap (#9 (a)).
        if backed && excerpt.source_sha256().is_none() {
            findings.push(Finding {
                severity: Severity::Warning,
                code: VerificationCode::UnpinnedDocumentExcerpt,
                excerpt_id: Some(excerpt.id().clone()),
                source_id: Some(excerpt.source_id().clone()),
                citation_id: None,
            });
        }
    }

    for source in sources {
        if !sources_with_excerpt.contains(&source.id) {
            findings.push(Finding {
                severity: Severity::Info,
                code: VerificationCode::UncitedSource,
                excerpt_id: None,
                source_id: Some(source.id.clone()),
                citation_id: None,
            });
        }
    }

    let warnings = findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    let infos = findings
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .count();

    VerificationReport {
        summary: VerificationSummary {
            citations: citations.len(),
            excerpts: excerpts.len(),
            document_backed_excerpts,
            pinned_excerpts,
            warnings,
            infos,
        },
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        sample_workspace, Anchor, Citation, Client, ClientId, Excerpt, ExcerptId, Matter, MatterId,
        SourceId, SourceRef, SourceType, StoredFile, Workspace,
    };

    fn client() -> Client {
        Client {
            id: ClientId::new("alfa"),
            name: "Alfa".to_string(),
        }
    }
    fn matter() -> Matter {
        Matter {
            id: MatterId::new("m"),
            client: ClientId::new("alfa"),
            title: "t".to_string(),
            subject: "s".to_string(),
        }
    }
    fn anchor() -> Anchor {
        Anchor {
            kind: "clausola".to_string(),
            value: "7.2".to_string(),
        }
    }
    fn ref_source(id: &str, kind: SourceType) -> SourceRef {
        SourceRef {
            id: SourceId::new(id),
            kind,
            title: id.to_string(),
            meta: String::new(),
            file: None,
        }
    }
    fn doc_source_with_file(id: &str, sha: &str) -> SourceRef {
        SourceRef {
            id: SourceId::new(id),
            kind: SourceType::Documento,
            title: "C.pdf".to_string(),
            meta: String::new(),
            file: Some(StoredFile {
                stored_name: format!("{id}.pdf"),
                original_name: "C.pdf".to_string(),
                byte_len: 3,
                sha256: sha.to_string(),
            }),
        }
    }

    #[test]
    fn sample_workspace_is_coherent_with_three_uncited_infos() {
        let report = verify(&sample_workspace());
        assert_eq!(
            report.summary,
            VerificationSummary {
                citations: 1,
                excerpts: 1,
                document_backed_excerpts: 0,
                pinned_excerpts: 0,
                warnings: 0,
                infos: 3,
            }
        );
        assert_eq!(report.findings.len(), 3);
        assert!(report
            .findings
            .iter()
            .all(|f| f.severity == Severity::Info && f.code == VerificationCode::UncitedSource));
        let sids: Vec<&str> = report
            .findings
            .iter()
            .filter_map(|f| f.source_id.as_ref().map(|s| s.0.as_str()))
            .collect();
        assert_eq!(sids, vec!["s2", "s3", "s4"]); // source order, s1 has the excerpt
    }

    #[test]
    fn orphan_excerpt_is_a_warning() {
        let ws = Workspace::new_with_evidence(
            client(),
            matter(),
            vec![ref_source("s1", SourceType::Documento)],
            vec![],
            vec![Excerpt::new("e1", SourceId::new("s1"), anchor(), "q", None).unwrap()],
            vec![], // no citation → e1 orphan
        )
        .unwrap();
        let report = verify(&ws);
        assert_eq!(report.summary.warnings, 1);
        let f = &report.findings[0];
        assert_eq!(f.severity, Severity::Warning);
        assert_eq!(f.code, VerificationCode::OrphanExcerpt);
        assert_eq!(f.excerpt_id.as_ref().unwrap().0, "e1");
    }

    #[test]
    fn uncited_source_is_info_and_does_not_count_as_warning() {
        let ws = Workspace::new_with_evidence(
            client(),
            matter(),
            vec![
                ref_source("s1", SourceType::Documento),
                ref_source("s2", SourceType::Norma),
            ],
            vec![],
            vec![Excerpt::new("e1", SourceId::new("s1"), anchor(), "q", None).unwrap()],
            vec![Citation::new("c1", "x", ExcerptId::new("e1")).unwrap()],
        )
        .unwrap();
        let report = verify(&ws);
        assert_eq!(report.summary.warnings, 0); // coherent
        assert_eq!(report.summary.infos, 1); // s2 uncited
        assert_eq!(report.findings[0].code, VerificationCode::UncitedSource);
        assert_eq!(report.findings[0].source_id.as_ref().unwrap().0, "s2");
    }

    #[test]
    fn unpinned_document_excerpt_warns_only_when_the_source_has_a_file() {
        let sha = "ab".repeat(32);
        // document source WITH a stored file, excerpt WITHOUT a pin → warning
        let unpinned = Workspace::new_with_evidence(
            client(),
            matter(),
            vec![doc_source_with_file("s1", &sha)],
            vec![],
            vec![Excerpt::new("e1", SourceId::new("s1"), anchor(), "q", None).unwrap()],
            vec![Citation::new("c1", "x", ExcerptId::new("e1")).unwrap()],
        )
        .unwrap();
        let r = verify(&unpinned);
        assert_eq!(r.summary.document_backed_excerpts, 1);
        assert_eq!(r.summary.pinned_excerpts, 0);
        assert_eq!(r.summary.warnings, 1);
        assert_eq!(
            r.findings[0].code,
            VerificationCode::UnpinnedDocumentExcerpt
        );

        // same source, excerpt WITH the matching pin → no warning, pinned counted
        let pinned = Workspace::new_with_evidence(
            client(),
            matter(),
            vec![doc_source_with_file("s1", &sha)],
            vec![],
            vec![
                Excerpt::new("e1", SourceId::new("s1"), anchor(), "q", Some(sha.clone())).unwrap(),
            ],
            vec![Citation::new("c1", "x", ExcerptId::new("e1")).unwrap()],
        )
        .unwrap();
        let r = verify(&pinned);
        assert_eq!(r.summary.document_backed_excerpts, 1);
        assert_eq!(r.summary.pinned_excerpts, 1);
        assert_eq!(r.summary.warnings, 0);
    }

    #[test]
    fn verify_is_deterministic() {
        let ws = sample_workspace();
        assert_eq!(verify(&ws), verify(&ws));
    }

    #[test]
    fn report_serializes_camelcase() {
        let json = serde_json::to_string(&verify(&sample_workspace())).unwrap();
        assert!(json.contains("documentBackedExcerpts"));
        assert!(json.contains("pinnedExcerpts"));
        assert!(json.contains("\"UncitedSource\""));
        assert!(json.contains("sourceId"));
    }
}
