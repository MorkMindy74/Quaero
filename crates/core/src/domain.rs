//! Quaero domain model (Cliente → Pratica → Fascicolo/vista → Fonte).
//!
//! Pure and Tauri-free (ADR-0011). A `DossierView` is our **Fascicolo as a
//! VIEW** over sources — not a physical folder (ADR-0008): a source may appear
//! in many dossiers (many-to-many). `SourceRef` is a **minimal citable
//! reference**, NOT yet a full Fonte with Estratto/Ancora (those come later).

use serde::{Deserialize, Serialize};

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
        }
    };
}

id_newtype!(ClientId);
id_newtype!(MatterId);
id_newtype!(SourceId);

/// The nine kinds of Fonte (ADR-0007).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    Documento,
    Norma,
    Giurisprudenza,
    Dottrina,
    Prassi,
    Dato,
    Nota,
    Memoria,
    FonteEsterna,
}

impl SourceType {
    /// Stable order — drives deterministic dynamic-dossier generation.
    pub fn all() -> [SourceType; 9] {
        use SourceType::*;
        [
            Documento,
            Norma,
            Giurisprudenza,
            Dottrina,
            Prassi,
            Dato,
            Nota,
            Memoria,
            FonteEsterna,
        ]
    }

    /// Display name of the dynamic Fascicolo that groups this type.
    pub fn dossier_name(self) -> &'static str {
        use SourceType::*;
        match self {
            Documento => "Documenti",
            Norma => "Norme",
            Giurisprudenza => "Giurisprudenza",
            Dottrina => "Dottrina",
            Prassi => "Prassi",
            Dato => "Dati",
            Nota => "Note",
            Memoria => "Memoria",
            FonteEsterna => "Fonti esterne",
        }
    }

    /// Stable slug for deterministic dynamic-dossier ids.
    pub fn slug(self) -> &'static str {
        use SourceType::*;
        match self {
            Documento => "documento",
            Norma => "norma",
            Giurisprudenza => "giurisprudenza",
            Dottrina => "dottrina",
            Prassi => "prassi",
            Dato => "dato",
            Nota => "nota",
            Memoria => "memoria",
            FonteEsterna => "fonte-esterna",
        }
    }
}

/// A subject (Cliente) for whom matters are handled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
}

/// A single professional engagement (Pratica) belonging to a Client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matter {
    pub id: MatterId,
    pub client: ClientId,
    pub title: String,
    pub subject: String,
}

/// A minimal citable reference (Fonte). Not yet an Estratto/Ancora.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub id: SourceId,
    pub kind: SourceType,
    pub title: String,
    pub meta: String,
}

/// Whether a Fascicolo/view is generated from source types or curated by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DossierKind {
    Dynamic,
    Manual,
}

/// A "Fascicolo" as a VIEW over sources (ADR-0008). A source may appear in many
/// dossiers (many-to-many); the dossier does not own/duplicate the source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DossierView {
    pub id: String,
    pub name: String,
    pub kind: DossierKind,
    pub sources: Vec<SourceId>,
}

impl DossierView {
    pub fn manual(id: impl Into<String>, name: impl Into<String>, sources: Vec<SourceId>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: DossierKind::Manual,
            sources,
        }
    }
}

/// Generate one **Dynamic** Fascicolo per `SourceType` actually present, in the
/// stable order of [`SourceType::all`]. Types with no sources produce nothing.
pub fn dynamic_dossiers(sources: &[SourceRef]) -> Vec<DossierView> {
    SourceType::all()
        .into_iter()
        .filter_map(|kind| {
            let ids: Vec<SourceId> = sources
                .iter()
                .filter(|s| s.kind == kind)
                .map(|s| s.id.clone())
                .collect();
            if ids.is_empty() {
                None
            } else {
                Some(DossierView {
                    id: format!("dyn-{}", kind.slug()),
                    name: kind.dossier_name().to_string(),
                    kind: DossierKind::Dynamic,
                    sources: ids,
                })
            }
        })
        .collect()
}

/// All dossiers that contain a given source — demonstrates the many-to-many
/// relation: the same Fonte can be viewed from several Fascicoli.
pub fn dossiers_for_source<'a>(
    source: &SourceId,
    dossiers: &'a [DossierView],
) -> Vec<&'a DossierView> {
    dossiers
        .iter()
        .filter(|d| d.sources.contains(source))
        .collect()
}

/// A whole matter context: client, matter, its sources, and its dossiers
/// (dynamic by type + manual). Pure, presentational seed for the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub client: Client,
    pub matter: Matter,
    pub sources: Vec<SourceRef>,
    pub dossiers: Vec<DossierView>,
}

/// Deterministic sample workspace (fixed ids, no random, no current date).
pub fn sample_workspace() -> Workspace {
    let client = Client {
        id: ClientId::new("alfa"),
        name: "Alfa S.r.l.".to_string(),
    };
    let matter = Matter {
        id: MatterId::new("rossi-bianchi"),
        client: client.id.clone(),
        title: "Rossi c. Bianchi".to_string(),
        subject: "Inadempimento contrattuale".to_string(),
    };
    let sources = vec![
        SourceRef {
            id: SourceId::new("s1"),
            kind: SourceType::Documento,
            title: "Contratto Rossi-Bianchi.pdf".to_string(),
            meta: "pag. 10–14".to_string(),
        },
        SourceRef {
            id: SourceId::new("s2"),
            kind: SourceType::Norma,
            title: "Art. 1453 c.c.".to_string(),
            meta: "Risoluzione per inadempimento".to_string(),
        },
        SourceRef {
            id: SourceId::new("s3"),
            kind: SourceType::Giurisprudenza,
            title: "Cass. civ. 12345/2024".to_string(),
            meta: "massima".to_string(),
        },
        SourceRef {
            id: SourceId::new("s4"),
            kind: SourceType::Nota,
            title: "Cliente disponibile a transigere".to_string(),
            meta: String::new(),
        },
    ];

    let mut dossiers = dynamic_dossiers(&sources);
    // A manual Fascicolo curated by the user — sources here ALSO remain in their
    // dynamic dossiers (many-to-many, no duplication).
    dossiers.push(DossierView::manual(
        "man-produzione-avversaria",
        "Produzione avversaria",
        vec![SourceId::new("s1"), SourceId::new("s3")],
    ));

    Workspace {
        client,
        matter,
        sources,
        dossiers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(id: &str, kind: SourceType) -> SourceRef {
        SourceRef {
            id: SourceId::new(id),
            kind,
            title: id.to_string(),
            meta: String::new(),
        }
    }

    #[test]
    fn dynamic_dossiers_group_sources_by_type() {
        let sources = vec![
            src("s1", SourceType::Documento),
            src("s2", SourceType::Norma),
            src("s3", SourceType::Documento),
        ];
        let dossiers = dynamic_dossiers(&sources);

        assert_eq!(dossiers.len(), 2);
        let documenti = dossiers.iter().find(|d| d.name == "Documenti").unwrap();
        assert_eq!(documenti.kind, DossierKind::Dynamic);
        assert_eq!(documenti.sources.len(), 2);
        let norme = dossiers.iter().find(|d| d.name == "Norme").unwrap();
        assert_eq!(norme.sources.len(), 1);
    }

    #[test]
    fn dynamic_dossiers_keep_stable_order() {
        // Norma added before Documento, but Documento must come first (all() order).
        let sources = vec![
            src("s1", SourceType::Norma),
            src("s2", SourceType::Documento),
        ];
        let dossiers = dynamic_dossiers(&sources);
        let names: Vec<&str> = dossiers.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["Documenti", "Norme"]);
    }

    #[test]
    fn a_source_can_belong_to_many_dossiers() {
        let ws = sample_workspace();
        let s1 = SourceId::new("s1");
        let views = dossiers_for_source(&s1, &ws.dossiers);
        // s1 is in its dynamic "Documenti" dossier AND in the manual one.
        assert!(views.len() >= 2);
        assert!(views.iter().any(|d| d.kind == DossierKind::Dynamic));
        assert!(views.iter().any(|d| d.kind == DossierKind::Manual));
    }

    #[test]
    fn dynamic_and_manual_are_distinguishable() {
        let ws = sample_workspace();
        assert!(ws.dossiers.iter().any(|d| d.kind == DossierKind::Dynamic));
        let manual = ws
            .dossiers
            .iter()
            .find(|d| d.kind == DossierKind::Manual)
            .unwrap();
        assert_eq!(manual.name, "Produzione avversaria");
    }

    #[test]
    fn sample_workspace_is_coherent() {
        let ws = sample_workspace();
        assert_eq!(ws.matter.client, ws.client.id);
        // every source id referenced by a dossier exists in the workspace sources.
        let known: Vec<&SourceId> = ws.sources.iter().map(|s| &s.id).collect();
        for dossier in &ws.dossiers {
            for sid in &dossier.sources {
                assert!(
                    known.contains(&sid),
                    "unknown source id in dossier {}",
                    dossier.id
                );
            }
        }
    }

    #[test]
    fn workspace_survives_serde_round_trip() {
        let ws = sample_workspace();
        let json = serde_json::to_string(&ws).unwrap();
        let back: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(ws, back);
        // newtype ids serialize transparently as bare strings.
        assert!(json.contains("\"rossi-bianchi\""));
    }
}
