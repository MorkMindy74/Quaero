import type { ButtonHTMLAttributes } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "ghost";
}

export function Button({ variant = "ghost", className = "", ...rest }: ButtonProps) {
  const base = "inline-flex items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors";
  const styles =
    variant === "primary"
      ? "bg-ink text-background hover:opacity-90"
      : "border border-transparent text-ink hover:bg-panel-2";
  return <button className={`${base} ${styles} ${className}`} {...rest} />;
}
