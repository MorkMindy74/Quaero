import type { ReactNode } from "react";

interface TabButtonProps {
  active?: boolean;
  onClick?: () => void;
  children: ReactNode;
}

export function TabButton({ active = false, onClick, children }: TabButtonProps) {
  return (
    <button
      role="tab"
      aria-selected={active}
      onClick={onClick}
      className={`whitespace-nowrap px-2 py-1 text-xs transition-colors ${
        active ? "border-b-2 border-accent-source text-ink" : "border-b-2 border-transparent text-muted hover:text-ink"
      }`}
    >
      {children}
    </button>
  );
}
