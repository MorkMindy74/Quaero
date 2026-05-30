# ADR-0011 — Sicurezza Tauri, IPC tipizzato e confine core/Tauri

## Stato
Accettata — 2026-05-30

## Decisione
Tre vincoli architetturali, in vigore fin dalla slice #2 (scheletro):

1. **Capabilities minime.** Le permission Tauri sono configurate esplicitamente in `src-tauri/capabilities/`; si abilita **solo** ciò che serve. In particolare NON si abilitano filesystem, shell, network/http, clipboard finché una slice non li richiede. Non si usa il globale `window.__TAURI__`: il frontend chiama il backend via `@tauri-apps/api/core` (`invoke`).
2. **IPC tipizzato.** I comandi non usano firme sparse tipo `greet(name) -> String`, ma tipi request/response dedicati e serializzabili con `serde` (es. `PingRequest`, `PingResponse`, `CommandError`).
3. **Confine core ↔ Tauri.** La crate `core` resta **pura**: tipi condivisi + `serde`, **zero dipendenze da Tauri**. È Tauri (`apps/desktop/src-tauri`) a dipendere da `core`, mai il contrario; i comandi Tauri fanno solo il mapping tra IPC e `core`.

## Perché
Quaero tratta documenti legali riservati: il modello di sicurezza deve nascere corretto, non essere aggiunto dopo (Tauri v2 usa le capabilities come meccanismo di controllo dei permessi). L'IPC tipizzato e la purezza di `core` mantengono il sistema modulare e testabile man mano che arriveranno comandi per documenti, fonti, pratiche, estratti e genealogia — coerente con il monorepo a moduli isolati (ADR-0003).

## Conseguenze
- `core` è testabile in isolamento senza il runtime Tauri.
- Aggiungere una permission è una decisione esplicita e tracciabile, non un default.
- I test del round-trip IPC includono un test frontend con **mock IPC** (`@tauri-apps/api/mocks`), non solo un test Rust.
