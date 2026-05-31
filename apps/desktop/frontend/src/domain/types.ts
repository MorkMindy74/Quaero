// TypeScript mirror of the `quaero-core` domain model (slice #5A).
// The real domain logic lives in Rust (`crates/core/src/domain.rs`); these are
// presentational types only. Variant names match the Rust serde output.

export type SourceType =
  | "Documento"
  | "Norma"
  | "Giurisprudenza"
  | "Dottrina"
  | "Prassi"
  | "Dato"
  | "Nota"
  | "Memoria"
  | "FonteEsterna";

export type DossierKind = "Dynamic" | "Manual";

export interface Client {
  id: string;
  name: string;
}

export interface Matter {
  id: string;
  client: string;
  title: string;
  subject: string;
}

/** Link from a SourceRef to its imported file content on disk (#6). Metadata
 *  only — the bytes live in the desktop blob store, never here. Mirrors
 *  `quaero_core::domain::StoredFile`. */
export interface StoredFile {
  storedName: string;
  originalName: string;
  byteLen: number;
  sha256: string;
}

/** Minimal citable reference (Fonte) — not yet an Estratto/Ancora.
 *  A Documento may carry a `file` link to its imported content (#6). */
export interface SourceRef {
  id: string;
  kind: SourceType;
  title: string;
  meta: string;
  file?: StoredFile;
}

/** A "Fascicolo" as a VIEW over sources (ADR-0008), many-to-many. */
export interface DossierView {
  id: string;
  name: string;
  kind: DossierKind;
  sources: string[];
}

/** Canonical manual Fascicolo — NO `kind`, so canonical state can never
 *  represent a dynamic dossier. Mirrors `quaero_core::domain::ManualDossier`. */
export interface ManualDossier {
  id: string;
  name: string;
  sources: string[];
}

/** Stable, layout-independent locator of an Excerpt within its Fonte (Ancora).
 *  Declarative in #8. Mirrors `quaero_core::domain::Anchor`. */
export interface Anchor {
  kind: string;
  value: string;
}

/** A verifiable portion of a Fonte that can support an Affermazione (ADR-0007).
 *  Mirrors `quaero_core::domain::Excerpt`. */
export interface Excerpt {
  id: string;
  sourceId: string;
  anchor: Anchor;
  quote: string;
  sourceSha256?: string;
}

/** Link between an Affermazione (`claim`) and the Excerpt that supports it.
 *  References an `excerptId` only — never a Fonte directly (ADR-0007).
 *  Mirrors `quaero_core::domain::Citation`. */
export interface Citation {
  id: string;
  claim: string;
  excerptId: string;
}

/** Canonical / persistable state: sources + user-curated manual dossiers +
 *  the anti-hallucination chain (excerpts/citations). Dynamic dossiers are NOT
 *  stored here — they are derived (see WorkspaceView). */
export interface Workspace {
  client: Client;
  matter: Matter;
  sources: SourceRef[];
  manualDossiers: ManualDossier[];
  excerpts?: Excerpt[];
  citations?: Citation[];
}

/** Citation-chain audit (#9). Mirrors `quaero_core::verify`. */
export type Severity = "Info" | "Warning";
export type VerificationCode = "OrphanExcerpt" | "UnpinnedDocumentExcerpt" | "UncitedSource";

export interface Finding {
  severity: Severity;
  code: VerificationCode;
  excerptId?: string;
  sourceId?: string;
  citationId?: string;
}

export interface VerificationSummary {
  citations: number;
  excerpts: number;
  documentBackedExcerpts: number;
  pinnedExcerpts: number;
  warnings: number;
  infos: number;
}

export interface VerificationReport {
  summary: VerificationSummary;
  findings: Finding[];
}

/** Derived, non-canonical view for the UI: dynamic (computed) + manual dossiers,
 *  the canonical excerpts/citations, and the derived citation-chain audit (#9).
 *  Mirrors `quaero_core::domain::WorkspaceView`. */
export interface WorkspaceView {
  client: Client;
  matter: Matter;
  sources: SourceRef[];
  dossiers: DossierView[];
  excerpts: Excerpt[];
  citations: Citation[];
  verification?: VerificationReport;
}

/** Display label for a source type (presentational only). */
export const SOURCE_TYPE_LABEL: Record<SourceType, string> = {
  Documento: "Documento",
  Norma: "Norma",
  Giurisprudenza: "Giurisprudenza",
  Dottrina: "Dottrina",
  Prassi: "Prassi",
  Dato: "Dato",
  Nota: "Nota",
  Memoria: "Memoria",
  FonteEsterna: "Fonte esterna",
};
