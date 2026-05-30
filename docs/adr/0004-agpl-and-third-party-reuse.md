# ADR-0004 — Licenza AGPL-3.0 e riuso di codice di terze parti

## Stato
Accettata — 2026-05-30

## Contesto
Quaero sarà open source con licenza **AGPL-3.0**, scelta per impedire l'appropriazione commerciale da parte di grandi aziende (la clausola di rete AGPL obbliga chi offre il software come servizio a ripubblicare le modifiche). Il metodo di sviluppo prevede di integrare, di volta in volta, funzionalità ispirate ad altre repository GitHub. Licenze verificate delle repo di riferimento:

- willchen96/mike — **AGPL-3.0**
- SemplificaAI/MikeRust — **AGPL-3.0**
- AnttiHero/lavern — **Apache-2.0**

## Decisione
1. Quaero è rilasciato sotto **AGPL-3.0**.
2. Il riuso di **codice** di terze parti è ammesso solo se la licenza della fonte è compatibile con AGPL-3.0:
   - codice **AGPL-3.0** (Mike, MikeRust) → riusabile direttamente;
   - codice **Apache-2.0** (Lavern) → assorbibile in AGPL-3.0 (compatibilità a senso unico);
   - codice senza licenza o con licenza incompatibile → **solo ispirazione**, niente copia di codice.
3. Per ogni porzione di codice riusata si registra fonte, file e licenza in un file `THIRD_PARTY.md`/`NOTICE`, conservando header di copyright e testi di licenza originali.

## Conseguenze
- Idee, architettura e UX delle repo di riferimento sono imitabili liberamente (non coperte da copyright); il vincolo riguarda solo il codice testuale.
- Prima di integrare codice esterno: verificare la licenza della fonte (e delle sue dipendenze) e aggiornare `THIRD_PARTY.md`.
- Niente marchi, nomi o loghi di terzi.
