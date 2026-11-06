---
ticket: 919
phase: outline
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Outline: insulation as explicit inputs

Plumbing first with today's values (no design moves), then the single-thickness clean-up, then the results, then `simulate`, then the density move as its own design change. No Rule Set changes.

## Implementation Overview

- [ ] PR 1: conductivity and density are inputs; heat leak and mass read them; numbers unchanged
- [ ] PR 2: one thickness: no 350 mm fallback on a Design path
- [ ] PR 3: U-value and heat leak per tank in the results
- [ ] PR 4: `simulate` and the Python API take the same inputs (after #930, or after messaging lane B1)
- [ ] PR 5: density default 96 kg/m³ for panel and PU foam (the design change, if approved)

## PR 1: two inputs, same numbers

`vdesign.rs` (INPUTS), `builtin_profile.json` (two `default_by_table` entries, `estimate`, #918's sources), `TankDesign` and `BargeInputs` fields, `tanks.rs:306` reads the input density, `ovd-format/src/gas_fuel.rs:109` and `holding_time.rs:126` read k from the capacity result. Defaults: k 0.026, ρ 60.

- Test that fails today (`crates/ovd-format/tests/insulation_inputs.rs`, `Label: property`): the example with `tank_insulation_conductivity_w_per_mk` typed 0.052 gives twice the heat leak; with density typed 120 the insulation mass doubles. Today the keys are unknown inputs.
- Test (`Label: sourced - Lee et al., J. Mar. Sci. Eng. 2021, 9, 36, Table 3; hand check in #918`): the example's heat leak per tank is 26.54 kW within 0.01 and the boil-off 746.85 kg/h.
- Test: a typed type other than the default gives the warning of design Q1.
- `tools/check_fixture_hashes.py` unchanged (same values).

## PR 2: one thickness

`ga_adapter.rs:130,238` read the input; `models.rs:222` default 0 with a comment; rename the constant to `LEGACY_SIMULATE_INSULATION_MM`, used only by `simulate`; fix `gas_arrangement.rs:1264`.

- **Test that fails today** (`Label: property`): a Design with thickness 123 mm; assert 123 in the steel diameter (OD − 2 × 0.123 − 2 × wall), the GA tank drawing, the capacity result, the heat leak (through the radii) and the insulation mass. Any 350 mm on a path fails it. Build a result with `insulation_thickness_mm: None` through `ga_adapter` to reach the fallback that today gives 350.
- `git grep "unwrap_or(350.0)"` finds nothing (a check in the PR body).

## PR 3: results

`boil_off` gains `u_value_w_per_m2k`, `heat_leak_per_tank_kw`, `boil_off_pct_per_day`; holding time prints the k it used from the result.

- Test that fails today (`Label: sourced`, hand values from #918): the example's `u_value_w_per_m2k` = 0.0759 within 1e-4 and `heat_leak_per_tank_kw` = 26.54.
- Unhashed or hashed: state in the PR; if hashed, regenerate examples and fixtures.

## PR 4: `simulate`

Message lane B1 or wait for #930. `simulate.rs:883-887`: read the two keys; a key left out keeps 0.026 and 60 (original engine).

- Test that fails today (`tests/test_simulate*.py`): `simulate` with `tank_insulation_conductivity_w_per_mk=0.052` doubles `boil_off.heat_leak_kw`.

## PR 5: density 96 kg/m³ (design change)

Profile rows for `panel` and `pu_foam` cargo rows to 96, `source` the conductivity's basis.

- Test that fails today (`Label: snapshot`, printed by the code): the example's insulation mass ≈ 214 t (#918: +80 t).
- `python3 tools/design_impact.py changelog`; `tools/refresh_examples.py`; fragment lists insulation mass, lightship, draught, cargo, GT per moved Option.

## Validation per PR

`cargo fmt --all --check`; clippy `-D warnings` on touched crates; their tests `--no-fail-fast`; touched pytest modules; `python3 tools/release_guard.py` and `--history`; `tools/check_fixture_hashes.py`; `bash tools/gates.sh` on PRs 1 and 5; `ovd-core` builds `no_std`.

## Open Questions

1. Python `simulate` with no thickness key: keep 350 mm (the original-engine convention) or take the profile default?
2. Density for panel and PU foam: move to 96 kg/m³ (PR 5, +80 t on the example) or keep 60 until maker data?
3. Ellipsoid head area in the heat leak (#918: about +45 % on 2:1 heads): here, or in #920?
4. Add PIR and vacuum perlite as types now (then key the tables on type, design option B), or later?
