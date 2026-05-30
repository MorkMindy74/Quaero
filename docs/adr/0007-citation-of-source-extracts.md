# ADR-0007 — Si citano Estratti di Fonte, non Fonti

## Stato
Accettata — 2026-05-30

## Decisione
Ogni **Affermazione** prodotta da Quaero è supportata da una o più **Citazioni**; ogni Citazione punta a un **Estratto di Fonte** (la porzione specifica e verificabile — es. "art. 1375 c.c., comma 1"; "Cass. 1234/2025, § 17"; "Contratto.pdf, pag. 8, clausola 4.2"), **non** a una Fonte intera. L'Estratto è identificato da un'**Ancora** stabile e indipendente dal layout, così che la Citazione resti valida anche se la Fonte viene ri-renderizzata. Le Fonti seguono una tassonomia a nove tipi: Documento, Norma, Giurisprudenza, Dottrina, Prassi, Dato, Nota, Memoria, Fonte Esterna.

## Perché
È il meccanismo anti-allucinazione primario del sistema: non basta sapere *da quale fonte* arriva un'affermazione, serve sapere *esattamente quale pezzo* la sostiene. Citare la Fonte intera renderebbe la verifica impossibile e riaprirebbe la porta alle allucinazioni.

## Conseguenze
- Il tipo condiviso in `core` non è un semplice riferimento alla Fonte, ma un Estratto di Fonte localizzato da un'Ancora.
- Il Motore citazioni produce Estratti; il Verificatore controlla che l'Estratto sostenga l'Affermazione.
- Surprising-by-default: uno sviluppatore potrebbe ingenuamente citare l'intera Fonte — questo ADR lo previene.
