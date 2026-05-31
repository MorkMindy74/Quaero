import type { ReactNode } from "react";

interface TabButtonProps {
  active?: boolean;
  onClick?: () => void;
  children: ReactNode;
  count?: number;
  alert?: boolean;
}

export function TabButton({ active = false, onClick, children, count, alert = false }: TabButtonProps) {
  return (
    <button
      role="tab"
      aria-selected={active}
      onClick={onClick}
      className={`inline-flex items-center whitespace-nowrap rounded px-2 py-1 text-xs transition-colors ${
        active ? "bg-panel text-ink" : "text-muted hover:text-ink"
      }`}
    >
      <span>{children}</span>
      {typeof count === "number" && (
        <span aria-hidden className="ml-1 rounded-full bg-panel-2 px-1 text-[10px] text-muted">
          {count}
        </span>
      )}
      {alert && (
        <span aria-hidden className="ml-1 font-bold text-accent-warning" title="alert">
          !
        </span>
      )}
    </button>
  );
}
