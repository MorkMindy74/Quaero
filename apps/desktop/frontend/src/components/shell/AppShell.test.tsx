import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import AppShell from "./AppShell";
import "../../i18n";

test("AppShell renders the five cockpit regions", () => {
  render(<AppShell />);
  expect(screen.getByTestId("region-topbar")).toBeInTheDocument();
  expect(screen.getByTestId("region-sidebar")).toBeInTheDocument();
  expect(screen.getByTestId("region-workspace")).toBeInTheDocument();
  expect(screen.getByTestId("region-context")).toBeInTheDocument();
  expect(screen.getByTestId("region-status")).toBeInTheDocument();
});
