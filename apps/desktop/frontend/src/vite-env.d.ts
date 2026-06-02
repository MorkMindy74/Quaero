// Minimal ambient declaration for Vite's `?url` asset imports (we keep tsconfig
// `types` narrow, so we don't pull all of `vite/client`). Used to bundle the
// pdf.js worker locally (no CDN) for the document text layer (#52).
declare module "*?url" {
  const src: string;
  export default src;
}
