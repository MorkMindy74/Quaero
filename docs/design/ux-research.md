# UX Discovery / Visual Research — Quaero

> **Primo giro**, basato su ricerca documentale + indicazioni dell'utente. Da arricchire con gli screenshot/esempi che l'utente invierà. La direzione estetica la decide l'utente; Claude non fa design "a gusto proprio".
>
> Voci marcate **[DA VERIFICARE]** non sono confermate: servono link/screenshot/testo dall'utente.

## Esito della Discovery — Design Direction v0.2 (30 May 2026)

Claude Design ha prodotto la **Quaero Design Direction v0.2 / Screen Spec**, trascritta in [`issue-03-screen-spec.md`](./issue-03-screen-spec.md). Esiti chiave:
- Direzione **fissata su Hybrid professional workspace** ("build the legal cockpit, not the chat"); **parchment** come accento semantico (documenti/estratti/citazioni/genealogia); dark differita post-#3.
- **Design token provvisori** con ruoli/temperatura/contrasto e **tre voci tipografiche** (Newsreader serif / Public Sans UI / IBM Plex Mono provenienza), **self-hosted** (no CDN).
- Schermata di riferimento "Matter workspace open", 14 componenti, 5 modi, 6 tab di contesto, acceptance + implementation brief.
- Anti-pattern confermati: niente chat-first clone di Mike, niente AI-slop, niente card SaaS, niente viola/gradienti, niente landing-page.

Il piano operativo (scope ridotto della #3) è in [`../plans/issue-03-cockpit-shell.md`](../plans/issue-03-cockpit-shell.md). Le sezioni seguenti restano la ricerca a monte.

## Direzione (decisa dall'utente)

**Quaero deve sembrare un cockpit legale professionale, non una chat AI.**

Deve comunicare: rigore · fiducia · controllo · fonti verificabili · memoria del lavoro · AI supervisionata dall'avvocato · prodotto premium · zero effetto giocattolo · zero estetica generica da AI.

## Principi visivi provvisori

1. **Legal cockpit, non chatbot.**
2. **Fonti e ragionamento sempre visibili.**
3. **Densità informativa alta, ma ordinata.**
4. **Interfaccia calma, autorevole, non rumorosa.**
5. **Command-first, ma non solo command line.**
6. **Sidebar e pannelli come strumenti di orientamento.**
7. **Ogni risposta ha un luogo dove mostrare Fonte, Estratto, Ancora e Genealogia.**
8. **Design distintivo, ma sobrio.**
9. **Niente gradienti viola generici, niente layout SaaS anonimo, niente estetica "AI template".**

> **Base provvisoria confermata dall'utente (2026-05-30):** cockpit non chatbot · fonti e ragionamento sempre visibili · densità alta ma calma · command-first con orientamento · AI supervisionata dall'avvocato · identità non generica · design come funzione, non decorazione. *(Ancora prima bozza, da affinare con gli screenshot.)*

## Visual Directions to Explore

> Tre direzioni visive alternative. La palette carta/pergamena del mockup resta una base. **Nessuna scelta definitiva** finché non arrivano altri screenshot.
>
> **Ruoli decisi (provvisori, 2026-05-30):**
> - **Direzione 3 — Hybrid professional workspace = PREFERITA / default della shell.**
> - **Direzione 1 — Legal parchment cockpit = ACCENTO documentale** (document viewer, citazioni, estratti, genealogia, stati "legal evidence").
> - **Direzione 2 — Dark legal command center = FUTURA / SECONDARIA** (dark mode, command palette, agent activity, log, reasoning trace). Non è il default.

### Direzione 1 — Legal parchment cockpit
- **Atmosfera:** base chiara; carta, pergamena, avorio, nero morbido; studio legale e documento; serif con cautela per titoli/documenti.
- **Vantaggi:** identità distintiva ("artigianato giuridico"), calda e autorevole; coerente col mockup; lontana dall'AI-template.
- **Rischi:** se eccessiva, effetto "vintage"/poco tecnico; il serif va dosato per non penalizzare la densità.
- **Dove funziona meglio:** workspace centrale, document/citation view, lettura di Fonti/Estratti.
- **Cosa prendere dai riferimenti:** grounding e citazioni di NotebookLM; sobrietà/densità di Linear.
- **Cosa evitare:** texture pesanti, skeumorfismo "pergamena" letterale, tono consumer.

### Direzione 2 — Dark legal command center
- **Atmosfera:** fondo scuro; pannelli tecnici; forte senso di controllo; adatto a command palette, agent activity, log, reasoning.
- **Vantaggi:** percezione di strumento professionale/tecnico; ottimo contrasto per stati, log, attività agenti; "controllo".
- **Rischi:** può risultare freddo/poco da studio legale; affaticamento nella lettura prolungata di documenti; rischio "estetica da dev tool".
- **Dove funziona meglio:** command palette, agent activity, pannello ragionamento/log, viste tecniche.
- **Cosa prendere dai riferimenti:** command palette di Raycast/Linear; trasparenza del ragionamento di Harvey.
- **Cosa evitare:** dark "gamer"/neon; perdita di calore e autorevolezza legale.

### Direzione 3 — Hybrid professional workspace
- **Atmosfera:** base chiara professionale; sidebar più scura; pannelli fonti ben evidenti; meno "pergamena", più SaaS premium legale.
- **Vantaggi:** equilibrio tra leggibilità (chiaro) e controllo (sidebar scura); fonti in evidenza; familiare e premium.
- **Rischi:** scivolare nel "SaaS anonimo" se non curato; perdere l'identità distintiva.
- **Dove funziona meglio:** shell complessiva, sidebar, pannello destro fonti/ragionamento.
- **Cosa prendere dai riferimenti:** struttura/contrasto sidebar-contenuto; gerarchia dei pannelli; densità calma di Linear; fonti di NotebookLM.
- **Cosa evitare:** palette/gradienti generici; look indistinguibile da un SaaS qualsiasi.

---

## Design Philosophy (riferimento concettuale)

### "Why Great Design Is More Than Just Good Looks" — Manikandan J.

- **Tipo:** principio concettuale (non riferimento visivo primario).
- **Tesi:** il design non è decorazione, è uno strumento strategico.
- **Per Quaero:** la UI deve *servire il lavoro legale*, non solo essere gradevole. Ogni elemento ha una funzione chiara — orientare l'avvocato, mostrare fonti, ridurre il rischio d'errore, rendere visibile il ragionamento, distinguere AI da validazione umana, aumentare fiducia e controllo. La coerenza visiva è parte della credibilità del prodotto. Ogni schermata deve raccontare cosa Quaero fa: legge, organizza, cita, verifica, produce bozze, conserva genealogia.
- **Cosa prendere:** design come linguaggio di business · come fiducia · come coerenza · come intenzione · come differenziazione · come racconto del valore del prodotto.
- **Cosa NON prendere:** non trasformare Quaero in una landing page marketing; non ragionare per "poster/banner/social"; niente CTA aggressive; non sacrificare densità informativa e rigore legale per estetica.
- **Influenza:** visual identity · consistency · CTA discipline · information hierarchy · trust · professional tone · product positioning. *(Non usarlo per decidere colori/layout specifici, ma per evitare una UI bella e inutile.)*

> **Principio guida:**
> *"Quaero's design is not decoration. It is an operational interface for legal judgment."*
> *"Il design di Quaero non è decorazione. È l'interfaccia operativa del giudizio legale assistito dall'AI."*

---

## Riferimenti — analisi

> Per ognuno: 1) cos'è · 2) perché rilevante · 3) cosa prendere · 4) cosa NON copiare · 5) area Quaero influenzata · 6) rischi.

### A. Riferimenti visivi / di prodotto

#### A1. Harvey (harvey.ai) — competitor diretto
1. **Cos'è:** piattaforma AI per servizi legali e professionali; ha un "Matter OS" che unifica documenti, precedenti, giurisprudenza, email e workflow in un unico workspace context-complete. Ha un design system maturo e blog pubblici sul proprio approccio al design.
2. **Perché rilevante:** è l'analogo più vicino a Quaero. Mostra come presentare l'AI legale in modo professionale; il loro design dichiara di puntare su **trasparenza e fiducia** (l'avvocato deve capire non solo l'output ma il ragionamento dietro).
3. **Cosa prendere:** centratura sul *matter* (= Pratica) come contesto; trasparenza del ragionamento; tono enterprise affidabile; drafting nello stesso thread; gestione documenti/citazioni.
4. **Cosa NON copiare:** branding e look Harvey; il loro stile è enterprise-neutro — Quaero vuole un'identità più distintiva ("artigianato giuridico" del mockup). Niente clonazione.
5. **Area Quaero:** workspace centrale · pannello fonti/ragionamento · document/citation view · tono professionale generale.
6. **Rischi:** sembrare un clone del competitor; ereditarne la freddezza enterprise; **[DA VERIFICARE]** dettagli UI recenti (screenshot aggiornati utili).

#### A2. Linear (linear.app)
1. **Cos'è:** product tool famoso per UI rapidissima, keyboard-first, densa ma calma, con command palette (Cmd+K) e un design system molto coerente.
2. **Perché rilevante:** incarna "premium tool, non giocattolo" e "densità ordinata + calma" — principi 3, 4, 8.
3. **Cosa prendere:** command palette; navigazione da tastiera; micro-interazioni sobrie; coerenza dei componenti; sensazione di velocità e controllo.
4. **Cosa NON copiare:** la palette/gradienti tendenti al viola e l'estetica "startup SaaS"; le metafore da issue-tracker.
5. **Area Quaero:** command palette · top bar · sidebar · densità/tono generale.
6. **Rischi:** scivolare nel "troppo startup, poco studio legale" (l'utente lo ha segnalato esplicitamente).

#### A3. Raycast (raycast.com)
1. **Cos'è:** launcher/command palette estensibile, keyboard-first, azioni rapide.
2. **Perché rilevante:** modello di interazione command-first (principio 5) e pattern di command palette/azioni rapide.
3. **Cosa prendere:** ergonomia della command palette; azioni contestuali rapide; scorciatoie.
4. **Cosa NON copiare:** il modello "launcher" come centro dell'app (Quaero è un workspace, non un launcher); l'estetica molto macOS.
5. **Area Quaero:** command palette · azioni rapide.
6. **Rischi:** far sembrare Quaero un launcher invece di un cockpit legale; command-first che sacrifica i pannelli di orientamento (principio 6).

#### A4. NotebookLM (Google)
1. **Cos'è:** strumento di ricerca AI *source-grounded*: i documenti sono fonti, le risposte citano i passaggi sorgente, layout fonti / chat / note.
2. **Perché rilevante:** è il riferimento canonico per la UX *source-grounded* — mappa quasi 1:1 sul modello Affermazione→Citazione→**Estratto**→Ancora di Quaero. Clic sulla citazione → salto al passaggio esatto.
3. **Cosa prendere:** citazioni come chip che linkano all'estratto esatto; pannello fonti; il pattern a tre aree (fonti / lavoro / output); il grounding visibile.
4. **Cosa NON copiare:** l'estetica consumer e il tono "friendly/soft" di Google — troppo poco autorevole per il legale.
5. **Area Quaero:** pannello destro fonti/ragionamento · document/citation view · workspace.
6. **Rischi:** risultare troppo consumer/soft, poco "studio legale".

#### A5. Refero (refero.design)
1. **Cos'è:** ampia galleria curata di UI reali (130k+ schermate, 36k+ pagine web, flussi), navigabile per tipo di pagina / pattern UX / elementi UI. Ha anche un MCP.
2. **Perché rilevante:** **fonte di pattern**, non uno stile da adottare. Utile per studiare come prodotti maturi risolvono cockpit, sidebar, pannelli, empty states, settings.
3. **Cosa prendere:** pattern di layout e flussi reali (orientamento, empty states, settings, densità).
4. **Cosa NON copiare:** è una galleria di UI **altrui, protette da copyright** — mai copiare una schermata specifica; solo ispirazione a livello di pattern.
5. **Area Quaero:** trasversale (tutte le aree, come libreria di pattern).
6. **Rischi:** copiare troppo da vicino un singolo prodotto (estetica/licenze).

#### A6. Mike — `willchen96/mike` (analogo open-source diretto) ⭐
1. **Cos'è:** "OSS AI Legal Platform" open-source (AGPL-3.0). Stack: Next.js + Express + Supabase (Auth/Postgres) + storage S3/R2; assistente legale documentale **online/cloud**. UX dal README: **Projects** (crea/apri un progetto e "chatti con i documenti"), upload + elaborazione documenti (DOC/DOCX→PDF), **model picker** multi-provider, settings **Account > Models & API Keys**, auth/sign-up. *(La fork **MikeRust** è il nostro riferimento di codice; Mike è il riferimento di UX/feature.)*
2. **Perché rilevante:** è l'analogo open-source più vicino a Quaero — stesso dominio, stesse primitive (progetto ≈ Pratica, documenti, chat citata, settings provider). Essendo AGPL, oltre ai flussi possiamo studiarne anche il codice (compatibile, con attribuzione in `THIRD_PARTY.md`).
3. **Cosa prendere:** organizzazione **project-centric** (≈ Pratica); flusso upload → elaborazione → interrogazione documenti; settings modelli/chiavi; struttura "assistente documentale legale"; eventuali soluzioni di codice riusabili (via MikeRust).
4. **Cosa NON copiare:** l'**architettura cloud** (Supabase/R2, documenti e chiavi verso provider esterni) — opposta al privacy-first/locale di Quaero (ADR-0001/0011); la metafora **chat-first** come centro (Quaero è cockpit-first, non chat-su-documenti); lo stile visivo (da verificare, rischio "SaaS Next.js generico").
5. **Area Quaero:** workspace centrale · sidebar (lista progetti ≈ pratiche) · settings (modelli/chiavi, ma in chiave locale) · document/citation view · top bar.
6. **Rischi:** copiare Mike troppo da vicino ⇒ Quaero diventa "ennesima chat-su-documenti" cloud, tradendo i principi 1–2 e il privacy-first; licenza AGPL ⇒ se si riusa codice, attribuire.

> ⚠️ **Warning (esplicito):** Mike è un riferimento di **feature/flusso**, **da NON copiare come architettura (cloud) né come metafora (chat-first)**. Stile osservato (screenshot): SaaS chiaro e minimale — sidebar `Assistant / Projects / Tabular Review / Workflows` + Assistant History, home con saluto centrato ("Hi, …") e box chat al centro, model picker, disclaimer "AI can make mistakes. Answers are not legal advice". È proprio l'impostazione **chat-first** che Quaero **non** adotta (Quaero è cockpit-first). Mike = riferimento UX/feature; **MikeRust** = possibile riferimento tecnico/codice; **Quaero** = prodotto diverso, locale, privacy-first, cockpit-first.

### B. Riferimenti di metodo / tooling (come costruiamo, non come appare)

#### B1. Anthropic "frontend-design" plugin + "Frontend Aesthetics Cookbook"
1. **Cos'è:** plugin ufficiale Anthropic per Claude Code (`anthropics/claude-code/plugins/frontend-design`); il cookbook è un `SKILL.md` che impone un framework a 4 domande — **purpose, tone, constraints, differentiation** — chiedendo un impegno estetico esplicito prima di scrivere CSS.
2. **Perché rilevante:** combatte direttamente l'"AI slop". Cita come centro statistico da evitare: **font Inter, gradienti viola su bianco, griglie a tre card** — *esattamente* ciò che l'utente vuole evitare (principio 9).
3. **Cosa prendere:** il framework purpose/tone/constraints/differentiation come checklist prima della #3; l'obbligo di scegliere una direzione precisa.
4. **Cosa NON copiare:** i suoi esempi "bold/maximalist/animati"; Quaero vuole sobrietà autorevole, non alto impatto decorativo.
5. **Area Quaero:** processo di design / definizione design token (non un'area UI specifica).
6. **Rischi:** spingere verso un'estetica troppo "d'effetto"; usarlo per generare UI generica.

#### B2. Superdesign.dev
1. **Cos'è:** design agent open-source (MIT) dentro l'IDE; genera/itera mockup UI in parallelo da prompt, in locale.
2. **Perché rilevante:** utile per **esplorare rapidamente** varianti di layout cockpit prima di scrivere codice.
3. **Cosa prendere:** velocità di iterazione su varianti di shell; esplorazione di alternative.
4. **Cosa NON copiare:** l'output AI "di default", che tende all'estetica generica (vedi B1).
5. **Area Quaero:** processo (prototipazione layout), non UI finale.
6. **Rischi:** over-design; adottare un mockup AI generico senza la direzione dell'utente. Licenza MIT (ok), ma niente asset di terzi.

#### B3. ui-ux-pro-max-skill (nextlevelbuilder)
1. **Cos'è:** skill Claude Code di "design intelligence" (50+ stili, 161 palette, 57 abbinamenti font, 99 linee guida UX, generatore di design system, check di accessibilità).
2. **Perché rilevante:** può accelerare la costruzione coerente dei design token e i controlli di accessibilità.
3. **Cosa prendere:** check accessibilità (contrasto, focus, ARIA), suggerimenti di scala tipografica/spacing, struttura del design system.
4. **Cosa NON copiare:** palette/stili "preconfezionati" che diluiscono l'identità distintiva di Quaero.
5. **Area Quaero:** design token / design system (processo).
6. **Rischi:** estetica template generica se si accettano i preset senza la direzione dell'utente.

### C. Opzionali / da valutare

#### C1. Claim-evidence interface (concetto astratto)
1. **Cos'è:** una **categoria** di UI che collega ogni affermazione alla prova che la sostiene (fact-checking, ricerca, legal-tech): affermazione → fonte → estratto → prova, con navigazione bidirezionale. *(Nessun prodotto specifico citato. "PaperTrail" resta da identificare: non lo usiamo come riferimento finché l'utente non invia un link preciso.)*
2. **Perché rilevante:** il pattern claim↔evidence è il cuore di Quaero (Affermazione ↔ Estratto di Fonte ↔ Genealogia).
3. **Cosa prendere:** visualizzazione del legame affermazione→prova; stati "verificato / non supportato"; navigazione bidirezionale claim↔fonte.
4. **Cosa NON copiare:** —(da definire quando avremo un prodotto di riferimento concreto).
5. **Area Quaero:** document/citation view · pannello ragionamento · agent activity (verificatore).
6. **Rischi:** trarre conclusioni da un prodotto non identificato; per ora resta a livello di principio.

---

## Mappa riferimenti → aree di Quaero

| Area Quaero | Riferimenti principali |
|---|---|
| Top bar | Linear, Harvey |
| Sidebar sinistra | Linear, Refero (pattern), Harvey |
| Workspace centrale | Harvey, NotebookLM |
| Pannello destro fonti/ragionamento | **NotebookLM**, claim-evidence [DA VERIFICARE], Harvey |
| Settings | Refero (pattern), Linear |
| Command palette | **Raycast**, **Linear** |
| Document/citation view | **NotebookLM**, claim-evidence [DA VERIFICARE] |
| Agent activity | Harvey (trasparenza ragionamento), claim-evidence [DA VERIFICARE] |
| Processo/design system | frontend-design plugin, ui-ux-pro-max, Superdesign |

## Rischi globali (da tenere d'occhio)

- **Copia estetica troppo evidente** di un singolo prodotto (Harvey/Linear) → problema di distintività e potenzialmente di licenze.
- **Licenze:** si imitano idee/pattern, mai codice o asset di terzi; le gallerie (Refero) mostrano UI altrui protette.
- **Over-design:** strumenti generativi (Superdesign, cookbook) spingono verso effetti; Quaero vuole sobrietà.
- **Troppo da startup, poco da studio legale:** rischio Linear/Raycast.
- **Troppo fredda e poco usabile:** rischio enterprise (Harvey) o eccesso command-line (Raycast).

## Aperti / da verificare (mi servono input dall'utente)

- **PaperTrail:** quale prodotto esattamente? Link/screenshot.
- **Mike (mikeoss.com):** screenshot delle schermate per valutarne lo stile visivo (il README dà i flussi, non l'aspetto).
- **Screenshot** delle schermate che ti piacciono di Harvey, Linear, Raycast, NotebookLM (così annoto pattern precisi, non impressioni generali).
- Eventuali **altri prodotti** non-legal che ti piacciono esteticamente.
- Conferma su **palette**: il mockup `UX/index.html` (carta/pergamena) resta la base, o vuoi esplorare alternative?

## Sintesi finale (proposta — da approvare)

> Prima bozza di sintesi. Da validare con altri screenshot prima di approvare la #3.

### Direzione visiva preferita
**Hybrid professional workspace** — base chiara professionale; sidebar sinistra più strutturata e leggermente più scura; workspace centrale pulito; **pannello destro forte** per Fonti, Estratti, Reasoning e Genealogia; accenti carta/pergamena nelle aree documentali; tono premium, legale, affidabile; niente estetica "startup AI generica".

### Direzioni secondarie
- **Legal parchment cockpit** → accento per le aree documentali (document viewer, citazioni, estratti, genealogia, stati "legal evidence").
- **Dark legal command center** → futura/secondaria: dark mode, command palette, agent activity, log, reasoning trace.

### 10 principi visivi (proposta definitiva)
1. **Cockpit, non chatbot** — l'unità è lo spazio di lavoro per Pratica, non la conversazione.
2. **Fonti e ragionamento sempre visibili** — il pannello destro è strutturale, non un extra.
3. **Ogni affermazione ha un luogo** per Fonte → Estratto → Ancora → Genealogia (claim-evidence).
4. **Densità alta ma calma** — molta informazione, gerarchia chiara, nessun rumore.
5. **Command-first con orientamento** — command palette per la velocità, sidebar/pannelli come bussola.
6. **AI supervisionata e distinguibile** — distinzione netta tra output AI (Bozza) e validazione umana (Documento/Atto); stati espliciti.
7. **Base chiara + accenti pergamena nei documenti** — Hybrid come default, parchment come accento.
8. **Identità distintiva e sobria** — premium legale; vietati i marker "AI slop" (Inter ovunque, gradienti viola, griglie a 3 card).
9. **Coerenza = credibilità** — un solo design system (token), stati e componenti coerenti.
10. **Design come funzione, non decorazione** — ogni elemento orienta, cita, riduce il rischio, aumenta controllo e fiducia.

### Pattern UI da prendere
- Citazioni come **chip che linkano all'Estratto esatto** + salto al passaggio (NotebookLM).
- **Command palette** Cmd+K (Linear, Raycast).
- **Sidebar strutturata** di orientamento (Linear; struttura project-centric di Mike/Harvey, non lo stile).
- **Pannello destro fonti/ragionamento** sempre presente; **trasparenza del ragionamento / agent activity** (NotebookLM, Harvey).
- **Densità calma**, micro-interazioni sobrie (Linear).
- Stati **"verificato / non supportato"** (pattern claim-evidence).
- Empty states / settings curati (libreria pattern Refero).

### Pattern UI da evitare
- **Chat-first come centro** (home "Hi, …" + box chat) — Mike.
- **Architettura/UX cloud** che espone dati — Mike.
- **Estetica AI-slop**: Inter ovunque, gradienti viola, griglia a 3 card (cookbook Anthropic).
- **Dark "gamer"/neon** o **SaaS anonimo** indistinguibile.
- **CTA marketing aggressive / landing-page vibes** (articolo Manikandan J.).
- **Launcher come unica modalità** (Raycast).
- Skeumorfismo "pergamena" pesante.

### Componenti chiave della futura #3 (cockpit shell + kit minimo)
- **Layout (5 regioni):** TopBar · LeftSidebar (strutturata) · MainWorkspace · RightContextPanel (Fonti/Estratti/Reasoning/Genealogia) · SettingsBlock.
- **Command palette** a livello di shell (placeholder, contenuti dopo).
- **UI kit:** Button, IconButton, Panel, Badge, SearchInput, SegmentedControl, ListRow (+ placeholder presentazionali CitationChip/SourceCard).
- **Design token** Hybrid (base chiara, sidebar più scura, accenti parchment) + tipografia (UI sans + serif per i documenti).
- *(La #3 resta presentazionale: nessun dominio, nessuna AI, nessun installer.)*

### Domande ancora aperte
- Confermi i **10 principi** come definitivi?
- **Quanto** più scura la sidebar (tonalità) nella direzione Hybrid?
- La **command palette** entra già nella #3 (shell) o in una slice successiva?
- Quando introdurre la **dark mode** (Direzione 2)?

### Screenshot / link che mi devi ancora mandare
- **Harvey, Linear, Raycast, NotebookLM** — schermate che ami (per pattern precisi).
- **PaperTrail** — link certo del prodotto (finché non arriva, resta solo il pattern astratto claim-evidence).
- Eventuali **altri prodotti** (anche non-legal) che ti piacciono esteticamente.

## Prossimo passo

Validata la sintesi con i tuoi screenshot, fissiamo i **design token** e le **prime decisioni** → solo allora si approva il piano operativo della #3 e si crea il branch `slice/3-cockpit-shell`.
