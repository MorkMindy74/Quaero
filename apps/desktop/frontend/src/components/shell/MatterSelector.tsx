import { useState } from "react";
import { useTranslation } from "react-i18next";
import { matters, type MockMatter } from "../../mock/data";

interface MatterSelectorProps {
  selected: MockMatter | null;
  onSelect: (matter: MockMatter) => void;
}

// Spec §3 comp 08: shows + switches the open Pratica. Selection drives the title only.
export function MatterSelector({ selected, onSelect }: MatterSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  return (
    <div className="relative">
      <button
        onClick={() => setOpen((v) => !v)}
        aria-haspopup="listbox"
        aria-expanded={open}
        className="inline-flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-panel-2"
      >
        <span className="text-muted">{t("matter.label")}:</span>
        <span>{selected ? selected.title : t("matter.select")}</span>
        <span className="text-muted">▾</span>
      </button>
      {open && (
        <ul role="listbox" className="absolute z-20 mt-1 w-64 rounded border border-hairline bg-panel py-1 shadow-md">
          {matters.map((matter) => (
            <li key={matter.id} role="option" aria-selected={selected?.id === matter.id}>
              <button
                onClick={() => {
                  onSelect(matter);
                  setOpen(false);
                }}
                className="block w-full px-3 py-1.5 text-left text-sm hover:bg-panel-2"
              >
                {matter.title}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
