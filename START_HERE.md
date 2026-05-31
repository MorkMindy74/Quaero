# START HERE — checkpoint Quaero

> Leggere **questo file per primo** a ogni nuova sessione, prima di scrivere codice. Le decisioni vivono negli ADR e nelle issue: non reinterpretarle, rileggerle.

## Cos'è Quaero (in una riga)

Un **Legal AI Workspace desktop, locale e privacy-first** per il diritto italiano. **Non** è una semplice chat su PDF: è uno scheletro applicativo fondato su Pratiche/Fascicoli/Fonte, citazioni ad **Estratti** di Fonte, **Genealogia** del Documento e una UI a cockpit legale. Vedi [`MANIFESTO.md`](MANIFESTO.md).

## Stato attuale (2026-05-31)

- Fase: **#2, #3, #5 (#5A/#5B/#5C), #6, #7, #8 e #9 COMPLETATE e mergiate in `main`.** Le **issue #5, #6, #7, #8 e #9 sono CHIUSE**: Pratiche (crea/apri/cerca, locale) + ingestione documenti/Evidence v1 + chat controllata stub-only + catena anti-allucinazione (Estratti/Citazioni) + Verificatore citazioni. Prossimo step: da decidere (vedi "Prossima sessione").
- Repo: `MorkMindy74/Quaero`, licenza **AGPL-3.0**. `main` @ **`159f954`**.
- **#2** walking skeleton: Cargo workspace + `quaero-core` (puro, no Tauri) + app Tauri (`ping` IPC → core) + frontend React/Vite/TS/Tailwind/i18next (IT default, EN, toggle) + CI minima.
- **#3** cockpit shell UI: 5 regioni, component kit, leaf mock (Source/Excerpt/Reasoning/Genealogy), card "Genealogia normativa" mock, refinement v0.3.
- **#5A** modello di dominio reale in `quaero-core` (Cliente→Pratica→Fascicolo/vista→Fonte) + UI mock tipizzata. **PR #20 mergiata**.
- **#5B** persistenza locale JSON del `Workspace` (create/open/search): helper puri in `quaero-core::persistence` + store desktop (`store.rs`, `std::fs`) + 3 comandi IPC + wrapper TS tipizzati. **PR #22 mergiata** (commit `e236449`).
- **#5C** UI minima collegata a create/open/search (**frontend-only**): lista Pratiche da `searchWorkspaces`, dialog "+ Nuova Pratica" → `createWorkspace`, apertura → `openWorkspace` nel pannello Sources; `slug()` per gli id; stati loading/error/empty. **PR #24 mergiata** (commit `fdd36ef`). Backend/IPC/filesystem/capability **non toccati**; Codex review non necessaria (frontend-only).
- **Issue #5 CHIUSA**: completata end-to-end con **#5A** (dominio canonico) + **#5B** (persistenza) + **#5C** (UI wiring). L'unificazione mock↔reale di TopCommandBar/MainWorkspace resta un **refinement UI futuro**, non parte dello scope di #5.
- **#6** ingestione documenti / Evidence v1: import di un file locale come **Fonte Documento**, byte in `app_data/files/<matterId>/`, registrazione canonica **`SourceRef + StoredFile`** (`storedName`/`originalName`/`byteLen`/`sha256`), byte **fuori** dal JSON, pubblicazione blob **esclusiva** (no overwrite), path safety, UI minima. **PR #26 mergiata** (commit `0d83ee5`). **Issue #6 CHIUSA.** Review Codex: giro 1 `changes-requested` (BLOCKER integrità blob) → fix → giro 2 `approve-with-notes` (nessun BLOCKER/SHOULD FIX residuo).
- **#7** chat controllata **stub-only offline**: pipeline UI → IPC → Rust → `ChatProvider` → `StubProvider` deterministico; **nessuna rete/API key/LLM reale/persistenza/accesso documenti/Evidence/citazioni**; isolamento per-Pratica (cambio Pratica → chat azzerata); risposte marcate "esplorativa · non verificata · senza citazioni · non parere legale" (ADR-0007). **PR #28 mergiata** (commit `3ef2557`). **Issue #7 CHIUSA.** Review Codex: giro 1 `changes-requested` (BLOCKER isolamento per-Pratica) → fix → giro 2 `approve`.
- **#8** **Citazioni ad Estratti** (cuore anti-allucinazione, ADR-0007): `Excerpt`/`Anchor`/`Citation` nel dominio canonico + persistenza + display read-only nella tab Estratti; invariante "si cita un Estratto, **mai** una Fonte" imposto dal tipo; `sourceSha256` validato contro `SourceRef.file.sha256`; retro-compatibile coi Workspace pre-#8. **PR #30 mergiata** (commit `6d496d8`). **Issue #8 CHIUSA.** Review Codex: giro 1 `changes-requested` (BLOCKER integrità `sourceSha256`) → fix → giro 2 `approve`.
- **#9** **Verificatore citazioni** (audit & spiegabilità): `verify(&Workspace)` **puro/deterministico** in `quaero-core::verify` → report **derivato** in `WorkspaceView` (mai persistito; `Workspace` canonico invariato); findings `Info`/`Warning` (niente `Error`) — `OrphanExcerpt`, `UnpinnedDocumentExcerpt` (solo con `StoredFile`), `UncitedSource` (Info, non degrada il verdetto); attestazione positiva (conteggi); tab **"Verifica"** read-only; verdetto `warnings==0` → "Catena coerente". **PR #32 mergiata** (commit `159f954`). **Issue #9 CHIUSA.** Review Codex leggera: giro 1 `changes-requested` (SHOULD FIX perf) → fix O(1) a comportamento invariato → verde.
- Test su `main`: **102 Rust** (66 unit core + 8 integration + 28 store desktop) + **42 frontend**; CI verde.
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

## Ingestione documenti / Evidence (#6, consolidato)

L'import vive nel core (`quaero-core`: `StoredFile`, `SourceRef.file`, `Workspace::with_source`, `hash::sha256_hex` via `sha2`) + store desktop (`store::import_document`). **PR #26 mergiata** (commit `0d83ee5`).

- **Import**: un file locale → **Fonte Documento** (`SourceRef` con `kind: Documento` + `file: StoredFile`). I **byte** vivono in `app_data/files/<matterId>/<storedName>`, **mai** nel JSON del Workspace.
- **`StoredFile`**: `storedName` (nome on-disk generato, sicuro), `originalName` (solo display, **mai** path), `byteLen`, **`sha256`** (hex lowercase) — fondamento minimo di **Evidence** (integrità verificabile, base delle future Ancore).
- **Pubblicazione blob esclusiva**: temp unico + `fs::hard_link` (fallisce se la destinazione esiste); un id che collide (con una Fonte del Workspace o con un blob orfano) **rigenera**, **mai overwrite** → `sha256`/`byteLen` sempre coerenti coi byte fisici. Ordine **blob → JSON** (worst case: blob orfano).
- **Path safety**: `matterId`/`sourceId`/estensione validati o generati; nessun path deriva da `originalName`.
- **Concorrenza**: **mutex per-matter in-process** → import concorrenti non perdono Fonti.
- **Trasporto**: byte via IPC (`<input type=file>` + `arrayBuffer`); **nessun `tauri-plugin-fs`/`-dialog`, nessuna nuova capability** (`core:default`). Cap **25 MB**.
- **`SourceRef.file` opzionale** (`serde(default)` + `skip_serializing_if`) → i Workspace pre-#6 restano caricabili.

**Limite accettato #6 v1**: il cap 25 MB protegge persistenza/disco ma il payload IPC viene **allocato prima** del controllo (vettore solo locale). Il protocollo **chunked/streaming** è un **candidato slice futura separata**, fuori da #6.

**Fuori da #6**: parsing PDF/DOCX, OCR, AI, preview/rendering, **Estratti/Ancore/Citazioni (#8)**, dedup, verifica periodica dell'hash, chunked import.

*(Consolidato dopo 2 giri di review avversariale Codex: BLOCKER integrità blob — overwrite su collisione id — risolto con pubblicazione esclusiva + rigenerazione; verdetto finale `approve-with-notes`, size-cap/DoS accettato come CAN WAIT fuori scope.)*

## Chat controllata (#7, consolidato)

La chat vive nella surface "Conversazione" via pipeline **UI → IPC → Rust → `ChatProvider`**. **PR #28 mergiata** (commit `3ef2557`).

- **`StubProvider` deterministico e offline** (in `quaero-core::chat`): stesso input → stesso output; **nessun LLM reale, rete, API key, segreto, accesso file, persistenza**. Comando IPC `chat_send` (`Result<_, String>`, `deny_unknown_fields`, cap lunghezza prompt).
- **Non-fondata per costruzione**: `grounded` sempre `false`, nessuna citazione; la UI mostra un **disclaimer permanente** + etichetta per-risposta "non verificata · senza citazioni · non parere legale" → ADR-0007 preservato (le Citazioni reali arrivano in #8).
- **Isolamento per-Pratica**: `ChatPanel` è keyato per `matter.id` → cambiando Pratica la chat si azzera; nessun bleed tra clienti. Chat **in-memory** (non persistita).
- **Nessuna nuova capability/plugin** (`core:default`).

**Fuori da #7** (slice futura dedicata, con preflight + Codex obbligatoria): **provider LLM reale** (locale/remoto) dietro `ChatProvider`, rete, API key/segreti, consenso utente, persistenza messaggi, memoria lunga, streaming, **grounding/Citazioni (#8)**.

*(Consolidato dopo 2 giri di review avversariale Codex: BLOCKER isolamento per-Pratica risolto con remount keyato per `matter.id`; verdetto finale `approve`.)*

## Citazioni ad Estratti (#8, consolidato)

Il cuore anti-allucinazione (ADR-0007) nel dominio canonico (`quaero-core::domain`). **PR #30 mergiata** (commit `6d496d8`).

- **`Excerpt { id, sourceId, anchor, quote, sourceSha256? }`** — porzione verificabile di una Fonte; valido-per-costruzione (campi privati, `new`/`RawExcerpt` `TryFrom`; rifiuta quote/anchor vuoti).
- **`Anchor { kind, value }`** — localizzatore logico indipendente dal layout (dichiarativo in #8; **nessun parsing**).
- **`Citation { id, claim, excerptId }`** — referenzia **solo** un Estratto: **impossibile citare una Fonte** (ADR-0007 imposto dal tipo; `deny_unknown_fields` rifiuta un `sourceId` intrufolato).
- **Integrità referenziale** in `Workspace::assemble` (su `new_with_evidence` e serde `TryFrom`): excerpt→source esistente, citation→excerpt esistente, id univoci.
- **Integrità Evidence**: se un Estratto pinna `sourceSha256`, deve combaciare **esattamente** con `SourceRef.file.sha256`; rifiutati mismatch e pin su Fonte senza file; `None` ammesso.
- **Retro-compatibilità**: `serde(default)` sui nuovi campi (Workspace pre-#8 caricabili); `skip_serializing_if` → JSON senza evidenze byte-identico.
- **UI**: tab "Estratti" mostra gli Estratti **reali** del workspace aperto (quote + Ancora + Fonte + claim citanti), stato vuoto chiaro, **nessun fallback mock**.

**Fuori da #8** (slice futura, es. #8B): creazione Estratti da UI, **clic→evidenzia** nel documento, parsing reale (PDF/DOCX), estrazione automatica / grounding con LLM reale.

*(Consolidato dopo 2 giri di review avversariale Codex: BLOCKER integrità `sourceSha256` — pin non validato — risolto con validazione contro il digest della Fonte; verdetto finale `approve`.)*

## Verificatore citazioni (#9, consolidato)

Audit **puro e derivato** della catena Estratto→Citazione (`quaero-core::verify`). **NON** ridefinisce la validità strutturale (già garantita da #8): produce qualità/copertura + attestazione positiva. **PR #32 mergiata** (commit `159f954`).

- **`verify(&Workspace) -> VerificationReport`** — puro, deterministico, **zero I/O** (niente byte/FS/parsing/LLM/rete).
- **Severità** `Info`/`Warning` (**niente `Error`**: gli errori strutturali non si caricano nemmeno). Findings: `OrphanExcerpt` (Warning), `UnpinnedDocumentExcerpt` (Warning, **solo** se la Fonte ha uno `StoredFile`), `UncitedSource` (**Info**, non degrada il verdetto). Ordine deterministico.
- **Attestazione positiva** (`summary`): `citations`, `excerpts`, `documentBackedExcerpts`, `pinnedExcerpts`, `warnings`, `infos`.
- **Verdetto**: `warnings == 0` → "Catena coerente"; altrimenti "Catena con N avvisi".
- **Report derivato** in `WorkspaceView.verification` (calcolato in `view()`, **mai persistito**); **`Workspace` canonico invariato**; **nessun IPC/dipendenza/capability nuovi**.
- **UI**: tab **"Verifica"** read-only, separata da "Estratti" (Audit vs Evidence); stato vuoto senza workspace, badge solo se `warnings>0`.
- **Seed**: report "Catena coerente" (1 citazione, 1 estratto, 0 document-backed, 0 pinnati, 0 warning, 3 `UncitedSource` Info).

**Fuori da #9** (slice future): **ri-hash fisico** dei file (accesso byte) per rilevare manomissioni su disco, parsing reale, clic→evidenzia, creazione Estratti da UI (#8B), grounding/LLM, export del report.

*(Consolidato dopo review avversariale Codex leggera: SHOULD FIX performance — lookup O(excerpts×sources) — risolto con set precomputato O(1) a comportamento invariato; verdetto finale verde.)*

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
| #5  | ✅ **CHIUSA** — Pratiche (Cliente→Pratica→Fascicolo): #5A dominio + #5B persistenza + #5C UI wiring (create/open/search). Unificazione mock↔reale = refinement UI futuro | AFK | #2, #3 |
| #6  | ✅ **CHIUSA** — Allega & ingerisci documento + Evidence (import→Fonte Documento + `sha256`; byte fuori JSON; pubblicazione esclusiva). Chunked import = slice futura | AFK | #5 |
| #7  | ✅ **CHIUSA** — Chat che risponde (senza citazioni): pipeline UI→IPC→Rust→ChatProvider→StubProvider offline; isolamento per-Pratica. Provider reale = slice futura | AFK | #6 |
| #8  | ✅ **CHIUSA** — Citazioni ad Estratti: Excerpt/Anchor/Citation canonici + persistenza + display read-only; "si cita un Estratto, non una Fonte"; `sourceSha256` validato. Creazione UI + clic→evidenzia = #8B futura | AFK | #7 |
| #9  | ✅ **CHIUSA** — Verificatore citazioni: `verify(&Workspace)` puro + report derivato in `WorkspaceView` + tab "Verifica" read-only; `UncitedSource` Info non degrada il verdetto. Re-hash fisico = slice futura | AFK | #8 |
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

**#2, #3, l'intera #5, #6, #7, #8 e #9 completate e mergiate; issue #5, #6, #7, #8 e #9 CHIUSE.** Pratiche end-to-end, documenti come Fonti (Evidence v1 con `sha256`), chat stub-only offline, catena anti-allucinazione Estratto→Citazione **modellata, validata e auditata** (Verificatore). Prossimo step da decidere insieme.

Candidati naturali: **#8B** (creazione Estratti da UI + clic→evidenzia + parsing reale del documento); **#10** (Privacy guard); oppure le slice di rinforzo — **provider LLM reale** dietro `ChatProvider` (preflight + Codex obbligatoria), **`chunked document import`** (robustezza payload IPC), **ri-hash fisico** dei file (verifica integrità su disco), **UI refinement** mock↔reale. Resta anche **#4 Installer** (HITL).

Regola operativa: niente codice prima di rileggere questo checkpoint e confermare il piano; per i **confini critici** (dominio canonico, persistenza, filesystem, IPC Tauri, parsing file caricati, dati cliente, AI che produce atti/citazioni, genealogia, migrazioni, cloud/connettori) → **Codex adversarial-review prima del merge**.

**Ambiente (Windows):** i comandi `cargo` richiedono l'ambiente VS Build Tools caricato — usare la **"x64 Native Tools Command Prompt for VS 2022"** oppure caricare `vcvars64.bat` (`...\BuildTools\VC\Auxiliary\Build\vcvars64.bat`), altrimenti il linker MSVC non è nel PATH. Verifica rapida:

```
rustc --version ; cargo --version ; node --version ; pnpm --version
```

Regola d'oro: **niente codice prima di aver riletto il checkpoint e confermato il piano.**
