import { render, screen } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, expect, test } from "vitest";
import AppShell from "./AppShell";
import "../../i18n";

beforeEach(() => {
  // #5C: the sidebar loads the saved-matters list on mount.
  mockIPC((cmd) => {
    if (cmd === "search_workspaces") return [];
  });
});

afterEach(() => clearMocks());

test("AppShell renders the five cockpit regions", () => {
  render(<AppShell />);
  expect(screen.getByTestId("region-topbar")).toBeInTheDocument();
  expect(screen.getByTestId("region-sidebar")).toBeInTheDocument();
  expect(screen.getByTestId("region-workspace")).toBeInTheDocument();
  expect(screen.getByTestId("region-context")).toBeInTheDocument();
  expect(screen.getByTestId("region-status")).toBeInTheDocument();
});
