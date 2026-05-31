// Pure id helpers for #5C. The user never types technical ids: these derive
// safe ids in the backend-allowed charset [A-Za-z0-9_-] (mirrors the desktop
// store's `safe_file_stem`). The backend remains the source of truth — these
// helpers only avoid sending obviously-invalid ids and never bypass validation.

/** Slugify a display string into the safe id charset; never returns empty. */
export function slug(input: string): string {
  const s = input
    .toLowerCase()
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "") // strip diacritics (à→a, è→e, …)
    .replace(/[^a-z0-9]+/g, "-") // any run of unsafe chars → single hyphen
    .replace(/^-+|-+$/g, "") // trim leading/trailing hyphens
    .replace(/-{2,}/g, "-"); // collapse repeats
  return s || "p";
}

/** Short suffix from the safe charset [0-9a-z]. Injectable RNG for tests. */
export function shortSuffix(rand: () => number = Math.random): string {
  let out = "";
  while (out.length < 6) {
    out += Math.floor(rand() * 36).toString(36);
  }
  return out.slice(0, 6);
}

/** Build a unique, safe matter id: `slug(title)-<suffix>`. Two matters with the
 *  same title never collide thanks to the random suffix. */
export function makeMatterId(title: string, rand: () => number = Math.random): string {
  return `${slug(title)}-${shortSuffix(rand)}`;
}
