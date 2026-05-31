# START HERE — checkpoint Quaero

> Leggere **questo file per primo** a ogni nuova sessione, prima di scrivere codice. Le decisioni vivono negli ADR e nelle issue: non reinterpretarle, rileggerle.

## Cos'è Quaero (in una riga)

Un **Legal AI Workspace desktop, locale e privacy-first** per il diritto italiano. **Non** è una semplice chat su PDF: è uno scheletro applicativo fondato su Pratiche/Fascicoli/Fonte, citazioni ad **Estratti** di Fonte, **Genealogia** del Documento e una UI a cockpit legale. Vedi [`MANIFESTO.md`](MANIFESTO.md).

## Stato attuale (2026-05-31)

- Fase: **#2, #3 e #5A COMPLETATE e mergiate in `main`.** Prossimo step: **#5B** (persistenza locale / create / open / search).
- Repo: `MorkMindy74/Quaero`, licenza **AGPL-3.0**. `main` @ **`7bbc694`**.
- **#2** walking skeleton: Cargo workspace + `quaero-core` (puro, no Tauri) + app Tauri (`ping` IPC → core) + frontend React/Vite/TS/Tailwind/i18next (IT default, EN, toggle) + CI minima.
- **#3** cockpit shell UI: 5 regioni, component kit, leaf mock (Source/Excerpt/Reasoning/Genealogy), card "Genealogia normativa" mock, refinement v0.3.
- **#5A** modello di dominio reale in `quaero-core` (Cliente→Pratica→Fascicolo/vista→Fonte) + UI mock tipizzata. **PR #20 mergiata** (commit `7bbc694`). **Issue #5 resta APERTA** per #5B.
- Test su `main`: **26 unit + 8 integration** (Rust) + **12 frontend**; CI verde.
- **Processo:** ogni modifica via **branch + PR** con CI verde (vedi `CONTRIBUTING.md`); niente commit diretti su `main`.
- Mockup estetico di riferimento: `UX/index.html`.

## Glossario core (dettaglio in [`CONTEXT.md`](CONTEXT.md))

```
struttura      Cliente → Pratica → Fascicolo (una vista, non una scatola) → Fonte
Fonte (9 tipi) Documento · Norma · Giurisprudenza · Dottrina · Prassi · Dato · Nota · Memoria · Fonte Esterna
anti-alluc.    Affermazione → Citazione → Estratto di Fonte → Ancora → Risposta
produzione     Output → Bozza → Documento → (Atto), con Genealogia (grafo di provenienza / DAG)
```

Backlog glossario (non bloccante): Strategia, Connettore, Timeline, Workflow/Task, Ruolo/Permessi.

## Contratto di dominio canonico (#5A, consolidato)

Il modello vive in `quaero-core::domain` (puro, Tauri-free). Invarianti imposti **dal tipo e dall'API pubblica**, non dalla convenzione:

- **`Workspace` valido-per-costruzione**: campi privati + accessor di sola lettura; si ottiene solo via `Workspace::new(...)` validante o serde (`RawWorkspace` + `TryFrom`).
- **Stato canonico vs vista derivata**: `Workspace` (canonico) contiene `sources` + `manualDossiers`; i **fascicoli dinamici** esistono **solo** nella `WorkspaceView` derivata (`Workspace::view()`), mai persistiti come stato canonico.
- **`ManualDossier` separato da `DossierView`**: il manuale non ha `kind` (non può rappresentare un dinamico); il prefisso **`dyn-` è riservato** ai dinamici.
- **`deny_unknown_fields`** su tutti i tipi canonici → nessun campo-ombra/derivato passa come canonico.
- **Integrità referenziale validata**: `matter.client == client.id`; id sorgenti e id manuali univoci; ogni Fonte di un manuale esiste in `sources`.
- **Wire `camelCase`** (`manualDossiers`) coerente Rust↔TypeScript.
- La **issue #5 NON è chiusa**: persistenza/create/open/search restano per #5B.

*(Consolidato dopo un loop di review avversariale Codex a 7 giri; verdetto finale `approve`.)*

## ADR approvati (`docs/adr/`) — 11 ADR

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
| #2  | ✅ Scheletro app (Tauri ↔ Rust) + i18n IT/EN | AFK | — |
| #3  | ✅ Design language "wow" + cockpit shell + component kit | HITL | #2 |
| #4  | Installer "wow" + login semplice | HITL | #2 |
| #5  | 🟡 Pratiche (Cliente→Pratica→Fascicolo) — **#5A ✅ dominio mergiato; #5B ⏳ persistenza/create/open/search** | AFK | #2, #3 |
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

## Prossima sessione

**#2, #3, #5A completate e mergiate.** Prossimo step: **#5B** — persistenza locale / create / open / search (issue **#5**, ancora aperta), che costruirà sul **contratto di dominio canonico** consolidato in #5A (vedi sezione sopra).

Per sviluppare #5B: leggere questo file, aprire la issue (`gh issue view 5 --comments`), confermare il piano, poi `/mattpocock-skills:tdd 5` su un branch dedicato `slice/5b-...`.

*(Restano anche, quando deciso: #4 Installer (HITL); #6→#15 spina dorsale del valore.)*

**Ambiente (Windows):** i comandi `cargo` richiedono l'ambiente VS Build Tools caricato — usare la **"x64 Native Tools Command Prompt for VS 2022"** oppure caricare `vcvars64.bat` (`...\BuildTools\VC\Auxiliary\Build\vcvars64.bat`), altrimenti il linker MSVC non è nel PATH. Verifica rapida:

```
rustc --version ; cargo --version ; node --version ; pnpm --version
```

Regola d'oro: **niente codice prima di aver riletto il checkpoint e confermato il piano.**
