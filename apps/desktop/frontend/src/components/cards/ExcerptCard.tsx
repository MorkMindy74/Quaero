import { Badge } from "../ui";
import type { MockExcerpt } from "../../mock/data";

interface ExcerptCardProps {
  excerpt: MockExcerpt;
}

// Parchment leaf (Spec §3 comp 11): a quoted Estratto anchored to a source.
export function ExcerptCard({ excerpt }: ExcerptCardProps) {
  return (
    <div className="rounded border border-hairline border-l-2 border-l-accent-source bg-parchment p-2">
      <p className="text-sm italic">“{excerpt.quote}”</p>
      <div className="mt-1 flex items-center justify-between gap-2 font-mono text-xs text-muted">
        <span className="truncate">{excerpt.source}</span>
        <Badge tone="source">{excerpt.anchor}</Badge>
      </div>
    </div>
  );
}
