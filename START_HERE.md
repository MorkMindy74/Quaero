# START HERE — checkpoint Quaero

> Leggere **questo file per primo** a ogni nuova sessione, prima di scrivere codice. Le decisioni vivono negli ADR e nelle issue: non reinterpretarle, rileggerle.

## Cos'è Quaero (in una riga)

Un **Legal AI Workspace desktop, locale e privacy-first** per il diritto italiano. **Non** è una semplice chat su PDF: è uno scheletro applicativo fondato su Pratiche/Fascicoli/Fonte, citazioni ad **Estratti** di Fonte, **Genealogia** del Documento e una UI a cockpit legale. Vedi [`MANIFESTO.md`](MANIFESTO.md).

## Stato attuale (2026-05-30)

- Fase: **Slice #2 (scheletro app) COMPLETATA e verde.** Prossima: #3 Design / #4 Installer (HITL, con l'utente), oppure #5 Pratiche.
- Repo: `MorkMindy74/Quaero`, licenza **AGPL-3.0**.
- Definiti: PRD, glossario di dominio, 11 ADR, 14 slice di lavoro.
- Implementato in #2: Cargo workspace + `quaero-core` (puro, no Tauri) + app Tauri (`ping` IPC → core) + frontend React/Vite/TS/Tailwind/i18next (IT default, EN, toggle) + CI minima. **2 test Rust + 3 test frontend** verdi; app **avviata in dev mode e verificata** visivamente.
- Mockup estetico di riferimento: `UX/index.html` (il design vero è la slice #3).

## Glossario core (dettaglio in [`CONTEXT.md`](CONTEXT.md))

```
struttura      Cliente → Pratica → Fascicolo (una vista, non una scatola) → Fonte
Fonte (9 tipi) Documento · Norma · Giurisprudenza · Dottrina · Prassi · Dato · Nota · Memoria · Fonte Esterna
anti-alluc.    Affermazione → Citazione → Estratto di Fonte → Ancora → Risposta
produzione     Output → Bozza → Documento → (Atto), con Genealogia (grafo di provenienza / DAG)
```

Backlog glossario (non bloccante): Strategia, Connettore, Timeline, Workflow/Task, Ruolo/Permessi.

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

## Prossima sessione

La slice **#2 è completata** (vedi *Stato attuale*). Prossime per piano:
- **#3 Design language** + **#4 Installer** — HITL, da fare insieme all'utente.
- **#5 Pratiche** — AFK, avvia la spina dorsale del dominio (Cliente→Pratica→Fascicolo→Fonte).

Per sviluppare la prossima slice (es. #5): leggere questo file, aprire la issue (`gh issue view 5 --comments`), confermare il piano, poi `/mattpocock-skills:tdd 5`.

**Ambiente (Windows):** i comandi `cargo` richiedono l'ambiente VS Build Tools caricato — usare la **"x64 Native Tools Command Prompt for VS 2022"** oppure caricare `vcvars64.bat` (`...\BuildTools\VC\Auxiliary\Build\vcvars64.bat`), altrimenti il linker MSVC non è nel PATH. Verifica rapida:

```
rustc --version ; cargo --version ; node --version ; pnpm --version
```

Regola d'oro: **niente codice prima di aver riletto il checkpoint e confermato il piano.**
