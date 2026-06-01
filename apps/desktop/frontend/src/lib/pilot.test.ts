import { expect, test } from "vitest";
import { nextActionKey } from "./pilot";

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
