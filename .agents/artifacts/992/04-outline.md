---
ticket: 992
phase: outline
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Outline: gas depth with larger tanks, less depth and a weather shield

Each phase is one PR to `main`. Failing tests first.

## Implementation Overview

- [ ] Phase 1: shield input and object, no effect on results when off
- [ ] Phase 2: weight, windage, freeboard treatment
- [ ] Phase 3: sightline and air draft checks
- [ ] Phase 4: new depth default (freeboard and structure)
- [ ] Phase 5: damage checks and the drawing

## Phase 1: input and object

Files: `crates/ovd-format/src/builtin_profile.json`, `profile.rs`, `objects.rs`, `vdesign.rs`.
- Test that fails today: an Option with `weather_shield` is accepted and echoed in results.
- Test: shield off leaves every example byte-equal.
Result: the input exists; nothing moves.

## Phase 2: weight, windage, freeboard

Files: `crates/ovd-core/src/loading_conditions.rs`, `models.rs`, freeboard code in `ovd-core`.
- Fails today: shield weight appears in lightship; windage area grows by the projected area.
- Fails today: a non-watertight shield leaves freeboard unchanged.
Result: shield-on examples move; shield-off do not.

## Phase 3: sightline and air draft

Files: `wheelhouse.rs`, `ga_adapter.rs`, `vessel_model.rs`.
- Fails today: shield height raises the sightline obstruction (#989 limit, #990 eye height).
- Fails today: air draft reports the highest of dome, shield and tank.
Result: two new rows in the trace.

## Phase 4: depth default

Files: `builtin_profile.json`, `vdesign.rs` (lines near 5176 and 6018), `catalogue/depth_rule.md` via `tools/gen_depth_rule.py`, `gas_arrangement.rs` (tank above deck amount).
- Fails today: the 140 x 29 m coastal LNG reference design depth is below 15.95 m and its tank OD is not smaller.
- Test: ratio outside the reference band gives a note.
Result: gas depth follows freeboard and structure.

## Phase 5: damage and drawing

Files: `damage_cases.rs`, `simulate.rs`, drawing code in `ga_adapter.rs`.
- Fails today: shield-on, non-watertight gives no added buoyancy in damage.
- Fails today: GA profile shows the shield outline.
Result: re-pins listed in the design (hashes, snapshots, catalogue).

## Validation per PR

`cargo fmt --all --check`; clippy `-D warnings` on `ovd-core`, `ovd-format`; touched crates' tests with `--no-fail-fast`; `python3 tools/release_guard.py` and `--history`; `tools/check_fixture_hashes.py`; `bash tools/gates.sh` for phases 4 and 5.

## Open Questions

1. Order: should the depth default (phase 4) ship before the shield, with tanks held at deck level?
2. Does phase 5 wait for #935 so the search screens damage?
