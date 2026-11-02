# Research note: how concept design reports are structured

For spec 0015 (`docs/specs/0015-concept-report-format.md`). Each source is marked **read** (I read the text) or
**not read** (cited from a secondary source or a catalogue; check before quoting).

## 1. Sources

| # | Source | Status | What it gives |
|---|---|---|---|
| S1 | SNAME, *Dr. James A. Lisnyk Student Ship Design Competition Rules*, 2021–2022 (March 2022), §V "Design report requirements" ([PDF](https://www.sname.org/sites/default/files/2022-03/2021-2022%20Final%20Lisnyk%20Rules%20and%20Entry%20Form_final.pdf)) | read | The closest thing to a profession-wide required contents list for a concept/preliminary ship design report |
| S2 | Virginia Tech AOE 4065/4066 ship design course, *ADF design report*, Team 5, 2007 (draft) ([PDF](https://www.aoe.vt.edu/content/dam/aoe_vt_edu/people/faculty/albrown5/design/2007adfteam5draftreport.pdf)) | read (contents) | A university concept design report structure (A. Brown's course) |
| S3 | ANSI/NISO Z39.18-2005 (R2010), *Scientific and Technical Reports — Preparation, Presentation, and Preservation*, Table 1 and §5.2 ([PDF](https://groups.niso.org/higherlogic/ws/public/download/12038/Z39_18_2005_R2010.pdf/latest)) | read | The general technical report convention: components, order, what a summary is |
| S4 | ABS, *Rules for Building and Classing Steel Vessels 2004*, Part 1, 1-1-7/1 "Hull Plans" ([PDF](https://ww2.eagle.org/content/dam/eagle/rules-and-guides/archives/conventional_ocean_service/2_steelvesselrules2004/part_1_e-nov04.pdf)) | read (2004 edition; superseded) | A class "plans to be submitted" list |
| S5 | ABS, *Guidance Notes on Review and Approval of Novel Concepts*, 2003, Section 4/2 "Required information to be submitted" (AIP) ([PDF](https://ww2.eagle.org/content/dam/eagle/rules-and-guides/archives/other/116_reviewandapprovalofnovelconcepts/pub116_novelconcepts.pdf)) | read (2003 edition; check for a newer one) | What class wants at the concept stage, for an approval in principle |
| S6 | ABS, *Rules for Building and Classing Steel Barges 2021*, Part 4 "Plans and data to be submitted" ([PDF](https://ww2.eagle.org/content/dam/eagle/rules-and-guides/archives/special_service/10_rulesforbuildingclassingsteelbarges_2021/barge-rules-july21.pdf)) | read (lists only) | Barge submission lists point back to Part 1, 1-5-4/3 |
| S7 | J. H. Evans, "Basic Design Concepts", *J. Am. Soc. Naval Engineers* 71 (1959) 671–678 | not read | The design spiral. Citation as given by secondary sources; they differ on the volume number |
| S8 | DNV Rules for Ships Pt.1 Ch.3 (documentation requirements) and DNV-CG-0157 *Stability documentation for approval* | not read | Catalogue entries only; the PDFs did not download. CG-0157 sets the minimum contents of stability documentation for approval |
| S9 | IMO MSC.1/Circ.1455 (2013), *Guidelines for the approval of alternatives and equivalents* | not read | Defines a "preliminary design" as a high-level design with the general arrangement and major systems, and a preliminary analysis report. Fragments only |
| S10 | UCL MSc Naval Architecture, ship design exercise (module MECH0084 and the Design Building Block papers of Andrews and Pawling) | not read (no report template found) | UCL stresses the architecture (arrangement) beside numerical balance |
| S11 | MIT 2N sequence (2.701–2.704) | not read (no public report outline found) | Course pages describe preliminary design to a set of mission requirements; no report outline is public |
| S12 | LR and BV submission lists | not searched in depth | — |
| S13 | OVD `tools/concept_report.py` (issue #727) and issue #799 | read | Today's standard parts; the ATB comparison need |

## 2. Findings by source

**S1, SNAME Lisnyk rules (§V).** The report must show the owner's requirements were understood, give
the approach and trade-offs behind each choice, identify technical risks, and give enough detail for a
technical evaluation. It must be organised by a work breakdown so every aspect is covered. The minimum
list, in the rules' order:
- (a) a summary with a table of principal characteristics, "the first item in the report";
- (b) weights in an accepted format for light ship, full load departure and arrival;
- (c) curves of form and floodable length; (d) trim and intact stability for all operating conditions;
  (e) damage stability for limiting cases;
- (f) drawings in an appendix, not in the body: lines, GA, capacity plan, machinery arrangement,
  structural midship section with scantlings;
- (g) speed and power; (h) electrical load; (i) seakeeping; (j) area and volume summary that adds to
  the enclosed volume; (k) manning; (l) HM&E systems; (m) propulsion plant trade-off; (n) endurance
  fuel; (o) mission (cargo) systems; (p) cost, build and operating; (q) technical risk with mitigation,
  including "shortcomings in the design that would be addressed in the next design phase";
- (r) the owner's requirements as an appendix.
It also lists the faults judges see most: topics missing or hard to find; poor figures; no method
explained; no rationale for critical decisions; internal inconsistency (one value, two figures); no
discussion of results. The committee wants evidence the team understands the tool, not "elegant
computer results".

**S2, Virginia Tech report.** Executive summary (with a principal characteristics table) first. Then:
1 Introduction, design process and plan; 2 Mission definition; 3 Concept exploration (trade-offs,
design space, synthesis model, effectiveness, risk, cost, optimisation, baseline); 4 Concept
development: 4.1 preliminary arrangement, 4.2 producibility, 4.3 hull form and deckhouse, 4.4
structure, 4.5 power and propulsion (resistance, propulsion, electric load, fuel), 4.6 mechanical and
electrical, 4.7 manning, 4.8 space and arrangements, 4.9 weights and loading (weights, loading
conditions), 4.10 hydrostatics and stability (intact, damage), 4.11 seakeeping, 4.12 cost; 5
Conclusions and future work; 6 References; appendices for the requirement documents, data summaries,
equipment list, weights and centres, space summary, models.

**S3, NISO Z39.18.** Front matter (cover, title, notice, abstract, contents), body (summary,
introduction, methods/assumptions/procedures, results and discussion, conclusions, recommendations,
references), back matter (appendices, symbols, glossary, index). The summary is required: it states the
problem, the main results and conclusions and a recommended course of action; it adds nothing new and
has no cross-references. An executive summary is for readers who will not read the whole report. The
introduction names the audience and does not hold findings.

**S4, ABS Part 1, 1-1-7/1 (2004).** Hull plans "for review or reference" begin with the vessel
specification and general arrangement, then the midship section and scantling plans, then capacity
plan and lines. The plans must give the design draught and speed and, for special cargo, the weights
and their distribution.

**S5, ABS novel concepts, AIP (2003).** At the concept stage class asks for: the scope and the
interfaces of the concept; design basis (operating envelope, environment, design life); functional
description; preliminary GA; preliminary structural drawings with primary scantlings; the codes and
standards applied and the justification; the novel aspects the codes do not cover and how they will be
justified; concept-level calculations against agreed criteria; the data sources. The AIP ends in
conditions for later approval.

**S7, the design spiral (not read).** As reproduced in later texts, the spiral passes in turn through
requirements, proportions, form, hydrostatics, freeboard and subdivision, arrangement, structure,
powering and machinery, weights, stability and cost, and goes round again. Reproductions differ in the
exact order; the original was not read. The point for a report: each step uses the results of the
steps before it.

## 3. Where they agree

1. **The summary leads**, with a principal characteristics table (S1 a, S2, S3). The spec puts a
   one-page summary first, after the cover (D2).
2. **Requirements first.** Owner's requirements (S1), mission definition (S2), design basis (S5). The spec's A1 Brief matches.
3. **Arrangement comes early in development** (S2 4.1 before hull form; S4 GA second after the
   specification; S10). The spec places the GA at A4, after form and cargo, and shows it in the body beside capacity.
4. **Weights, then loading conditions, then intact, then damage stability** (S1 b–e, S2 4.9–4.10). The spec matches (A7 to A11).
5. **Drawings and data in appendices**, not the body (S1 f; S2 appendices). Matches Appendix B.
6. **Risk and next-phase shortcomings are a required part** (S1 q; S2 §5; S5 AIP conditions). Matches A14.
7. **Rationale and method beside every result** (S1 faults list; S3 methods). The spec's `data_box` per part serves this.
8. **Codes and standards named up front, deviations stated** (S5). A1 lists the Rule Sets; A12 and the "not evaluated" block state the gaps.

## 4. Where the spec's order differs from the sources

| Point | Sources | Spec | Why |
|---|---|---|---|
| Powering before weights | S2 (4.5 before 4.9); spiral | Structure, powering and machinery come before weights (D3) | Machinery mass enters the lightship |
| Freeboard with hydrostatics | Spiral; S1 (c) floodable length with curves of form | Freeboard sits beside hydrostatics and stability | Freeboard sets the laden draught |
| Structure and arrangement | S2: arrangement, hull form, structure; S4: GA, then structure | GA at A4, structure at A5 | Agrees |
| Seakeeping | S1 (i), S2 4.11 | A subsection marked "not evaluated" | OVD has no motions (#731) |
| Manning, electrical load, HM&E | S1 (h, k, l), S2 4.6–4.7 | Marked "not evaluated" in A4 and A6 | The manning model is in spec 0011's research; electrical load is not modelled |
| Concept exploration and trade studies | S2 §3; S1 trade-off rationale | Appendix B6 | The body is an executive summary (D8) |
| Conclusions at the end | S2 §5, S3 | Summary first (D2); limits and open items close the body | S3 allows a summary at the front |
| Operating cost | S1 (p) | Build cost only | Not modelled; a non-goal |
| Introduction naming the audience | S3 | The cover and summary name the client as the reader (D1) | Agrees in function |

## 5. Lessons from today's reports (no project content)

- Reports already written with OVD used finding-first section titles and a one-page summary of findings.
  The spec keeps that with a fixed title plus a one-line finding (D4).
- A per-report script grew to several hundred lines of narrative with figures typed inside f-strings.
  Even when the figures came from variables, nothing checked that each one traced to a run field. This
  is the case for filled blanks and a trace map (spec sections 6 and 11).
- An "open items before approval in principle" table (item, why, how it closes) was the most useful
  part for an external reviewer. It mirrors S5's AIP conditions. The spec builds it from the run (A14).
- Rules not evaluated were listed at the end of the rules section only. A reader of the stability
  section did not see that damage stability was missing. This is the case for the in-place
  "not evaluated" block (spec section 5).

## 6. Gaps in this research

- DNV Pt.1 Ch.3, DNV-CG-0157, LR and BV submission lists not read (downloads failed or not tried).
- Evans (1959) not read; spiral order taken from reproductions.
- No public MIT 2N or UCL report template found.
- ISO: no ISO standard for technical report structure is in force that I could confirm (ISO 5966 was
  withdrawn); ISO 2145 (numbering of divisions in written documents) fits section numbering but was
  not read.
