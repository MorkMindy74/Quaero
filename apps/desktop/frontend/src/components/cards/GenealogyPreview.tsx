import type { MockGenealogyNode } from "../../mock/data";

interface GenealogyPreviewProps {
  nodes: MockGenealogyNode[];
}

// Parchment leaf (Spec §3 comp 13): compact lineage Fonte → Bozza → Documento.
export function GenealogyPreview({ nodes }: GenealogyPreviewProps) {
  return (
    <div className="flex flex-wrap items-center gap-1 rounded border border-hairline bg-parchment p-2 font-mono text-xs">
      {nodes.map((node, i) => (
        <span key={node.id} className="flex items-center gap-1">
          <span
            className={
              node.kind === "human"
                ? "text-accent-human"
                : node.kind === "ai"
                  ? "text-accent-source"
                  : "text-ink"
            }
            title={node.kind === "human" ? "validato (umano)" : node.kind === "ai" ? "AI" : "fonte"}
          >
            {node.label}
          </span>
          {i < nodes.length - 1 && <span className="text-muted">→</span>}
        </span>
      ))}
    </div>
  );
}
