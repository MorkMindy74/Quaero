# Come si contribuisce a Quaero

Regole di processo per lo sviluppo. Si applicano a ogni slice e a ogni hotfix.
(In italiano per ora; vedi ADR-0005 per la strategia linguistica.)

## Regola d'oro: niente commit diretti su `main`

`main` è sempre verde e protetto. **Nessuno committa direttamente su `main`.**
Ogni modifica — feature, slice, hotfix, doc — passa da un branch dedicato e una Pull Request.

## Flusso per ogni slice / hotfix

1. **Branch dedicato** dalla `main` aggiornata:
   - slice: `slice/<n>-<breve-descrizione>` (es. `slice/3-cockpit-shell`)
   - hotfix: `hotfix/<breve-descrizione>` (es. `hotfix/ci-pnpm-setup`)
2. **Sviluppo in TDD** (vedi le skill di processo): test prima, codice minimo per il verde.
3. **Commit puliti**, messaggi in inglese, descrittivi.
4. **Push** del branch e **apertura PR** verso `main`. Nel corpo: `Closes #<issue>` per collegare e chiudere automaticamente la issue al merge.
5. **CI verde obbligatoria**: la PR non si mergia finché la CI non è verde.
6. **Squash merge** (storia lineare e pulita).
7. **Delete branch** dopo il merge (locale e remoto).
8. Aggiornare la `main` locale: `git checkout main && git pull --prune`.

## Gate di qualità (devono essere verdi)

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `pnpm test`
- `pnpm build`

La CI (`.github/workflows/ci.yml`) esegue esattamente questi gate.

## Smoke test umano prima del merge (regola stabile)

I gate automatici **non bastano** per le feature che cambiano il comportamento a
runtime o che l'utente vede/usa direttamente. **Ogni feature visibile, rischiosa
o che introduce nuovo comportamento runtime richiede uno smoke test umano prima
del merge.** Vale in particolare per:

- provider LLM **reale** (locale o remoto);
- OSINT / raccolta dati;
- export **DOCX/PDF**;
- import / parsing documenti;
- UI importante;
- privacy/security a **runtime**;
- accesso **byte/filesystem**;
- **dati cliente**;
- connettori esterni.

Lo smoke test è una verifica manuale del comportamento reale (non solo test
verdi): si avvia l'app/funzione e si osserva l'esito atteso. Va fatto **prima**
del merge; se per ragioni di processo non è stato possibile (regola introdotta
dopo l'apertura della PR), va eseguito **subito dopo il merge e prima di iniziare
qualunque altra slice**, annotandolo nel checkpoint.

## Ambiente Windows

I comandi `cargo` richiedono l'ambiente VS Build Tools caricato: usare la
**"x64 Native Tools Command Prompt for VS 2022"** o caricare `vcvars64.bat`,
altrimenti il linker MSVC non è nel PATH. Vedi `START_HERE.md`.

## Documenti di riferimento

- `START_HERE.md` — checkpoint di sessione (stato, ADR, issue, ordine).
- `MANIFESTO.md` — l'idea forte di Quaero.
- `CONTEXT.md` — glossario di dominio.
- `docs/adr/` — decisioni architetturali.
- `docs/plans/` — piani operativi per slice.
