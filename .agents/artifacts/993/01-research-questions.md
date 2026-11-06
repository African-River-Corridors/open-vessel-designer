---
ticket: 993
phase: research-questions
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Research Questions: arrangement style "tanks in hold, flat deck"

## Key Context Pointers

- Ticket #993; related #992 (larger tanks, less depth, weather shield); parent damage work #935.
- `crates/ovd-core/src/aft_arrangement.rs`: `ArrangementStyle`, `ARRANGEMENT_STYLES`, `compute`, `sightline_obstruction_m`.
- `crates/ovd-core/src/gas_arrangement.rs`: `search`, `fit`, `base_inputs` (dome height and diameter).

## Questions

1. What does `ArrangementStyle` change today, and where is the style read?
2. Where does gas depth come from, and what stops a depth that follows the tank OD?
3. What does the sightline code take as the obstruction, and can it take the dome tops?
4. How does the damage model treat the deck, the hold and the tank domes?
5. What is lightship made of, and what must a deck and hatches add?
6. Which drawings show the tanks, and which fixtures move?
