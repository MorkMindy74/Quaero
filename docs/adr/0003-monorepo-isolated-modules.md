# ADR-0003 — Monorepo a moduli isolati (una funzionalità = una cartella)

## Stato
Accettata — 2026-05-30

## Contesto
Il committente ha posto un vincolo esplicito: niente repository monolitica in cui un cambiamento rischia di rompere tutto. Si deve sempre sapere su quale cartella lavorare, e ogni funzionalità deve poter essere modificata senza rompere il resto.

## Decisione
Quaero è un **monorepo a moduli isolati**. Ogni deep module del PRD è una **crate Rust separata** in un Cargo workspace, compilabile e testabile in isolamento. I moduli comunicano solo tramite contratti stabili definiti nella crate `core`.

Struttura:

```
quaero/
├─ apps/desktop/        # guscio Tauri (frontend + glue sottile)
├─ crates/
│  ├─ core/             # tipi condivisi (es. CitationAnchor)
│  ├─ matters/          # Archivio pratiche (#1)
│  ├─ ingestion/        # Ingestione documenti (#2)        [testato]
│  ├─ citations/        # Motore citazioni (#3)            [testato]
│  ├─ verifier/         # Verificatore citazioni (#4)      [testato]
│  ├─ privacy-guard/    # Privacy guard (#5)               [testato]
│  ├─ connectors/       # Connettori legali (#6) — trait plugin
│  ├─ orchestrator/     # Orchestratore agenti (#7)
│  └─ drafting/         # Redazione & export + CNS (#8)
├─ frontend/            # UI (evoluzione di UX/index.html) (#9)
└─ docs/adr/ + CONTEXT.md
```

## Conseguenze
- Cambiare una crate non rompe le altre, salvo modifica del contratto condiviso in `core` — che diventa il punto di attenzione per le revisioni.
- Estensibilità (requisito del committente): connettori, agenti e servizi sono punti di estensione formali (trait/registrazione), così nuove funzionalità AI e non si aggiungono come plugin.
- Ogni issue di lavoro è mappabile a una singola cartella → ambito chiaro per ogni intervento.
