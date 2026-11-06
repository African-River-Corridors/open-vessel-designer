---
ticket: 935
phase: research-questions
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Research Questions: damage survival inside the arrangement search

## Key Context Pointers

- Ticket #935; evidence in the findings comment on #914; parent #734; related #915 (say what stops a larger tank), #923 (`--no-damage`), #921 (damage run time).
- Search: `crates/ovd-core/src/gas_arrangement.rs` (`search`, `passes_all`, `fit`).
- Final damage run: `crates/ovd-format/src/simulate.rs` (IGC 2.7 block) and `crates/ovd-format/src/damage.rs`.
- Damage engine: `crates/ovd-core/src/damage_cases.rs`, `damage_stability.rs`, `measured.rs`.
- Rule data: `crates/ovd-format/rules/igc-2g/*.json`; rule text: IGC (MSC.370(93)) 2.3, 2.6, 2.7.

## Questions

1. How does the search order and stop its costly stage, and what does that stage read today?
2. Where are the IGC 2.7 damage cases built and run, and which parts live in `ovd-core` (`no_std`) and which in `ovd-format`?
3. What does a candidate already carry that a damage run needs (hull, IS Code conditions, vessel model, spaces, Rule Set tables)?
4. How does the Rule Trace turn damage verdicts into `igc/2.7*` rows, and what does it do when no damage was computed?
5. Which other damage Rule Sets does a gas run evaluate (ADN 9.3.1.15, ES-TRIN 28.03), and under what conditions?
6. How is the diameter stepped today, and how would a 0.1 m step be expressed in the search's own terms (side clearance)?
7. Does `--no-damage` (#923) exist on `main`? What does a skipped damage run look like in the results?
8. Which tests and examples cover the search and the damage block, and which hashes move when the tank moves?
