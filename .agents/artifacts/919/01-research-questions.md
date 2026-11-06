---
ticket: 919
phase: research-questions
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Research Questions: insulation as explicit inputs

## Key Context Pointers

- Ticket #919; part of #823. Research input: the #918 comment of 2026-11-05 (table of values, hand U-value and heat leak). Related: #920 (insulation tables), #875 (holding-time solve), #872 (Gas view), #873 (report), #930 and lane B1 (`simulate.rs`), #932 (no drift).
- Code: `crates/ovd-core/src/{gas_fuel,tanks,models,barge_capacity,ga_adapter}.rs`; `crates/ovd-format/src/{gas_fuel,holding_time,simulate,eval,vdesign}.rs`; `builtin_profile.json`.

## Questions

1. Where does each insulation value (type, thickness, conductivity, density) enter the engine, and which paths can reach a value that is not the input?
2. Which results print the conductivity, heat leak and boil-off today, and which do not (U-value, heat leak per tank)?
3. How does `simulate` treat a key left out, and what does that mean for the 350 mm constant?
4. What does the `tank_insulation_type` input decide today?
5. What values and sources did #918 find, and do the hand U-value and heat leak match the engine?
6. Which tests and examples pin insulation numbers, and what moves if a default changes?
