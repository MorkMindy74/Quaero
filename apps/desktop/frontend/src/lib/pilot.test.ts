import { expect, test } from "vitest";
import {
  nextActionKey,
  currentWorkflowStep,
  workflowStepIndex,
  WORKFLOW_STEPS,
} from "./pilot";

test("nextActionKey: no sources → import a document", () => {
  expect(nextActionKey({ sources: 0, excerpts: 0, citations: 0 })).toBe("pilot.next.importSource");
});

test("nextActionKey: sources but no excerpts → create the first Estratto", () => {
  expect(nextActionKey({ sources: 2, excerpts: 0, citations: 0 })).toBe("pilot.next.createExcerpt");
});

test("nextActionKey: excerpts but no citations → add a Citazione", () => {
  expect(nextActionKey({ sources: 1, excerpts: 3, citations: 0 })).toBe("pilot.next.addCitation");
});

test("nextActionKey: has citations → export the report", () => {
  expect(nextActionKey({ sources: 1, excerpts: 3, citations: 1 })).toBe("pilot.next.export");
});

// --- Lawyer Workflow UX V1 (#62) ---

test("currentWorkflowStep: maps counts to the guided step", () => {
  expect(currentWorkflowStep({ sources: 0, excerpts: 0, citations: 0 })).toBe("load");
  expect(currentWorkflowStep({ sources: 1, excerpts: 0, citations: 0 })).toBe("find");
  expect(currentWorkflowStep({ sources: 1, excerpts: 2, citations: 0 })).toBe("claims");
  expect(currentWorkflowStep({ sources: 1, excerpts: 2, citations: 1 })).toBe("exportReview");
});

test("workflowStepIndex advances monotonically along the path", () => {
  expect(WORKFLOW_STEPS.map((s) => s.id)).toEqual(["load", "find", "claims", "exportReview"]);
  expect(workflowStepIndex("load")).toBe(0);
  expect(workflowStepIndex("exportReview")).toBe(3);
  expect(workflowStepIndex(currentWorkflowStep({ sources: 1, excerpts: 0, citations: 0 }))).toBe(1);
});
