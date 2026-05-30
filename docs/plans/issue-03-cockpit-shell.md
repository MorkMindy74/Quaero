# Piano operativo — Issue #3: Cockpit shell (scope ridotto)

> ⚠️ **BOZZA — NON APPROVATO.** Piano di esecuzione della #3. Si approva (e si crea il branch `slice/3-cockpit-shell`) **solo dopo** il merge della PR documentale e il via dell'utente.
>
> Riferimento di build vincolante: [`docs/design/issue-03-screen-spec.md`](../design/issue-03-screen-spec.md) (Screen Spec v0.2). Contesto: [`docs/design/ux-research.md`](../design/ux-research.md).

## Esito della UX Discovery

La fase di UX Discovery ha prodotto la **Design Direction v0.2 / Screen Spec** (Claude Design). Direzione fissata: **Hybrid professional workspace** ("build the legal cockpit, not the chat"). La spec definisce schermata, token, tipografia, acceptance e implementation brief. Questo piano **riduce l'ambito implementativo** della #3 ai soli componenti necessari per la shell.

## Obiettivo

Renderizzare la schermata *"Matter workspace open"* come **shell React a 5 regioni**, con mode switcher e tab del pannello destro funzionanti su **contenuto mock statico**. Nessun dominio, nessuna AI, nessuna persistenza.

## Componenti in scope per la #3

**Shell / regioni:**
- `AppShell` — grid 5 regioni, theme + language context
- `TopCommandBar` — strip globale (wordmark, ⌘K, matter, model/status, IT/EN, settings)
- `LeftSidebar` — nav (Workspace/Pratiche/Knowledge) + liste Recenti/Pinned
- `MainWorkspace` — header pratica + surface del modo attivo
- `RightContextPanel` — pannello evidence permanente con 6 tab
- `SettingsBlock` — identità + accesso settings (ancorato in fondo alla sidebar)
- `ModeSwitcher` — segmented control sui 5 modi (deve funzionare)
- `StatusStrip` — strumentazione + segnale "local & private"

**Component primitives minimi:**
- `Button` · `Panel` · `Badge` · `SearchInput` · `TabButton` · `ListRow`

> **Nota di raccordo con la Screen Spec v0.2:** la spec elenca 14 componenti, inclusi i leaf `SourceCard`, `ExcerptCard`, `ReasoningStep`, `GenealogyPreview` e i controlli `CommandPaletteTrigger`, `MatterSelector`. In questo scope ridotto i contenuti dei 6 tab e dei 5 modi sono renderizzati **componendo le primitive** (`Panel`/`ListRow`/`Badge`) con dati mock. Da confermare in fase di approvazione se i leaf-card e `MatterSelector`/`CommandPaletteTrigger` entrano nella #3 (come da acceptance §10 della spec: ⌘K stub, MatterSelector che aggiorna l'header) o se alcuni slittano a una slice di rifinitura. **Decisione dell'utente all'approvazione.**

## Cosa DEVE funzionare (interazioni)

- **ModeSwitcher** commuta i 5 modi (placeholder).
- **RightContextPanel** cambia tra le 6 tab (default Sources).
- **MatterSelector** aggiorna la label dell'header *(se incluso — vedi nota di raccordo)*.
- **⌘K** apre/chiude un overlay stub *(se incluso)*.
- **Toggle IT/EN** (regressione #2) e **ping** (regressione #2) restano verdi.

## Cosa resta FUORI dalla #3

- ❌ niente **Pratiche reali** (solo liste/selettore mock)
- ❌ niente **Fonti reali** (solo card/righe mock)
- ❌ niente **AI** (nessuna chiamata a modelli)
- ❌ niente **ingestion** di documenti
- ❌ niente **persistenza** (stato si resetta al reload)
- ❌ niente **backend nuovo** (`core`/`src-tauri` invariati)
- ❌ niente **GenealogyGraph reale** (solo preview/placeholder)
- ❌ niente **installer** (è la #4)
- ❌ niente **#5 anticipata** (nessun modello dati Pratica/Fascicolo/Fonte)
- ❌ niente **dark mode** (luce di default; dark differita post-#3)

## Design tokens & tipografia

Da `docs/design/issue-03-screen-spec.md` §8–§9: token come CSS variables (background/panel/parchment/border/text/muted + accenti source/human/verified/warning); **pergamena solo** su documento/estratto/citazione/genealogia. Font self-hosted woff2: Newsreader (heading) · Public Sans (UI) · IBM Plex Mono (status/provenienza); **no CDN a runtime**.

## File / cartelle (frontend only)

```
apps/desktop/frontend/
├─ tailwind.config.js          # design token (§8) come tema
├─ src/index.css · src/styles/  # base tipografica + @font-face self-host
├─ public/fonts/                # woff2 self-hosted (Newsreader/Public Sans/IBM Plex Mono)
├─ src/components/ui/           # primitives: Button, Panel, Badge, SearchInput, TabButton, ListRow
├─ src/components/shell/        # AppShell, TopCommandBar, LeftSidebar, MainWorkspace,
│                               #   RightContextPanel, SettingsBlock, ModeSwitcher, StatusStrip
├─ src/app/App.tsx              # compone AppShell (sostituisce il guscio minimo #2)
├─ src/mock/                    # array mock statici (pratiche, fonti, estratti, step, nodi)
└─ **/*.test.tsx                # smoke test (§11)
```
**Nessun file Rust toccato.** Regressioni #2 (`ping`, toggle lingua) preservate.

## Acceptance criteria

Vedi [`issue-03-screen-spec.md` §10](../design/issue-03-screen-spec.md). In sintesi: 5 regioni visibili · pannello destro permanente · ModeSwitcher sui 5 modi · status local/privacy presente · nessun dominio/AI/persistenza/installer · #2 verde.

## Test frontend minimi

Vedi spec §11: render 5 regioni · mode toggle · pannello destro presente (default Sources) · tab switch · toggle IT/EN non regredito · ping #2 intatto.

## Note di processo

- Branch dedicato: `slice/3-cockpit-shell` (da creare **solo dopo** merge della PR documentale + approvazione).
- PR verso `main` con `Closes #3`, CI verde obbligatoria, squash merge, delete branch (vedi `CONTRIBUTING.md`).
