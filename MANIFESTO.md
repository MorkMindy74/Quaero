# Manifesto di Quaero — l'idea forte

> Da leggere prima di scrivere qualsiasi riga di codice. Se una scelta di implementazione contraddice questo manifesto, è la scelta a essere sbagliata, non il manifesto.

## Quaero non è una chat AI su PDF

È facile, partendo a generare codice, ritrovarsi con "un'app che funziona": una casella di testo che risponde su un documento. **Quella non è Quaero.** Sarebbe l'ennesimo wrapper, indifendibile e sostituibile.

## La prima fase di Quaero è uno *scheletro applicativo*, fondato su quattro pilastri

1. **Pratiche / Fascicoli / Fonte** — il lavoro dell'avvocato è strutturato: Cliente → Pratica → Fascicolo → Fonte. Il Fascicolo è una *vista*, non una scatola: Quaero non replica il faldone di carta, lo supera. Le Fonti hanno una tassonomia ricca (9 tipi), non sono "file generici".

2. **Citazioni ad Estratti di Fonte** — ogni Affermazione di Quaero è ancorata non a una Fonte intera, ma all'**Estratto** preciso che la sostiene (pagina+clausola, comma, §). È il meccanismo anti-allucinazione: non basta sapere *da dove* arriva una frase, serve sapere *esattamente quale pezzo* la regge.

3. **Genealogia del Documento** — ogni Documento prodotto conserva il suo **grafo di provenienza**: Fonti, Estratti, prompt, Output AI, Bozze, versioni, interventi umani, validazioni, firme. In ambito legale la genealogia di un documento vale quasi quanto il documento stesso: abilita audit, responsabilità professionale, spiegabilità e certificazione del lavoro AI-assisted.

4. **UI a cockpit legale, privacy-first e locale** — non un chatbox, ma uno spazio di lavoro (pratiche, evidenze, agent run, fonti) curato esteticamente, dove "nessun dato della pratica esce dalla macchina" è una garanzia di prodotto, non un'opzione.

## La conseguenza pratica

Si costruisce **prima lo scheletro** (struttura, dominio, provenienza), **poi** l'intelligenza ci si appoggia sopra. L'AI è un inquilino di un'architettura solida, non le fondamenta. Questo è ciò che rende Quaero difendibile e degno di fiducia per un avvocato.

Riferimenti: [`CONTEXT.md`](CONTEXT.md) (glossario), [`docs/adr/`](docs/adr) (decisioni), [`START_HERE.md`](START_HERE.md) (checkpoint).
