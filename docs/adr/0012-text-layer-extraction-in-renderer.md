# ADR-0012 — Livello testo dei Documenti: estrazione nel renderer, sidecar locale derivato

## Stato
Accettata — 2026-06-02 (implementata nella slice #52, PR #53)

## Contesto
Per arrivare all'AI Evidence Assistant (proposta automatica di Estratti e
Citazioni, ADR-0007) serve che il contenuto testuale di un Documento sia
disponibile e ancorabile. Finora una Fonte Documento conservava **solo i byte
del file + i metadati** (`StoredFile` con `sha256`, `byteLen`, `storedName`):
nessun testo estratto, quindi nessuna base su cui ancorare Estratti. Questa ADR
fissa **come** si ottiene quel testo, in modo coerente con il confine di
sicurezza già stabilito (ADR-0011) e con la natura di dato legale riservato.

## Decisione
Introduciamo un **livello testo** per le Fonti Documento, con questi vincoli
(tutti in vigore dalla slice #52):

1. **Estrazione nel renderer/webview, mai in Rust.** Il testo è prodotto nel
   webview: `TextDecoder` UTF-8 per `.txt`/`.md`, **pdf.js** (`pdfjs-dist`) per i
   PDF testuali. Il documento è input **non fidato**; il parsing resta confinato
   al sandbox del webview.
2. **Nessun parsing PDF lato Rust.** Il backend non interpreta mai il contenuto
   del documento.
3. **Core Rust puro.** Lo store desktop **valida e persiste** soltanto: riceve il
   testo già estratto e scrive un file affiancato. Nessuna logica di parsing nel
   core (coerente con ADR-0011).
4. **Text layer come sidecar locale** `files/<matter>/<storedName>.txt` (UTF-8),
   accanto al blob immutabile, con scrittura atomica (temp + rename) e pulizia del
   temp su qualsiasi errore pre-rename.
5. **Derivato dal blob immutabile.** Il sidecar è legato strutturalmente alla
   versione `sha256`-pinnata del file (il blob non viene mai sovrascritto): il
   livello testo è un derivato, non una seconda fonte di verità.
6. **Verifica `sha256` sotto lock prima della scrittura.** `set_source_text`
   riceve l'`expected_sha256` catturato all'inizio dell'estrazione e lo verifica,
   sotto il lock per-Pratica, contro il `SourceRef.file.sha256` caricato; su
   mismatch rifiuta senza scrivere. La coerenza testo↔versione è imposta dal
   backend, non solo dal renderer.
7. **Zero egress.** Worker pdf.js **bundlato localmente** (niente CDN), nessuna
   risorsa remota (cMap/font); `isEvalSupported:false` (mitiga CVE-2024-4367).
8. **Nessuna nuova capability Tauri.** Il frontend invia il testo via IPC tipizzato
   esistente, come già fa con i byte all'import (ADR-0011).

Tre stati persistiti: **Available** (testo presente), **Empty** (file supportato,
nessun testo utile — es. PDF scansionato), **Absent** (nessun sidecar / Fonte
senza file). Il renderer aggiunge due stati non persistiti: **Failed** e
**Unsupported**.

## Perché
Quaero tratta documenti legali riservati: il parsing di input non fidato deve
stare dove il modello di sicurezza è più forte (il sandbox del webview), non nel
processo nativo, e nessun dato deve lasciare il dispositivo. Tenere il core puro
(ADR-0011) e il testo come **derivato verificabile** del blob immutabile mantiene
una sola fonte di verità (i byte pinnati) e una coerenza imposta per costruzione,
prerequisito per ancorare in modo affidabile Estratti e Citazioni (ADR-0007).

## Conseguenze
- Esiste una base testuale locale e coerente su cui le slice future potranno
  costruire l'**AI Evidence Assistant** (proposta di Estratti/Citazioni ancorati).
- Il livello testo è ricostruibile: cancellare il sidecar riporta semplicemente
  allo stato `Absent`; il blob e i metadati restano intatti.
- `pdfjs-dist` (Apache-2.0) entra come dipendenza **solo frontend**, registrata in
  `THIRD_PARTY.md` (ADR-0004), pinnata, con worker locale.
- **Fuori scope** (non decisi né implementati qui): **OCR** e **PDF scansionati**
  (restano `Empty`), estrazione **DOCX** e altri formati (restano `Unsupported`),
  viewer/evidenziazione, ed estrazione automatica all'import (oggi on-demand).
