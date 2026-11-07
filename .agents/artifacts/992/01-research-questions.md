---
ticket: 992
phase: research-questions
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Research Questions: gas depth with larger tanks, less depth and a weather shield

## Key Context Pointers

- Ticket #992. Related: #989 (sightline limit per Service Area), #990 (eye height 1.65 m), #935 (damage in the search).
- Depth default: `crates/ovd-format/src/builtin_profile.json` (`depth_to_beam_ratio`), `catalogue/depth_rule.md`, `crates/ovd-format/src/vdesign.rs` (`default_by("depth_to_beam_ratio", ..)`, around lines 5176 and 6018).
- Tank fit: `crates/ovd-core/src/gas_arrangement.rs`. Sightline and air draft: `wheelhouse.rs`, `ga_adapter.rs`, `vessel_model.rs`.

## Questions

1. What sets gas depth today, and what is the evidence behind 0.55?
2. What does the tank-fit search take as a depth limit, and what stops a tank standing above deck?
3. Does depth or freeboard help the governing damage case?
4. What does a tank above deck do to the sightline and the air draft?
5. Where do freeboard and load-line read the deck and superstructure, and how would a non-watertight shield be treated?
6. Where would a shield input live (Option, Defaults Profile, Service Area), and which drawings and hashes move?
