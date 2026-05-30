# ADR-0008 — Gerarchia Cliente → Pratica → Fascicolo → Fonte, con Fascicoli molti-a-molti

## Stato
Accettata — 2026-05-30

## Decisione
Il modello di dominio è gerarchico: **Cliente → Pratica → Fascicolo → Fonte**. Pratica e Fascicolo sono concetti distinti (non sinonimi): la Pratica è l'incarico professionale, il Fascicolo è un raggruppamento organizzativo di Fonti al suo interno. Una **Fonte può appartenere a più Fascicoli** (relazione molti-a-molti). I Fascicoli sono per default **dinamici** — derivati automaticamente dal Tipo di Fonte (Documento, Norma, Giurisprudenza, Dottrina, …) — e possono essere anche **manuali**, creati e curati dall'utente.

## Perché
Gli studi legali distinguono l'incarico (Pratica) dai suoi sotto-raggruppamenti (Fascicoli); fonderli imporrebbe un refactoring costoso quando i documenti crescono. La cardinalità molti-a-molti evita di replicare il limite del faldone di carta ("un foglio, una cartella") e abilita le viste dinamiche senza duplicare le Fonti. Poiché la tassonomia delle Fonti esiste già (vedi [[0007-citation-of-source-extracts]]), i Fascicoli dinamici costano pochissimo e offrono fin dalla V1 l'organizzazione automatica.

## Considered Options
- **Pratica == Fascicolo (sinonimi)**: scartato, debito tecnico e modello non aderente alla realtà degli studi.
- **Fascicolo fisso, una Fonte in un solo Fascicolo**: scartato, imita l'armadio di carta e preclude le viste dinamiche.
- **Solo viste dinamiche, niente Fascicoli manuali**: scartato per la V1, toglie all'utente il controllo organizzativo.

## Conseguenze
- Relazione Fonte ↔ Fascicolo molti-a-molti nel modello dati.
- I Fascicoli dinamici sono una proiezione del Tipo di Fonte, non una struttura da mantenere a mano.
- Quaero "supera il faldone": l'organizzazione è una vista sulle Fonti della Pratica, non un contenitore fisico esclusivo.
