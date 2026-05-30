import type { ReactNode } from "react";

interface ListRowProps {
  title: ReactNode;
  meta?: ReactNode;
  trailing?: ReactNode;
  active?: boolean;
  onClick?: () => void;
}

export function ListRow({ title, meta, trailing, active = false, onClick }: ListRowProps) {
  return (
    <button
      onClick={onClick}
      className={`flex w-full items-center justify-between gap-2 rounded px-2 py-1.5 text-left transition-colors ${
        active ? "bg-panel-2" : "hover:bg-panel-2"
      }`}
    >
      <span className="min-w-0">
        <span className="block truncate text-sm">{title}</span>
        {meta && <span className="block truncate text-xs text-muted">{meta}</span>}
      </span>
      {trailing}
    </button>
  );
}
