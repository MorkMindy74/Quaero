// Static mock data for the #3 shell. No domain logic, no persistence, no backend.
// These are presentational fixtures only (Screen Spec v0.2).
import type { WorkspaceView } from "../domain/types";

export interface MockMatter {
  id: string;
  title: string;
  meta: string;
}

export const matters: MockMatter[] = [
  { id: "rossi", title: "Rossi c. Bianchi", meta: "Inadempimento contrattuale · Fascicolo 04" },
  { id: "conti", title: "Eredità Conti", meta: "Successione · Fascicolo 02" },
  { id: "appalto", title: "Appalto SRL", meta: "Contratto d'appalto · Fascicolo 07" },
];

export const recentMatters: MockMatter[] = [matters[0], matters[1]];
export const pinnedMatters: MockMatter[] = [matters[2]];

export interface MockSource {
  id: string;
  type: string;
  title: string;
  meta: string;
  verified: boolean;
}

export const sources: MockSource[] = [
  { id: "s1", type: "Documento", title: "Contratto Rossi-Bianchi.pdf", meta: "pag. 10–14", verified: true },
  { id: "s2", type: "Norma", title: "Art. 1453 c.c.", meta: "Risoluzione per inadempimento", verified: true },
  { id: "s3", type: "Giurisprudenza", title: "Cass. civ. 12345/2024", meta: "massima", verified: true },
];

export interface MockExcerpt {
  id: string;
  quote: string;
  source: string;
  anchor: string;
}

export const excerpts: MockExcerpt[] = [
  { id: "e1", quote: "Il Fornitore potrà recedere con preavviso di quindici giorni…", source: "Contratto Rossi-Bianchi.pdf", anchor: "pag. 8 · clausola 7.2" },
  { id: "e2", quote: "Nei contratti con prestazioni corrispettive…", source: "Art. 1453 c.c.", anchor: "comma 1" },
];

export interface MockReasoningStep {
  id: string;
  index: number;
  claim: string;
  sources: number;
  verified: boolean;
}

export const reasoningSteps: MockReasoningStep[] = [
  { id: "r1", index: 1, claim: "La clausola 7.2 consente il recesso con preavviso di 15 giorni.", sources: 1, verified: true },
  { id: "r2", index: 2, claim: "L'art. 1453 c.c. richiede un inadempimento di non scarsa importanza.", sources: 1, verified: true },
  { id: "r3", index: 3, claim: "Il recesso unilaterale espone Alfa a perdita di continuità.", sources: 2, verified: false },
];

export interface MockGenealogyNode {
  id: string;
  label: string;
  kind: "fonte" | "ai" | "human";
}

export const genealogyNodes: MockGenealogyNode[] = [
  { id: "g1", label: "Fonte", kind: "fonte" },
  { id: "g2", label: "Bozza v1", kind: "ai" },
  { id: "g3", label: "Bozza v2", kind: "ai" },
  { id: "g4", label: "Documento", kind: "human" },
];

export interface MockMemoryItem {
  id: string;
  key: string;
  note: string;
}

export const memoryItems: MockMemoryItem[] = [
  { id: "m1", key: "Strategia", note: "Cliente disponibile a transigere." },
  { id: "m2", key: "Scadenza", note: "Verificare la prescrizione entro 30 giorni." },
];

export interface MockNormativeVersion {
  id: string;
  date: string;
  label: string;
  current: boolean;
}

export interface MockNormativeGenealogy {
  norma: string;
  status: string;
  timeline: MockNormativeVersion[];
  linkedSources: string[];
}

// Mock only — illustrative labels, no scraping, no real normative data.
export const normativeGenealogy: MockNormativeGenealogy = {
  norma: "Art. 1375 c.c.",
  status: "versione vigente",
  timeline: [
    { id: "nv1", date: "1942", label: "testo originario", current: false },
    { id: "nv2", date: "1985", label: "modifica", current: false },
    { id: "nv3", date: "oggi", label: "vigente", current: true },
  ],
  linkedSources: ["Art. 1175 c.c.", "Cass. civ. 12345/2024"],
};

// Derived view mirroring `quaero_core::domain::sample_workspace().view()`
// (dynamic dossiers + manual). The derivation logic is tested in Rust; here the
// already-derived result is static (no domain logic in the frontend).
export const workspaceView: WorkspaceView = {
  client: { id: "alfa", name: "Alfa S.r.l." },
  matter: {
    id: "rossi-bianchi",
    client: "alfa",
    title: "Rossi c. Bianchi",
    subject: "Inadempimento contrattuale",
  },
  sources: [
    { id: "s1", kind: "Documento", title: "Contratto Rossi-Bianchi.pdf", meta: "pag. 10–14" },
    { id: "s2", kind: "Norma", title: "Art. 1453 c.c.", meta: "Risoluzione per inadempimento" },
    { id: "s3", kind: "Giurisprudenza", title: "Cass. civ. 12345/2024", meta: "massima" },
    { id: "s4", kind: "Nota", title: "Cliente disponibile a transigere", meta: "" },
  ],
  dossiers: [
    { id: "dyn-documento", name: "Documenti", kind: "Dynamic", sources: ["s1"] },
    { id: "dyn-norma", name: "Norme", kind: "Dynamic", sources: ["s2"] },
    { id: "dyn-giurisprudenza", name: "Giurisprudenza", kind: "Dynamic", sources: ["s3"] },
    { id: "dyn-nota", name: "Note", kind: "Dynamic", sources: ["s4"] },
    { id: "man-produzione-avversaria", name: "Produzione avversaria", kind: "Manual", sources: ["s1", "s3"] },
  ],
  // #8: the demo fallback shows NO mock excerpts/citations — real Estratti come
  // only from an opened workspace; otherwise the Estratti tab is an empty state.
  excerpts: [],
  citations: [],
};

export interface MockAgentRow {
  id: string;
  label: string;
  status: "idle" | "running" | "done";
}

export const agentActivity: MockAgentRow[] = [
  { id: "a1", label: "Analista documentale", status: "done" },
  { id: "a2", label: "Ricercatore normativo", status: "done" },
  { id: "a3", label: "Verificatore citazioni", status: "running" },
];
