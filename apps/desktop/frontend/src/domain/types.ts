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

/** Minimal citable reference (Fonte) — not yet an Estratto/Ancora. */
export interface SourceRef {
  id: string;
  kind: SourceType;
  title: string;
  meta: string;
}

/** A "Fascicolo" as a VIEW over sources (ADR-0008), many-to-many. */
export interface DossierView {
  id: string;
  name: string;
  kind: DossierKind;
  sources: string[];
}

export interface Workspace {
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
