import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import "../../i18n";
import type { Excerpt } from "../../domain/types";
import type { CitationCandidate } from "../../lib/ipc";
import { CitationCandidatesPanel } from "./CitationCandidatesPanel";

const EXCERPTS: Excerpt[] = [
  { id: "e1", sourceId: "s1", anchor: { kind: "pagina", value: "8" }, quote: "Il conduttore è tenuto." },
  { id: "e2", sourceId: "s1", anchor: { kind: "clausola", value: "7.2" }, quote: "Recesso del fornitore." },
];

function renderPanel(opts: {
  propose?: () => Promise<CitationCandidate[]>;
  accept?: () => Promise<boolean>;
}) {
  const cands: CitationCandidate[] = [
    { excerptId: "e1", claim: "Affermazione su e1.", reason: "motivo", valid: true },
    { excerptId: "ghost", claim: "Affermazione orfana.", reason: "motivo", valid: false },
  ];
  const onPropose = vi.fn(async () => (opts.propose ? opts.propose() : cands));
  const onAccept = vi.fn(async (_e: string, _c: string) => (opts.accept ? opts.accept() : true));
  render(<CitationCandidatesPanel excerpts={EXCERPTS} onPropose={onPropose} onAccept={onAccept} />);
  return { onPropose, onAccept };
}

test("proposes citation candidates (not saved), valid + invalid", async () => {
  renderPanel({});
  expect(screen.queryAllByTestId("citation-candidate")).toHaveLength(0);
  fireEvent.click(screen.getByRole("button", { name: "Proponi Citazioni" }));
  await waitFor(() => expect(screen.getAllByTestId("citation-candidate")).toHaveLength(2));
  const statuses = screen.getAllByTestId("citation-candidate-status");
  expect(statuses[0]).toHaveTextContent("non salvata");
  expect(statuses[1]).toHaveTextContent("non valida");
  // The linked excerpt quote is shown for the valid one.
  expect(screen.getByText("“Il conduttore è tenuto.”")).toBeInTheDocument();
});

test("only the valid candidate is approvable; approve uses the canonical path", async () => {
  const { onAccept } = renderPanel({});
  fireEvent.click(screen.getByRole("button", { name: "Proponi Citazioni" }));
  await screen.findAllByTestId("citation-candidate");

  // Exactly one approve button (the invalid candidate has none).
  const approveBtns = screen.getAllByRole("button", { name: "Approva → crea Citazione" });
  expect(approveBtns).toHaveLength(1);

  fireEvent.click(approveBtns[0]);
  await waitFor(() => expect(onAccept).toHaveBeenCalledWith("e1", "Affermazione su e1."));
  await waitFor(() =>
    expect(screen.getAllByTestId("citation-candidate-status")[0]).toHaveTextContent(
      "Citazione creata",
    ),
  );
});

test("discarding a candidate removes it without persisting", async () => {
  const { onAccept } = renderPanel({});
  fireEvent.click(screen.getByRole("button", { name: "Proponi Citazioni" }));
  await screen.findAllByTestId("citation-candidate");

  fireEvent.click(screen.getAllByRole("button", { name: "Scarta" })[0]);
  await waitFor(() => expect(screen.getAllByTestId("citation-candidate")).toHaveLength(1));
  expect(onAccept).not.toHaveBeenCalled();
});

test("double-clicking Approva creates the Citation only once (re-entrancy guard)", async () => {
  let resolveAccept!: (v: boolean) => void;
  const { onAccept } = renderPanel({
    accept: () =>
      new Promise<boolean>((r) => {
        resolveAccept = r;
      }),
  });
  fireEvent.click(screen.getByRole("button", { name: "Proponi Citazioni" }));
  await screen.findAllByTestId("citation-candidate");

  const btn = screen.getByRole("button", { name: "Approva → crea Citazione" });
  fireEvent.click(btn);
  fireEvent.click(btn);
  resolveAccept(true);
  await waitFor(() =>
    expect(screen.getAllByTestId("citation-candidate-status")[0]).toHaveTextContent(
      "Citazione creata",
    ),
  );
  expect(onAccept).toHaveBeenCalledTimes(1);
});

test("a stale candidate is not approvable after the excerpt set changes (re-check)", async () => {
  const cand: CitationCandidate[] = [
    { excerptId: "e1", claim: "Affermazione su e1.", reason: "m", valid: true },
  ];
  const onPropose = vi.fn(async () => cand);
  const onAccept = vi.fn(async (_e: string, _c: string) => true);
  const { rerender } = render(
    <CitationCandidatesPanel excerpts={EXCERPTS} onPropose={onPropose} onAccept={onAccept} />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Proponi Citazioni" }));
  await screen.findByTestId("citation-candidate");

  // The open Pratica's excerpt set changes: e1 is no longer present.
  rerender(
    <CitationCandidatesPanel excerpts={[EXCERPTS[1]]} onPropose={onPropose} onAccept={onAccept} />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Approva → crea Citazione" }));

  await waitFor(() =>
    expect(screen.getByTestId("citation-candidate-status")).toHaveTextContent("non valida"),
  );
  expect(onAccept).not.toHaveBeenCalled();
});

test("empty proposal shows a hint", async () => {
  renderPanel({ propose: async () => [] });
  fireEvent.click(screen.getByRole("button", { name: "Proponi Citazioni" }));
  expect(await screen.findByText(/Nessun Estratto senza Citazione/i)).toBeInTheDocument();
  expect(screen.queryAllByTestId("citation-candidate")).toHaveLength(0);
});
