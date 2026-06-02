import { render, screen, fireEvent } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import "../../i18n";
import type { Excerpt } from "../../domain/types";
import { NewCitationDialog } from "./NewCitationDialog";

const EXCERPT: Excerpt = {
  id: "e1",
  sourceId: "s1",
  anchor: { kind: "pagina", value: "8" },
  quote: "Il conduttore è tenuto.",
};

// #57: a lawyer must never lose typed text by clicking outside the dialog.
test("clicking outside the dialog does NOT close it", () => {
  const onClose = vi.fn();
  render(
    <NewCitationDialog excerpt={EXCERPT} onClose={onClose} onSubmit={vi.fn(async () => true)} error={null} />,
  );
  // The backdrop is the dialog container; clicking it must not close.
  fireEvent.click(screen.getByRole("dialog"));
  expect(onClose).not.toHaveBeenCalled();
});

test("Escape closes the dialog; Cancel closes it too", () => {
  const onClose = vi.fn();
  render(
    <NewCitationDialog excerpt={EXCERPT} onClose={onClose} onSubmit={vi.fn(async () => true)} error={null} />,
  );
  fireEvent.keyDown(window, { key: "Escape" });
  expect(onClose).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Annulla" }));
  expect(onClose).toHaveBeenCalledTimes(2);
});
