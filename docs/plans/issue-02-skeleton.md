# Piano operativo — Issue #2: Scheletro app (Tauri ↔ Rust) + i18n IT/EN

> Mini piano. **Non è codice**: è la mappa da confermare a inizio sessione prima di sviluppare in TDD.

## Obiettivo della slice

Avere il **walking skeleton**: l'app desktop si avvia, mostra il guscio del workspace, il frontend comunica con il backend Rust in un round-trip end-to-end, e sono cablate da subito due fondamenta trasversali — il **Cargo workspace con la crate `core`** e l'**i18n** (italiano default, inglese già presente). È la base tecnica su cui poggiano tutte le altre slice; non contiene logica di dominio.

## File / cartelle che si toccheranno (atteso)

```
quaero/
├─ Cargo.toml                 # workspace Rust (membri: apps/desktop, crates/*)
├─ apps/
│  └─ desktop/                # app Tauri (entrypoint, comandi, config)
│     ├─ src/                 # codice Rust dell'app
│     └─ tauri.conf.*         # configurazione Tauri
├─ crates/
│  └─ core/                   # tipi condivisi (per ora minimi)
├─ frontend/                  # UI servita nel webview (evoluzione di UX/index.html)
│  └─ locales/                # it.(json) , en.(json)
└─ (build tooling frontend: es. Vite)
```

Nota: `UX/index.html` resta come riferimento estetico; il porting reale del design è la #3, non la #2.

## Stack previsto

- **Tauri 2.x** + **Rust** (backend), webview per il frontend (ADR-0002).
- **Cargo workspace** multi-crate (ADR-0003).
- **i18n** con stringhe esternalizzate in file di locale (ADR-0005); libreria i18n frontend **da confermare** a inizio sessione.
- Build tooling frontend (es. Vite) e framework UI **da confermare** (default: minimale, niente lock-in inutile).

## Test minimi (TDD)

- **Round-trip**: un comando Rust (es. `ping`/`greet`) invocato dal frontend restituisce il valore atteso → coperto da test sul lato Rust.
- **i18n**: i file di locale `it`/`en` si caricano e una chiave nota risolve nella stringa corretta per ciascuna lingua; l'italiano è il default.
- **Workspace**: `cargo test` gira sull'intero workspace e la crate `core` compila/testa in isolamento.

## Acceptance criteria (da issue #2)

- [ ] L'app si avvia su Windows come applicazione Tauri e mostra il guscio del workspace
- [ ] Round-trip dimostrabile frontend → comando Rust → frontend
- [ ] Cargo workspace con crate `core` per i tipi condivisi
- [ ] Tutte le stringhe UI provengono da file di locale (`it`, `en`); switch lingua funzionante su almeno una stringa
- [ ] Italiano è la lingua predefinita
- [ ] Codice, commenti e commit in inglese

## Rischi tecnici

- **Toolchain Windows**: Tauri richiede Rust stabile, WebView2 e i build tools MSVC. Verificare l'ambiente prima di iniziare.
- **Scelta framework/i18n frontend**: rischio di lock-in o over-engineering. Mitigazione: partire minimale, decidere consapevolmente a inizio sessione.
- **Firma installer**: NON è in questa slice (è la #4). Qui l'app gira in dev, non firmata.
- **Tentazione di anticipare il design**: il guscio deve restare grezzo; la cura estetica è la #3.

## Cosa NON fare nella #2

- ❌ Nessuna logica di dominio: niente Pratiche, Fascicoli, Fonti, citazioni, AI.
- ❌ Nessun pannello reale (Evidence, Agent Run, chat funzionante) — solo il guscio.
- ❌ Nessun installer né firma (sono la #4).
- ❌ Nessun design system rifinito (è la #3): qui basta un guscio tematizzato in modo essenziale.
- ❌ Niente connettori, nessuna rete verso l'esterno.
- ❌ Non saltare i test per "andare veloci": la #2 fissa il pattern TDD per tutto il resto.
