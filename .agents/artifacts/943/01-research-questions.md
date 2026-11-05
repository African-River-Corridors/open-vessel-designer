---
ticket: 943
phase: research-questions
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Research Questions: deckhouse tiers sized from what they hold

## Key Context Pointers

- Ticket #943 (found in the review of #942). Related: #912 (openings, being built on `ticket/912-deckhouse-openings`), #783 (GA drawing), #911, #894.
- Spec `docs/specs/0011-stern-arrangement.md` (D3, D4, D6; section 2 says the rooms give "the deckhouse footprint and tier count").
- Code: `crates/ovd-core/src/aft_arrangement.rs` (`size_deckhouse`), `rooms.rs`, `barge_capacity.rs` (two-pass wheelhouse), `tonnage.rs`, `crates/ovd-format/src/cost.rs`, `eval.rs`, `run.rs`.
- Sweep case: `tools/design_impact.py:575` ("sweep gas river-only deckhouse 140x29").

## Questions

1. How does `size_deckhouse` choose the footprint and the tier count today, and where does the sightline add tiers?
2. What does the rooms table hold (rooms, stores, stairs, service spaces), and what does it leave out?
3. Where does the wheelhouse stand (x and z), and what sets the eye height a fixed wheelhouse needs?
4. Who reads the deckhouse's area, volume, tiers and footprint: mass, cost, tonnage, Rule Trace, results JSON, app?
5. How do the #912 openings read the deckhouse shape (the `Frame`)?
6. Which tests and sweep cases pin the deckhouse today, and what moves if the area changes?
7. Which Rules bear on it: ES-TRIN 7.02 (unobstructed view), SOLAS V/22.1, ES-TRIN 15 and MLC A3.1 (rooms, headroom), IGC 3.2.4 (openings)?
