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
id_newtype!(ExcerptId);
id_newtype!(CitationId);

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
#[serde(deny_unknown_fields)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
}

/// A single professional engagement (Pratica) belonging to a Client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matter {
    pub id: MatterId,
    pub client: ClientId,
    pub title: String,
    pub subject: String,
}

/// Descriptor linking a `SourceRef` to its imported file content on disk.
/// Metadata only — the bytes live in the desktop blob store, never here. The
/// `sha256` digest pins the imported content (integrity / future Ancora base).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredFile {
    /// Safe on-disk filename (generated; never derived from `original_name`).
    pub stored_name: String,
    /// Original filename provided by the user (display only, never a path).
    pub original_name: String,
    /// Size of the stored content, in bytes.
    pub byte_len: u64,
    /// Lowercase hex SHA-256 of the imported bytes.
    pub sha256: String,
}

/// A minimal citable reference (Fonte). Not yet an Estratto/Ancora. A Documento
/// may carry a [`StoredFile`] link to its imported content (#6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    pub id: SourceId,
    pub kind: SourceType,
    pub title: String,
    pub meta: String,
    /// Present for imported Documento sources; `None` for reference-only Fonti.
    /// `serde(default)` keeps pre-#6 workspaces (without this field) loadable;
    /// `skip_serializing_if` keeps file-less sources' JSON unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<StoredFile>,
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

/// Reserved id prefix for **generated dynamic** dossiers. Manual dossiers may
/// never use it, so dynamic and manual views can't collide on id.
pub const DYNAMIC_DOSSIER_PREFIX: &str = "dyn-";

/// Error constructing a canonical [`ManualDossier`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualDossierError {
    /// The id uses the reserved `dyn-` prefix of generated dynamic dossiers.
    ReservedDynamicPrefix,
}

impl std::fmt::Display for ManualDossierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManualDossierError::ReservedDynamicPrefix => write!(
                f,
                "manual dossier id must not use the reserved `{DYNAMIC_DOSSIER_PREFIX}` prefix"
            ),
        }
    }
}

impl std::error::Error for ManualDossierError {}

/// **Canonical** manual Fascicolo. By construction it can only be manual — it
/// has NO `kind`, so a dynamic dossier can never be represented in canonical
/// state. Its id may not use the reserved `dyn-` prefix. Both invariants are
/// enforced on construction AND on deserialization (via `RawManualDossier`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RawManualDossier")]
pub struct ManualDossier {
    // Private fields: the only ways to obtain a `ManualDossier` are the checked
    // constructor and the validating serde `TryFrom` path, so an invalid id
    // (e.g. `dyn-*`) can never be built — not even by an in-process Rust caller
    // (an external `ManualDossier { id: .. }` literal does not compile).
    id: String,
    name: String,
    sources: Vec<SourceId>,
}

impl ManualDossier {
    /// Build a manual dossier, rejecting ids in the reserved `dyn-` namespace.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        sources: Vec<SourceId>,
    ) -> Result<Self, ManualDossierError> {
        let id = id.into();
        if id.starts_with(DYNAMIC_DOSSIER_PREFIX) {
            return Err(ManualDossierError::ReservedDynamicPrefix);
        }
        Ok(Self {
            id,
            name: name.into(),
            sources,
        })
    }

    /// Read-only accessor for the dossier id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Read-only accessor for the dossier name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Read-only accessor for the dossier's source ids.
    pub fn sources(&self) -> &[SourceId] {
        &self.sources
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

/// Wire shape for deserializing a [`ManualDossier`]: `deny_unknown_fields`
/// rejects smuggled fields (e.g. `kind`), then `TryFrom` enforces the id rule.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawManualDossier {
    id: String,
    name: String,
    sources: Vec<SourceId>,
}

impl TryFrom<RawManualDossier> for ManualDossier {
    type Error = ManualDossierError;

    fn try_from(raw: RawManualDossier) -> Result<Self, Self::Error> {
        ManualDossier::new(raw.id, raw.name, raw.sources)
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
                    id: format!("{}{}", DYNAMIC_DOSSIER_PREFIX, kind.slug()),
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

// --- Anti-hallucination chain (#8): Estratto · Ancora · Citazione ----------
//
// ADR-0007: an Affermazione is supported by **Estratti di Fonte**, never by a
// whole Fonte. The types below enforce that *by construction*: a [`Citation`]
// can only reference an [`ExcerptId`] — there is no field through which it could
// point at a `SourceId`. The Ancora is a **layout-independent logical locator**
// (declarative in #8; not computed from parsing).

/// A stable, layout-independent locator of an [`Excerpt`] within its Fonte
/// (ADR: "Ancora"). Declarative in #8 (e.g. `kind: "clausola", value: "7.2"`);
/// it points at a logical unit, not a rendered page coordinate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub kind: String,
    pub value: String,
}

/// Error constructing an [`Excerpt`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcerptError {
    /// The quoted text is empty.
    EmptyQuote,
    /// The anchor kind or value is empty.
    EmptyAnchor,
}

impl std::fmt::Display for ExcerptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExcerptError::EmptyQuote => write!(f, "excerpt quote must not be empty"),
            ExcerptError::EmptyAnchor => write!(f, "excerpt anchor kind/value must not be empty"),
        }
    }
}

impl std::error::Error for ExcerptError {}

/// A **verifiable portion of a Fonte** that can support an Affermazione (ADR-0007).
/// Belongs to a [`SourceRef`] (`source_id`), carries the verbatim `quote`, an
/// [`Anchor`], and — for Documento Fonti with a [`StoredFile`] — an optional
/// `source_sha256` pinning the content version the excerpt was taken from.
/// Valid by construction: fields are private; built via [`Excerpt::new`] or the
/// serde `TryFrom` path, both rejecting an empty quote / anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RawExcerpt")]
pub struct Excerpt {
    id: ExcerptId,
    source_id: SourceId,
    anchor: Anchor,
    quote: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_sha256: Option<String>,
}

impl Excerpt {
    pub fn new(
        id: impl Into<String>,
        source_id: SourceId,
        anchor: Anchor,
        quote: impl Into<String>,
        source_sha256: Option<String>,
    ) -> Result<Self, ExcerptError> {
        let quote = quote.into();
        if quote.trim().is_empty() {
            return Err(ExcerptError::EmptyQuote);
        }
        if anchor.kind.trim().is_empty() || anchor.value.trim().is_empty() {
            return Err(ExcerptError::EmptyAnchor);
        }
        Ok(Self {
            id: ExcerptId::new(id),
            source_id,
            anchor,
            quote,
            source_sha256,
        })
    }

    pub fn id(&self) -> &ExcerptId {
        &self.id
    }
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }
    pub fn quote(&self) -> &str {
        &self.quote
    }
    pub fn source_sha256(&self) -> Option<&str> {
        self.source_sha256.as_deref()
    }
}

/// Wire shape for [`Excerpt`]: `deny_unknown_fields` then `TryFrom` validation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawExcerpt {
    id: String,
    source_id: SourceId,
    anchor: Anchor,
    quote: String,
    #[serde(default)]
    source_sha256: Option<String>,
}

impl TryFrom<RawExcerpt> for Excerpt {
    type Error = ExcerptError;

    fn try_from(raw: RawExcerpt) -> Result<Self, Self::Error> {
        Excerpt::new(
            raw.id,
            raw.source_id,
            raw.anchor,
            raw.quote,
            raw.source_sha256,
        )
    }
}

/// Error constructing a [`Citation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationError {
    /// The claim text is empty.
    EmptyClaim,
}

impl std::fmt::Display for CitationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CitationError::EmptyClaim => write!(f, "citation claim must not be empty"),
        }
    }
}

impl std::error::Error for CitationError {}

/// The link between an Affermazione (`claim`) and the [`Excerpt`] that supports
/// it. **Crucially, it references only an [`ExcerptId`]** — there is no field to
/// cite a Fonte directly, so ADR-0007 ("cite Estratti, not Fonti") holds by
/// construction. Valid by construction: built via [`Citation::new`] or serde
/// `TryFrom`, both rejecting an empty claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RawCitation")]
pub struct Citation {
    id: CitationId,
    claim: String,
    excerpt_id: ExcerptId,
}

impl Citation {
    pub fn new(
        id: impl Into<String>,
        claim: impl Into<String>,
        excerpt_id: ExcerptId,
    ) -> Result<Self, CitationError> {
        let claim = claim.into();
        if claim.trim().is_empty() {
            return Err(CitationError::EmptyClaim);
        }
        Ok(Self {
            id: CitationId::new(id),
            claim,
            excerpt_id,
        })
    }

    pub fn id(&self) -> &CitationId {
        &self.id
    }
    pub fn claim(&self) -> &str {
        &self.claim
    }
    pub fn excerpt_id(&self) -> &ExcerptId {
        &self.excerpt_id
    }
}

/// Wire shape for [`Citation`]: `deny_unknown_fields` rejects a smuggled
/// `sourceId` (a Citation may never reference a Fonte), then `TryFrom` validates.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawCitation {
    id: String,
    claim: String,
    excerpt_id: ExcerptId,
}

impl TryFrom<RawCitation> for Citation {
    type Error = CitationError;

    fn try_from(raw: RawCitation) -> Result<Self, Self::Error> {
        Citation::new(raw.id, raw.claim, raw.excerpt_id)
    }
}

/// Why a [`Workspace`] is not a valid canonical document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    /// `matter.client` does not point at `client.id`.
    ClientMismatch,
    /// Two sources share the same id.
    DuplicateSourceId(String),
    /// Two manual dossiers share the same id.
    DuplicateManualDossierId(String),
    /// A manual dossier references a source id absent from `sources`.
    DanglingManualSource { dossier: String, source: String },
    /// Two excerpts share the same id.
    DuplicateExcerptId(String),
    /// An excerpt references a source id absent from `sources`.
    DanglingExcerptSource { excerpt: String, source: String },
    /// Two citations share the same id.
    DuplicateCitationId(String),
    /// A citation references an excerpt id absent from `excerpts`.
    DanglingCitationExcerpt { citation: String, excerpt: String },
    /// An excerpt pins a `sourceSha256` but the referenced source has no file.
    ExcerptShaWithoutFile { excerpt: String, source: String },
    /// An excerpt's `sourceSha256` does not match the referenced file's digest.
    ExcerptShaMismatch { excerpt: String, source: String },
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::ClientMismatch => write!(f, "matter.client must equal client.id"),
            WorkspaceError::DuplicateSourceId(id) => write!(f, "duplicate source id: {id}"),
            WorkspaceError::DuplicateManualDossierId(id) => {
                write!(f, "duplicate manual dossier id: {id}")
            }
            WorkspaceError::DanglingManualSource { dossier, source } => {
                write!(
                    f,
                    "manual dossier {dossier} references unknown source {source}"
                )
            }
            WorkspaceError::DuplicateExcerptId(id) => write!(f, "duplicate excerpt id: {id}"),
            WorkspaceError::DanglingExcerptSource { excerpt, source } => {
                write!(f, "excerpt {excerpt} references unknown source {source}")
            }
            WorkspaceError::DuplicateCitationId(id) => write!(f, "duplicate citation id: {id}"),
            WorkspaceError::DanglingCitationExcerpt { citation, excerpt } => {
                write!(
                    f,
                    "citation {citation} references unknown excerpt {excerpt}"
                )
            }
            WorkspaceError::ExcerptShaWithoutFile { excerpt, source } => write!(
                f,
                "excerpt {excerpt} pins a sha256 but source {source} has no stored file"
            ),
            WorkspaceError::ExcerptShaMismatch { excerpt, source } => write!(
                f,
                "excerpt {excerpt} sha256 does not match the stored file of source {source}"
            ),
        }
    }
}

impl std::error::Error for WorkspaceError {}

/// **Canonical / persistable** matter state, **valid by construction**. Holds
/// only canonical data: the matter's sources and the user-curated **manual**
/// dossiers. Dynamic dossiers are NOT stored here — they are derived on demand
/// (see [`Workspace::view`]). Fields are private; the only ways to obtain a
/// `Workspace` are the validating [`Workspace::new`] or the serde `TryFrom`
/// path, so an incoherent state (client/matter mismatch, duplicate ids, dangling
/// manual source refs, shadow `dossiers`) can never exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RawWorkspace")]
pub struct Workspace {
    client: Client,
    matter: Matter,
    sources: Vec<SourceRef>,
    manual_dossiers: Vec<ManualDossier>,
    // Anti-hallucination chain (#8). `skip_serializing_if` keeps pre-#8
    // workspaces' on-disk shape byte-for-byte unchanged when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    excerpts: Vec<Excerpt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    citations: Vec<Citation>,
}

impl Workspace {
    /// Build a canonical workspace (no excerpts/citations), enforcing
    /// referential integrity. Backward-compatible 4-arg constructor.
    pub fn new(
        client: Client,
        matter: Matter,
        sources: Vec<SourceRef>,
        manual_dossiers: Vec<ManualDossier>,
    ) -> Result<Self, WorkspaceError> {
        Self::assemble(
            client,
            matter,
            sources,
            manual_dossiers,
            Vec::new(),
            Vec::new(),
        )
    }

    /// Build a canonical workspace including the anti-hallucination chain
    /// (Estratti + Citazioni), enforcing full referential integrity (#8).
    pub fn new_with_evidence(
        client: Client,
        matter: Matter,
        sources: Vec<SourceRef>,
        manual_dossiers: Vec<ManualDossier>,
        excerpts: Vec<Excerpt>,
        citations: Vec<Citation>,
    ) -> Result<Self, WorkspaceError> {
        Self::assemble(
            client,
            matter,
            sources,
            manual_dossiers,
            excerpts,
            citations,
        )
    }

    fn assemble(
        client: Client,
        matter: Matter,
        sources: Vec<SourceRef>,
        manual_dossiers: Vec<ManualDossier>,
        excerpts: Vec<Excerpt>,
        citations: Vec<Citation>,
    ) -> Result<Self, WorkspaceError> {
        if matter.client != client.id {
            return Err(WorkspaceError::ClientMismatch);
        }

        let mut seen_sources = std::collections::HashSet::new();
        for source in &sources {
            if !seen_sources.insert(&source.id) {
                return Err(WorkspaceError::DuplicateSourceId(source.id.0.clone()));
            }
        }

        let mut seen_dossiers = std::collections::HashSet::new();
        for dossier in &manual_dossiers {
            if !seen_dossiers.insert(dossier.id()) {
                return Err(WorkspaceError::DuplicateManualDossierId(
                    dossier.id().to_string(),
                ));
            }
        }

        for dossier in &manual_dossiers {
            for source in dossier.sources() {
                if !seen_sources.contains(source) {
                    return Err(WorkspaceError::DanglingManualSource {
                        dossier: dossier.id().to_string(),
                        source: source.0.clone(),
                    });
                }
            }
        }

        // #8: excerpts must reference an existing source; ids unique. If an
        // excerpt pins a `sourceSha256`, it must match the referenced source's
        // StoredFile digest exactly (Evidence integrity) — a pin on a fileless
        // source, or a mismatching digest, is rejected. Runs on both the public
        // constructor and the serde `TryFrom` path (no deserialization bypass).
        let mut seen_excerpts = std::collections::HashSet::new();
        for excerpt in &excerpts {
            if !seen_excerpts.insert(excerpt.id()) {
                return Err(WorkspaceError::DuplicateExcerptId(excerpt.id().0.clone()));
            }
            let source = match sources.iter().find(|s| &s.id == excerpt.source_id()) {
                Some(source) => source,
                None => {
                    return Err(WorkspaceError::DanglingExcerptSource {
                        excerpt: excerpt.id().0.clone(),
                        source: excerpt.source_id().0.clone(),
                    });
                }
            };
            if let Some(sha) = excerpt.source_sha256() {
                match source.file.as_ref() {
                    None => {
                        return Err(WorkspaceError::ExcerptShaWithoutFile {
                            excerpt: excerpt.id().0.clone(),
                            source: excerpt.source_id().0.clone(),
                        });
                    }
                    Some(file) if file.sha256 != sha => {
                        return Err(WorkspaceError::ExcerptShaMismatch {
                            excerpt: excerpt.id().0.clone(),
                            source: excerpt.source_id().0.clone(),
                        });
                    }
                    Some(_) => {}
                }
            }
        }

        // #8: citations must reference an existing excerpt; ids unique.
        let mut seen_citations = std::collections::HashSet::new();
        for citation in &citations {
            if !seen_citations.insert(citation.id()) {
                return Err(WorkspaceError::DuplicateCitationId(citation.id().0.clone()));
            }
            if !seen_excerpts.contains(citation.excerpt_id()) {
                return Err(WorkspaceError::DanglingCitationExcerpt {
                    citation: citation.id().0.clone(),
                    excerpt: citation.excerpt_id().0.clone(),
                });
            }
        }

        Ok(Self {
            client,
            matter,
            sources,
            manual_dossiers,
            excerpts,
            citations,
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn matter(&self) -> &Matter {
        &self.matter
    }

    pub fn sources(&self) -> &[SourceRef] {
        &self.sources
    }

    pub fn manual_dossiers(&self) -> &[ManualDossier] {
        &self.manual_dossiers
    }

    pub fn excerpts(&self) -> &[Excerpt] {
        &self.excerpts
    }

    pub fn citations(&self) -> &[Citation] {
        &self.citations
    }

    /// Return a new workspace with one more source, re-validating the whole
    /// graph (e.g. rejects a duplicate source id). Used by document import (#6);
    /// preserves existing excerpts/citations. Never mutates in place.
    pub fn with_source(self, source: SourceRef) -> Result<Self, WorkspaceError> {
        let mut sources = self.sources;
        sources.push(source);
        Workspace::new_with_evidence(
            self.client,
            self.matter,
            sources,
            self.manual_dossiers,
            self.excerpts,
            self.citations,
        )
    }

    /// Derive the read/render view (dynamic dossiers + manual) plus the
    /// canonical excerpts/citations (cloned). Recomputed from canonical state.
    pub fn view(&self) -> WorkspaceView {
        WorkspaceView {
            client: self.client.clone(),
            matter: self.matter.clone(),
            sources: self.sources.clone(),
            dossiers: all_dossier_views(&self.sources, &self.manual_dossiers),
            excerpts: self.excerpts.clone(),
            citations: self.citations.clone(),
        }
    }
}

/// Wire shape for deserializing a [`Workspace`]: `deny_unknown_fields` rejects
/// shadow/derived fields (e.g. a top-level `dossiers`), then `TryFrom` enforces
/// referential integrity via [`Workspace::new`].
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawWorkspace {
    client: Client,
    matter: Matter,
    sources: Vec<SourceRef>,
    manual_dossiers: Vec<ManualDossier>,
    // `default` keeps pre-#8 persisted workspaces (without these fields) loadable.
    #[serde(default)]
    excerpts: Vec<Excerpt>,
    #[serde(default)]
    citations: Vec<Citation>,
}

impl TryFrom<RawWorkspace> for Workspace {
    type Error = WorkspaceError;

    fn try_from(raw: RawWorkspace) -> Result<Self, Self::Error> {
        Workspace::new_with_evidence(
            raw.client,
            raw.matter,
            raw.sources,
            raw.manual_dossiers,
            raw.excerpts,
            raw.citations,
        )
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
    pub excerpts: Vec<Excerpt>,
    pub citations: Vec<Citation>,
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
            file: None,
        },
        SourceRef {
            id: SourceId::new("s2"),
            kind: SourceType::Norma,
            title: "Art. 1453 c.c.".to_string(),
            meta: "Risoluzione per inadempimento".to_string(),
            file: None,
        },
        SourceRef {
            id: SourceId::new("s3"),
            kind: SourceType::Giurisprudenza,
            title: "Cass. civ. 12345/2024".to_string(),
            meta: "massima".to_string(),
            file: None,
        },
        SourceRef {
            id: SourceId::new("s4"),
            kind: SourceType::Nota,
            title: "Cliente disponibile a transigere".to_string(),
            meta: String::new(),
            file: None,
        },
    ];
    // A manual Fascicolo curated by the user. Its sources ALSO appear in their
    // derived dynamic dossiers (many-to-many, no duplication of canonical data).
    let manual_dossiers = vec![ManualDossier::new(
        "man-produzione-avversaria",
        "Produzione avversaria",
        vec![SourceId::new("s1"), SourceId::new("s3")],
    )
    .expect("sample manual dossier id is valid")];

    // #8 seed: an Estratto of the Documento s1, and a Citazione that cites it
    // (an Affermazione supported by that Estratto — never by the whole Fonte).
    let excerpts = vec![Excerpt::new(
        "e1",
        SourceId::new("s1"),
        Anchor {
            kind: "clausola".to_string(),
            value: "7.2".to_string(),
        },
        "Il Fornitore potrà recedere con preavviso di quindici giorni.",
        None,
    )
    .expect("sample excerpt is valid")];
    let citations = vec![Citation::new(
        "c1",
        "La clausola 7.2 consente il recesso con preavviso di 15 giorni.",
        ExcerptId::new("e1"),
    )
    .expect("sample citation is valid")];

    Workspace::new_with_evidence(
        client,
        matter,
        sources,
        manual_dossiers,
        excerpts,
        citations,
    )
    .expect("sample workspace is internally consistent")
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
            file: None,
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
        assert!(!ws.manual_dossiers().is_empty());
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
        let ws = sample_workspace();
        // Reclassify s1 from Documento to Norma (s1 was the only Documento) and
        // rebuild via the public constructor (still a valid graph).
        let mut sources = ws.sources().to_vec();
        sources
            .iter_mut()
            .find(|s| s.id == SourceId::new("s1"))
            .unwrap()
            .kind = SourceType::Norma;
        let ws = Workspace::new(
            ws.client().clone(),
            ws.matter().clone(),
            sources,
            ws.manual_dossiers().to_vec(),
        )
        .expect("reclassified workspace is still valid");

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
        assert_eq!(a.matter().client, a.client().id);
        // every source referenced by a manual dossier exists in the sources.
        let known: Vec<&SourceId> = a.sources().iter().map(|s| &s.id).collect();
        for dossier in a.manual_dossiers() {
            for sid in dossier.sources() {
                assert!(
                    known.contains(&sid),
                    "unknown source id in {}",
                    dossier.id()
                );
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

    #[test]
    fn canonical_workspace_rejects_nested_shadow_view_fields() {
        // A corrupted/version-skewed file carrying derived view bits inside nested
        // canonical structs must be rejected, not silently accepted.
        let payloads = [
            // client carries a shadow `dossiers`
            r#"{"client":{"id":"a","name":"A","dossiers":[]},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[],"manualDossiers":[]}"#,
            // matter carries a shadow `dossiers`
            r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s","dossiers":[]},"sources":[],"manualDossiers":[]}"#,
            // a source carries a shadow `dossiers`
            r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[{"id":"s1","kind":"Documento","title":"t","meta":"","dossiers":[]}],"manualDossiers":[]}"#,
            // a source carries a shadow `manualDossiers`
            r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[{"id":"s1","kind":"Documento","title":"t","meta":"","manualDossiers":[]}],"manualDossiers":[]}"#,
        ];
        for json in payloads {
            assert!(
                serde_json::from_str::<Workspace>(json).is_err(),
                "nested shadow field must be rejected: {json}"
            );
        }
    }

    #[test]
    fn manual_dossier_with_normal_id_is_ok() {
        assert!(ManualDossier::new("man-x", "X", vec![]).is_ok());
    }

    #[test]
    fn manual_dossier_with_reserved_dyn_prefix_is_rejected() {
        assert_eq!(
            ManualDossier::new("dyn-documento", "X", vec![]),
            Err(ManualDossierError::ReservedDynamicPrefix)
        );
    }

    #[test]
    fn deserializing_a_manual_dossier_with_reserved_dyn_prefix_is_rejected() {
        let json = r#"{
            "client": {"id":"alfa","name":"Alfa"},
            "matter": {"id":"m","client":"alfa","title":"t","subject":"s"},
            "sources": [],
            "manualDossiers": [{"id":"dyn-documento","name":"X","sources":[]}]
        }"#;
        assert!(
            serde_json::from_str::<Workspace>(json).is_err(),
            "a loaded manual dossier must not use the reserved dyn- prefix"
        );
    }

    #[test]
    fn view_dossier_ids_are_unique_no_dynamic_manual_collision() {
        let view = sample_workspace().view();
        let total = view.dossiers.len();
        let mut ids: Vec<&str> = view.dossiers.iter().map(|d| d.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "dossier view ids must be unique");
    }

    fn client(id: &str) -> Client {
        Client {
            id: ClientId::new(id),
            name: id.to_uppercase(),
        }
    }

    fn matter(id: &str, client: &str) -> Matter {
        Matter {
            id: MatterId::new(id),
            client: ClientId::new(client),
            title: "t".to_string(),
            subject: "s".to_string(),
        }
    }

    #[test]
    fn workspace_new_rejects_client_matter_mismatch() {
        let result = Workspace::new(client("alfa"), matter("m", "beta"), vec![], vec![]);
        assert_eq!(result, Err(WorkspaceError::ClientMismatch));
    }

    #[test]
    fn workspace_new_rejects_dangling_manual_source() {
        let manual = ManualDossier::new("man-x", "X", vec![SourceId::new("ghost")]).unwrap();
        let result = Workspace::new(client("alfa"), matter("m", "alfa"), vec![], vec![manual]);
        assert!(matches!(
            result,
            Err(WorkspaceError::DanglingManualSource { .. })
        ));
    }

    #[test]
    fn workspace_new_rejects_duplicate_source_ids() {
        let sources = vec![
            src("s1", SourceType::Documento),
            src("s1", SourceType::Norma),
        ];
        let result = Workspace::new(client("alfa"), matter("m", "alfa"), sources, vec![]);
        assert!(matches!(result, Err(WorkspaceError::DuplicateSourceId(_))));
    }

    #[test]
    fn workspace_new_rejects_duplicate_manual_dossier_ids() {
        let m1 = ManualDossier::new("man-x", "X", vec![]).unwrap();
        let m2 = ManualDossier::new("man-x", "Y", vec![]).unwrap();
        let result = Workspace::new(client("alfa"), matter("m", "alfa"), vec![], vec![m1, m2]);
        assert!(matches!(
            result,
            Err(WorkspaceError::DuplicateManualDossierId(_))
        ));
    }

    #[test]
    fn workspace_new_accepts_a_valid_graph() {
        let sources = vec![src("s1", SourceType::Documento)];
        let manual = ManualDossier::new("man-x", "X", vec![SourceId::new("s1")]).unwrap();
        assert!(Workspace::new(client("alfa"), matter("m", "alfa"), sources, vec![manual]).is_ok());
    }

    #[test]
    fn with_source_adds_a_valid_source() {
        let ws = Workspace::new(client("alfa"), matter("m", "alfa"), vec![], vec![]).unwrap();
        let doc = SourceRef {
            id: SourceId::new("doc-1"),
            kind: SourceType::Documento,
            title: "Contratto.pdf".to_string(),
            meta: "12 byte".to_string(),
            file: Some(StoredFile {
                stored_name: "doc-1.pdf".to_string(),
                original_name: "Contratto.pdf".to_string(),
                byte_len: 12,
                sha256: "ab".repeat(32),
            }),
        };
        let ws = ws.with_source(doc).unwrap();
        assert_eq!(ws.sources().len(), 1);
        assert_eq!(ws.sources()[0].id, SourceId::new("doc-1"));
        assert!(ws.sources()[0].file.is_some());
        // it surfaces in the derived "Documenti" dynamic dossier
        assert!(ws.view().dossiers.iter().any(|d| d.name == "Documenti"));
    }

    #[test]
    fn with_source_rejects_duplicate_id() {
        let ws = Workspace::new(
            client("alfa"),
            matter("m", "alfa"),
            vec![src("s1", SourceType::Documento)],
            vec![],
        )
        .unwrap();
        let dup = src("s1", SourceType::Norma);
        assert!(matches!(
            ws.with_source(dup),
            Err(WorkspaceError::DuplicateSourceId(_))
        ));
    }

    #[test]
    fn source_ref_with_file_round_trips() {
        let doc = SourceRef {
            id: SourceId::new("doc-1"),
            kind: SourceType::Documento,
            title: "C.pdf".to_string(),
            meta: String::new(),
            file: Some(StoredFile {
                stored_name: "doc-1.pdf".to_string(),
                original_name: "C.pdf".to_string(),
                byte_len: 3,
                sha256: "ba7816bf".to_string(),
            }),
        };
        let json = serde_json::to_string(&doc).unwrap();
        // camelCase wire shape for StoredFile
        assert!(json.contains("storedName"));
        assert!(json.contains("originalName"));
        assert!(json.contains("byteLen"));
        assert!(json.contains("sha256"));
        let back: SourceRef = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn source_ref_without_file_is_backward_compatible() {
        // pre-#6 JSON (no `file`) still deserializes, with file = None…
        let legacy = r#"{"id":"s1","kind":"Documento","title":"t","meta":"m"}"#;
        let parsed: SourceRef = serde_json::from_str(legacy).unwrap();
        assert!(parsed.file.is_none());
        // …and a file-less source serializes WITHOUT a `file` key (unchanged shape).
        let json = serde_json::to_string(&parsed).unwrap();
        assert!(!json.contains("file"));
    }

    #[test]
    fn workspace_json_with_documento_carries_sha256_not_bytes() {
        let ws = Workspace::new(client("alfa"), matter("m", "alfa"), vec![], vec![]).unwrap();
        let ws = ws
            .with_source(SourceRef {
                id: SourceId::new("doc-1"),
                kind: SourceType::Documento,
                title: "C.pdf".to_string(),
                meta: String::new(),
                file: Some(StoredFile {
                    stored_name: "doc-1.pdf".to_string(),
                    original_name: "C.pdf".to_string(),
                    byte_len: 3,
                    sha256: "deadbeef".to_string(),
                }),
            })
            .unwrap();
        let json = crate::persistence::to_json(&ws).unwrap();
        assert!(json.contains("sha256"));
        assert!(json.contains("deadbeef"));
        assert!(json.contains("storedName"));
        // the canonical JSON references the file by name + digest, never bytes.
        assert!(!json.contains("bytes"));
    }

    #[test]
    fn deserializing_incoherent_workspace_is_rejected() {
        // matter.client (b) does not match client.id (a).
        let json = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"b","title":"t","subject":"s"},"sources":[],"manualDossiers":[]}"#;
        assert!(serde_json::from_str::<Workspace>(json).is_err());
        // a manual dossier referencing a non-existent source.
        let dangling = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[],"manualDossiers":[{"id":"man-x","name":"X","sources":["ghost"]}]}"#;
        assert!(serde_json::from_str::<Workspace>(dangling).is_err());
    }

    // --- #8 Estratti / Ancore / Citazioni ---------------------------------

    fn anchor(kind: &str, value: &str) -> Anchor {
        Anchor {
            kind: kind.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn excerpt_new_rejects_empty_quote_or_anchor() {
        assert_eq!(
            Excerpt::new("e1", SourceId::new("s1"), anchor("k", "v"), "   ", None),
            Err(ExcerptError::EmptyQuote)
        );
        assert_eq!(
            Excerpt::new("e1", SourceId::new("s1"), anchor("", "v"), "q", None),
            Err(ExcerptError::EmptyAnchor)
        );
        assert_eq!(
            Excerpt::new("e1", SourceId::new("s1"), anchor("k", ""), "q", None),
            Err(ExcerptError::EmptyAnchor)
        );
        assert!(Excerpt::new("e1", SourceId::new("s1"), anchor("k", "v"), "q", None).is_ok());
    }

    #[test]
    fn citation_new_rejects_empty_claim() {
        assert_eq!(
            Citation::new("c1", "  ", ExcerptId::new("e1")),
            Err(CitationError::EmptyClaim)
        );
        assert!(Citation::new("c1", "x", ExcerptId::new("e1")).is_ok());
    }

    #[test]
    fn sample_workspace_round_trips_with_excerpts_and_citations() {
        let ws = sample_workspace();
        assert_eq!(ws.excerpts().len(), 1);
        assert_eq!(ws.citations().len(), 1);
        // the citation points at the excerpt, and the excerpt at a real source
        assert_eq!(ws.citations()[0].excerpt_id(), ws.excerpts()[0].id());
        assert!(ws
            .sources()
            .iter()
            .any(|s| &s.id == ws.excerpts()[0].source_id()));
        let json = serde_json::to_string(&ws).unwrap();
        assert!(json.contains("\"excerpts\""));
        assert!(json.contains("\"citations\""));
        assert!(json.contains("sourceId"));
        let back: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(ws, back);
        // the view carries them too
        let view = ws.view();
        assert_eq!(view.excerpts.len(), 1);
        assert_eq!(view.citations.len(), 1);
    }

    #[test]
    fn workspace_without_evidence_omits_those_keys_on_serialize() {
        // backward-compatible on-disk shape: no excerpts/citations keys when empty
        let ws = Workspace::new(client("alfa"), matter("m", "alfa"), vec![], vec![]).unwrap();
        let json = serde_json::to_string(&ws).unwrap();
        assert!(!json.contains("excerpts"));
        assert!(!json.contains("citations"));
    }

    #[test]
    fn pre_8_json_without_evidence_still_loads() {
        // a pre-#8 persisted workspace (no excerpts/citations) must still load
        let json = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[{"id":"s1","kind":"Documento","title":"t","meta":""}],"manualDossiers":[]}"#;
        let ws: Workspace = serde_json::from_str(json).unwrap();
        assert!(ws.excerpts().is_empty());
        assert!(ws.citations().is_empty());
    }

    #[test]
    fn workspace_rejects_dangling_excerpt_source() {
        let excerpt =
            Excerpt::new("e1", SourceId::new("ghost"), anchor("k", "v"), "q", None).unwrap();
        let r = Workspace::new_with_evidence(
            client("alfa"),
            matter("m", "alfa"),
            vec![],
            vec![],
            vec![excerpt],
            vec![],
        );
        assert!(matches!(
            r,
            Err(WorkspaceError::DanglingExcerptSource { .. })
        ));
    }

    #[test]
    fn workspace_rejects_dangling_citation_excerpt() {
        let citation = Citation::new("c1", "x", ExcerptId::new("ghost")).unwrap();
        let r = Workspace::new_with_evidence(
            client("alfa"),
            matter("m", "alfa"),
            vec![],
            vec![],
            vec![],
            vec![citation],
        );
        assert!(matches!(
            r,
            Err(WorkspaceError::DanglingCitationExcerpt { .. })
        ));
    }

    #[test]
    fn workspace_rejects_duplicate_excerpt_and_citation_ids() {
        let src1 = src("s1", SourceType::Documento);
        let e =
            |id: &str| Excerpt::new(id, SourceId::new("s1"), anchor("k", "v"), "q", None).unwrap();
        let dup_ex = Workspace::new_with_evidence(
            client("alfa"),
            matter("m", "alfa"),
            vec![src1.clone()],
            vec![],
            vec![e("e1"), e("e1")],
            vec![],
        );
        assert!(matches!(dup_ex, Err(WorkspaceError::DuplicateExcerptId(_))));

        let c = |id: &str| Citation::new(id, "x", ExcerptId::new("e1")).unwrap();
        let dup_cit = Workspace::new_with_evidence(
            client("alfa"),
            matter("m", "alfa"),
            vec![src1],
            vec![],
            vec![e("e1")],
            vec![c("c1"), c("c1")],
        );
        assert!(matches!(
            dup_cit,
            Err(WorkspaceError::DuplicateCitationId(_))
        ));
    }

    #[test]
    fn a_citation_cannot_reference_a_fonte_directly() {
        // hostile JSON: a citation carrying `sourceId` instead of `excerptId`.
        // deny_unknown_fields rejects `sourceId` (and `excerptId` is missing).
        let json = r#"{
            "client":{"id":"a","name":"A"},
            "matter":{"id":"m","client":"a","title":"t","subject":"s"},
            "sources":[{"id":"s1","kind":"Documento","title":"t","meta":""}],
            "manualDossiers":[],
            "excerpts":[{"id":"e1","sourceId":"s1","anchor":{"kind":"k","value":"v"},"quote":"q"}],
            "citations":[{"id":"c1","claim":"x","sourceId":"s1"}]
        }"#;
        assert!(
            serde_json::from_str::<Workspace>(json).is_err(),
            "a citation must reference an excerpt, never a Fonte"
        );
    }

    #[test]
    fn excerpt_wire_shape_is_camelcase_with_optional_sha() {
        let with_sha = Excerpt::new(
            "e1",
            SourceId::new("s1"),
            anchor("clausola", "7.2"),
            "q",
            Some("ab".repeat(32)),
        )
        .unwrap();
        let json = serde_json::to_string(&with_sha).unwrap();
        assert!(json.contains("sourceId"));
        assert!(json.contains("\"anchor\""));
        assert!(json.contains("sourceSha256"));
        let back: Excerpt = serde_json::from_str(&json).unwrap();
        assert_eq!(with_sha, back);

        // sha omitted when None
        let no_sha = Excerpt::new(
            "e2",
            SourceId::new("s1"),
            anchor("clausola", "7.2"),
            "q",
            None,
        )
        .unwrap();
        let json = serde_json::to_string(&no_sha).unwrap();
        assert!(!json.contains("sourceSha256"));
    }

    fn doc_with_file(id: &str, sha: &str) -> SourceRef {
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
    fn excerpt_sha_matching_the_source_file_is_accepted() {
        let sha = "ab".repeat(32);
        let ex = Excerpt::new(
            "e1",
            SourceId::new("s1"),
            anchor("k", "v"),
            "q",
            Some(sha.clone()),
        )
        .unwrap();
        let ws = Workspace::new_with_evidence(
            client("alfa"),
            matter("m", "alfa"),
            vec![doc_with_file("s1", &sha)],
            vec![],
            vec![ex],
            vec![],
        );
        assert!(ws.is_ok());
    }

    #[test]
    fn excerpt_sha_mismatch_is_rejected() {
        let ex = Excerpt::new(
            "e1",
            SourceId::new("s1"),
            anchor("k", "v"),
            "q",
            Some("cd".repeat(32)),
        )
        .unwrap();
        let r = Workspace::new_with_evidence(
            client("alfa"),
            matter("m", "alfa"),
            vec![doc_with_file("s1", &"ab".repeat(32))],
            vec![],
            vec![ex],
            vec![],
        );
        assert!(matches!(r, Err(WorkspaceError::ExcerptShaMismatch { .. })));
    }

    #[test]
    fn excerpt_sha_on_a_source_without_file_is_rejected() {
        let ex = Excerpt::new(
            "e1",
            SourceId::new("s1"),
            anchor("k", "v"),
            "q",
            Some("ab".repeat(32)),
        )
        .unwrap();
        // s1 is a Norma without a StoredFile → a sha pin is meaningless
        let r = Workspace::new_with_evidence(
            client("alfa"),
            matter("m", "alfa"),
            vec![src("s1", SourceType::Norma)],
            vec![],
            vec![ex],
            vec![],
        );
        assert!(matches!(
            r,
            Err(WorkspaceError::ExcerptShaWithoutFile { .. })
        ));
    }

    #[test]
    fn loaded_excerpt_with_mismatching_sha_is_rejected_via_serde() {
        // serde/RawWorkspace path must enforce the same sha integrity check.
        let json = r#"{
            "client":{"id":"a","name":"A"},
            "matter":{"id":"m","client":"a","title":"t","subject":"s"},
            "sources":[{"id":"s1","kind":"Documento","title":"t","meta":"","file":{"storedName":"s1.pdf","originalName":"C.pdf","byteLen":3,"sha256":"aaaa"}}],
            "manualDossiers":[],
            "excerpts":[{"id":"e1","sourceId":"s1","anchor":{"kind":"k","value":"v"},"quote":"q","sourceSha256":"bbbb"}]
        }"#;
        assert!(serde_json::from_str::<Workspace>(json).is_err());

        // matching sha loads fine
        let ok = r#"{
            "client":{"id":"a","name":"A"},
            "matter":{"id":"m","client":"a","title":"t","subject":"s"},
            "sources":[{"id":"s1","kind":"Documento","title":"t","meta":"","file":{"storedName":"s1.pdf","originalName":"C.pdf","byteLen":3,"sha256":"aaaa"}}],
            "manualDossiers":[],
            "excerpts":[{"id":"e1","sourceId":"s1","anchor":{"kind":"k","value":"v"},"quote":"q","sourceSha256":"aaaa"}]
        }"#;
        assert!(serde_json::from_str::<Workspace>(ok).is_ok());
    }

    #[test]
    fn loaded_excerpt_with_empty_quote_is_rejected() {
        let json = r#"{
            "client":{"id":"a","name":"A"},
            "matter":{"id":"m","client":"a","title":"t","subject":"s"},
            "sources":[{"id":"s1","kind":"Documento","title":"t","meta":""}],
            "manualDossiers":[],
            "excerpts":[{"id":"e1","sourceId":"s1","anchor":{"kind":"k","value":"v"},"quote":"   "}]
        }"#;
        assert!(serde_json::from_str::<Workspace>(json).is_err());
    }
}
