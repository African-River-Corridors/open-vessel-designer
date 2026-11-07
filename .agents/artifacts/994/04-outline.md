---
ticket: 994
phase: outline
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Outline: arrangement style "wheelhouse forward"

One PR per phase to `main`. Each test fails on `main` first.

## Implementation Overview

Five PRs, stacked (the Service Area gate folds into PR 4; re-pins go into each PR's changelog fragment).

- [ ] PR 1: the style key and the forward layout (D1, D2)
- [ ] PR 2: sightline from the forward eye and the wing check (D3)
- [ ] PR 3: openings with the cargo face aft, and the new Rule rows (D4)
- [ ] PR 4: the hazard-study flag in results and report, and the coastal warning (D5, D6)
- [ ] PR 5: weight, trim and drawings (D7)

## PR 1: key and layout

Files: `crates/ovd-core/src/aft_arrangement.rs`, `crates/ovd-format/src/vdesign.rs`.
- Test that fails today: `from_key("wheelhouse_forward")` is `Some`; a river vessel with that style puts the house forward of the first tank (or over the forepeak) and machinery aft. `Label: property`.
- Result: a layout with the house forward; no Rule changes yet.

## PR 2: sightline and wing check

Files: `crates/ovd-core/src/wheelhouse.rs`, `aft_arrangement.rs`.
- Test: with the eye forward and tanks behind it, the blind distance is 0 and lift needed is 0; an obstruction ahead of the eye still counts.
- Test: wing check passes when a wing sees along the full hull side; fails with a wing short of the aft end of the house.
- Result: new Measures; cameras listed as an open item.

## PR 3: openings and Rule rows

Files: `crates/ovd-core/src/deckhouse_openings.rs`, `crates/ovd-format/rules/igc-2g/<next>.json`.
- Test: a door on the aft face fails 3.2.4.1; a door on the fore face passes; a side door inside the zone fails.
- Test: a vent mast 20 m from an opening fails 8.2.11.1; 25 m passes.
- Result: new `igc-2g` version; older pins give the same output as today.

## PR 4: hazard-study flag

Files: `crates/ovd-format/src/run.rs`, report writer.
- Test: results hold `hazard_study_required: true` with at least one reason; the report's first section names it; other styles hold `false`.

## PR 4 (continued): coastal warning

Test: coastal with this style gives a named warning and a hazard-study reason (bow motions); river and estuary give neither. No Service Area fails the style (maintainer's choice at the gate).

## PR 5: weight, trim, drawings

Test: the longitudinal centre moves forward by the house mass moment; total mass unchanged to rounding; the drawing shows the house under the wheelhouse (snapshot).

## Re-pins

Each PR runs `python3 tools/design_impact.py changelog`; expect no moved Option; re-pin only new snapshots by running the code.
