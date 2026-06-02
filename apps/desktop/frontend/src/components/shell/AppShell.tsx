import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { TopCommandBar } from "./TopCommandBar";
import { LeftSidebar } from "./LeftSidebar";
import { MainWorkspace } from "./MainWorkspace";
import { RightContextPanel, type TabId } from "./RightContextPanel";
import { StatusStrip } from "./StatusStrip";
import { CommandPalette } from "./CommandPalette";
import { NewMatterDialog } from "./NewMatterDialog";
import { matters, type MockMatter } from "../../mock/data";
import {
  openWorkspace as ipcOpenWorkspace,
  importDocument,
  addExcerpt,
  addCitation,
  exportMarkdown,
  updateExcerpt,
  deleteExcerpt,
  updateCitation,
  deleteCitation,
  getSourceText,
  setSourceText,
  proposeEvidence,
  acceptEvidenceCandidate,
  proposeEvidenceLocal,
  requestEvidenceConsent,
  evidenceProviderKind,
  proposeCitations,
} from "../../lib/ipc";
import type { LocalEvidenceResult } from "../../lib/ipc";
import { useWorkspaces } from "../../lib/useWorkspaces";
import type { WorkspaceView } from "../../domain/types";

/** #6 import cap, mirrors the backend MAX_IMPORT_BYTES (25 MB). */
const MAX_IMPORT_BYTES = 25 * 1024 * 1024;

// AppShell (Screen Spec v0.2, comp 01): owns the 5-region grid.
// #5C: the left sidebar + context panel are wired to the real local persistence
// (create/open/search via #5B IPC). The TopCommandBar / MainWorkspace surfaces
// stay on the #3 mock for now (out of scope here).
export default function AppShell() {
  const { t } = useTranslation();
  const [matter, setMatter] = useState<MockMatter | null>(matters[0]);
  const [paletteOpen, setPaletteOpen] = useState(false);

  const workspaces = useWorkspaces();
  const [open, setOpen] = useState<WorkspaceView | null>(null);
  const [openError, setOpenError] = useState<string | null>(null);
  const [newOpen, setNewOpen] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [addExcerptError, setAddExcerptError] = useState<string | null>(null);
  const [addCitationError, setAddCitationError] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
  // #58: is the local Ollama Evidence provider opted-in AND loopback-valid?
  const [evidenceLocalEnabled, setEvidenceLocalEnabled] = useState(false);
  // #62: active right-panel tab, lifted here so the central guide can jump to it.
  const [rightTab, setRightTab] = useState<TabId>("sources");

  const openById = async (id: string) => {
    setOpenError(null);
    try {
      setOpen(await ipcOpenWorkspace(id));
    } catch {
      setOpen(null);
      setOpenError("matters.errorOpen");
    }
  };

  const handleImport = async (file: File) => {
    setImportError(null);
    if (!open) return;
    if (file.size > MAX_IMPORT_BYTES) {
      setImportError(t("documents.tooLarge"));
      return;
    }
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      setOpen(await importDocument(open.matter.id, file.name, bytes));
    } catch {
      setImportError(t("documents.importError"));
    }
  };

  const handleAddExcerpt = async (args: {
    sourceId: string;
    anchorKind: string;
    anchorValue: string;
    quote: string;
    note?: string;
  }): Promise<boolean> => {
    setAddExcerptError(null);
    if (!open) return false;
    try {
      setOpen(await addExcerpt({ matterId: open.matter.id, ...args }));
      return true;
    } catch {
      setAddExcerptError(t("excerpts.addError"));
      return false;
    }
  };

  const handleAddCitation = async (excerptId: string, claim: string): Promise<boolean> => {
    setAddCitationError(null);
    if (!open) return false;
    try {
      setOpen(await addCitation({ matterId: open.matter.id, excerptId, claim }));
      return true;
    } catch {
      setAddCitationError(t("citations.addError"));
      return false;
    }
  };

  const handleUpdateExcerpt = async (
    excerptId: string,
    args: { sourceId: string; anchorKind: string; anchorValue: string; quote: string; note?: string },
  ): Promise<boolean> => {
    if (!open) return false;
    try {
      setOpen(
        await updateExcerpt({
          matterId: open.matter.id,
          excerptId,
          anchorKind: args.anchorKind,
          anchorValue: args.anchorValue,
          quote: args.quote,
          note: args.note,
        }),
      );
      return true;
    } catch {
      return false;
    }
  };

  const handleDeleteExcerpt = async (excerptId: string): Promise<boolean> => {
    if (!open) return false;
    try {
      setOpen(await deleteExcerpt(open.matter.id, excerptId));
      return true;
    } catch {
      // Refused (e.g. still cited) → the panel shows a clear block message.
      return false;
    }
  };

  const handleUpdateCitation = async (citationId: string, claim: string): Promise<boolean> => {
    if (!open) return false;
    try {
      setOpen(await updateCitation(open.matter.id, citationId, claim));
      return true;
    } catch {
      return false;
    }
  };

  const handleDeleteCitation = async (citationId: string): Promise<boolean> => {
    if (!open) return false;
    try {
      setOpen(await deleteCitation(open.matter.id, citationId));
      return true;
    } catch {
      return false;
    }
  };

  // #55 Evidence Assistant: approving a candidate creates a real Estratto
  // (server-side verified against the text layer) and refreshes the open view.
  const handleAcceptEvidence = async (
    matterId: string,
    sourceId: string,
    anchorKind: string,
    anchorValue: string,
    quote: string,
    note?: string,
  ): Promise<boolean> => {
    if (!open) return false;
    try {
      setOpen(await acceptEvidenceCandidate(matterId, sourceId, anchorKind, anchorValue, quote, note));
      return true;
    } catch {
      // Refused (e.g. quote not in the text layer) → the panel shows a message.
      return false;
    }
  };

  // #58: the panel signals consent (after its dialog); the backend issues a
  // one-shot, source-bound token which the local proposal then consumes. The
  // token never lives in the panel.
  const handleProposeEvidenceLocal = async (
    matterId: string,
    sourceId: string,
  ): Promise<LocalEvidenceResult> => {
    const token = await requestEvidenceConsent(matterId, sourceId);
    return proposeEvidenceLocal(matterId, sourceId, token);
  };

  const handleExportMarkdown = async (): Promise<boolean> => {
    setExportError(null);
    if (!open) return false;
    try {
      const md = await exportMarkdown(open.matter.id);
      // Download via Blob in the webview — no Rust file write, no save dialog.
      const blob = new Blob([md], { type: "text/markdown" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${open.matter.id}.md`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      URL.revokeObjectURL(url);
      return true;
    } catch {
      setExportError(t("export.error"));
      return false;
    }
  };

  useEffect(() => {
    // Reflect the backend's honest provider posture (opt-in + loopback check).
    evidenceProviderKind()
      .then((kind) => setEvidenceLocalEnabled(kind === "ollamaLocal"))
      .catch(() => setEvidenceLocalEnabled(false));
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="grid h-screen grid-rows-[auto_minmax(0,1fr)_auto] bg-background text-ink">
      <TopCommandBar matter={matter} onSelectMatter={setMatter} onOpenPalette={() => setPaletteOpen(true)} />
      <div className="grid min-h-0 grid-cols-[260px_minmax(0,1fr)_348px]">
        <LeftSidebar
          items={workspaces.items}
          loading={workspaces.loading}
          error={workspaces.error}
          query={workspaces.query}
          onQueryChange={workspaces.setQuery}
          onOpen={(id) => void openById(id)}
          onNew={() => setNewOpen(true)}
          activeId={open?.matter.id ?? null}
        />
        <MainWorkspace
          matter={matter}
          workspace={open ?? undefined}
          onGoToTab={setRightTab}
          onExport={handleExportMarkdown}
        />
        <RightContextPanel
          workspace={open ?? undefined}
          onImportFile={open ? (file) => void handleImport(file) : undefined}
          importError={importError}
          onAddExcerpt={open ? handleAddExcerpt : undefined}
          addExcerptError={addExcerptError}
          onAddCitation={open ? handleAddCitation : undefined}
          addCitationError={addCitationError}
          onProposeCitations={open ? () => proposeCitations(open.matter.id) : undefined}
          onExportMarkdown={open ? handleExportMarkdown : undefined}
          exportError={exportError}
          onUpdateExcerpt={open ? handleUpdateExcerpt : undefined}
          onDeleteExcerpt={open ? handleDeleteExcerpt : undefined}
          onUpdateCitation={open ? handleUpdateCitation : undefined}
          onDeleteCitation={open ? handleDeleteCitation : undefined}
          onGetSourceText={open ? getSourceText : undefined}
          onSetSourceText={open ? setSourceText : undefined}
          onProposeEvidence={open ? proposeEvidence : undefined}
          onAcceptEvidence={open ? handleAcceptEvidence : undefined}
          evidenceLocalEnabled={evidenceLocalEnabled}
          onProposeEvidenceLocal={open ? handleProposeEvidenceLocal : undefined}
          tab={rightTab}
          onTabChange={setRightTab}
        />
      </div>
      <StatusStrip />
      {paletteOpen && <CommandPalette onClose={() => setPaletteOpen(false)} />}
      {newOpen && (
        <NewMatterDialog
          error={workspaces.error}
          onClose={() => setNewOpen(false)}
          onCreate={async (clientName, title, subject) => {
            const summary = await workspaces.createMatter(clientName, title, subject);
            if (summary) await openById(summary.id);
            return summary;
          }}
        />
      )}
      {openError && (
        <div role="alert" className="sr-only">
          {openError}
        </div>
      )}
    </div>
  );
}
