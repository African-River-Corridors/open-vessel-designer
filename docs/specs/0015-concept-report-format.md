# Spec 0015: A standard concept design report

Status: **agreed by the maintainer, 2026-11-02** (decisions D1–D15 taken with the maintainer in two
rounds). 
Builds on issue #727 (`tools/concept_report.py`, the first standard parts) and issue #728. Related:
#799 (the ATB option in the report), spec 0012 (damage stability), spec 0013 (plate-level steel),
spec 0014 (ATB), #730 (freeboard), #731 (seakeeping). Report engine: Open Report Designer (ORD),
spec 0001 of that repo. Research: `docs/research/concept-report-format.md`.

Concept and preliminary design only, never approval (ADR 0011). A concept report says so on its cover
and in every footer.

The report is meant to look like the final report once the engine is trusted. So it holds no asks and
no challenge questions, and no language model writes any of it. Work that needs a judgement call is
outside the report (section 10).

## 1. Problem

OVD can write part of a concept design report. `tools/concept_report.py` builds fixed parts from an
`ovd run --json` Option: main particulars, lightship, steel, hydrostatics, stability, powering, a
compliance matrix, a paragraph per governing Rule, the Rules not evaluated, run provenance and the GA
sheets. It places them in five sections and one appendix.

That is half a standard. The rest of every real report is written per project, by hand or by a
language model:

- **No fixed order.** Each project script picks its own sections and its own order. Two reports of two
  designs cannot be read side by side, and a reviewer cannot learn where to look.
- **No build-up.** The five sections do not take the reader from the brief to the verdict one step at a
  time. The brief, the cargo system, the arrangement, damage stability, freeboard, cost and the open
  items have no home in the tool, so each project adds them, or leaves them out without saying so.
- **Silent gaps.** A section the run cannot fill simply does not appear. The reader cannot tell "not
  checked" from "not relevant" or "forgotten".
- **Free narrative.** Project text sits anywhere, at any length, and can type a figure. A typed figure
  drifts from the run the moment the design changes.
- **No vessel types.** The parts are written for a gas barge. A liquid, dry bulk, hopper or deck barge,
  or a pushed or articulated formation, needs other subsections, and has nowhere to put them.
- **One design only.** The tool shows one Option with an optional reference column. Issue #799 needs an
  ATB option beside a self-propelled one.
- **No command.** The report is a script in `tools/`, not part of `ovd`.

## 2. Goals and non-goals

**Goals**

1. One skeleton for every concept report: the same headings in the same order for every design.
2. The client or owner is the main reader (D1). The body is about ten pages and reads like an
   executive summary (D8). The full technical chapters, and the evidence behind them, are appendices.
3. The order builds up understanding: each section uses only what an earlier section has set (D3).
4. Every figure, table and chart comes from the run. No typed figure, anywhere.
5. A section OVD cannot compute still appears, marked **not evaluated**, with the reason and what would
   close it.
6. Each part has a one-line finding and a short reading. Software makes both from the run, by rules
   and fixed sentence templates (D4, section 6). No typed figures.
7. Vessel-type modules add type-specific subsections inside the fixed sections (D5).
8. One command, `ovd report`, and a Python API, writing an ORD document and its PDF, HTML and Word.
9. A report reads as a first read: it never refers to an earlier revision or draft.
10. Tests that prove every figure traces to a run field.

**Non-goals**

- Approval documents. A concept report is not a stability booklet, a class submission or a loading
  manual. It may list what such a submission would need (Appendix A14).
- New engineering. This spec arranges what the run already computes. Where the run has nothing, the
  report says "not evaluated"; it does not estimate.
- Asks and challenge questions inside the report (D2), and a cover note that carries them (D11).
  Judgement-call work belongs to a future OVD agent template (section 10).
- A report designer UI. The desktop app may show the report later; that is a separate spec.
- Writing prose with a language model. The builder uses none, at any step (D9, section 6).

## 3. Principles

| # | Principle |
|---|---|
| P1 | **Fixed skeleton.** Headings, numbering and order are the same for every design. A module adds subsections inside a section; it never adds, removes or moves a top-level section. |
| P2 | **Build-up order.** A section may refer back to any earlier section, never forward. Only the summary gathers from all (D3). |
| P3 | **Figures from the run.** Every number in the document comes from a run field, or from a constant the report code names and cites (for example a rule limit already in the trace). The builder keeps a trace map from each table cell, chart series and sentence blank to its JSON pointer. |
| P4 | **Not evaluated, never absent.** A section, subsection, table row or chart with no run data appears with a fixed "not evaluated" block. It is never blank, zero or dropped. |
| P5 | **Findings and readings by rule.** Text about the design is made by rules over the run and filled into fixed sentence templates. A figure in the text is inserted from the run by name; it is never typed. Text may not claim a check the run did not make, or claim approval (section 6). |
| P6 | **First read.** No text refers to an earlier revision, draft or version of the report. Change history lives outside the document (section 10). |
| P7 | **Concept only.** The ADR 0011 notice is on the cover and in every footer. The ORD status cannot pass the class's highest non-approval state. |
| P8 | **Deterministic.** The same run, options and engine version give a byte-identical ORD document. |
| P9 | **No asks in the report.** The report states findings and limits. It never asks the reader to decide or to challenge (D2). |

## 4. The skeleton

The report has three zones, in this order: the **body** (about ten pages), **Appendix A** (the full
technical chapters, in build-up order) and **Appendix B** (the evidence). Each body part is a digest
of one or more Appendix A chapters and points to them.

### 4.1 Body: about ten pages, executive-summary in character

| Part | Title | Pages | Digest of | What the client reads |
|---|---|---|---|---|
| — | Cover, control block, notice | 0.5 | ORD `control`, `notice`; `engine_version` | What the document is, its revision and status, and that it is concept only |
| — | Summary | 1 | gathered from all parts | One page that stands alone: principal characteristics and the verdict (D2) |
| 1 | The vessel and its brief | 1 | A1, A2 | What it must do, under which rules, its size and shape; the options-compared table |
| 2 | Capacity and arrangement | 1 | A3, A4 | How much cargo and what limits it; the general arrangement |
| 3 | Structure, powering and machinery | 1.5 | A5, A6 | The hull steel, speed and power, propulsors and machinery |
| 4 | Weights | 0.75 | A7 | Lightship and deadweight by item |
| 5 | Floating, freeboard and stability | 1.25 | A8, A9, A10 | Draughts, freeboard carried and required, intact stability |
| 6 | Damage stability | 1 | A11 | Whether it survives the rule damages |
| 7 | Compliance | 1 | A12 | Every Rule, its outcome and its margin |
| 8 | Cost | 0.5 | A13 | What it costs, on which rates and price date |
| 9 | Limits and open items | 0.5 | A14 | What the run did not evaluate, and where its methods are outside their range |
| | **Total** | **10** | | |

The page budget is a target the renderer checks (a warning above 12 pages, not an error). Each body
part holds: its finding line, its key table or figure (comparison columns when other Options are
named, D6), and its reading (about 120 words). Summary: a table of principal characteristics (L, B,
D, T laden and light, displacement, lightship, deadweight, cargo, installed power, service speed,
governing Rule Sets), a verdict line from the trace (counts of pass, warn, fail, not evaluated; the
worst outcome) and the design in a few sentences made by the same rules as every finding. No asks.

Why this order (D3). Strict build-up puts structure, powering and machinery before weights, because
the hull steel and the machinery masses enter the lightship. Powering uses the displacement the form
section sets; the weights section then closes the loop with the lightship the run reports. Freeboard
sits with hydrostatics because it is read at the same draughts. Compliance follows every check it
summarises; cost follows compliance; limits close the body.

### 4.2 Appendix A: the technical chapters, in build-up order

| § | Title | What the reader learns | Main run fields |
|---|---|---|---|
| A1 | Brief | What the vessel must do and under which rules | `vessel`, `cargo`, `phase`, `rule_trace.scope`, `rule_trace.rule_sets`, `rule_trace.service_area`, design inputs |
| A2 | Main dimensions and form | The size and shape, and what bounds each dimension | `dims`, `hydrostatics` (coefficients, Lwl, Bwl), `geometry` (form, stern, rakes), `results.governs`, `results.draught_max_m` |
| A3 | Cargo system and capacity | How much cargo, in what, and whether volume or weight binds | `results.cargo_*`, `arrangement`, `spaces` (cargo kinds), module fields |
| A4 | General arrangement | Where everything is | `ovd draw` sheets, `spaces`, `drawing`, `sightline`, `results.air_draft_*` |
| A5 | Structure | The midship section, the scantlings and the hull steel | `structure` (spec 0013), `cost_masses.steel`, `calibration` |
| A6 | Powering, propulsion and machinery | How fast, on what power, with which propulsors, stern and machinery | `powering`, `geometry` (stern, propeller), `boil_off`, `powering.fuel_gas`, stern block |
| A7 | Weights | Lightship by item, deadweight by item, centres | `cost_masses.masses`, `stern_block_mass`, `results.lightship_t`, `results.deadweight_t`, `loading_conditions.consumables_t`, `calibration` |
| A8 | Hydrostatics and loading conditions | How it floats in each condition | `hydrostatic_table`, `hydrostatics`, `loading_conditions.conditions` |
| A9 | Freeboard | The freeboard carried against the freeboard required | `results.freeboard_m`, freeboard rows of `rule_trace` (#730) |
| A10 | Intact stability | Whether it stands up, condition by condition | `stability`, `curves`, `stability_marks`, intact rows of `rule_trace` |
| A11 | Damage stability | Whether it survives the rule damages | `damage`, `marpol_damage` (spec 0006, spec 0012), damage rows of `rule_trace` |
| A12 | Compliance | Every Rule, its outcome, and what was not evaluated | `rule_trace` (rows, verdict, warnings) |
| A13 | Cost | What it costs, on which rates and price date | `cost` |
| A14 | Limits and open items | What could not be checked, and what would close it | gathered: not evaluated, warnings, calibration labels, method ranges |

Each chapter has: **fixed content** (always built from the run), its **finding and reading** (section
6) and its **not-evaluated trigger** (when the "not evaluated" block replaces fixed content). The
finding and reading of a chapter are the same text the body part shows. They are made once.

**A1 Brief.** Fixed: mission inputs as typed (cargo kind and capacity target, route or Service Area,
waterway limits, design speed, phase), the governing Rule Sets with edition, the service limits the
run applies (draught cap, air draught limit, sea state if typed). Not evaluated: never; the brief is
the input.

**A2 Main dimensions and form.** Fixed: main dimensions, form coefficients (Cb, Cm, Cp, Cwp) with the
length they are on, ratios L/B, B/T, B/D and L/D, hull and stern form names, a "what bounds each
dimension" table (the input, the limit or the fit rule that sets it, from `checks` and the fit
result) and the options-compared table (D6). Figure: body plan or lines, when drawn. Not evaluated:
lines figure, when the run has no lines drawing.

**A3 Cargo system and capacity.** Fixed: cargo volume and mass at the fill ratio, volume-bound and
weight-bound limits and which governs, a table of cargo Spaces (volume, centre, fill). Module
subsections (section 7).

**A4 General arrangement.** Fixed: the GA sheets (side elevation, plan, midship section) from
`ovd draw`; a Spaces summary by kind whose volumes add to the enclosed volume (checked); sightline and
air draught. Not evaluated: accommodation and manning (until the manning model of spec 0011 lands),
deck equipment.

**A5 Structure.** Fixed: midship section with scantlings, hull steel by the method that drives the
lightship and by the comparison method, with the gap; material; calibration of the steel method. Not
evaluated: longitudinal strength, local strength, fatigue.

**A6 Powering, propulsion and machinery.** Fixed: the resistance method named as the engine runs it,
with its validated range and whether this hull is inside it; power and speed by environment;
installed power and margin; propulsor, propeller diameter and blade load; stern arrangement;
machinery items with their masses; endurance or fuel and gas balance; bollard pull. Not evaluated:
electrical load, manoeuvring. Module subsections (section 7).

**A7 Weights.** Fixed: lightship by item (the items add to the lightship; checked), margins,
deadweight by item, KG and LCG per item where the run has them, the lightship calibration label. Not
evaluated: centres the run does not hold.

**A8 Hydrostatics and loading conditions.** Fixed: hydrostatic table by draught, curves of form
chart, each loading condition item by item (mass, VCG, LCG, free surface), with draught, trim and GM.

**A9 Freeboard.** Fixed: freeboard carried at the laden draught, freeboard required and its basis, the
corrections applied. Not evaluated: the ICLL corrections the engine does not apply (#730).

**A10 Intact stability.** Fixed: the criteria set, GZ curves per condition, a criteria table
(criterion, value, limit, margin, outcome) from the trace, the lowest-GM condition. Not evaluated:
weather criterion when windage is not typed; seakeeping and motions (#731), as a subsection once the engine holds them.

**A11 Damage stability.** Fixed: rule set and standard of damage, the case table, the governing case
per condition with its floating position and GZ, the governing-case drawing (spec 0012). Not
evaluated: any hull type or rule set the run has no damage cases for.

**A12 Compliance.** Fixed: trace counts; a matrix with one row per governing or warning Rule (Rule,
clause, requirement, value, limit, outcome, verified or not); a paragraph per governing Rule built
from its trace lines; the list of Rules not evaluated. The full trace goes to Appendix B2.
Plain-words lines per Rule come from the rules data, not from the project.

**A13 Cost.** Fixed: currency, price base date, rates and their basis, cost by item, yard margin,
total; the label "example rates" when the rates are the engine defaults. Not evaluated: operating
cost.

**A14 Limits and open items.** Fixed: an open-items table built from the run (item, why, how it
closes): every not-evaluated section and Rule, every warning, every uncalibrated method, every method
run outside its range. It states facts; it asks
nothing.

### 4.3 Appendix B: the evidence

Evidence and studies (D7), after the technical chapters.

| § | Title | Holds |
|---|---|---|
| B1 | Run provenance | Engine version, commit, file engine version, input file hash, results hash and hash status per Option |
| B2 | Full rule trace | Every Rule row, clause by clause |
| B3 | Calibration | The evidence behind each calibrated method, with ranges |
| B4 | Drawings | The drawings list and the full sheets |
| B5 | Full tables | Hydrostatics, loading condition items, Spaces |
| B6 | Trade studies | When supplied; the section is present and says "none supplied" otherwise |
| B7 | Sources | Documents cited, by edition and paragraph |
| B8 | Symbols and terms | From the glossary |

## 5. "Not evaluated" behaviour

- Every heading of section 4 and every module subsection is always present.
- When a part has no run data, a fixed block replaces its content: **Not evaluated.** What this part
  would show; why it is not evaluated (the run's reason, or the missing engine capability with its
  issue number); what would close it.
- Inside a table, a missing value is printed "not evaluated", never blank, never zero.
- A not-evaluated part has no reading. Its finding line says "Not evaluated" with the reason; a person
  cannot fill the gap with prose.
- Every not-evaluated part and Rule appears again in Appendix A14 and in body part 9.
- In the body, a not-evaluated part gets one line in its finding and one row in the limits part
  (body part 9). In the appendix it gets the full block, with the reason and what would close it (D12).
- "Not applicable" is a separate state. It is printed only when a rule or a module says so (for
  example probabilistic damage for a barge, spec 0012 D4), with the reason.

## 6. Findings and readings

Each part has two pieces of text about the design (D4):

- a **finding**: one line, 25 words or fewer;
- a **reading**: about 120 words (limit 150), three to five sentences.

The summary, each body part and each Appendix A chapter has a finding. Each body part and the summary
also has a reading; an Appendix A chapter shows its body part's reading. There are no challenge
questions and no asks (D2, D4).

### 6.1 A figure in the text is inserted, never typed

A sentence is a fixed template with named blanks, for example `{governing_rule}` and `{margin}`. The
builder fills each blank from the run by name and writes the value with its unit. It records the
blank and its JSON pointer in the trace map. A numeral typed into a template is an error. Because the
text is filled from the run, a changed design changes the text, and a typed figure cannot go stale.

### 6.2 Findings are made by software, with no language model

Findings and readings are deterministic. No language model is used, anywhere in the build (D9). Three
steps:

1. **Rules over the run.** A finding rule is a named test on run fields. It decides what is true: which
   constraint binds, which Rule governs and by how much, how many checks pass, which weight item is
   largest, how the lead option differs from the others.
2. **Templates.** Each rule has one or more fixed sentences with blanks. The rule picks the sentence
   that fits (for example "volume binds" or "weight binds") and the builder fills the blanks.
3. **Assembly.** A reading is an ordered list of rules for the part. The builder keeps each sentence
   whose rule applies, drops the rest, and stops at the word limit. A part with no data gets the
   not-evaluated sentence instead.

The same run and engine version give the same text, byte for byte (P8). The rules and templates live
in data files beside the report code, so a maintainer can change a sentence without changing code.

### 6.3 Worked examples

The figures below are generic and only show the shape.

| Part | Rule (what it tests) | Template | Filled |
|---|---|---|---|
| Capacity | which limit binds: the smallest of the volume, weight and draught limits | "Cargo capacity is set by {binding_limit}: {cargo_t} against {next_limit_t} if {next_limit} alone applied." | "Cargo capacity is set by the draught cap: 1,850 t against 2,300 t if volume alone applied." |
| Intact stability | the criterion with the smallest margin, over all conditions | "The governing intact criterion is {criterion} in the {condition} condition: {value} against a limit of {limit}." | "The governing intact criterion is area under the righting arm curve to 30 degrees in the laden condition: 0.18 m rad against a limit of 0.055 m rad." |
| Compliance | counts of each outcome in the trace, and the Rule with the smallest margin | "{pass} of {total} checks pass, {warn} warn, {fail} fail and {not_evaluated} are not evaluated. The tightest margin is {rule}." | "41 of 46 checks pass, 2 warn, 0 fail and 3 are not evaluated. The tightest margin is the minimum metacentric height." |
| Weights | the largest lightship item and its share | "The largest lightship item is {item}, {share} of the lightship of {lightship_t}." | "The largest lightship item is hull steel, 38 % of the lightship of 410 t." |
| Options | the differences between the lead option and each comparison | "{lead} carries {delta_cargo} {more_less} cargo than {other}, needs {delta_power} {more_less} installed power and costs {delta_cost} {more_less}." | "Option B carries 120 t more cargo than Option A, needs 300 kW more installed power and costs 4 % more." |

A reading for the capacity part joins the capacity sentence, a sentence on how full the cargo spaces
are and, for a gas barge, a sentence on boil-off margin, each from its own rule.

### 6.4 Rules that apply to all text

| Rule | Detail |
|---|---|
| Length | Finding 25 words or fewer; reading about 120 words, limit 150. A breach is an error. The exact limits are deferred (section 12) |
| Figures | Every figure is a filled blank. A typed numeral is an error, except inside a rule citation (`ES-TRIN 28.03(3)`), a Rule id, or an issue reference |
| Claims | May state what a rule found, compare, and cite a source by document, edition and paragraph. May not claim a check the trace does not hold, and may not use "approved", "certified" or "compliant" without a trace outcome |
| Scope | Speaks about this design only. No project-confidential material belongs in a public example |
| First read | No reference to earlier revisions, drafts or "what changed" (section 10) |
| No asks | No question to the reader, no "please confirm", no "we recommend you check" (P9) |
| Override | A person may supply a replacement reading for a part in a narrative file keyed `<part>.reading`. The checker applies the rules above to it. The default is the generated text |

The checker runs in `ovd report check` and in `ovd report` itself.

### 6.5 No model polish

Every sentence comes from a template (D9). Plain, a little repetitive, and byte-identical every time.
There is no optional rewording step and no hook for one. A person may replace a reading through the
narrative file (section 6.4); the checker applies the same rules to it.

## 7. Vessel-type modules

A module adds subsections inside fixed sections (D5: one skeleton, with modules). It chooses itself
from the run (`cargo`, `vessel`, the Formation kind). More than one module may apply (a gas barge in
an ATB formation). The subsections sit in the Appendix A chapter named; the body digest shows them
only when they change the finding. The set:

| Module | Adds |
|---|---|
| Gas (IGC, ADN type G) | A3 tanks: type, diameter, design pressure, clearances; A6 boil-off and gas balance; A11 IGC damage cases (spec 0012) |
| Liquid bulk (ADN type C/N, MARPOL I) | A3 cargo and slop tanks by fill; A11 MARPOL damage; A14 oil outflow, when evaluated |
| Dry bulk | A3 hold volumes; A10 grain or cargo-shift stability, when evaluated |
| Hopper | A3 hopper volume and load; A10 the opening and split states, when evaluated |
| Deck cargo | A3 deck area and deck load; A10 windage of the cargo |
| Pushed or articulated formation (spec 0007, spec 0014) | A2 the Formation and each unit; A4 the Notch and coupler; A10 each unit's own stability; A6 Formation power, calibration error, Time Connected; side-by-side comparison with a self-propelled option (#799) |

A module may not add a top-level section, change the order, or add text outside its subsections.

## 8. More than one option

One Option leads the report (D6). Up to three other Options of the same run may be named as
comparisons. They appear as extra columns in the principal-characteristics table, the weights and
capacity tables, the powering table and the compliance counts, and in one "Options compared" table in
body part 1 (Appendix A2 holds the full one). Every other table shows the lead Option only.

## 9. The `ovd report` command, the Python API and ORD output

**Command line.**

```
ovd report <file.vdesign> --option <name> [--compare <name> ...] [--narrative <overrides.yaml>]
           [--number <doc-no>] [--revision <letter>] [--date <yyyy-mm-dd>]
           [--format ord,pdf,html,docx | all] -o <out-dir>
ovd report --from-run <run.json> --option <name> ...      # no engine run; read a saved run
ovd report check <file.vdesign|run.json> --option <name> [--narrative <overrides.yaml>]
ovd report --findings [--option <name> <file>]            # list findings and the rule behind each
```

`ovd report` runs the design (or reads a saved run), builds the ORD document, checks it (`ord check`
and the text checker) and renders the asked formats. Exit 1 on any error.

**Python.** `ovd.report.build(run, option, compare=(), narrative=None, control=None) -> dict` (the
ORD document); `ovd.report.check(doc_or_run, narrative=None) -> list[Finding]`;
`ovd.report.findings(run, option) -> list[FindingText]`; `ovd.report.render(doc, formats, out_dir)`.
`tools/concept_report.py` becomes a thin wrapper, then is removed. Deferred: whether the builder moves
into Rust beside the engine, so the desktop app and the browser build share it.

**ORD.** The report is an ORD document: one JSON file, rendered to PDF, HTML and Word by ORD's three
writers, with ORD's structure parity between them. Class: `technical-note` today; a `concept-report`
class with its own number prefix and lifecycle is deferred. The builder uses ORD's `summary`, `notice`,
`control`, sections, tables, figures, `data_box` (one "Data and limits" box per part, listing the run
fields and method limits it rests on) and appendices.

**Look (D15).** OVD has its own report template, a new one derived from the report template the project
already uses. It has the OVD blue on the cover page, a general arrangement drawing of the vessel as the
cover image, and the OVD colours for accents in the body. The colours are the tokens of the OVD style
guide (`docs/design/style-guide.md`, section 2; `website/tokens.css`), not new ones:

| Place | Token | Value |
|---|---|---|
| Cover ground | `--blueprint` | `#24496c` |
| Cover edge and dark bands | `--blueprint-deep` | `#1a3753` |
| Lettering on the cover | `--white-ink` | `#eef6f6` |
| Frame and rule ornaments | `--gilt` | `#a78654` |
| Rules and marks that carry meaning | `--gilt-mark` | `#8a6a35` |
| Body headings and text | `--ink` | `#1c3550` |
| Table heads and insets | `--surface-2` | `#efe4c9` |
| Links, key-figure accents | `--accent` | `#24496c` |

Pass, warn and fail keep the status tokens (`--pass`, `--warn`, `--fail`), each with a word and a
shape as well as a colour. The cover drawing is the general arrangement side elevation from `ovd draw`,
drawn in white ink on the blueprint ground. A drawing on a body page stays black on white, as the
Drawing Format Standard prints it. A project may pass its own ORD template to change the look.

**Units and number format (D14).** SI units. Thousands separators. A set number of decimals per
quantity, held in one table in the builder's data files, for example: lengths to two decimals, draughts
to two decimals, masses to the whole tonne, power to the whole kilowatt, percentages to one decimal.
The same table sets the digits in a table cell and in a filled sentence, so the two never disagree.

## 10. Review asks, judgement calls, revision and the first-read rule

**No cover note (D11).** The report never carries a review ask or a challenge question, and OVD does not
write a cover note. The open-items table (A14) states the facts a reviewer needs. It states facts and
asks nothing.

**Roadmap pointer, not part of this spec.** Work that needs a judgement call, such as deciding what to
ask a client or how to answer a reviewer, belongs to a future OVD agent template. The template will
ship with memories, workflows, skills and tools, so that users can do this kind of work with an agent.
This spec does not design it. The report has no hook for it.

**Revision.**
- A report is read as if it were the only version. No text refers to an earlier revision, draft or
  version: no "changed since rev A", "now", "no longer", "as before", "updated".
- The control block carries the revision letter and date only. It has no change table.
- Change history lives outside the document: in the version control of the design file and any
  narrative file, and in the review record of each issue. A project may keep a separate change note.
- Only a released, citable report of another design may be referenced, by number and revision, as a
  source.
- The text checker flags revision language (a word list, as a warning; the maintainer decides if it
  becomes an error).

## 11. Testing

1. **Golden reports.** Each public example with a report (gas, liquid bulk, an ATB formation) has a
   stored ORD document. A change to the builder that moves it must update the golden file in the same
   change, and the diff is reviewed.
2. **Every figure traced.** The trace map covers every number in the document. A test walks the ORD
   document, finds every numeral outside citations and ids, and fails on any number with no entry.
3. **Trace that can fail.** A perturbation test changes one run field per section and asserts that
   the matching figure moves, and no other. A trace map that merely lists pointers cannot pass it.
4. **Not evaluated.** A test deletes each run block in turn and asserts that every heading is still
   present and the right part reads "not evaluated".
5. **Findings.** Per finding rule: a perturbation that flips its test (volume binds to weight binds)
   flips the sentence; a rule with no data drops its sentence; no template holds a numeral; a filled
   reading stays inside the word limit. A typed figure, a forward reference, an approval claim, an
   ask and revision language each fail the text checker.
6. **Modules.** Each module's subsections appear for its vessel type and for no other.
7. **ORD.** `ord check` passes; the three formats pass ORD's parity check.
8. **Determinism.** Two builds of the same input are byte-identical.
9. **Page budget.** The rendered body of each golden report is about ten pages.
10. **No engine change.** No results hash moves (report only).

## 12. Decisions (design tree)

Round 1 (D1–D8) and round 2 (D9–D15) were both decided with the maintainer on 2026-11-02. No decision
is open.

| # | Decision | Options | Decided, and why | State |
|---|---|---|---|---|
| D1 | Primary reader | expert reviewer; client; both, layered | **The client or owner.** The report goes to the person who decides. Technical detail goes to the appendices, where the expert reads it | Decided |
| D2 | Summary | leads the report; ends it; both | **First, one page that stands alone, with principal characteristics and the verdict.** The report must look like the final report once the engine is trusted, so it holds no asks and no challenge questions. Review asks are not in the report (D11) | Decided |
| D3 | Order principle | strict build-up; skeleton as proposed; by reader question | **Strict build-up.** Each section uses only earlier ones. Structure, powering and machinery come before weights; freeboard sits beside hydrostatics | Decided |
| D4 | Narrative | finding and reading per section; summary only; free prose | **Per section, a one-line finding and a short reading of about 120 words, no typed figures.** No challenge questions. A figure in the text is inserted from the run by name, never typed. Findings are made by rules and fixed templates, with no language model by default (section 6) | Decided |
| D5 | Vessel types | modules inside one skeleton; one skeleton per type; one skeleton, no modules | **Modules inside one skeleton.** Reports of different types stay comparable; a module adds subsections only | Decided |
| D6 | More than one option | one leads, others as columns; every table side by side; one design per report | **One option leads; up to three others as comparison columns in the key tables, plus one options-compared table.** Keeps every other table readable | Decided |
| D7 | Appendices | evidence only; evidence and studies; minimal | **Evidence and studies:** provenance, full rule trace, calibration, drawings, full tables, trade studies, sources | Decided |
| D8 | Length | 20–30 pages; 40; no target; other | **Other: the body is about ten pages and reads as an executive summary.** The full technical chapters (the step-by-step build-up) move to Appendix A, followed by the evidence of D7 in Appendix B | Decided |
| D9 | Findings and readings: how made | fully templated, no model; templated plus optional checked model polish | **Fully templated.** Rules and fixed sentences only. No language model anywhere in the build. A model could reword a sentence, but then the build is no longer a pure function of the run, and every reword needs its own check. Plain and repeatable beats smooth | Decided |
| D10 | What the ten-page body holds and in what order | the eleven parts of section 4.1; five grouped parts; summary, matrix and figures only | **The eleven parts of section 4.1:** cover, summary, vessel and brief, capacity and arrangement, structure and powering, weights, floating and stability, damage stability, compliance, cost, limits. It follows the build-up of D3 and gives each Appendix A chapter a digest | Decided |
| D11 | Cover note for review asks | OVD generates its open points, a person adds the questions; written by hand, no tool; no cover note, asks go in the review tool | **No cover note.** The report carries no asks, so there is nothing for a note to carry. Judgement-call work belongs to a future OVD agent template with memories, workflows, skills and tools (roadmap pointer, section 10; not in this spec) | Decided |
| D12 | "Not evaluated" in body and appendix | one line in the body and the full block in the appendix; full block in both; open-items part only | **Body: one line in the finding and one row in the limits part. Appendix: the full block, with the reason and what would close it.** The client sees every gap and is not held up by it; the expert finds the detail | Decided |
| D13 | Figures in the body | which GA views, the midship section, GZ curve, damage case drawing, lines | **Body figures: the GA side elevation and plan, the midship section, the governing righting-arm curve and the governing damage case.** Those four answer what it is, how it is built and whether it stands up and survives. Other drawings go to Appendix B4 | Decided |
| D14 | Units and number formatting | SI with set digits per quantity; fixed significant figures; reader's locale | **SI, thousands separators, and a set number of decimals per quantity** (for example lengths to two decimals, masses to the whole tonne). Readers compare reports; one rule per quantity keeps figures comparable and a table agrees with the text | Decided |
| D15 | Default look | OVD's own neutral style with a house style as a template; a house style as the default; plain | **A new OVD report template, derived from the report template the project already uses: the OVD blue on the cover, a general arrangement drawing of the vessel as the cover image, OVD colours for body accents** (section 9). The look of the report matches the look of the tool, with colours from the existing tokens | Decided |

Deferred, not open decisions: exact word limits; seakeeping and manning placement; whether the builder
moves to Rust; an ORD `concept-report` class; the revision-language list.

## 13. Build tickets (proposed; filed after this spec is merged)

Each ticket has a title, what it does, how it is accepted, and what blocks it.

**T1. Skeleton and page budget**
- What: the body parts, Appendix A chapters and Appendix B sections as fixed ORD sections. Every heading
  is present. The "not evaluated" block (body line plus limits row; full appendix block). The page-budget
  check (warning above 12 pages). Today's parts move into their chapters.
- Accept: a report builds for each public example with every heading of section 4 present; deleting any
  run block leaves every heading and shows "not evaluated" in the right place (section 11, tests 4, 9).
- Blocked by: none.

**T2. Trace map**
- What: record a JSON pointer for every table cell, chart series and sentence blank.
- Accept: the every-figure test finds no number without an entry; the perturbation test moves only the
  matching figure when one run field changes (tests 2, 3).
- Blocked by: T1.

**T3. Findings engine and text checker**
- What: finding rules, sentence templates as data, the reading assembler, the narrative override file,
  and the text checker (figures, claims, asks, first read, length). No language model, no polish hook.
- Accept: per finding rule, a flipped test flips the sentence; no template holds a numeral; a filled
  reading stays inside the word limit; a typed figure, a forward reference, an approval claim, an ask
  and revision language each fail the checker (test 5).
- Blocked by: T1, T2.

**T4. New fixed parts from existing run fields**
- What: brief, "what bounds each dimension", cargo Spaces, Spaces summary, loading conditions item by
  item, freeboard, cost, open-items table and the body digests.
- Accept: each part is built only from run fields, with its not-evaluated trigger; the Spaces add to the
  enclosed volume and the lightship items add to the lightship (checked).
- Blocked by: T1.

**T5. Units and number format table**
- What: the set decimals per quantity in one data table; the builder formats every cell and every filled
  blank from it; thousands separators.
- Accept: a figure prints the same in a table and in a sentence; a test fails on a quantity with no
  entry in the table.
- Blocked by: T1.

**T6. `ovd report` command and Python API**
- What: `ovd report`, `ovd report check`, `ovd report --findings`; the `ovd.report` Python API;
  `tools/concept_report.py` becomes a wrapper.
- Accept: the command writes ORD, PDF, HTML and Word for a `.vdesign` file and for a saved run; exit 1 on
  any error; two builds are byte-identical (test 8).
- Blocked by: T1, T3.

**T7. OVD report template**
- What: a new ORD template derived from the project's existing report template: OVD blue cover, the GA
  side elevation as the cover image in white ink, OVD colour tokens for body accents (section 9).
- Accept: the template uses only tokens of `docs/design/style-guide.md` section 2 (a test reads both);
  the cover meets the contrast of the token pairs; ORD parity holds across the three formats (test 7).
- Blocked by: T1, T6.

**T8. Vessel-type modules: gas and liquid bulk**
- What: the module mechanism and the first two modules (section 7).
- Accept: each module's subsections appear for its vessel type and for no other (test 6).
- Blocked by: T1.

**T9. Formation module and comparison Options (#799)**
- What: the pushed or articulated formation module; up to three comparison Options as columns.
- Accept: an ATB option sits beside a self-propelled option in the key tables and in the options-compared
  table; every other table shows the lead Option only.
- Blocked by: T8.

**T10. Damage stability chapter**
- What: Appendix A11 and body part 6 from the outputs of spec 0012, with the governing-case drawing.
- Accept: the case table, the governing case per condition and the drawing come from the run; a hull
  type with no damage cases reads "not evaluated".
- Blocked by: T1, the spec 0012 tickets.

**T11. Golden reports and determinism**
- What: a stored ORD document for each public example with a report (gas, liquid bulk, an ATB formation).
- Accept: a builder change that moves a golden file updates it in the same change; the body of each is
  about ten pages; no results hash moves (tests 1, 9, 10).
- Blocked by: T4, T5, T6, T8.

**T12. Docs: report format page and a worked example**
- What: the report format page, and the worked example report attached to the release.
- Accept: the page lists the skeleton, the not-evaluated rule and the units table; the example is the
  140 × 29 m worked example.
- Blocked by: T11.

## 14. Not in scope

Approval documents (stability booklet, loading manual, class submission); asks, challenge questions and
a cover note; the OVD agent template named in section 10; a report editor UI; operating cost and
life-cycle cost; translations (ORD handles language when it can). A language model in the builder is out
of scope.
