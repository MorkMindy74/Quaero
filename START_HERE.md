# START HERE — checkpoint Quaero

> Leggere **questo file per primo** a ogni nuova sessione, prima di scrivere codice. Le decisioni vivono negli ADR e nelle issue: non reinterpretarle, rileggerle.

## Cos'è Quaero (in una riga)

Un **Legal AI Workspace desktop, locale e privacy-first** per il diritto italiano. **Non** è una semplice chat su PDF: è uno scheletro applicativo fondato su Pratiche/Fascicoli/Fonte, citazioni ad **Estratti** di Fonte, **Genealogia** del Documento e una UI a cockpit legale. Vedi [`MANIFESTO.md`](MANIFESTO.md).

## Stato attuale (2026-05-31)

- Fase: **#2, #3, #5A e #5B COMPLETATE e mergiate in `main`.** #5B ha aggiunto la **persistenza locale** (create/open/search). Prossimo step: da decidere (vedi "Prossima sessione").
- Repo: `MorkMindy74/Quaero`, licenza **AGPL-3.0**. `main` @ **`e236449`**.
- **#2** walking skeleton: Cargo workspace + `quaero-core` (puro, no Tauri) + app Tauri (`ping` IPC → core) + frontend React/Vite/TS/Tailwind/i18next (IT default, EN, toggle) + CI minima.
- **#3** cockpit shell UI: 5 regioni, component kit, leaf mock (Source/Excerpt/Reasoning/Genealogy), card "Genealogia normativa" mock, refinement v0.3.
- **#5A** modello di dominio reale in `quaero-core` (Cliente→Pratica→Fascicolo/vista→Fonte) + UI mock tipizzata. **PR #20 mergiata**.
- **#5B** persistenza locale JSON del `Workspace` (create/open/search): helper puri in `quaero-core::persistence` + store desktop (`store.rs`, `std::fs`) + 3 comandi IPC + wrapper TS tipizzati. **PR #22 mergiata** (commit `e236449`). **Issue #5 resta APERTA** (PR con `Refs #5`).
- Test su `main`: **57 Rust** (31 unit core + 8 integration + 18 store desktop) + **16 frontend**; CI verde.
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

## Contratto di persistenza locale (#5B, consolidato)

La persistenza vive in `quaero-core::persistence` (puro: solo (de)serializzazione, nessun I/O) + uno store nel crate desktop (`apps/desktop/src-tauri/src/store.rs`, I/O `std::fs`). **PR #22 mergiata** (commit `e236449`).

- **Solo `Workspace` canonico** viene salvato: la firma del writer accetta `&Workspace`, mai `WorkspaceView`; i **fascicoli dinamici non sono mai persistiti**.
- **Caricamento sempre validato**: `from_json` → `serde_json::from_str::<Workspace>` → `RawWorkspace`/`TryFrom` (integrità referenziale, rifiuto `dyn-`, `deny_unknown_fields` su uno `dossiers` fantasma).
- **Un file JSON per Pratica** in `app_data_dir()/workspaces/<id>.json` (fuori dal repo).
- **`create` esclusiva e atomica**: temp file **unico** + `fs::hard_link` (fallisce se la destinazione esiste) → tra create concorrenti **esattamente una vince** (`AlreadyExists` per le altre); **cleanup del temp incondizionato** (write/publish fallite o successo).
- **Path safety**: id `[A-Za-z0-9_-]`, niente `..`/`/`/`\`; **`file_stem == matter.id`** imposto in load (`open` rifiuta gli incoerenti, `search` li salta).
- **`search` minima**: lista su metadati (nome cliente / titolo pratica), substring case-insensitive, query vuota = tutte; niente indici/full-text/ranking/semantica.
- **`open`** ritorna la `WorkspaceView` derivata (dinamici ricalcolati); il canonico resta la fonte di verità.
- **IPC**: 3 comandi sottili `Result<_, String>` (niente panic). **Nessuna nuova capability Tauri** (`core:default`), **nessun `tauri-plugin-fs`**.

**Fuori da #5B** (slice/fasi successive): integrazione UI completa di create/open/search; ingestione documenti/Fonti (#6); update/delete; cifratura at-rest; migrazioni di schema; locking multi-processo; ricerca full-text/semantica; genealogia reale (#15).

*(Consolidato dopo 2 giri di review avversariale Codex: BLOCKER `create` concorrente + SHOULD FIX `search`/file ostili + SHOULD FIX cleanup temp — tutti risolti; verdetto finale senza BLOCKER né SHOULD FIX.)*

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
| #5  | 🟡 Pratiche (Cliente→Pratica→Fascicolo) — **#5A ✅ dominio; #5B ✅ persistenza create/open/search mergiati**; restano UI completa + ingestione | AFK | #2, #3 |
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

**#2, #3, #5A e #5B completate e mergiate.** La issue **#5** resta **aperta**: il dominio (#5A) e la persistenza locale (#5B) ci sono, ma restano da cablare la **UI completa** delle Pratiche (create/open/search) e l'**ingestione delle Fonti** (#6). Prossimo step da decidere insieme.

Candidati naturali: completare il **cablaggio UI** di create/open/search su #5B; oppure **#6** (allega & ingerisci documento + Evidence), che apre la spina dorsale #6→#8. Restano anche **#4 Installer** (HITL) e #9→#15.

Regola operativa: niente codice prima di rileggere questo checkpoint e confermare il piano; per i **confini critici** (dominio canonico, persistenza, filesystem, IPC Tauri, parsing file caricati, dati cliente, AI che produce atti/citazioni, genealogia, migrazioni, cloud/connettori) → **Codex adversarial-review prima del merge**.

**Ambiente (Windows):** i comandi `cargo` richiedono l'ambiente VS Build Tools caricato — usare la **"x64 Native Tools Command Prompt for VS 2022"** oppure caricare `vcvars64.bat` (`...\BuildTools\VC\Auxiliary\Build\vcvars64.bat`), altrimenti il linker MSVC non è nel PATH. Verifica rapida:

```
rustc --version ; cargo --version ; node --version ; pnpm --version
```

Regola d'oro: **niente codice prima di aver riletto il checkpoint e confermato il piano.**
