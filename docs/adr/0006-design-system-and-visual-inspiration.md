# ADR-0006 — Design system e politica di ispirazione visiva

## Stato
Accettata — 2026-05-30

## Contesto
L'estetica è una priorità esplicita del committente: l'utente deve fare "wow". Il committente non è un designer e si ispira a gallerie di design (refero.design, superdesign.dev). Serve coerenza visiva su ogni schermata e una regola chiara sull'uso delle fonti di ispirazione.

## Decisione
- Quaero adotta un **design system**: i design token (palette carta/pergamena, serif per i documenti, spaziature, ombre, raggi — già presenti nel mockup `UX/index.html`) sono l'unica fonte di verità, e un component kit ne deriva tutti i componenti.
- L'identità visiva di partenza è quella del mockup ("artigianato giuridico"): calda, curata, distintiva rispetto ai competitor freddi/aziendali.
- **Ispirazione visiva**: stili, layout e micro-interazioni di siti come refero.design e superdesign.dev possono essere **imitati** (non coperti da copyright). **Non** si copiano codice o asset proprietari di terzi.

## Conseguenze
- Esiste una slice dedicata "Design language" (issue #3, HITL) che formalizza token e component kit prima delle schermate ricche di UI.
- Ogni nuova schermata usa il component kit → coerenza e qualità garantite.
- Le decisioni di design passano da revisione umana (HITL).
