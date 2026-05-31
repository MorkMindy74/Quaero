import { useCallback, useEffect, useState } from "react";
import {
  createWorkspace,
  searchWorkspaces,
  type WorkspaceSummary,
} from "./ipc";
import type { Client, Matter } from "../domain/types";
import { makeMatterId, slug } from "./slug";

export interface UseWorkspaces {
  items: WorkspaceSummary[];
  loading: boolean;
  error: string | null;
  query: string;
  setQuery: (q: string) => void;
  refresh: () => Promise<void>;
  /** Create a Pratica from display fields; derives safe ids. Returns the
   *  summary on success, or null on error (message exposed via `error`). */
  createMatter: (
    clientName: string,
    matterTitle: string,
    subject?: string,
  ) => Promise<WorkspaceSummary | null>;
}

/**
 * Frontend data layer for #5C: wraps the #5B IPC commands (search/create) with
 * list + loading/error state. No domain logic, no persistence — the backend
 * stays the source of truth (we never bypass its validation). `rand` is
 * injectable so tests get deterministic matter ids.
 */
export function useWorkspaces(rand: () => number = Math.random): UseWorkspaces {
  const [items, setItems] = useState<WorkspaceSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");

  const run = useCallback(async (q: string) => {
    setLoading(true);
    setError(null);
    try {
      const results = await searchWorkspaces(q);
      setItems(results ?? []);
    } catch (e) {
      setError(String(e));
      setItems([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void run(query);
  }, [query, run]);

  const refresh = useCallback(() => run(query), [run, query]);

  const createMatter = useCallback(
    async (clientName: string, matterTitle: string, subject = "") => {
      setError(null);
      const clientId = slug(clientName);
      const client: Client = { id: clientId, name: clientName };
      const matter: Matter = {
        id: makeMatterId(matterTitle, rand),
        client: clientId,
        title: matterTitle,
        subject,
      };
      try {
        const summary = await createWorkspace(client, matter);
        await run(query);
        return summary;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [run, query, rand],
  );

  return { items, loading, error, query, setQuery, refresh, createMatter };
}
