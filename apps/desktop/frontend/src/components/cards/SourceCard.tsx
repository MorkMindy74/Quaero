import { Badge } from "../ui";
import type { MockSource } from "../../mock/data";

interface SourceCardProps {
  source: MockSource;
  selected?: boolean;
  onSelect?: () => void;
}

// Parchment leaf (Spec §3 comp 10). Presentational, mock props only.
export function SourceCard({ source, selected = false, onSelect }: SourceCardProps) {
  return (
    <button
      onClick={onSelect}
      className={`block w-full rounded border bg-parchment p-2 text-left transition-colors ${
        selected ? "border-accent-source" : "border-hairline hover:border-accent-source"
      }`}
    >
      <div className="flex items-center justify-between">
        <Badge tone="source">{source.type}</Badge>
        {source.verified && (
          <span title="verificato" aria-label="verificato" className="h-2 w-2 rounded-full bg-accent-verified" />
        )}
      </div>
      <div className="mt-1 truncate text-sm">{source.title}</div>
      <div className="font-mono text-xs text-muted">{source.meta}</div>
    </button>
  );
}
