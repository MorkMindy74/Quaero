//! Quaero domain model (Cliente → Pratica → Fascicolo/vista → Fonte).
//!
//! Pure and Tauri-free (ADR-0011). A `DossierView` is our **Fascicolo as a
//! VIEW** over sources — not a physical folder (ADR-0008): a source may appear
//! in many dossiers (many-to-many). `SourceRef` is a **minimal citable
//! reference**, NOT yet a full Fonte with Estratto/Ancora (those come later).
//!
//! Canonical vs derived boundary (post adversarial review):
//! - [`Workspace`] is the **canonical / persistable** contract: it stores only
//!   `sources` and user-curated **manual** dossiers.
//! - **Dynamic** dossiers are **derived** from `sources` by [`dynamic_dossiers`]
//!   and only ever live inside a [`WorkspaceView`] (computed for the UI). They
//!   are never persisted as canonical state, so they cannot go stale.

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

/// **Canonical** manual Fascicolo. By construction it can only be manual — it
/// has NO `kind`, so a dynamic dossier can never be represented in canonical
/// state. The boundary is enforced by the type, not by convention.
/// `deny_unknown_fields` rejects any payload trying to smuggle in a `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualDossier {
    pub id: String,
    pub name: String,
    pub sources: Vec<SourceId>,
}

impl ManualDossier {
    pub fn new(id: impl Into<String>, name: impl Into<String>, sources: Vec<SourceId>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sources,
        }
    }

    /// Render this canonical manual dossier as a (Manual) view entry.
    pub fn to_view(&self) -> DossierView {
        DossierView {
            id: self.id.clone(),
            name: self.name.clone(),
            kind: DossierKind::Manual,
            sources: self.sources.clone(),
        }
    }
}

/// Generate one **Dynamic** Fascicolo per `SourceType` actually present, in the
/// stable order of [`SourceType::all`]. Types with no sources produce nothing.
/// This is a pure derivation from `sources` — never persisted as canonical state.
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

/// All Fascicolo views for rendering: derived **dynamic** dossiers (from the
/// current `sources`) followed by the canonical **manual** ones.
pub fn all_dossier_views(sources: &[SourceRef], manual: &[ManualDossier]) -> Vec<DossierView> {
    let mut views = dynamic_dossiers(sources);
    views.extend(manual.iter().map(ManualDossier::to_view));
    views
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

/// **Canonical / persistable** matter state. Holds only canonical data: the
/// matter's sources and the user-curated **manual** dossiers. Dynamic dossiers
/// are NOT stored here — they are derived on demand (see [`Workspace::view`]).
/// `deny_unknown_fields` rejects shadow/derived fields (e.g. a top-level
/// `dossiers`) so a saved `WorkspaceView` cannot pass as canonical state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Workspace {
    pub client: Client,
    pub matter: Matter,
    pub sources: Vec<SourceRef>,
    pub manual_dossiers: Vec<ManualDossier>,
}

impl Workspace {
    /// Derive the read/render view (dynamic dossiers + manual). The view is
    /// always recomputed from the current `sources`, so it cannot go stale.
    pub fn view(&self) -> WorkspaceView {
        WorkspaceView {
            client: self.client.clone(),
            matter: self.matter.clone(),
            sources: self.sources.clone(),
            dossiers: all_dossier_views(&self.sources, &self.manual_dossiers),
        }
    }
}

/// A **derived, non-canonical** view of a [`Workspace`] for the UI. Combines
/// computed dynamic dossiers with the manual ones. This is NOT a persistence
/// schema; it is recomputed from canonical state and may be serialized only to
/// hand the already-derived result to the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceView {
    pub client: Client,
    pub matter: Matter,
    pub sources: Vec<SourceRef>,
    pub dossiers: Vec<DossierView>,
}

/// Deterministic sample canonical workspace (fixed ids, no random, no current
/// date). Contains sources + a manual dossier only; dynamic dossiers are derived
/// via [`Workspace::view`].
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
    // A manual Fascicolo curated by the user. Its sources ALSO appear in their
    // derived dynamic dossiers (many-to-many, no duplication of canonical data).
    let manual_dossiers = vec![ManualDossier::new(
        "man-produzione-avversaria",
        "Produzione avversaria",
        vec![SourceId::new("s1"), SourceId::new("s3")],
    )];

    Workspace {
        client,
        matter,
        sources,
        manual_dossiers,
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
    fn view_combines_derived_dynamic_and_canonical_manual() {
        let view = sample_workspace().view();
        // dynamic dossiers are present (derived)…
        assert!(view
            .dossiers
            .iter()
            .any(|d| d.kind == DossierKind::Dynamic && d.name == "Documenti"));
        // …and the manual one is present too.
        assert!(view
            .dossiers
            .iter()
            .any(|d| d.kind == DossierKind::Manual && d.name == "Produzione avversaria"));
    }

    #[test]
    fn manual_dossiers_are_canonical_manual_only_by_type() {
        let ws = sample_workspace();
        assert!(!ws.manual_dossiers.is_empty());
        // `ManualDossier` has no `kind` field, so canonical state cannot represent
        // a dynamic dossier at all. In the derived view it surfaces as Manual.
        let view = ws.view();
        assert!(view
            .dossiers
            .iter()
            .any(|d| d.kind == DossierKind::Manual && d.name == "Produzione avversaria"));
    }

    #[test]
    fn canonical_workspace_rejects_a_dynamic_dossier_in_manual_field() {
        // Hostile payload trying to smuggle a Dynamic dossier into a manual one.
        let json = r#"{
            "client": {"id":"alfa","name":"Alfa"},
            "matter": {"id":"m","client":"alfa","title":"t","subject":"s"},
            "sources": [],
            "manualDossiers": [{"id":"x","name":"X","sources":[],"kind":"Dynamic"}]
        }"#;
        let parsed: Result<Workspace, _> = serde_json::from_str(json);
        assert!(
            parsed.is_err(),
            "ManualDossier must reject a smuggled kind/Dynamic field"
        );
    }

    #[test]
    fn canonical_workspace_rejects_top_level_shadow_dossiers() {
        // Valid manualDossiers + an extra top-level `dossiers` (derived view state).
        // deny_unknown_fields on Workspace must reject it so a saved WorkspaceView
        // cannot pass as a canonical document under #5B.
        let json = r#"{
            "client": {"id":"alfa","name":"Alfa"},
            "matter": {"id":"m","client":"alfa","title":"t","subject":"s"},
            "sources": [],
            "manualDossiers": [],
            "dossiers": [{"id":"dyn-x","name":"X","kind":"Dynamic","sources":[]}]
        }"#;
        let parsed: Result<Workspace, _> = serde_json::from_str(json);
        assert!(
            parsed.is_err(),
            "canonical Workspace must reject a shadow top-level `dossiers` field"
        );
    }

    #[test]
    fn canonical_workspace_wire_shape_is_camelcase() {
        let json = serde_json::to_string(&sample_workspace()).unwrap();
        assert!(json.contains("manualDossiers"));
        assert!(!json.contains("manual_dossiers"));
        assert!(!json.contains("\"dossiers\""));
        assert!(!json.contains("Dynamic"));
    }

    #[test]
    fn a_source_can_belong_to_dynamic_and_manual() {
        let view = sample_workspace().view();
        let s1 = SourceId::new("s1");
        let views = dossiers_for_source(&s1, &view.dossiers);
        assert!(views.len() >= 2);
        assert!(views.iter().any(|d| d.kind == DossierKind::Dynamic));
        assert!(views.iter().any(|d| d.kind == DossierKind::Manual));
    }

    #[test]
    fn changing_source_type_refreshes_dynamic_view_without_staleness() {
        let mut ws = sample_workspace();
        // Reclassify s1 from Documento to Norma. s1 was the only Documento.
        ws.sources
            .iter_mut()
            .find(|s| s.id == SourceId::new("s1"))
            .unwrap()
            .kind = SourceType::Norma;

        let view = ws.view();
        // The "Documenti" dynamic dossier disappears — no stale membership.
        assert!(!view.dossiers.iter().any(|d| d.name == "Documenti"));
        // s1 is now grouped under "Norme".
        let norme = view.dossiers.iter().find(|d| d.name == "Norme").unwrap();
        assert!(norme.sources.contains(&SourceId::new("s1")));
    }

    #[test]
    fn canonical_contract_does_not_persist_dynamic_dossiers() {
        let ws = sample_workspace();
        let json = serde_json::to_string(&ws).unwrap();
        // The canonical Workspace serializes manual dossiers only. No DossierKind
        // is present at all (ManualDossier has none): neither Dynamic nor Manual,
        // and no derived dynamic dossier ids. (SourceRef.kind = SourceType is fine.)
        assert!(json.contains("Produzione avversaria"));
        assert!(!json.contains("\"Dynamic\""));
        assert!(!json.contains("\"Manual\""));
        assert!(!json.contains("dyn-"));
    }

    #[test]
    fn workspace_view_carries_dynamic_dossiers_when_serialized() {
        let view = sample_workspace().view();
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("\"Dynamic\""));
        assert!(json.contains("Documenti"));
    }

    #[test]
    fn sample_workspace_is_deterministic_and_coherent() {
        let a = sample_workspace();
        let b = sample_workspace();
        assert_eq!(a, b);
        assert_eq!(a.matter.client, a.client.id);
        // every source referenced by a manual dossier exists in the sources.
        let known: Vec<&SourceId> = a.sources.iter().map(|s| &s.id).collect();
        for dossier in &a.manual_dossiers {
            for sid in &dossier.sources {
                assert!(known.contains(&sid), "unknown source id in {}", dossier.id);
            }
        }
    }

    #[test]
    fn canonical_workspace_survives_serde_round_trip() {
        let ws = sample_workspace();
        let json = serde_json::to_string(&ws).unwrap();
        let back: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(ws, back);
        assert!(json.contains("\"rossi-bianchi\""));
    }
}
