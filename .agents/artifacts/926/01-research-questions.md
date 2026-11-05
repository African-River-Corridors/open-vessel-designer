---
ticket: 926
phase: research-questions
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Research Questions: tank design pressure from the Service Area

## Key Context Pointers

- Ticket #926; follows #317, #577. Blocked by #925 (Service Area required). Coordinate with #875 (holding-time solve of pressure and insulation), #927 (igc-2g 5.0.0 makes the minimum block). `simulate.rs:870` is shared with lane B1.
- Profile: `crates/ovd-format/src/builtin_profile.json` (`tank_design_pressure_mpa`), `profile.rs`, `vdesign.rs` (`TABLE_KEYS`), `session.rs`.
- Engine: `crates/ovd-core/src/tanks.rs`, `pressure_vessel.rs`; `crates/ovd-format/src/{run,simulate,gas_arrangement}.rs`.
- Rules: `rules/igc-2g/3.3.0.json` (`igc/4.23.1.2/min-design-pressure`), `rules/ovd/3.1.0.json`. Text: IGC 4.23.1.2; ADN 9.3.1.23.1, 9.3.1.24.1.

## Questions

1. How does a `null` design pressure become the IGC 4.23.1.2 minimum today, and which entry points honour it?
2. How are profile defaults chosen by context (`default_by_table`), and can the context include the Service Area?
3. What happens to a defaulted input when a table key changes in a session?
4. What does the IGC row read and how does it report? What does a `note` consequence do?
5. Which `ovd` Rule Set version is next, and how is Applicability on `service_area` written?
6. What does the hand-worked Po give for the 140 × 29 m example's tank, and does it match the engine?
7. How would the results show how the pressure was set (typed, default, IGC minimum)?
