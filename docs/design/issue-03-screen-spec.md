# Quaero — Screen Spec v0.2 (Issue #3)

> Trascrizione tecnica della **Quaero Design Direction v0.2 — Screen Spec** prodotta da Claude Design (30 May 2026). Supersedes la direzione v0.1. È la *build sheet* della #3: una schermata, 14 componenti, 5 modi, 6 tab di contesto — con contenuto, stati, interazioni e una linea netta tra ciò che è *mockato* nella #3 e ciò che è *fuori scope*. **Nessun codice qui.**
>
> Scope: **frontend shell only (React)** · Schermata: *"Matter workspace open"* · Dati: **mock statici, no backend** · Dipende da #2 (i18n) che deve restare verde.
>
> **Deliverable:** una shell a 5 regioni renderizzata, con un mode switcher funzionante su contenuto placeholder.

## 1. Direzione visuale

**Build the legal cockpit, not the chat.** Base = **Hybrid professional workspace**: sfondo carta calda, testo inchiostro, slate profondo per la struttura (~90% di ogni schermata). La **pergamena è semantica, non decorativa**: solo su documenti, estratti, citazioni e genealogia. Hairline (bordi 1px) invece di ombre; ombre morbide riservate all'unico layer flottante (palette overlay). Il colore porta significato (fonte/validazione/verificato/warning), mai come decorazione. **Niente** dark di default, **niente** viola, **niente** gradienti, **niente** card SaaS generiche, **niente** clone chat-first di Mike, **niente** effetto landing-page.

## 2. Layout a 5 regioni — schermata "Matter workspace open"

L'unica schermata che la #3 deve renderizzare: una Pratica è aperta; il modo *Conversation* è attivo; il pannello destro mostra *Sources*.

```
┌─ TopCommandBar (02) ───────────────────────────────────────────────┐
│ ◴ Quaero   ⌘K   Pratica: Rossi c. Bianchi ▾   ● local  IT/EN  model ▾  ⚙ │
├───────────────┬───────────────────────────────┬────────────────────┤
│ LeftSidebar 03│ MainWorkspace 04              │ RightContextPanel 05│
│ Workspace     │ ModeSwitcher 09:             │ Sources Excerpts    │
│ Pratiche      │ Conversation·Review·Drafting │ Reason Memory       │
│ Knowledge     │ ·Reasoning·Genealogy         │ Geneal. Agent       │
│ Recenti…      │                              │                     │
│ Pinned…       │ Rossi c. Bianchi             │ • Contratto.pdf     │
│               │ Inadempimento · Fascicolo 04 │ • Art. 1453 c.c.    │
│ SettingsBlock │ ▤ ACTIVE MODE SURFACE        │ • Cass. 12345/2024  │
│ 06            │   (placeholder)              │                     │
├───────────────┴───────────────────────────────┴────────────────────┤
│ StatusStrip (14): ● local & private · index 100% · matter: aperta · citazioni 6/6 │
└─────────────────────────────────────────────────────────────────────┘
```

Regioni: **02 TopCommandBar · 03 LeftSidebar · 04 MainWorkspace · 05 RightContextPanel · 14 StatusStrip.**

## 3. Component tree

```
<AppShell>                       // 01 — owns grid, theme, language context
├─ <TopCommandBar>               // 02
│  ├─ <CommandPaletteTrigger />  // 07
│  ├─ <MatterSelector />         // 08
│  └─ model · IT/EN · settings entry
├─ <LeftSidebar>                 // 03
│  ├─ nav · recenti · pinned
│  └─ <SettingsBlock />          // 06
├─ <MainWorkspace>               // 04
│  ├─ <ModeSwitcher />           // 09 — Conversation·Review·Drafting·Reasoning·Genealogy
│  └─ <ActiveModeSurface />      // placeholder per mode (§4 modes)
├─ <RightContextPanel>           // 05 — tabs (§5)
│  ├─ <SourceCard />             // 10
│  ├─ <ExcerptCard />            // 11
│  ├─ <ReasoningStep />          // 12
│  └─ <GenealogyPreview />       // 13
└─ <StatusStrip />               // 14
```

## 4. Component specs (×14)

Per ognuno: scopo · contenuto visibile · stati minimi · interazioni minime · cosa è mockato in #3 · cosa resta fuori.

1. **AppShell** (root) — Owns the 5-region CSS grid; provides theme + language context. Visible: nulla di proprio (compone le 4 regioni + status strip). Stati: `default` (dark futuro, non in #3). Interazioni: nessuna diretta; tiene lo stato lingua IT/EN. Mock: region size fisse, nessun collapse responsive richiesto. Out: routing, auth, persistence, window chrome/installer.
2. **TopCommandBar** (regione 1) — strip globale di comando+orientamento, non scrolla mai. Visible: wordmark · ⌘K · MatterSelector · model/status · IT/EN · settings entry. Stati: `default` · dot model online/offline. Interazioni: click sui figli (delegato); barra statica. Mock: nome modello + dot hard-coded. Out: switch modello reale, notifiche, account menu.
3. **LeftSidebar** (regione 2) — navigazione primaria + accesso pratiche; settings ancorato in fondo. Visible: Workspace · Pratiche · Knowledge; liste Recenti & Pinned; SettingsBlock. Stati: nav item `active/idle/hover`; lista vuota ("nessuna pratica"). Interazioni: seleziona destinazione (evidenzia, no route); hover. Mock: liste statiche; active via local state. Out: caricamento reale pratiche, drag-pin, collapse/resize rail.
4. **MainWorkspace** (regione 3) — ospita header pratica, ModeSwitcher, surface del modo attivo. Visible: titolo+meta pratica · tab modi · placeholder modo attivo. Stati: 1 di 5 modi attivo; empty "no-matter". Interazioni: reagisce a ModeSwitcher; scambia il placeholder. Mock: ogni modo = placeholder etichettato (§4 modes). Out: chat reale, rendering documenti, logica editor di drafting.
5. **RightContextPanel** (regione 4) — superficie *evidence* permanente, **mai un drawer**, sempre visibile. Visible: 6 tab (§5) + lista card del tab attivo. Stati: tab attivo; per-tab `populated/empty`. Interazioni: cambia tab (local state); card statiche. Mock: 2–3 card mock per tab; Sources attivo di default. Out: retrieval live, verifica citazioni, scritture in memoria.
6. **SettingsBlock** (in sidebar) — identità + accesso impostazioni, ancorato in fondo. Visible: avatar/iniziali · nome avvocato · ⚙. Stati: `default/hover`. Interazioni: click → non apre nulla ancora (noop/log). Mock: identità statica ("Avv. M. Rossi"). Out: pannello settings reale, piano/tier, sign-out.
7. **CommandPaletteTrigger** (in top bar) — accesso alla palette ⌘K; segnala il design command-first. Visible: bottone con hint ⌘K + label. Stati: `default/hover/focus`. Interazioni: click / ⌘K → apre overlay stub (lista vuota). Mock: overlay apre/chiude; nessun comando reale. Out: command registry, fuzzy search, azioni.
8. **MatterSelector** (in top bar) — mostra + cambia la Pratica aperta. Visible: nome pratica + ▾; dropdown di pratiche mock. Stati: `closed/open`; item selezionato. Interazioni: apri menu, scegli pratica → aggiorna label header. Mock: 3–4 pratiche statiche; la selezione guida solo il titolo. Out: load dati pratica, create/close.
9. **ModeSwitcher** (in workspace) — cambia il workspace tra i 5 modi. Visible: segmented control Conversation · Review · Drafting · Reasoning · Genealogy. Stati: 1 segmento `active`; hover/focus. Interazioni: click → set modo attivo (local state). **Deve funzionare in #3.** Mock: lo switch scambia solo placeholder. Out: funzionalità per-modo, persistenza ciclo tastiera.
10. **SourceCard** (parchment leaf) — una fonte (doc, norma, giurisprudenza). Visible: tipo · titolo · meta breve (data/cite); superficie pergamena. Stati: `default/hover/selected`; dot `verified` (mock). Interazioni: click seleziona (evidenzia); no navigazione. Mock: props da array statico di 3 fonti. Out: apertura documento, stato verifica reale.
11. **ExcerptCard** (parchment leaf) — un passaggio citato (Estratto) ancorato a una fonte. Visible: testo citazione · ref fonte · ancora (§/pag); pergamena + bordo brass. Stati: `default/hover`; badge "anchored". Interazioni: click → scrollerebbe la fonte (noop in #3). Mock: 2 estratti statici con ancore finte. Out: risoluzione ancora reale, sync highlight.
12. **ReasoningStep** (leaf) — un passo della reasoning trace dell'AI. Visible: indice step · claim breve · n. fonti collegate. Stati: `default`; marker `verified/unverified` (mock). Interazioni: expand/collapse (local). Mock: 3 step statici; flag verificato hard-coded. Out: chain-of-thought reale, grounding live.
13. **GenealogyPreview** (parchment leaf) — lineage compatto: Fonte → Bozza v1 → v2 → Documento. Visible: catena di nodi con label versione + marker sigillo validazione. Stati: nodo `AI / human-validated`; nodo corrente. Interazioni: hover nodo → tooltip (mock); click noop. Mock: catena statica 3–4 nodi. Out: grafo genealogia completo, diff, restore versione.
14. **StatusStrip** (regione 5) — strumentazione discreta; porta il segnale privacy. Visible: `local & private` · % indicizzazione · stato pratica · conteggio verifica citazioni. Stati: privacy `local/online`; index `idle/running/done`. Interazioni: nessuna (read-only); tooltip opzionale. Mock: valori statici; "local & private" sempre mostrato. Out: pipeline di indicizzazione reale, motore di verifica.

## 5. Right context panel — tab spec

Sei tab, sempre presenti. **Default attivo = Sources.** Ogni tab ha un tipo di card e un empty state. Il pannello è **permanente, non collassa mai** in #3.

| Tab | Contenuto | Card | Empty state |
|---|---|---|---|
| **Sources** *(default)* | tutte le Fonti della pratica | SourceCard (10) | "Nessuna fonte caricata." |
| **Excerpts** | Estratti di Fonte ancorati | ExcerptCard (11) | "Nessun estratto." |
| **Reasoning** | reasoning trace compatta | ReasoningStep (12) | "Nessun ragionamento registrato." |
| **Memory** | memoria pratica / contesto salvato | righe chiave–nota | "Memoria vuota." |
| **Genealogy** | lineage dell'output attivo | GenealogyPreview (13) | "Nessuna genealogia." |
| **Agent Activity** | log run agenti (idle/running/done) + ultimo index | righe status | "Nessuna attività." |

## 6. Workspace modes (×5)

Il ModeSwitcher commuta queste 5 surface dentro MainWorkspace. In #3 ognuna è un **placeholder etichettato** — nessun comportamento reale. *Reasoning* e *Genealogy* appaiono anche come tab del pannello destro: il modo è la superficie di lavoro a tutta larghezza, la tab è il riassunto di contesto persistente.

1. **Conversation** *(default)* — Q&A grounded sulle Fonti della pratica. Placeholder: thread vuoto + composer disabilitato + riga "grounded in N sources".
2. **Review** — review documenti/evidenze: griglia Estratti / review-table. Placeholder: superficie pergamena + skeleton tabella (col: Fonte · Estratto · Esito).
3. **Drafting** — Output → Bozza → Documento. Canvas di bozza con genealogia + sigillo validazione. Placeholder: blocco "documento" pergamena + marker sigillo AI/umano, non editabile.
4. **Reasoning** — reasoning trace a tutta larghezza. Placeholder: lista verticale di 3 ReasoningStep, flag verificato mock.
5. **Genealogy** — lineage documento a dimensione piena. Placeholder: catena orizzontale di nodi (GenealogyPreview espanso).
- **No-matter state** — nessuna pratica aperta: prompt calmo a selezionare/aprire una pratica. Placeholder: "Seleziona una pratica" centrato + azione apri.

## 7. Visual hierarchy

1. **Cattura per primo l'occhio:** titolo pratica + contenuto documento/fonte attivo nel workspace. Type più grande (serif), massimo contrasto.
2. **Sempre visibile:** le 5 regioni, il pannello destro, matter selector, trigger ⌘K, status local/privacy. Mai nascosti, mai scrollati via.
3. **Secondario:** liste di navigazione, label modi, indicatore modello, label tab. Presenti e leggibili, ma più quieti (sans/mono, contrasto minore).
4. **Non deve mai competere:** chrome, bordi, stile command-bar, decorazioni. Mai sovrastare documento o fonti.

## 8. Design tokens (provvisori)

Valori direzionali — l'hex finale può variare, ma ruolo, temperatura e contrasto relativo sono fissi. Gli accenti condividono lightness/chroma variando la tinta (da definire in OKLCH).

| Token | Direzione | Ruolo |
|---|---|---|
| `background` | warm off-white · ~`#FAF8F3` | base app / canvas workspace |
| `panel` | white on paper · ~`#FFFFFF` / `#F4EFE6` | sidebar, card, superfici rialzate |
| `parchment` | warm paper · ~`#F0E8D7` | **solo** documenti · estratti · citazioni · genealogia |
| `border` | soft warm grey · ~`#E6DFD1` | hairline, divider |
| `text` | near-black ink · ~`#1B1A17` | testo primario, titoli |
| `muted-text` | warm grey · ~`#837C6E` | meta, label, copy secondario |
| `accent-source` | muted brass · ~`#8A6C2E` | fonti, citazioni, ancore |
| `accent-human` | oxblood · ~`#7A322E` | validazione umana / sigillo override |
| `accent-verified` | muted green · ~`#3F6B4A` | citazione verificata · index OK · privacy ok |
| `accent-warning` | amber · ~`#B26B1E` | non verificato · attenzione · index stale |

## 9. Typography usage

Tre voci, ognuna con un solo compito. Nessun font fuori da questi ruoli.

- **Serif** — *giudizio & titoli documento solo* (candidato: **Newsreader**). Uso: titoli pratica, heading documento/sezione, heading bozza. Mai per controlli UI, label o body.
- **Sans** — *UI operativa* (candidato: **Public Sans**). Uso: nav, bottoni, liste, body, empty states, label modi. La voce di default dell'interfaccia.
- **Mono** — *provenienza · sistema · status* (candidato: **IBM Plex Mono**). Uso: token, citazioni, ancore, status strip, hint ⌘K, label tecniche.

**Italiano + offline:** i tre candidati coprono Latin Extended-A (à è é ì ò ù, maiuscole accentate). Requisito privacy-first/desktop: **self-host dei font** (woff2 bundled), **niente CDN Google a runtime**; definire fallback stack così la shell renderizza prima che i font si assestino.

## 10. Acceptance criteria visuali

#3 è fatta quando tutto regge. ✓ = deve essere vero · ✕ = deve essere assente.

- ✓ Shell a 5 regioni visibile: top bar · left · main · right · status.
- ✓ Pannello destro **permanente** — presente al load, mai collassato.
- ✓ ModeSwitcher funziona su tutti i 5 modi placeholder.
- ✓ MatterSelector presente e cambia la label header.
- ✓ CommandPaletteTrigger presente (⌘K apre overlay stub).
- ✓ Status local/privacy presente nello status strip.
- ✕ Nessuna logica di dominio reale — nulla calcola risultati legali.
- ✕ Nessuna AI — nessuna chiamata a modelli.
- ✕ Nessuna persistenza — lo stato si resetta al reload.
- ✕ Nessun installer/packaging — gira come frontend dev.

## 11. Test frontend minimi

Livello smoke. Render componenti + le poche interazioni che devono funzionare.

- ✓ Renderizza 5 regioni — assert che ogni root regione è nel document.
- ✓ Mode toggle — click su ogni modo setta active + scambia placeholder.
- ✓ Pannello destro presente — RightContextPanel montato; tab default = Sources.
- ✓ Tab switch — click su una tab cambia la lista card attiva.
- ✓ Toggle IT/EN ancora funzionante — switch lingua di #2 non regredito.
- ✓ Ping #2 intatto — integrazione i18n/build esistente passa ancora.

## 12. Claude Code Implementation Brief

> **Quaero · Issue #3 · frontend shell — "Build the legal cockpit, not the chat."**

- **Build:** la schermata "Matter workspace open" (§2) come shell React: `AppShell` con 5 regioni e i 14 componenti di §3–4.
- **Layout:** CSS grid — top bar (fissa) · left sidebar · main · right panel (permanente) · status strip. Il pannello destro non collassa mai.
- **Interattivo (devono davvero funzionare):** ModeSwitcher (5 modi) · tab RightContextPanel (6, default Sources) · MatterSelector (aggiorna header). Più overlay stub ⌘K + toggle IT/EN (da #2).
- **Data:** solo array mock statici — pratiche, fonti, estratti, reasoning step, nodi genealogia. No fetch, no store, no persistenza.
- **Modi:** ognuno dei 5 = superficie placeholder etichettata (§6). Nessuna funzionalità per-modo.
- **Tokens:** usa la tabella §8 come CSS variables. Pergamena solo su superfici documento/estratto/citazione/genealogia.
- **Type:** Newsreader (heading) · Public Sans (UI) · IBM Plex Mono (status/provenienza). Self-host woff2, no CDN a runtime.
- **Style guard:** no dark default · no viola · no gradienti · no card SaaS · no home chat-first · no banda landing-page (§7/§1).
- **Done when:** tutti gli acceptance §10 passano e gli smoke test §11 sono verdi; #2 resta verde.
- **Out of scope:** AI reale, retrieval, verifica, rendering documenti, editor drafting, persistenza, settings reali, installer, dark mode.

---

*Trascritto da "Quaero Design Direction v0.2 — Screen Spec" (Claude Design, 30 May 2026), che supersedes la v0.1 come riferimento di build. Mike OSS & Harvey usati solo per pattern.*
