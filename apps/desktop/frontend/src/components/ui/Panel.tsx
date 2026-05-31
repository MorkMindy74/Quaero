import type { ReactNode } from "react";

interface PanelProps {
  title?: ReactNode;
  children?: ReactNode;
  parchment?: boolean;
  className?: string;
}

// Panel: hairline-bordered surface. parchment=true only for document/excerpt/
// citation/genealogy surfaces (Screen Spec v0.2 §7).
export function Panel({ title, children, parchment = false, className = "" }: PanelProps) {
  return (
    <section className={`rounded border border-hairline ${parchment ? "bg-parchment" : "bg-panel"} ${className}`}>
      {title && (
        <header className="border-b border-hairline px-3 py-2 font-mono text-[11px] uppercase tracking-wide text-muted">
          {title}
        </header>
      )}
      <div className="p-3">{children}</div>
    </section>
  );
}
