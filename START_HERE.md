# START HERE — checkpoint Quaero

> Leggere **questo file per primo** a ogni nuova sessione, prima di scrivere codice. Le decisioni vivono negli ADR e nelle issue: non reinterpretarle, rileggerle.

## Cos'è Quaero (in una riga)

Un **Legal AI Workspace desktop, locale e privacy-first** per il diritto italiano. **Non** è una semplice chat su PDF: è uno scheletro applicativo fondato su Pratiche/Fascicoli/Fonte, citazioni ad **Estratti** di Fonte, **Genealogia** del Documento e una UI a cockpit legale. Vedi [`MANIFESTO.md`](MANIFESTO.md).

## Stato attuale (2026-05-30)

- Fase: **pianificazione completata, sviluppo non ancora iniziato.**
- Repo: `MorkMindy74/Quaero`, licenza **AGPL-3.0**.
- Definiti: PRD, glossario di dominio, 9 ADR, 14 slice di lavoro. Nessuna riga di codice applicativo ancora scritta.
- Unico artefatto pre-esistente: `UX/index.html` (mockup statico dell'interfaccia, fissa la visione estetica).

## Glossario core (dettaglio in [`CONTEXT.md`](CONTEXT.md))

```
struttura      Cliente → Pratica → Fascicolo (una vista, non una scatola) → Fonte
Fonte (9 tipi) Documento · Norma · Giurisprudenza · Dottrina · Prassi · Dato · Nota · Memoria · Fonte Esterna
anti-alluc.    Affermazione → Citazione → Estratto di Fonte → Ancora → Risposta
produzione     Output → Bozza → Documento → (Atto), con Genealogia (grafo di provenienza / DAG)
```

Backlog glossario (non bloccante): Strategia, Connettore, Timeline, Workflow/Task, Ruolo/Permessi.

## ADR approvati (`docs/adr/`)

| ADR | Decisione |
|-----|-----------|
| 0001 | Desktop-first, online in futuro |
| 0002 | Stack Tauri + Rust |
| 0003 | Monorepo a moduli isolati (una funzionalità = una cartella / crate) |
| 0004 | AGPL-3.0 + politica di riuso terze parti (`THIRD_PARTY.md`) |
| 0005 | Lingua: codice in inglese, UI bilingue IT/EN da subito, docs IT per ora |
| 0006 | Design system + politica di ispirazione visiva |
| 0007 | Si citano **Estratti di Fonte**, non Fonti (anti-allucinazione) |
| 0008 | Gerarchia Pratica/Fascicolo; il Fascicolo è una vista (molti-a-molti) |
| 0009 | Genealogia del Documento come **grafo di provenienza** (audit/responsabilità) |
| 0010 | Stack frontend: React + TypeScript + Vite + Tailwind + i18next + Vitest + RTL |
| 0011 | Sicurezza Tauri (capabilities minime), IPC tipizzato, `core` resta Tauri-free |

## Issue aperte (GitHub)

- **#1** — PRD (fondazione) + addendum architetturale.
- Slice di lavoro **#2 → #15** (figlie del PRD). Tipo: AFK = un agente può prenderla; HITL = serve una decisione umana.

| Issue | Slice | Tipo | Bloccata da |
|------|-------|------|-------------|
| #2  | Scheletro app (Tauri ↔ Rust) + i18n IT/EN | AFK | — |
| #3  | Design language "wow" + component kit | HITL | #2 |
| #4  | Installer "wow" + login semplice | HITL | #2 |
| #5  | Pratiche (Cliente→Pratica→Fascicolo, locale) | AFK | #2, #3 |
| #6  | Allega & ingerisci documento + Evidence | AFK | #5 |
| #7  | Chat che risponde (senza citazioni) | AFK | #6 |
| #8  | Citazioni ad Estratti + clic→evidenzia | AFK | #7 |
| #9  | Verificatore citazioni | AFK | #8 |
| #10 | Privacy guard | AFK | #8 |
| #11 | Valuta clausola + Bozza + export DOCX | AFK | #8 |
| #12 | Connettore Normattiva | AFK | #10 |
| #13 | Connettore giurisprudenza | AFK | #12 |
| #14 | Firma digitale CNS | HITL | #11 |
| #15 | Vista Genealogia / Provenance Graph | AFK | #11 |

## Ordine consigliato di sviluppo

1. **#2 Scheletro** (sblocca tutto) → poi in parallelo **#3 Design** e **#4 Installer** (HITL, con te).
2. **#5 Pratiche** → **#6 Ingestione** → **#7 Chat** → **#8 Citazioni**: la spina dorsale del valore.
3. **#9 Verificatore** + **#10 Privacy guard**: i due moduli critici testati (anti-allucinazione + privacy).
4. **#11 Redazione/DOCX** → **#15 Genealogia**; connettori **#12/#13**; **#14 Firma CNS** in coda.

## Quando ripartiamo sulla #2

1. Leggere questo file e [`docs/plans/issue-02-skeleton.md`](docs/plans/issue-02-skeleton.md).
2. Confermare il piano della #2.
3. Solo allora iniziare, con approccio **TDD**. Comando:

```
/mattpocock-skills:tdd 2
```

(in alternativa, aprire prima la issue con `gh issue view 2 --comments`).

Prima di iniziare, verificare l'ambiente:

```
rustc --version ; cargo --version ; node --version ; pnpm --version ; rustup show
```

Regola d'oro: **niente codice prima di aver riletto il checkpoint e confermato il piano.** La #2 non produce funzionalità, produce confini architetturali corretti.
