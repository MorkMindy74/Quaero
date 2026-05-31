import { useEffect, useState } from "react";
import { TopCommandBar } from "./TopCommandBar";
import { LeftSidebar } from "./LeftSidebar";
import { MainWorkspace } from "./MainWorkspace";
import { RightContextPanel } from "./RightContextPanel";
import { StatusStrip } from "./StatusStrip";
import { CommandPalette } from "./CommandPalette";
import { matters, type MockMatter } from "../../mock/data";

// AppShell (Screen Spec v0.2, comp 01): owns the 5-region grid. Default state =
// "Matter workspace open" — a matter is open, Conversation mode, Sources tab.
export default function AppShell() {
  const [matter, setMatter] = useState<MockMatter | null>(matters[0]);
  const [paletteOpen, setPaletteOpen] = useState(false);

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
        <LeftSidebar matter={matter} onSelectMatter={setMatter} />
        <MainWorkspace matter={matter} />
        <RightContextPanel />
      </div>
      <StatusStrip />
      {paletteOpen && <CommandPalette onClose={() => setPaletteOpen(false)} />}
    </div>
  );
}
