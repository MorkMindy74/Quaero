import { render, screen, waitFor } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, expect, test } from "vitest";
import { StatusStrip } from "./StatusStrip";
import "../../i18n";

afterEach(() => clearMocks());

test("#37 posture reflects the offline stub provider by default", async () => {
  mockIPC((cmd) => {
    if (cmd === "ping") return { reply: "pong: status" };
    if (cmd === "chat_provider_kind") return "stub";
  });
  render(<StatusStrip />);
  await waitFor(() =>
    expect(screen.getByTestId("status-privacy")).toHaveTextContent("chat stub offline"),
  );
  // truthful in every case: nothing leaves the device
  expect(screen.getByTestId("status-privacy")).toHaveTextContent("nessun dato esce dal dispositivo");
});

test("#37 posture reflects an active local model when ollamaLocal is selected", async () => {
  mockIPC((cmd) => {
    if (cmd === "ping") return { reply: "pong: status" };
    if (cmd === "chat_provider_kind") return "ollamaLocal";
  });
  render(<StatusStrip />);
  await waitFor(() =>
    expect(screen.getByTestId("status-privacy")).toHaveTextContent("modello locale attivo"),
  );
  expect(screen.getByTestId("status-privacy")).toHaveTextContent("nessun dato lascia il dispositivo");
});

test("#37 posture falls back to the offline stub line if the provider query fails", async () => {
  mockIPC((cmd) => {
    if (cmd === "ping") return { reply: "pong: status" };
    if (cmd === "chat_provider_kind") throw new Error("unavailable");
  });
  render(<StatusStrip />);
  // default state stays "stub" → no crash, offline posture shown
  await waitFor(() =>
    expect(screen.getByTestId("status-privacy")).toHaveTextContent("chat stub offline"),
  );
});
