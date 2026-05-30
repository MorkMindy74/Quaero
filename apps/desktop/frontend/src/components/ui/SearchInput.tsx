import type { InputHTMLAttributes } from "react";

type SearchInputProps = InputHTMLAttributes<HTMLInputElement>;

export function SearchInput({ className = "", ...rest }: SearchInputProps) {
  return (
    <input
      type="search"
      className={`w-full rounded border border-hairline bg-panel px-2 py-1 text-sm outline-none placeholder:text-muted ${className}`}
      {...rest}
    />
  );
}
