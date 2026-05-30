# ADR-0009 — Genealogia del Documento come grafo di provenienza

## Stato
Accettata — 2026-05-30

## Decisione
Ogni Documento derivato da Quaero conserva una **Genealogia** completa e ricostruibile. La provenienza non è un attributo secondario, ma un **requisito architetturale di prima classe**.

La Genealogia registra il percorso che porta alla formazione del Documento: Fonti, Estratti di Fonte, prompt/istruzioni, Output AI, Bozze, versioni intermedie, modifiche umane, validazioni, esportazioni ed eventuali firme.

La Genealogia è modellata come **grafo di provenienza (DAG), non come catena lineare**, perché un Documento può derivare da più Fonti, più Bozze, più interventi umani e più passaggi di revisione, e può confluire in più versioni.

## Perché
In ambito legale la genealogia di un Documento vale quasi quanto il Documento stesso: abilita audit, responsabilità professionale, spiegabilità, controllo anti-allucinazione e futura certificazione del lavoro AI-assisted. Non basta sapere *che cosa* ha scritto Quaero: bisogna sapere *come ci è arrivato*, con quali Fonti, con quali passaggi e chi lo ha validato. Se la provenienza non è tracciata dal primo giorno, ricostruirla dopo è praticamente impossibile.

## Principi
1. Ogni Documento derivato da Quaero conserva la propria Genealogia completa.
2. La Genealogia include Fonti, Estratti, prompt, Output AI, Bozze, versioni e interventi umani.
3. Ogni nodo del grafo ha metadati minimi: autore (AI o umano), timestamp, modello AI usato (se presente), Fonti utilizzate, Estratti citati, versione precedente, tipo di operazione (generazione, modifica, validazione, export, firma).
4. La Genealogia è consultabile dall'utente.
5. La Genealogia non viene cancellata quando una Bozza diventa Documento.
6. La validazione dell'avvocato non elimina l'origine AI: la qualifica.
7. Una Bozza validata diventa Documento della Pratica, quindi Fonte citabile.
8. Un Atto è un tipo specifico di Documento validato, non la categoria generale.

## Conseguenze
- Il modello dati in `core` rappresenta la provenienza come grafo (nodi tipizzati + archi), non come campo "versione precedente" singolo.
- Si lega ad [[0007-citation-of-source-extracts]]: gli Estratti di Fonte sono nodi della Genealogia.
- Esiste una vista utente della Genealogia (consultabilità, principio 4).
