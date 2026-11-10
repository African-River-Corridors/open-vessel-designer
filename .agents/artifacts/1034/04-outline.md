---
ticket: 1034
phase: outline
date: 2026-11-08
status: approved (the maintainer's gate, 2026-11-09)
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a85c10002f3e7e752f587d5ac0e745834532a76f
---

# Outline: river draught and air-draft limits, and the cost of the damage search

Each phase is one PR, stacked on the one before. Failing tests first. The outline follows the recommended answers in `03-design.md`; the gate (2026-11-09) answered Q3 with no "no limit", so the tests below use numbers, never a typed "no limit". See "Gate outcome" in `03-design.md`.

## Implementation Overview

- [ ] Phase 1: the probe hands its share answer to the run pass (D4a). No result moves.
- [ ] Phase 2: the two limits are a required answer for river-only Options (D1, D1b). No saved file moves.
- [ ] Phase 3: the climb share rule (D2). River-only answers can move; each is named.
- [ ] Follow-up issues opened (D4b, D4c, D5), no code.

## Phase 1: one share search per run

Files: `crates/ovd-format/src/simulate.rs` (`simulate_once`, the probe), `crates/ovd-core/src/gas_arrangement.rs` (`probe_depth`, `fit_with`, `share_stage_one`).
- Test: every fixture, example and website design gives byte-equal results and hashes (`tools/check_fixture_hashes.py`).
- Fails today: on the 100 x 20 m river-only LNG barge, `share_stage_one` runs once per run, not twice (count the calls through a test seam). Label: rule.
- PR body: CPU seconds before and after on the 100 x 20 m and 110 x 11.4 m river designs and the 140 x 29 m coastal LNG reference design, three runs each, release build.
Result: the river share search costs about half.

## Phase 2: required river limits

Files: `crates/ovd-format/src/builtin_profile.json` (basis text of `draught_limit_m`, `air_draft_limit_m`, `bridge_clearance_m`), `vdesign.rs` (the required-answer rule, the legacy warning, the `Missing` paths), `class_input.rs` (a class fill counts as an answer), `api.rs` and `src/ovd/simulate.py` (the `TypeError`), `src/ovd/calibration.py` (a number for the air-draft limit), `crates/ovd-cli/src/main.rs` (`--draught-limit`, `--air-draft-limit` on `new` and `fill`), `app/src/lib/InputField.svelte` and `LeftPanel.svelte` (no value, no "none" choice), the locale files for the new messages, `docs/python-api.md`, `docs/basis/08-waterway-and-route.md`, a `changes/` note.
- Fails today: `fill --as-new` of a river-only Option with neither limit gives one error naming both inputs and the three fixes; the same text from `run`, `validate`, the app session and Python `simulate`.
- Fails today: `--draught-limit 3.5 --air-draft-limit 9.1` writes both typed; the Option runs. `none` is refused: there is no "no limit".
- Fails today: a typed `null` for either limit, and `None` in Python `simulate`, is not an answer: one message (`TypeError` in Python).
- Fails today: a Vessel Class pick satisfies both (CEMT VIa fills 4.5 m and 7.0 m, marked `class`).
- Fails today: a saved river-only file storing both as default `null` runs with the same results hash as on `main`, plus one validation warning (not in the results).
- Fails today: `air_draft_limit_m` above a typed `bridge_clearance_m` gives a validation warning.
- Test: a coastal-and-river Option with no limits runs as today, no new message.
- Test edits, not re-pins: the river-only tests that build Options from typed inputs get numbers (the limits their case needs, high enough not to bind). Their values do not move; a value that would move is named in the PR body.
Result: no new river-only Option runs without an answer for both limits.

## Phase 3: the climb share rule

Files: `crates/ovd-core/src/gas_arrangement.rs` (`fit_with`, `share_stage_one`, `share_better`; a public `SHARE_CLIMB_MIN_GAIN` of 1 % and the step rule `climb_continues(from_t, to_t)`), `crates/ovd-format/src/simulate.rs` (`arrangement.share_climb_from`, only when the climb moved the share), `run.rs` (`summary_lines`: the rule, and the share the climb left with its cargo), `builtin_profile.json` (the `tank_top_above_deck_od_fraction` basis), a `changes/` note.

Re-measured 2026-11-10 on the Phase 2 branch (PR #1055, `206ba491`, release build; each share typed as a limit, `ovd run --json`). No figure moved against the design note (D2) or research section 5. Phase 2 made both limits required, so "high limits that do not bind" are now numbers: 3.5 m draught and 40 m air draft, what the Phase 2 tests use. With them the 100 x 20 m barge gives the same answer as the old "no limits" run (0.375, 2,350.73 t).

| Case | Share | Tanks | Depth (m) | Cargo (t) | Base today | Climb |
|---|---|---|---|---|---|---|
| 100 x 20 m river LNG, 4.5 / 9.1 m | 0.5 | 2 x 2 x 6.1 m | 4.80 | 1,328.13 | **picks** | start |
| | 0.375 | 2 x 2 x 6.6 m | 5.22 | 1,558.95 | | **picks** (+17 %) |
| | 0.25 | 2 x 2 x 6.6 m | 6.05 | 1,526.83 | | falls: stop |
| 100 x 20 m, 3.5 / 40 m | 0.5 | fails intact | | | | |
| | 0.375 | 2 x 2 x 8.1 m | 6.16 | 2,350.73 | **picks** | **picks** |
| | 0.25 | 2 x 2 x 8.1 m | 7.18 | 2,345.42 | | falls: stop |
| | 0.125 | 1 x 2 x 13.0 m | 12.48 | 3,054.99 | | never reached (air draft 15.6 m) |
| 110 x 11.4 m river LNG, 3.5 / 40 m | 0.5 | 1 x 2 x 7.6 m | 4.90 | 1,163.05 | **picks** | start |
| | 0.375 | 1 x 2 x 8.6 m | 6.48 | 1,461.09 | | **picks** (+26 %) |
| | 0.25, 0.125 | 1 x 2 x 8.6 m | 7.55, 8.63 | 1,461.09 | | gain 0 %: stop |
| 110 x 11.4 m, 4.5 / 9.1 m | 0.5 | 1 x 2 x 6.6 m | 4.80 | 861.97 | **picks** | **picks** |
| | 0.375 | 1 x 2 x 6.6 m | 5.22 | 853.70 | | falls: stop |

The done-tests are committed on the branch (`ticket/1034-p3-climb`), each headed `// DONE-TEST #1034 P3 — do not edit in the build`:
- Fails today (`crates/ovd-format/tests/share_climb.rs`): the 100 x 20 m barge at 4.5 / 9.1 m picks share 0.375, 2 x 2 x 6.6 m, 5.22 m deep, 1,558.95 t (today: 0.5, 1,328.13 t), and the summary line reads "the most cargo within the limits" and "0.500 carries 1328.1 t". Label: snapshot.
- Fails today (same file): the 110 x 11.4 m tanker at 3.5 / 40 m picks 0.375, 1,461.09 t (today: 0.5, 1,163.05 t); the lower shares carry the same cargo on a deeper hull, a gain under 1 %. Label: rule, on a snapshot.
- Guard, passes today but for the summary line (same file): the 100 x 20 m barge at 3.5 / 40 m stays at 0.375, 2,350.73 t; the climb stops when the cargo falls at 0.25 and never reaches the 12.5 m hull at 0.125. "Most cargo over every share" would take that hull. Label: snapshot.
- Fails today, does not compile (`crates/ovd-core/tests/share_climb.rs`): `SHARE_CLIMB_MIN_GAIN == 0.01`; `climb_continues` goes on at a gain over 1 % and stops at a smaller gain, a tie or a fall. Label: property.
- Test (`crates/ovd-format/tests/damage_budget.rs`, extended): the coastal-and-river 100 x 20 m barge with a typed share of 0.5 and two along keeps one damage budget of 24 runs over every share the climb visits, and one count that never falls.
- Mutation proof: with the climb step forced to 0 (tallest that passes), the first test fails; with `climb_continues` always true (most cargo over every share), the third test fails.
- Re-pins: river-only designs whose answer moves, every one named through `tools/design_impact.py changelog`; the share-searched fixtures and examples through `tools/check_fixture_hashes.py` (`gas_adn_river_two_wide`, `gas_deckhouse_self_propelled`, `gas_igc_2g_exact`, `gas_igc_2g_self_propelled`, the 140 x 29 m sensitivities example). No Rule Set data changes.
- PR body: the timing table of design D3 re-measured on the merged tree (CPU seconds; the base figures of 2026-11-10 are in the build brief).
Result: river-only designs carry the most cargo within the limits.

## Follow-up issues (no code here)

- D4b: `ovd run` reruns an Option without damage when its stored hash is at the unscreened stage (adds 4 to 7 s on a coastal run).
- D4c: infeasible designs run 15 to 56 CPU s; bound the stability stage of a share that cannot beat the closest answer.
- D5: let a river-only run choose 2, 3 or 4 tanks along on the intact checks.

## Landing order

Build after #1033 (default depth), which is in build now and may move the depths these tables use. Phase 1 has no dependency and may go first. Whoever lands second merges `main`, runs `tools/check_fixture_hashes.py` and `tools/design_impact.py changelog`, and re-measures.
