import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import "../../i18n";
import type { SourceRef } from "../../domain/types";

// Mock the extraction orchestrator (keeps pdf.js out of jsdom) and sha256Hex.
vi.mock("../../lib/extractText", () => ({ extractDocumentText: vi.fn() }));
vi.mock("../../lib/textLayer", async (importOriginal) => {
  const orig = await importOriginal<typeof import("../../lib/textLayer")>();
  return { ...orig, sha256Hex: vi.fn() };
});

import { SourceTextPanel } from "./SourceTextPanel";
import { extractDocumentText } from "../../lib/extractText";
import type { ExtractOutcome } from "../../lib/extractText";
import { sha256Hex } from "../../lib/textLayer";
import type { SourceText } from "../../lib/ipc";

const SHA = "a".repeat(64);

// jsdom's File lacks a working arrayBuffer(); stub a minimal File-like object.
function fakeFile(bytes = [1, 2, 3]): File {
  return {
    name: "c.pdf",
    arrayBuffer: async () => new Uint8Array(bytes).buffer,
  } as unknown as File;
}

function doc(originalName: string): SourceRef {
  return {
    id: "s1",
    kind: "Documento",
    title: originalName,
    meta: "",
    file: { storedName: "doc-1-0.pdf", originalName, byteLen: 3, sha256: SHA },
  };
}

beforeEach(() => {
  vi.mocked(extractDocumentText).mockReset();
  vi.mocked(sha256Hex).mockReset();
});
afterEach(() => vi.clearAllMocks());

function renderPanel(opts: {
  source: SourceRef;
  get: () => Promise<SourceText>;
  set?: () => Promise<SourceText>;
}) {
  const onGet = vi.fn(async (_mid: string, _sid: string) => opts.get());
  const onSet = vi.fn(
    async (_mid: string, _sid: string, _sha: string, _text: string) =>
      (opts.set ? opts.set() : ({ status: "available", text: "" } as SourceText)),
  );
  render(<SourceTextPanel matterId="m" source={opts.source} onGet={onGet} onSet={onSet} />);
  return { onGet, onSet };
}

test("renders the persisted 'available' state and toggles a read-only preview", async () => {
  renderPanel({ source: doc("c.pdf"), get: async () => ({ status: "available", text: "Articolo 1." }) });
  expect(await screen.findByTestId("text-layer-status")).toHaveTextContent("Testo disponibile");
  fireEvent.click(screen.getByRole("button", { name: "Mostra testo" }));
  expect(screen.getByTestId("text-layer-preview")).toHaveTextContent("Articolo 1.");
});

test("renders 'absent' with an extract action", async () => {
  renderPanel({ source: doc("c.pdf"), get: async () => ({ status: "absent" }) });
  expect(await screen.findByTestId("text-layer-status")).toHaveTextContent("Testo non disponibile");
  expect(screen.getByLabelText("Estrai testo")).toBeInTheDocument();
});

test("renders 'empty' state (supported file, no useful text)", async () => {
  renderPanel({ source: doc("scan.pdf"), get: async () => ({ status: "empty" }) });
  expect(await screen.findByTestId("text-layer-status")).toHaveTextContent("Testo vuoto");
});

test("derives 'unsupported' from the filename without querying the store", async () => {
  const get = vi.fn(async (_mid: string, _sid: string) => ({ status: "absent" }) as SourceText);
  render(<SourceTextPanel matterId="m" source={doc("atto.docx")} onGet={get} onSet={vi.fn()} />);
  expect(await screen.findByTestId("text-layer-status")).toHaveTextContent("Formato non supportato");
  expect(get).not.toHaveBeenCalled();
  expect(screen.queryByLabelText("Estrai testo")).not.toBeInTheDocument();
});

test("a corrupt sidecar (store error) shows 'failed', not 'absent'", async () => {
  renderPanel({
    source: doc("c.pdf"),
    get: async () => {
      throw new Error("corrupt sidecar");
    },
  });
  expect(await screen.findByTestId("text-layer-status")).toHaveTextContent("Estrazione fallita");
});

test("extracting a matching file stores the text and shows 'available'", async () => {
  vi.mocked(sha256Hex).mockResolvedValue(SHA); // matches the source digest
  vi.mocked(extractDocumentText).mockResolvedValue({ kind: "text", text: "estratto ok" });
  const { onSet } = renderPanel({
    source: doc("c.pdf"),
    get: async () => ({ status: "absent" }),
    set: async () => ({ status: "available", text: "estratto ok" }),
  });
  await screen.findByTestId("text-layer-status");

  fireEvent.change(screen.getByLabelText("Estrai testo"), { target: { files: [fakeFile()] } });

  await waitFor(() =>
    expect(screen.getByTestId("text-layer-status")).toHaveTextContent("Testo disponibile"),
  );
  expect(onSet).toHaveBeenCalledWith("m", "s1", SHA, "estratto ok");
});

test("stale extraction never repaints a newly-selected Fonte (BLOCKER regression)", async () => {
  const SHA_A = "a".repeat(64);
  const SHA_B = "b".repeat(64);
  const srcA: SourceRef = {
    id: "s1",
    kind: "Documento",
    title: "A.pdf",
    meta: "",
    file: { storedName: "a.pdf", originalName: "A.pdf", byteLen: 3, sha256: SHA_A },
  };
  const srcB: SourceRef = {
    id: "s2",
    kind: "Documento",
    title: "B.pdf",
    meta: "",
    file: { storedName: "b.pdf", originalName: "B.pdf", byteLen: 3, sha256: SHA_B },
  };

  vi.mocked(sha256Hex).mockResolvedValue(SHA_A); // matches A, lets extraction start
  let resolveExtract!: (v: ExtractOutcome) => void;
  vi.mocked(extractDocumentText).mockReturnValue(
    new Promise<ExtractOutcome>((res) => {
      resolveExtract = res;
    }),
  );
  const onGet = vi.fn(async (_mid: string, _sid: string) => ({ status: "absent" }) as SourceText);
  const onSet = vi.fn(
    async (_mid: string, _sid: string, _sha: string, _text: string) =>
      ({ status: "available", text: "SEGRETO DI A" }) as SourceText,
  );

  const { rerender } = render(
    <SourceTextPanel matterId="m" source={srcA} onGet={onGet} onSet={onSet} />,
  );
  await screen.findByTestId("text-layer-status");

  // Start extraction on Fonte A; it stays pending.
  fireEvent.change(screen.getByLabelText("Estrai testo"), { target: { files: [fakeFile()] } });
  await waitFor(() => expect(extractDocumentText).toHaveBeenCalled());

  // The lawyer selects Fonte B before A's extraction resolves (same instance).
  rerender(<SourceTextPanel matterId="m" source={srcB} onGet={onGet} onSet={onSet} />);
  await waitFor(() =>
    expect(screen.getByTestId("text-layer-status")).toHaveTextContent("Testo non disponibile"),
  );

  // A's extraction now resolves: text is still persisted to A (correct target)…
  resolveExtract({ kind: "text", text: "SEGRETO DI A" });
  await waitFor(() => expect(onSet).toHaveBeenCalledWith("m", "s1", SHA_A, "SEGRETO DI A"));

  // …but B's visible panel must NOT be repainted with A's text/status.
  expect(screen.getByTestId("text-layer-status")).toHaveTextContent("Testo non disponibile");
  expect(screen.queryByTestId("text-layer-preview")).not.toBeInTheDocument();
  expect(screen.queryByText("SEGRETO DI A")).not.toBeInTheDocument();
});

test("a file whose sha256 differs from the original is refused, nothing stored", async () => {
  vi.mocked(sha256Hex).mockResolvedValue("b".repeat(64)); // mismatch
  const { onSet } = renderPanel({ source: doc("c.pdf"), get: async () => ({ status: "absent" }) });
  await screen.findByTestId("text-layer-status");

  fireEvent.change(screen.getByLabelText("Estrai testo"), { target: { files: [fakeFile([9])] } });

  expect(await screen.findByRole("alert")).toHaveTextContent("non corrisponde all'originale");
  expect(onSet).not.toHaveBeenCalled();
  expect(extractDocumentText).not.toHaveBeenCalled();
});
