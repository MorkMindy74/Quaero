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

/** Canonical / persistable state: sources + user-curated manual dossiers only.
 *  Dynamic dossiers are NOT stored here — they are derived (see WorkspaceView). */
export interface Workspace {
  client: Client;
  matter: Matter;
  sources: SourceRef[];
  manualDossiers: ManualDossier[];
}

/** Derived, non-canonical view for the UI: dynamic (computed) + manual dossiers.
 *  Mirrors `quaero_core::domain::WorkspaceView`. */
export interface WorkspaceView {
  client: Client;
  matter: Matter;
  sources: SourceRef[];
  dossiers: DossierView[];
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
