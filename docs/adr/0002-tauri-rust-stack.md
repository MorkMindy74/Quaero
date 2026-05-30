# ADR-0002 — Stack tecnologico: Tauri + Rust

## Stato
Accettata — 2026-05-30

## Contesto
Quaero è desktop-first (vedi [[0001-desktop-first-online-later]]) e deve: installarsi come un normale programma con installer piccolo e firmato, essere privacy-friendly e performante, e permettere il riuso di codice da repository di riferimento. Le repo di ispirazione sono Mike (TypeScript, AGPL-3.0), MikeRust (Rust, AGPL-3.0, versione locale) e Lavern (TypeScript, Apache-2.0).

## Decisione
Il guscio applicativo è **Tauri** con backend **Rust** e frontend web. I moduli di dominio sono **crate Rust** in un Cargo workspace; il frontend evolve dal mockup `UX/index.html`.

Motivazioni:
- Installer piccolo e firmato (`.exe`/`.msi` su Windows), avvio come app nativa → adatto a utenti non tecnici.
- Rust: sicurezza di memoria, performance, ottimo per elaborazione documentale locale.
- Allineato a **MikeRust** (Rust, AGPL-3.0), da cui è possibile riusare codice. Le idee/UX di Mike e Lavern restano imitabili liberamente.

## Conseguenze
- Il riuso di codice TypeScript da Mike/Lavern richiede riscrittura/port in Rust oppure confinamento nel frontend; le idee si riusano comunque liberamente.
- La struttura a moduli isolati è realizzata come Cargo workspace (vedi [[0003-monorepo-isolated-modules]]).
- L'autenticazione iniziale deve restare semplicissima (login locale / chiave di licenza), senza configurazioni manuali.
