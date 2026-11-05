---
ticket: 913
phase: research-questions
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Research Questions: igc-2g 4.0.0, IGC 3.2.4.1 and 3.2.4.2 on placed openings

## Key Context Pointers

- Ticket #913; blocked by #912 (placed openings, in build on `ticket/912-deckhouse-openings`, read at commit `1fcb2a4e`, uncompiled) and #898 (igc-2g 3.3.0, merged).
- Rule Set: `crates/ovd-format/rules/igc-2g/3.3.0.json`; registry `crates/ovd-format/src/rule_set.rs`; released list `rules/RELEASED.json`.
- Spec 0004 "Version numbers" and "Object-scoped inputs" (`docs/specs/0004-rules-engine.md`).
- Rule text: IGC (MSC.370(93)) 1.2.17, 1.2.31, 3.2.4.1, 3.2.4.2, 3.2.5.
- Evaluation harness: `evaluation/` (README, `cases/`, `oracle/`).

## Questions

1. What do the four `igc/3.2.4.1/*` rows read today, and which other Rule Sets read the same inputs?
2. What does #912 give the Rule Trace: object type, Measures, and their values for the default layout?
3. Can a Rule filter openings by kind, space, face and gas-tightness? What object-scoped inputs exist, and how are they declared?
4. Can a formula limit read an Option-level Measure (L of ICLL) while the Rule is on an opening?
5. How does the trace pick the worst object of a Rule (the governing row)?
6. How is a new major version registered, and does the default coastal stack follow it without a profile edit?
7. Which #898 gaps (`igc/3.6.1/airlock`, `solas/V.22.1.3/conning-arc`) could evaluate once openings are placed?
8. How would boundary cases be expressed in the evaluation harness, and is there a case kind that fits?
9. What happens to a coastal gas Option with a container-block stern (no deckhouse, so no openings)?
