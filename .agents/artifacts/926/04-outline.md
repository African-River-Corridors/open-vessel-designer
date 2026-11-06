---
ticket: 926
phase: outline
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Outline: tank design pressure from the Service Area

Starts after #925 merges (the Service Area is then always known). First the profile can key on the Service Area with no value change; then the coastal default moves; then the river-only reference row; then the Python path.

## Implementation Overview

- [ ] PR 1: `service_area` is a table key; no default changes
- [ ] PR 2: coastal Type C defaults to the IGC 4.23.1.2 minimum (the design change)
- [ ] PR 3: `ovd` 3.2.0 reference row on river-only
- [ ] PR 4: Python `simulate` follows the Service Area (after messaging lane B1)

## PR 1: a fourth table key

`vdesign.rs`: `TABLE_KEYS` adds `service_area`; `table_combinations` walks both values. `profile.rs`: accept a three-key `by` as "any Service Area". `builtin_profile.json`: the 11 `by` lists gain `service_area`; no row changes.

- Test that fails today (`profile.rs` unit, `Label: property`): a profile table row with `when: { service_area: ["coastal-and-river"] }` loads and `table_default` picks it only for that context. Today the loader rejects the key.
- Test: a profile with a three-key `by` still loads (`Label: property`).
- Hashes unchanged (`tools/check_fixture_hashes.py`).

## PR 2: coastal default is the IGC minimum

`builtin_profile.json`: a first row for `tank_design_pressure_mpa`, `when: { service_area: coastal-and-river, cargo_containment: type_c_tanks }`, `value: null`, basis citing IGC 4.23.1.2. Results: `drawing.tank.design_pressure_basis`.

- **Test that fails today** (`crates/ovd-format/tests/design_pressure.rs`): the 140 × 29 m example as a new coastal LNG Option, pressure untyped; tank design pressure = 0.4149 MPa within 1e-4 (`Label: sourced - IGC 4.23.1.2`, the hand-worked value in the research). Today 0.40.
- Test: typed 0.50 MPa is kept, basis `typed`; typed 0.35 MPa gives `igc/4.23.1.2/min-design-pressure` = `warn` (`fail` after #927).
- Test (`Label: property`): two tanks of the same hull, the larger has the higher Po.
- Test (session): a coastal Option switched to river-only refills the pressure to 0.40; back to coastal gives `null`.
- Design impact: `python3 tools/design_impact.py changelog`; regenerate fixtures, website examples and `examples/` (`tools/refresh_examples.py`); the fragment lists pressure, wall, mass, lightship, draught, cargo for each moved coastal Option.

## PR 3: `ovd` 3.2.0 reference row

New `rules/ovd/3.2.0.json` = 3.1.0 plus `ovd/tank/igc-min-design-pressure-reference` (id to be confirmed), as design Q2. Register it.

- Test that fails today: the river-only LNG twin of the example has the row, value 0.40, limit 0.4149, outcome `warn`, and the run's verdict is not blocked by it.
- Test: on coastal the row is `not applicable`.
- Evaluation: an `ovd` reference case at Po − 0.001 (warn) and Po + 0.001 (pass), Po from an independent oracle of the formula. `python3 evaluation/run.py --check`.
- `ovd rules --provenance` passes (minor: adds a Rule).

## PR 4: Python `simulate`

`simulate.rs:870-880`: a missing key on a coastal Type C hull sets `design_pressure_igc_minimum`. Message lane B1 first; this block is theirs.

- Test that fails today (`tests/test_simulate*.py`): `simulate` of the example's params with `service_area="coastal-and-river"` and no pressure key gives 0.4149 MPa; river-only gives 0.40.

## Validation per PR

`cargo fmt --all --check`; clippy `-D warnings` on `ovd-core`, `ovd-format`; touched crates' tests `--no-fail-fast`; touched pytest modules; `python3 tools/release_guard.py` and `--history`; `tools/check_fixture_hashes.py`; `bash tools/gates.sh` and `ovd rules --provenance` on PRs 2 and 3.

## Open Questions

1. The reference row's id: `ovd/tank/igc-min-design-pressure-reference`, or a name the maintainer prefers?
2. Should a coastal Python `simulate` call with no pressure key take Po (design Q4), or stay at 0.40 for API stability?
3. Order with #935: both move coastal gas Options. Which merges first?
