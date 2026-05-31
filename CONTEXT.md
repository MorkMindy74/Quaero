# Quaero

Glossario del linguaggio di dominio di Quaero, assistente legale AI per il diritto italiano. Questo file è solo un glossario: nessun dettaglio implementativo (attributi, schema, tecnologie non vanno qui).

## Pratiche

Gerarchia: **Cliente → Pratica → Fascicolo → Documento** (e più in generale → Fonte).

**Cliente**:
Il soggetto per cui l'avvocato lavora (es. Alfa S.r.l., Banca Beta, Rossi Mario). Un Cliente può avere più Pratiche.
_Avoid_: Assistito, parte

**Pratica**:
Il singolo incarico professionale (es. "Recupero credito verso Gamma S.r.l.", "Operazione UTP posizione Rossi", "Parere sulla garanzia MCC"). Appartiene a un Cliente e contiene uno o più Fascicoli, oltre a chat, memoria, strategia e attività.
_Avoid_: caso, matter

**Fascicolo**:
Una **vista organizzativa** sulle Fonti di una Pratica — non un contenitore fisico esclusivo. Una stessa Fonte può comparire in più Fascicoli senza essere duplicata (molti-a-molti). Un Fascicolo può essere **dinamico** (generato dal sistema, uno per Tipo di Fonte: Documenti, Norme, Giurisprudenza, Dottrina, Prassi, Dati, Note, Memoria, Fonti esterne) o **manuale** (creato e curato dall'utente: es. Produzione avversaria, Atti processuali, Compliance, Strategia, Bozze, Due diligence). I Fascicoli **dinamici** sono **viste derivate**, ricalcolate dalle Fonti e mai salvate come stato canonico; i **manuali** sono stato canonico curato dall'utente. *(Modellato in #5A: stato canonico vs vista derivata; in #5B viene **persistito solo il canonico** — i dinamici non si salvano. Vedi `START_HERE.md`.)*
_Avoid_: cartella, faldone, folder, scatola, contenitore

## Fonti

**Fonte**:
Qualunque entità verificabile che può essere citata a supporto di un'affermazione. È il concetto generale; ha nove tipi. Il **Documento** è solo uno di questi.

**Documento**:
Un file salvato in una Pratica, **acquisito** dall'esterno (es. Contratto.pdf, Sentenza.pdf, Bilancio.xlsx) **o derivato** da una Bozza validata dall'avvocato. È una Fonte citabile. Un **Atto** ne è un sottotipo.
_Avoid_: file, allegato

**Norma**:
Una fonte normativa primaria o secondaria (es. art. 1375 c.c., TUB, TUF, Codice della crisi, Regolamento UE, circolare ministeriale). Tipo di Fonte.

**Giurisprudenza**:
Una pronuncia di un organo giudicante (es. Cassazione, Sezioni Unite, Consiglio di Stato, Corte Costituzionale, CGUE, CEDU). Categoria autonoma, distinta da Norma. Tipo di Fonte.

**Dottrina**:
Un'opera scientifica o interpretativa (es. articolo, commentario, trattato, nota a sentenza, libro). Tipo di Fonte.

**Prassi**:
Soft-law e orientamenti di autorità, non vincolanti come la Norma (es. circolari Banca d'Italia, istruzioni MCC, linee guida AGCM, FAQ CONSOB, orientamenti EBA, provvedimenti IVASS). Tipo di Fonte.

**Dato**:
Una prova fattuale, né normativa né giurisprudenziale (es. visura camerale, bilancio, KPI, serie storica, estratto conto). Tipo di Fonte.

**Nota**:
Un singolo appunto dell'utente, citabile internamente ma non autorevole (es. "Cliente disponibile a transigere"). Tipo di Fonte.

**Memoria**:
Conoscenza consolidata e persistente su una Pratica o un Cliente, diversa dalla Nota (che è un singolo appunto): preferenze, decisioni già prese, strategia concordata, istruzioni permanenti. Tipo di Fonte.

**Fonte Esterna**:
Una risorsa esterna che non rientra negli altri tipi (es. sito web, report FMI/BCE/EBA, banca dati generica), identificata da URL e timestamp. Tipo di Fonte.

## Citazione

**Affermazione**:
Una singola asserzione contenuta in una Risposta di Quaero. Ogni Affermazione deve essere supportata da una o più Fonti.
_Avoid_: claim, frase

**Estratto di Fonte**:
La porzione specifica e verificabile di una Fonte che sostiene un'Affermazione (es. "Contratto.pdf, pag. 8, clausola 4.2"; "Cass. 1234/2025, § 17"; "art. 1375 c.c., comma 1"). È l'unità che viene effettivamente citata: una Risposta cita Estratti di Fonte, non Fonti intere.
_Avoid_: snippet, passaggio

**Ancora**:
Il localizzatore stabile e indipendente dal layout che identifica un Estratto di Fonte all'interno della sua Fonte, così che resti valido anche se la Fonte viene ri-renderizzata.
_Avoid_: anchor, riferimento, posizione

**Citazione**:
Il collegamento, in una Risposta, tra un'Affermazione e l'Estratto di Fonte che la sostiene.
_Avoid_: reference, rimando

**Risposta**:
L'output che Quaero produce a una richiesta dell'avvocato in una Pratica, composto da Affermazioni ciascuna corredata di Citazioni.
_Avoid_: output, messaggio

## Produzione e provenienza

**Output**:
Qualunque artefatto generato da Quaero, testuale, strutturato o visuale (es. bozza di atto, memo, parere, email, timeline, strategia, checklist, report).
_Avoid_: artefatto, risultato

**Bozza**:
Un Output **non ancora validato** da un essere umano. Conserva i metadati di origine (autore AI, modello, data di generazione, origine/prompt, stato di revisione). Va sempre verificata prima di diventare Documento.
_Avoid_: draft, abbozzo

**Atto**:
Un sottotipo di Documento: un atto giudiziario (es. comparsa, ricorso, memoria, citazione, appello). Non è la categoria generale dell'output. Non tutte le Bozze diventano Atti; molte diventano semplici Documenti.

**Genealogia**:
Il **grafo di provenienza** (DAG, non una catena lineare) completo e ricostruibile di un Documento: Fonti, Estratti di Fonte, prompt, Output AI, Bozze, versioni intermedie, interventi umani, validazioni, export e firme. Ogni nodo porta metadati minimi (autore AI/umano, timestamp, modello, Fonti/Estratti, tipo di operazione). Non viene cancellata quando una Bozza diventa Documento; la validazione umana qualifica l'origine AI, non la elimina. È il fondamento di audit, responsabilità professionale, spiegabilità e certificazione del lavoro AI-assisted.
_Avoid_: provenienza, storia, lineage, catena

## Flagged ambiguities

**Fonte Esterna vs Norma/Giurisprudenza recuperate online**: una Norma recuperata da Normattiva resta una **Norma**, non una Fonte Esterna; una pronuncia recuperata da una banca dati resta **Giurisprudenza**. La classificazione segue il *tipo di contenuto*, non la provenienza. "Fonte Esterna" è il tipo residuale per risorse esterne prive di un tipo legale più ricco.

**Risposta vs Output**: la **Risposta** è l'output conversazionale in chat (Affermazioni + Citazioni). L'**Output** è un artefatto producibile/esportabile (bozza di atto, memo, report…) che, una volta validato, diventa Documento. Entrambi sono generati dalla macchina e devono portare con sé la Genealogia.

## Backlog glossario (da definire, non bloccanti per partire)

Termini non ancora consolidati, da grigliare quando diventano rilevanti: **Strategia**, **Connettore**, **Timeline**, **Workflow/Task**, **Ruolo/Permessi**.

## Example dialogue

> **Dev**: L'avvocato chiede "la clausola 7.2 è rischiosa?". Quaero risponde. Cosa cita?
> **Esperto**: Non cita "il Contratto". Cita un *Estratto di Fonte*: Contratto.pdf, pag. 8, clausola 7.2. E se collega l'art. 1375 c.c., cita "art. 1375 c.c., comma 1", non l'articolo intero.
> **Dev**: Quindi se domani impagino di nuovo il PDF e la clausola finisce a pag. 9?
> **Esperto**: L'Ancora deve reggere lo stesso: punta alla clausola 7.2, non alla pagina 8 in quanto tale. La Citazione resta valida.
> **Dev**: E un appunto tipo "cliente vuole transigere"?
> **Esperto**: È una Nota — una Fonte citabile internamente, ma non autorevole. Diversa dalla Memoria, che è la strategia consolidata della Pratica.
