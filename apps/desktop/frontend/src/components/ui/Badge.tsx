import type { ReactNode } from "react";

type Tone = "default" | "source" | "verified" | "warning" | "human";

interface BadgeProps {
  children: ReactNode;
  tone?: Tone;
}

const tones: Record<Tone, string> = {
  default: "text-muted border-hairline",
  source: "text-accent-source border-accent-source",
  verified: "text-accent-verified border-accent-verified",
  warning: "text-accent-warning border-accent-warning",
  human: "text-accent-human border-accent-human",
};

export function Badge({ children, tone = "default" }: BadgeProps) {
  return (
    <span className={`inline-flex items-center rounded-full border px-2 py-0.5 font-mono text-[11px] ${tones[tone]}`}>
      {children}
    </span>
  );
}
