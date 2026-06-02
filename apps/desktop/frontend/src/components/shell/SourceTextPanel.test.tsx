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

test("a file whose sha256 differs from the original is refused, nothing stored", async () => {
  vi.mocked(sha256Hex).mockResolvedValue("b".repeat(64)); // mismatch
  const { onSet } = renderPanel({ source: doc("c.pdf"), get: async () => ({ status: "absent" }) });
  await screen.findByTestId("text-layer-status");

  fireEvent.change(screen.getByLabelText("Estrai testo"), { target: { files: [fakeFile([9])] } });

  expect(await screen.findByRole("alert")).toHaveTextContent("non corrisponde all'originale");
  expect(onSet).not.toHaveBeenCalled();
  expect(extractDocumentText).not.toHaveBeenCalled();
});
