import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import "../../i18n";
import { WorkflowGuide } from "./WorkflowGuide";

test("empty Pratica → first step 'Carica un documento', CTA jumps to sources", () => {
  const onGoToTab = vi.fn();
  const onExport = vi.fn(async () => true);
  render(
    <WorkflowGuide
      sources={0}
      excerpts={0}
      citations={0}
      verificationWarnings={null}
      onGoToTab={onGoToTab}
      onExport={onExport}
    />,
  );
  expect(screen.getByTestId("workflow-card-title")).toHaveTextContent("Carica un documento");
  fireEvent.click(screen.getByRole("button", { name: "Carica documento" }));
  expect(onGoToTab).toHaveBeenCalledWith("sources");
  expect(onExport).not.toHaveBeenCalled();
});

test("with citations → final step exports and surfaces the chain verdict", async () => {
  const onGoToTab = vi.fn();
  const onExport = vi.fn(async () => true);
  render(
    <WorkflowGuide
      sources={1}
      excerpts={2}
      citations={1}
      verificationWarnings={2}
      onGoToTab={onGoToTab}
      onExport={onExport}
    />,
  );
  expect(screen.getByTestId("workflow-card-title")).toHaveTextContent("Controlla ed esporta");
  // Primary action exports AND shows visible confirmation (the lawyer must see it).
  fireEvent.click(screen.getByRole("button", { name: "Esporta report" }));
  expect(onExport).toHaveBeenCalledTimes(1);
  await waitFor(() => expect(screen.getByTestId("workflow-export-done")).toBeInTheDocument());
  // Secondary jumps to the evidence check.
  fireEvent.click(screen.getByRole("button", { name: "Apri Controllo prove" }));
  expect(onGoToTab).toHaveBeenCalledWith("verify");
  // Counts + #9 verdict are shown.
  expect(screen.getByTestId("workflow-card")).toHaveTextContent("Affermazioni: 1");
  expect(screen.getByTestId("workflow-card")).toHaveTextContent("2 avvisi");
});

test("middle step 'claims' jumps to the Estratti tab", () => {
  const onGoToTab = vi.fn();
  render(
    <WorkflowGuide
      sources={1}
      excerpts={2}
      citations={0}
      verificationWarnings={0}
      onGoToTab={onGoToTab}
      onExport={vi.fn(async () => true)}
    />,
  );
  expect(screen.getByTestId("workflow-card-title")).toHaveTextContent(
    "Trasforma i passaggi in affermazioni",
  );
  fireEvent.click(screen.getByRole("button", { name: "Proponi affermazioni" }));
  expect(onGoToTab).toHaveBeenCalledWith("excerpts");
});
