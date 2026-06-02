import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import "../../i18n";
import type { SourceRef } from "../../domain/types";
import type { EvidenceCandidate } from "../../lib/ipc";
import { EvidenceCandidatesPanel } from "./EvidenceCandidatesPanel";

const SOURCE: SourceRef = {
  id: "s1",
  kind: "Documento",
  title: "Contratto.pdf",
  meta: "",
  file: { storedName: "doc-1-0.pdf", originalName: "Contratto.pdf", byteLen: 3, sha256: "a".repeat(64) },
};

const CANDS: EvidenceCandidate[] = [
  { quote: "Articolo 1. Il conduttore è tenuto.", anchorKind: "paragrafo", anchorValue: "1", reason: "motivo uno" },
  { quote: "Articolo 2. Recesso.", anchorKind: "paragrafo", anchorValue: "2", reason: "motivo due" },
];

function renderPanel(opts: {
  propose?: () => Promise<EvidenceCandidate[]>;
  accept?: () => Promise<boolean>;
}) {
  const onPropose = vi.fn(async (_m: string, _s: string) => (opts.propose ? opts.propose() : CANDS));
  const onAccept = vi.fn(
    async (_m: string, _s: string, _ak: string, _av: string, _q: string, _n?: string) =>
      (opts.accept ? opts.accept() : true),
  );
  render(
    <EvidenceCandidatesPanel matterId="m" source={SOURCE} onPropose={onPropose} onAccept={onAccept} />,
  );
  return { onPropose, onAccept };
}

test("proposes candidates (not saved) on demand", async () => {
  renderPanel({});
  // No candidates before the lawyer asks.
  expect(screen.queryAllByTestId("evidence-candidate")).toHaveLength(0);
  fireEvent.click(screen.getByRole("button", { name: "Proponi Evidence" }));
  await waitFor(() => expect(screen.getAllByTestId("evidence-candidate")).toHaveLength(2));
  expect(screen.getByText("“Articolo 1. Il conduttore è tenuto.”")).toBeInTheDocument();
  // Both start as "non salvato".
  const badges = screen.getAllByTestId("evidence-candidate-status");
  expect(badges[0]).toHaveTextContent("non salvato");
});

test("approving a candidate creates a real Estratto with the right args", async () => {
  const { onAccept } = renderPanel({});
  fireEvent.click(screen.getByRole("button", { name: "Proponi Evidence" }));
  await screen.findAllByTestId("evidence-candidate");

  fireEvent.click(screen.getAllByRole("button", { name: "Approva → crea Estratto" })[0]);

  await waitFor(() =>
    expect(onAccept).toHaveBeenCalledWith(
      "m",
      "s1",
      "paragrafo",
      "1",
      "Articolo 1. Il conduttore è tenuto.",
      "motivo uno",
    ),
  );
  await waitFor(() =>
    expect(screen.getAllByTestId("evidence-candidate-status")[0]).toHaveTextContent("Estratto creato"),
  );
});

test("discarding a candidate removes it without persisting", async () => {
  const { onAccept } = renderPanel({});
  fireEvent.click(screen.getByRole("button", { name: "Proponi Evidence" }));
  await screen.findAllByTestId("evidence-candidate");

  fireEvent.click(screen.getAllByRole("button", { name: "Scarta" })[0]);

  await waitFor(() => expect(screen.getAllByTestId("evidence-candidate")).toHaveLength(1));
  expect(onAccept).not.toHaveBeenCalled();
});

test("double-clicking Approva creates the Estratto only once (re-entrancy guard)", async () => {
  let resolveAccept!: (v: boolean) => void;
  const { onAccept } = renderPanel({
    accept: () =>
      new Promise<boolean>((r) => {
        resolveAccept = r;
      }),
  });
  fireEvent.click(screen.getByRole("button", { name: "Proponi Evidence" }));
  await screen.findAllByTestId("evidence-candidate");

  const approveBtn = screen.getAllByRole("button", { name: "Approva → crea Estratto" })[0];
  fireEvent.click(approveBtn); // first approval, IPC in flight (pending)
  fireEvent.click(approveBtn); // second click before it resolves — must be ignored

  resolveAccept(true);
  await waitFor(() =>
    expect(screen.getAllByTestId("evidence-candidate-status")[0]).toHaveTextContent("Estratto creato"),
  );
  expect(onAccept).toHaveBeenCalledTimes(1);
});

test("a rejected approval (quote not in text) surfaces an error, nothing created", async () => {
  renderPanel({ accept: async () => false });
  fireEvent.click(screen.getByRole("button", { name: "Proponi Evidence" }));
  await screen.findAllByTestId("evidence-candidate");

  fireEvent.click(screen.getAllByRole("button", { name: "Approva → crea Estratto" })[0]);

  expect(await screen.findByRole("alert")).toHaveTextContent("non presente nella Fonte");
  expect(screen.getAllByTestId("evidence-candidate-status")[0]).toHaveTextContent("non salvato");
});

test("empty proposal (no text layer) shows a hint", async () => {
  renderPanel({ propose: async () => [] });
  fireEvent.click(screen.getByRole("button", { name: "Proponi Evidence" }));
  expect(await screen.findByText(/estrai prima il testo/i)).toBeInTheDocument();
  expect(screen.queryAllByTestId("evidence-candidate")).toHaveLength(0);
});
