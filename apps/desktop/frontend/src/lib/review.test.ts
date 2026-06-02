import { expect, test } from "vitest";
import { reviewRows } from "./review";
import type { Citation, Excerpt, SourceRef, VerificationReport } from "../domain/types";

const sources: SourceRef[] = [
  { id: "s1", kind: "Documento", title: "Contratto.pdf", meta: "" },
];
const excerpts: Excerpt[] = [
  { id: "e1", sourceId: "s1", anchor: { kind: "pagina", value: "7" }, quote: "Il Fornitore può recedere…" },
  { id: "e2", sourceId: "s1", anchor: { kind: "clausola", value: "7.2" }, quote: "…con preavviso di 15 giorni." },
];

test("empty workspace → no rows", () => {
  expect(reviewRows({ sources: [], excerpts: [], citations: [] })).toEqual([]);
});

test("a Citazione yields a coherent row with claim, fonte title and quote", () => {
  const citations: Citation[] = [{ id: "c1", excerptId: "e1", claim: "Recesso ammesso." }];
  const rows = reviewRows({ sources, excerpts: [excerpts[0]], citations });
  expect(rows).toHaveLength(1);
  expect(rows[0]).toMatchObject({
    kind: "citation",
    excerptId: "e1",
    fonte: "Contratto.pdf",
    estratto: "Il Fornitore può recedere…",
    anchor: "pagina 7",
    claim: "Recesso ammesso.",
    esito: "coherent",
  });
});

test("a Citazione whose Estratto has a #9 Warning is flagged as warning", () => {
  const citations: Citation[] = [{ id: "c1", excerptId: "e1", claim: "X" }];
  const verification: VerificationReport = {
    summary: {
      citations: 1, excerpts: 1, documentBackedExcerpts: 1, pinnedExcerpts: 0, warnings: 1, infos: 0,
    },
    findings: [{ severity: "Warning", code: "UnpinnedDocumentExcerpt", excerptId: "e1" }],
  };
  const rows = reviewRows({ sources, excerpts: [excerpts[0]], citations, verification });
  expect(rows[0].esito).toBe("warning");
});

test("an Estratto with no Citazione becomes an 'uncited' row (no claim)", () => {
  const rows = reviewRows({ sources, excerpts: [excerpts[1]], citations: [] });
  expect(rows).toHaveLength(1);
  expect(rows[0]).toMatchObject({ kind: "uncited", excerptId: "e2", claim: null, esito: "uncited" });
});

test("stable order: Citazioni first, then uncited Estratti", () => {
  const citations: Citation[] = [{ id: "c1", excerptId: "e2", claim: "Y" }];
  const rows = reviewRows({ sources, excerpts, citations });
  expect(rows.map((r) => [r.kind, r.excerptId])).toEqual([
    ["citation", "e2"],
    ["uncited", "e1"],
  ]);
});

test("fonte falls back to the sourceId when the Fonte is missing", () => {
  const rows = reviewRows({ sources: [], excerpts: [excerpts[0]], citations: [] });
  expect(rows[0].fonte).toBe("s1");
});

test("a Citazione referencing a missing Estratto is dropped (defensive)", () => {
  const citations: Citation[] = [{ id: "c1", excerptId: "ghost", claim: "Z" }];
  expect(reviewRows({ sources, excerpts: [], citations })).toEqual([]);
});
