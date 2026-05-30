# ADR-0005 — Strategia linguistica e internazionalizzazione

## Stato
Accettata — 2026-05-30

## Contesto
Quaero nasce per il mercato italiano ma punta a diffusione mondiale. Il committente legge meglio l'italiano e non è tecnico. I costi di traduzione differiscono molto a seconda dell'artefatto: il codice e l'infrastruttura i18n sono costosi da convertire a posteriori, i documenti di pianificazione no.

## Decisione
- **Codice** (identificatori, commenti) e **messaggi di commit**: in **inglese**, da subito.
- **Interfaccia utente**: **bilingue dal primo giorno** tramite i18n, con stringhe esternalizzate in file di locale. **Italiano lingua predefinita**, inglese già presente.
- **Documenti di pianificazione** (issue, ADR, PRD, README): in **italiano per ora**; una passata di traduzione in inglese prima del lancio pubblico.

## Conseguenze
- La slice di fondazione (#2) cabla l'i18n da subito: nessuna stringa hard-coded nella UI.
- Aggiungere nuove lingue in futuro = aggiungere un file di locale, senza toccare il codice.
- Prima del lancio mondiale: tradurre README e documentazione pubblica in inglese.
