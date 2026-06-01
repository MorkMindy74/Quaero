//! Grounded Markdown export of a Pratica (decomposition of #12 — Markdown only).
//!
//! Pure: `Workspace -> String`. No I/O, no LLM, no network, no new dependency.
//! It renders ONLY what the user authored — Citazioni, Estratti and Fonti — plus
//! the derived #9 verification summary. Nothing is generated or inferred, so the
//! document stays **grounded/verifiable**. Client text is escaped minimally and
//! quotes go into blockquotes; no HTML, no Markdown tables.

use std::collections::HashSet;

use crate::domain::{Excerpt, ExcerptId, SourceId, SourceRef, SourceType, Workspace};
use crate::verify::verify;

/// Italian label for a Fonte type (display only).
fn source_type_label(kind: &SourceType) -> &'static str {
    match kind {
        SourceType::Documento => "Documento",
        SourceType::Norma => "Norma",
        SourceType::Giurisprudenza => "Giurisprudenza",
        SourceType::Dottrina => "Dottrina",
        SourceType::Prassi => "Prassi",
        SourceType::Dato => "Dato",
        SourceType::Nota => "Nota",
        SourceType::Memoria => "Memoria",
        SourceType::FonteEsterna => "Fonte esterna",
    }
}

/// Minimal inline escape: collapse newlines/tabs to spaces and neutralise
/// backslash and backtick so client text cannot open a code span or break the
/// line. Deliberately conservative — readable, not typographically perfect.
fn inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '\n' | '\r' | '\t' => out.push(' '),
            c => out.push(c),
        }
    }
    out.trim().to_string()
}

/// Render `s` as a Markdown blockquote: each physical line prefixed with `> `,
/// with the same minimal escaping applied per line. Always ends with a blank line.
fn blockquote(s: &str) -> String {
    let mut out = String::new();
    for line in s.split('\n') {
        let escaped: String = line
            .chars()
            .map(|c| match c {
                '\\' => "\\\\".to_string(),
                '`' => "\\`".to_string(),
                '\r' | '\t' => " ".to_string(),
                c => c.to_string(),
            })
            .collect();
        out.push_str("> ");
        out.push_str(escaped.trim_end());
        out.push('\n');
    }
    out.push('\n');
    out
}

/// First 12 hex chars of a sha256 (+ ellipsis), for compact display.
fn short_sha(sha: &str) -> String {
    if sha.len() > 12 {
        format!("{}…", &sha[..12])
    } else {
        sha.to_string()
    }
}

fn find_excerpt<'a>(ws: &'a Workspace, id: &ExcerptId) -> Option<&'a Excerpt> {
    ws.excerpts().iter().find(|e| e.id() == id)
}

fn find_source<'a>(ws: &'a Workspace, id: &SourceId) -> Option<&'a SourceRef> {
    ws.sources().iter().find(|s| &s.id == id)
}

/// A "Title (Tipo) · sha256 abc…" line for a Fonte, given the excerpt's pin.
fn source_line(source: Option<&SourceRef>, pin: Option<&str>) -> String {
    match source {
        Some(s) => {
            let sha = pin
                .or(s.file.as_ref().map(|f| f.sha256.as_str()))
                .map(|h| format!(" · sha256 {}", short_sha(h)))
                .unwrap_or_default();
            format!(
                "{} ({}){}",
                inline(&s.title),
                source_type_label(&s.kind),
                sha
            )
        }
        None => "(fonte mancante)".to_string(),
    }
}

/// Render the Pratica as a grounded Markdown report. Sections: header, Verifica
/// della catena (#9 verdict + counts), Citazioni (Affermazione → Estratto →
/// Fonte), Estratti non citati, Fonti.
pub fn workspace_to_markdown(ws: &Workspace) -> String {
    let mut out = String::new();

    // Header
    out.push_str("# Quaero — Report Evidence\n\n");
    out.push_str(&format!("**Cliente:** {}\n\n", inline(&ws.client().name)));
    out.push_str(&format!("**Pratica:** {}\n\n", inline(&ws.matter().title)));
    let subject = ws.matter().subject.trim();
    if !subject.is_empty() {
        out.push_str(&format!("**Materia:** {}\n\n", inline(subject)));
    }

    // Verifica della catena (#9): verdict + counts only (no per-finding list).
    let report = verify(ws);
    let s = &report.summary;
    let verdict = if s.warnings == 0 {
        "Catena coerente".to_string()
    } else {
        format!("Catena con {} avvisi", s.warnings)
    };
    out.push_str("## Verifica della catena\n\n");
    out.push_str(&format!("**Esito:** {verdict}\n\n"));
    out.push_str(&format!(
        "- Citazioni: {} · Estratti: {} (document-backed: {}, pinnati: {})\n",
        s.citations, s.excerpts, s.document_backed_excerpts, s.pinned_excerpts
    ));
    out.push_str(&format!("- Avvisi: {} · Info: {}\n\n", s.warnings, s.infos));

    // Citazioni: Affermazione → quote (blockquote) → Ancora → Fonte.
    out.push_str("## Citazioni\n\n");
    if ws.citations().is_empty() {
        out.push_str("_Nessuna Citazione._\n\n");
    } else {
        for c in ws.citations() {
            out.push_str(&format!("### «{}»\n\n", inline(c.claim())));
            match find_excerpt(ws, c.excerpt_id()) {
                Some(ex) => {
                    out.push_str(&blockquote(ex.quote()));
                    out.push_str(&format!(
                        "- **Estratto:** {} {}\n",
                        inline(&ex.anchor().kind),
                        inline(&ex.anchor().value)
                    ));
                    out.push_str(&format!(
                        "- **Fonte:** {}\n\n",
                        source_line(find_source(ws, ex.source_id()), ex.source_sha256())
                    ));
                }
                // Referential integrity guarantees this never happens; render
                // defensively rather than panic.
                None => out.push_str("_(estratto mancante)_\n\n"),
            }
        }
    }

    // Estratti non citati: authored Estratti that no Citazione references.
    let cited: HashSet<&ExcerptId> = ws.citations().iter().map(|c| c.excerpt_id()).collect();
    let orphans: Vec<&Excerpt> = ws
        .excerpts()
        .iter()
        .filter(|e| !cited.contains(e.id()))
        .collect();
    out.push_str("## Estratti non citati\n\n");
    if orphans.is_empty() {
        out.push_str("_Nessuno._\n\n");
    } else {
        for ex in orphans {
            out.push_str(&blockquote(ex.quote()));
            out.push_str(&format!(
                "- {} {} · Fonte: {}\n\n",
                inline(&ex.anchor().kind),
                inline(&ex.anchor().value),
                source_line(find_source(ws, ex.source_id()), ex.source_sha256())
            ));
        }
    }

    // Fonti
    out.push_str("## Fonti\n\n");
    if ws.sources().is_empty() {
        out.push_str("_Nessuna Fonte._\n\n");
    } else {
        for sr in ws.sources() {
            let sha = sr
                .file
                .as_ref()
                .map(|f| format!(" · sha256 {}", short_sha(&f.sha256)))
                .unwrap_or_default();
            out.push_str(&format!(
                "- {} ({}){}\n",
                inline(&sr.title),
                source_type_label(&sr.kind),
                sha
            ));
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Anchor, Citation, Client, ClientId, Matter, MatterId, SourceId, StoredFile, Workspace,
    };

    fn client() -> Client {
        Client {
            id: ClientId::new("alfa"),
            name: "Alfa S.r.l.".to_string(),
        }
    }
    fn matter() -> Matter {
        Matter {
            id: MatterId::new("m"),
            client: ClientId::new("alfa"),
            title: "Rossi c. Bianchi".to_string(),
            subject: "locazione".to_string(),
        }
    }
    fn doc(id: &str, title: &str, sha: &str) -> SourceRef {
        SourceRef {
            id: SourceId::new(id),
            kind: SourceType::Documento,
            title: title.to_string(),
            meta: String::new(),
            file: Some(StoredFile {
                stored_name: format!("{id}.pdf"),
                original_name: title.to_string(),
                byte_len: 3,
                sha256: sha.to_string(),
            }),
        }
    }
    fn anchor(kind: &str, value: &str) -> Anchor {
        Anchor {
            kind: kind.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn renders_header_verification_and_chain() {
        let sha = "ab".repeat(32);
        let ex = Excerpt::new(
            "e1",
            SourceId::new("s1"),
            anchor("clausola", "7.2"),
            "Il conduttore può recedere.",
            Some(sha.clone()),
        )
        .unwrap();
        let cit = Citation::new("c1", "Recesso con preavviso.", ExcerptId::new("e1")).unwrap();
        let ws = Workspace::new_with_evidence(
            client(),
            matter(),
            vec![doc("s1", "Contratto.pdf", &sha)],
            vec![],
            vec![ex],
            vec![cit],
        )
        .unwrap();

        let md = workspace_to_markdown(&ws);
        assert!(md.contains("# Quaero — Report Evidence"));
        assert!(md.contains("**Cliente:** Alfa S.r.l."));
        assert!(md.contains("**Pratica:** Rossi c. Bianchi"));
        assert!(md.contains("**Materia:** locazione"));
        assert!(md.contains("## Verifica della catena"));
        assert!(md.contains("Catena coerente"));
        assert!(md.contains("Citazioni: 1 · Estratti: 1"));
        // chain: claim, quote (blockquote), anchor, source
        assert!(md.contains("### «Recesso con preavviso.»"));
        assert!(md.contains("> Il conduttore può recedere."));
        assert!(md.contains("**Estratto:** clausola 7.2"));
        assert!(md.contains("Contratto.pdf (Documento) · sha256 abababababab…"));
    }

    #[test]
    fn lists_uncited_excerpts_and_sources() {
        let ex = Excerpt::new(
            "e1",
            SourceId::new("s1"),
            anchor("pagina", "3"),
            "frammento",
            None,
        )
        .unwrap();
        let ws = Workspace::new_with_evidence(
            client(),
            matter(),
            vec![doc("s1", "Atto.pdf", &"cd".repeat(32))],
            vec![],
            vec![ex],
            vec![], // no citations → the excerpt is uncited
        )
        .unwrap();
        let md = workspace_to_markdown(&ws);
        assert!(md.contains("## Estratti non citati"));
        assert!(md.contains("> frammento"));
        assert!(md.contains("## Fonti"));
        assert!(md.contains("Atto.pdf (Documento)"));
        // no citations section content
        assert!(md.contains("_Nessuna Citazione._"));
    }

    #[test]
    fn empty_workspace_still_renders_valid_sections() {
        let ws = Workspace::new(client(), matter(), vec![], vec![]).unwrap();
        let md = workspace_to_markdown(&ws);
        assert!(md.contains("**Cliente:** Alfa S.r.l."));
        assert!(md.contains("_Nessuna Citazione._"));
        assert!(md.contains("## Estratti non citati\n\n_Nessuno._"));
        assert!(md.contains("## Fonti\n\n_Nessuna Fonte._"));
    }

    #[test]
    fn client_text_is_escaped_no_code_span_or_line_break() {
        // A hostile claim/quote with backticks and newlines must not break out.
        let ex = Excerpt::new(
            "e1",
            SourceId::new("s1"),
            anchor("k", "v"),
            "riga1\nriga2 con `code`",
            None,
        )
        .unwrap();
        let cit = Citation::new("c1", "claim con `tick` e\nnewline", ExcerptId::new("e1")).unwrap();
        let ws = Workspace::new_with_evidence(
            client(),
            matter(),
            vec![doc("s1", "F.pdf", &"ab".repeat(32))],
            vec![],
            vec![ex],
            vec![cit],
        )
        .unwrap();
        let md = workspace_to_markdown(&ws);
        // backticks neutralised (escaped) — no raw code span survives
        assert!(!md.contains("`code`"));
        assert!(!md.contains("`tick`"));
        assert!(md.contains("\\`code\\`"));
        // the claim heading is single-line (newline collapsed to space)
        assert!(md.contains("### «claim con \\`tick\\` e newline»"));
        // the multi-line quote becomes two blockquote lines
        assert!(md.contains("> riga1"));
        assert!(md.contains("> riga2 con \\`code\\`"));
    }
}
