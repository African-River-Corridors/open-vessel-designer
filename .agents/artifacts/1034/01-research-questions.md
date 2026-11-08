---
ticket: 1034
phase: research-questions
date: 2026-11-08
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a85c10002f3e7e752f587d5ac0e745834532a76f
---

# Research Questions: river draught and air-draft limits, and the cost of the damage search

## Key Context Pointers

- Ticket #1034. Related: #992 (tank tops above the deck; P5, PR #1027, the river-only share search), #935 (damage in the search; PR #1020; tanks along, PR #1030), #991 (`bridge_clearance_m`, a warning only), #925 (the Service Area as a required input, the precedent for "required"), #94 (Vessel Classes fill the limits).
- Inputs: `crates/ovd-format/src/builtin_profile.json` (`draught_limit_m`, `air_draft_limit_m`, `bridge_clearance_m`, `tank_top_above_deck_*`), `crates/ovd-format/src/vdesign.rs` (`INPUTS`, `Missing`, required inputs), `crates/ovd-format/src/class_input.rs` (a class fills the limits).
- Search: `crates/ovd-core/src/gas_arrangement.rs` (`fit_with`, `share_stage_one`, `search`, `target_depth`, `DEFAULT_DAMAGE_RUN_CAP`). Run: `crates/ovd-format/src/simulate.rs` (`simulate_once`, the depth probe and passes; the draught limits).
- Python API: `src/ovd/simulate.py`; CLI: `crates/ovd-cli/src/main.rs`; app: `app/src/lib/LeftPanel.svelte`, `InputField.svelte`.
- Class limits: `catalogue/vessel_classes.yaml` (ECMT Resolution 92/2, Table 1).

## Questions

1. What do `draught_limit_m` and `air_draft_limit_m` do today in a gas run: in the search, in the depth, and in the checks? Does either stop a river-only hull from growing deep?
2. How does `bridge_clearance_m` (#991) differ from `air_draft_limit_m`, and where does each act?
3. What does "required" mean in this code today (the Service Area, #925), on each path: saved file, `fill`, `fill --as-new`, `new`, the app session, and `simulate` in Python?
4. Where can a limit come from besides typing: a Vessel Class, a Service Area default, a route? What values do published sources give for river draughts and bridge heights?
5. How does the river-only share search (#992 P5) pick a share, and what does "most cargo" mean in the ranking (volume, or tonnage at the draught cap)?
6. On the 100 x 20 m river-only LNG barge, with realistic river limits, what do "tallest share that passes" and "most cargo within the limits" pick, at what cargo, depth and air draft?
7. How long does one IGC 2.7 damage run take, and how many run per design, on the 140 x 29 m coastal LNG reference design, an 89 x 16 m coastal design, the 100 x 20 m river barge and a second river design?
8. Why did the 100 x 20 m river barge go from about 2.5 s to about 10 s with #1027? The PR said damage screening; the test `damage_budget.rs` says a river-only run screens no IGC damage. Which is right?
9. What does each search option cost in run time: the current search, every share searched, most cargo within the limits, and the damage-run cap at 12, 24 and 48?
