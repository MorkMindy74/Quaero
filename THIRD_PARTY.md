# Codice e fonti di terze parti

Quaero è rilasciato sotto **AGPL-3.0** (vedi `LICENSE`). Questo file registra ogni porzione di codice di terze parti riusata nel progetto, per rispettare le licenze e mantenere la tracciabilità (vedi [ADR-0004](docs/adr/0004-agpl-and-third-party-reuse.md)).

## Regola

Prima di integrare codice esterno:
1. verificare che la licenza della fonte sia compatibile con AGPL-3.0 (AGPL-3.0 ✅, Apache-2.0 ✅, MIT/BSD ✅; licenze incompatibili o assenti → solo ispirazione, niente copia);
2. conservare header di copyright e testo di licenza originali;
3. aggiungere una riga alla tabella qui sotto.

> Le *idee, architettura e UX* possono essere imitate liberamente: il vincolo riguarda solo il codice testuale.

## Repository di riferimento

| Progetto | Licenza | Uso previsto |
|---|---|---|
| [willchen96/mike](https://github.com/willchen96/mike) | AGPL-3.0 | ispirazione UX/feature; codice riusabile |
| [SemplificaAI/MikeRust](https://github.com/SemplificaAI/MikeRust) | AGPL-3.0 | base Rust locale; codice riusabile |
| [AnttiHero/lavern](https://github.com/AnttiHero/lavern) | Apache-2.0 | feature da assorbire in AGPL |
| [strukto-ai/mirage](https://github.com/strukto-ai/mirage) | Apache-2.0 | filesystem virtuale/sandbox per agenti — tecnica candidata per il Privacy guard (futuro, vedi issue #9) |

## Codice effettivamente riusato

_(nessuno ancora — aggiungere qui man mano: file di destinazione, fonte, file originale, licenza, commit)_

| Dove (Quaero) | Da (progetto/file) | Licenza | Commit |
|---|---|---|---|
| — | — | — | — |
