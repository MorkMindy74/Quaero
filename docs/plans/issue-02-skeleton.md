# Piano operativo — Issue #2: Scheletro app (Tauri ↔ Rust) + i18n IT/EN

> Mini piano. **Non è codice**: è la mappa da confermare a inizio sessione prima di sviluppare in TDD.
>
> **Frase chiave:** la #2 non deve produrre funzionalità, deve produrre **confini architetturali corretti**. È la differenza tra un progetto che parte bene e un prototipo che dopo tre settimane diventa fragile.

## Obiettivo della slice

Walking skeleton robusto: l'app desktop si avvia, mostra un guscio minimo del workspace, il frontend fa un round-trip IPC end-to-end verso il backend Rust, e sono cablate da subito le fondamenta trasversali — Cargo workspace con `core` pura, stack frontend, i18n IT/EN, sicurezza Tauri, IPC tipizzato, toolchain bloccata e CI minima. **Nessuna logica di dominio.**

## Stack fissato (ADR-0010 / ADR-0002)

- **Backend**: Tauri 2.x + Rust (toolchain MSVC su Windows).
- **Frontend**: React + TypeScript + **Vite** + Tailwind CSS + **i18next / react-i18next**.
- **Test**: `cargo test` (Rust) + **Vitest** + React Testing Library (frontend), con mock IPC Tauri (`@tauri-apps/api/mocks`).
- **Package manager**: pnpm (workspace).

## Struttura cartelle (convenzione Tauri `src-tauri`)

```
quaero/
├─ Cargo.toml                 # workspace Rust
├─ Cargo.lock
├─ rust-toolchain.toml        # toolchain bloccata
├─ package.json               # script root + packageManager pinnato
├─ pnpm-lock.yaml
├─ pnpm-workspace.yaml
├─ .github/workflows/ci.yml   # CI minima
├─ crates/
│  └─ core/                   # tipi condivisi PURI (serde, zero Tauri)
│     ├─ Cargo.toml
│     └─ src/lib.rs
├─ apps/
│  └─ desktop/
│     ├─ src-tauri/
│     │  ├─ Cargo.toml
│     │  ├─ tauri.conf.json
│     │  ├─ capabilities/default.json   # permission minime
│     │  └─ src/
│     │     ├─ lib.rs · main.rs
│     │     └─ commands/ping.rs
│     └─ frontend/
│        ├─ package.json · vite.config.ts · index.html
│        └─ src/
│           ├─ app/ · components/ · main.tsx
│           └─ i18n/{index.ts, locales/{it.json, en.json}}
└─ UX/index.html              # SOLO riferimento estetico (design vero = #3)
```

## Contratto IPC tipizzato (ADR-0011)

- Comando iniziale **`ping`** (non `greet`), con tipi dedicati: `PingRequest`, `PingResponse`, `CommandError`, serializzabili con `serde`.
- Frontend chiama via `@tauri-apps/api/core` (`invoke("ping", …)`); **niente** `window.__TAURI__` globale.
- **`core` resta pura**: tipi + serde, zero dipendenze da Tauri. Tauri dipende da `core`, mai il contrario; i comandi fanno solo mapping IPC ↔ core.

## Sicurezza Tauri (ADR-0011) — già nella #2

- Creare `src-tauri/capabilities/default.json` con le sole permission necessarie.
- NON abilitare filesystem, shell, http/network, clipboard (nessuna slice li richiede ancora).
- Configurazione capabilities esplicita in `tauri.conf.json`.

## i18n (ADR-0005) — regola precisa

Nella #2: **tutte le stringhe visibili del guscio UI** passano da i18n (it/en). Italiano default. Esempio chiavi (`it.json`):

```json
{
  "app.name": "Quaero",
  "workspace.welcome": "Cosa vuoi cercare, analizzare o costruire oggi?",
  "nav.workspace": "Workspace",
  "nav.matters": "Pratiche",
  "nav.knowledge": "Knowledge",
  "action.ping": "Test connessione"
}
```

## Toolchain bloccata + script root

- File di pinning: `rust-toolchain.toml`, `Cargo.lock`, `pnpm-lock.yaml`, `packageManager` in `package.json`.
- Vite dentro Tauri: porta fissa + `strictPort: true`; `tauri.conf.json` con `devUrl`, `frontendDist`, `beforeDevCommand`, `beforeBuildCommand`.
- Script root (illustrativi):

```json
{
  "scripts": {
    "dev": "pnpm --filter @quaero/desktop dev",
    "build": "pnpm --filter @quaero/desktop build",
    "test": "pnpm --filter @quaero/desktop test",
    "check": "pnpm test && cargo test --workspace",
    "lint": "pnpm --filter @quaero/desktop lint"
  }
}
```

## Prerequisiti Windows + verifica ambiente

Tauri su Windows richiede Microsoft C++ Build Tools (MSVC) e Microsoft Edge WebView2; toolchain Rust MSVC consigliata. Node 20.19+ (o 22.12+) per Vite; Vitest richiede Node ≥20 e Vite ≥6.

Comando di verifica ambiente (da documentare in README/START_HERE):

```
rustc --version ; cargo --version ; node --version ; pnpm --version ; rustup show
```

## CI minima (`.github/workflows/ci.yml`) — già nella #2

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm install --frozen-lockfile
pnpm test
pnpm build
```

(Niente build dell'installer Tauri: è la #4.)

## Test minimi (TDD)

1. **Rust unit test**: il comando/handler `ping` restituisce `PingResponse`; la crate `core` compila/testa in isolamento.
2. **Frontend test (Vitest + RTL)**: la UI chiama `invoke("ping")`, il **mock IPC** restituisce una risposta, la risposta viene mostrata a schermo.
3. **Workspace**: `cargo test --workspace`, `pnpm test`, `pnpm build` passano.

## Acceptance criteria (rivisti)

- [ ] Repo configurata come Cargo workspace
- [ ] `crates/core` compila in isolamento e non dipende da Tauri
- [ ] App Tauri desktop si avvia in dev mode su Windows
- [ ] Frontend Vite/React/TypeScript mostra un guscio minimo del workspace
- [ ] IPC round-trip dimostrabile con comando `ping`
- [ ] `ping` usa request/response tipizzati e serializzabili (`serde`)
- [ ] Test Rust del comando/core
- [ ] Test frontend con mock IPC
- [ ] i18n IT/EN configurato; italiano lingua predefinita
- [ ] Switch lingua funzionante su almeno una stringa visibile
- [ ] Tauri capabilities minime configurate; nessuna permission non necessaria abilitata
- [ ] `cargo test --workspace`, `pnpm test`, `pnpm build` passano
- [ ] CI minima attiva (fmt, clippy, test Rust, test frontend, build frontend)
- [ ] Comando di verifica ambiente documentato
- [ ] Codice, commenti e commit in inglese

## Rischi tecnici

- **Toolchain Windows**: Rust MSVC + WebView2 + C++ Build Tools. Verificare l'ambiente prima di iniziare.
- **Firma installer**: NON in questa slice (è la #4). Qui l'app gira in dev, non firmata.
- **Tentazione di anticipare il design**: il guscio resta grezzo; la cura estetica è la #3.

## Cosa NON fare nella #2

- ❌ Nessuna logica di dominio: niente Pratiche, Fascicoli, Fonti, citazioni, AI.
- ❌ Nessun pannello reale (Evidence, Agent Run, chat funzionante) — solo il guscio.
- ❌ Nessun installer né firma (sono la #4).
- ❌ Nessun design system rifinito (è la #3): guscio tematizzato in modo essenziale.
- ❌ Niente connettori, nessuna rete/permission verso l'esterno.
- ❌ Non saltare i test per "andare veloci": la #2 fissa il pattern TDD per tutto il resto.

## Riferimenti

- Tauri — Calling Rust / `invoke`: https://v2.tauri.app/develop/calling-rust/
- Tauri — Mocking IPC nei test: https://v2.tauri.app/develop/tests/mocking/
- Tauri — Capabilities (sicurezza): https://v2.tauri.app/security/capabilities/
- Tauri — Vite (`devUrl`, `strictPort`): https://v2.tauri.app/start/frontend/vite/
- Tauri — Prerequisiti (Windows): https://v2.tauri.app/start/prerequisites/
- Vite — Guide: https://vite.dev/guide/
- Vitest — Guide: https://vitest.dev/guide/
- i18next: https://www.i18next.com/
- React Testing Library: https://testing-library.com/docs/react-testing-library/intro/
