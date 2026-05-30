# ADR-0010 — Stack frontend: React + TypeScript + Vite + Tailwind + i18next

## Stato
Accettata — 2026-05-30

## Decisione
Il frontend di Quaero (dentro `apps/desktop/frontend`, servito nel webview Tauri) usa: **React + TypeScript**, build con **Vite**, stile con **Tailwind CSS**, internazionalizzazione con **i18next / react-i18next**, test con **Vitest** + **React Testing Library**.

## Perché
La UI di Quaero è un cockpit con sidebar, pannelli, tab, stato, document viewer e reasoning panel: partire senza framework sarebbe un falso risparmio. React+TS+Vite è uno standard maturo e ben documentato (anche in monorepo); Tailwind si sposa con il design system (ADR-0006); i18next copre JSON/fallback/namespace/TypeScript (ADR-0005); React Testing Library spinge a testare la UI come la usa l'utente, non i dettagli interni.

## Considered Options
- **Vanilla / nessun framework**: scartato — la complessità della UI a cockpit lo renderebbe presto ingestibile.
- **Electron + framework**: già scartato in ADR-0002 (installer pesante, meno privacy).

## Conseguenze
- Vincoli di versione: Node 20.19+ (o 22.12+) per Vite; Vitest richiede Node ≥20 e Vite ≥6.
- Vite dentro Tauri va configurato con porta fissa e `strictPort: true` (devUrl prevedibile per Tauri).
- Il design system (ADR-0006) sarà implementato come token Tailwind + componenti React nella slice #3.
