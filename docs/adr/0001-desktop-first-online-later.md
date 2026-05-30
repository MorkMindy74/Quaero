# ADR-0001 — Desktop-first, online in futuro

## Stato
Accettata — 2026-05-30

## Contesto
Quaero è un assistente legale AI per il diritto italiano, usato da avvocati con dati di clienti coperti da segreto professionale e GDPR. La promessa di prodotto è "nessun dato della pratica esce dalla macchina". Gli utenti hanno scarsa confidenza con l'informatica: il prodotto deve installarsi e funzionare come un normale programma.

## Decisione
Quaero è primariamente un'**applicazione desktop locale**. I dati delle pratiche risiedono e vengono elaborati sulla macchina dell'utente. Una eventuale versione **online resta possibile in futuro**: i moduli vanno progettati dietro interfacce che non assumano l'esecuzione locale, così che un backend remoto opzionale possa essere aggiunto senza riscrivere il nucleo.

## Conseguenze
- Privacy-by-design come default; i connettori esterni sono opt-in e segnalati (vedi Privacy guard nel PRD).
- I moduli non devono dipendere da dettagli "solo-locale" (es. path assoluti, accesso diretto al filesystem sparso): l'accesso a storage/rete passa da interfacce astratte.
- L'esperienza di installazione e autenticazione deve essere banale per utenti non tecnici (vedi [[0002-tauri-rust-stack]]).
