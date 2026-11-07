---
ticket: 992
phase: outline
date: 2026-11-07
status: draft (maintainer decisions folded in 2026-11-07)
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Outline: gas depth with larger tanks standing above the deck under a weather shield

Each phase is one PR, stacked on the one before. Failing tests first. This outline also covers the parts of #993 that survive (see design D1b); #993 is closed as superseded.

The input is `tank_top_above_deck_m`. Unset means "the Service Area default" only from phase 5. Until then, unset keeps today's 0.55 x beam depth, so nothing moves.

## Implementation Overview

- [ ] Phase 1: the input, and depth from the tank (opt-in, includes the 0 m hold)
- [ ] Phase 2: shield object, weight, windage, freeboard treatment
- [ ] Phase 3: sightline, air draft, dome trunks and hold weight lines
- [ ] Phase 4: damage and drawings
- [ ] Phase 5: Service Area defaults on (coastal 2.0 m with shield, river-only OD/2, ocean 0 m) and re-pins

## Phase 1: input and depth

Files: `crates/ovd-format/src/builtin_profile.json`, `profile.rs`, `vdesign.rs` (near lines 5176 and 6018), `crates/ovd-core/src/gas_arrangement.rs` (`fit`, `base_inputs`), Python and format input schemas.
- Fails today: `tank_top_above_deck_m = 0` is accepted; a negative value or one above the tank OD is an error that names the limit.
- Fails today: with the input set, depth = double bottom + support + OD + 0.3 - 0.3 - h (0 m: OD + DB + support + 0.3), floored by draught + freeboard + margin. Label: snapshot.
- Test: input unset, every example byte-equal.
- Test: the profile holds the default rules by Service Area (`metres` or `od_fraction`), read but not yet applied.
Result: depth is an output of `fit` when the input is set.

## Phase 2: shield, weight, windage, freeboard

Files: `objects.rs`, `vdesign.rs`, `loading_conditions.rs`, `models.rs`, freeboard code in `ovd-core`.
- Fails today: `weather_shield` is accepted and echoed in results; ignored at 0 m with a note.
- Fails today: shield weight appears in lightship; windage grows by the projected area.
- Fails today: a non-watertight shield leaves freeboard unchanged and gives no hull-girder credit. A note says a trunk is not modelled.
Result: shield-on moves; shield-off does not.

## Phase 3: sightline, air draft, domes, hold weights

Files: `wheelhouse.rs`, `ga_adapter.rs`, `vessel_model.rs`, `barge_capacity.rs`.
- Fails today: obstruction = depth + highest of dome, tank top, shield (#989 limit, #990 eye height).
- Fails today: air draft = the max of dome, shield, tank top; a note names which sets it.
- Fails today: at 0 m the hold weights (flat deck, hatch covers, dome coamings, cradles) appear in lightship with named sources, without double counting bulkhead steel.
Result: rows in the trace.

## Phase 4: damage and drawings

Files: `damage_cases.rs`, `simulate.rs`, `compartments.rs`, `gas_damage.rs` (after #935), drawing code in `ga_adapter.rs`.
- Fails today: shield-on, non-watertight adds no buoyancy; shield on and off give the same intact-buoyancy volume.
- Fails today: at 0 m a side breach across the bulkhead between tank rows floods both holds; the governing case is named.
- Fails today: the GA profile shows the shield outline (dashed, "weather shield, not watertight"), the deck, domes and the air-draft dimension.
Result: damage and drawings follow the input.

## Phase 5: defaults and re-pins

Files: `builtin_profile.json`, `vdesign.rs`, `catalogue/depth_rule.md` via `tools/gen_depth_rule.py`.
- Fails today: an Option with no depth and no height on the 140 x 29 m coastal LNG reference design gets 2.0 m and a shield; depth is below 15.95 m and the cause is stated.
- Test: river-only gets OD/2, no shield; ocean gets 0 m.
- Test: ratio outside 0.49 to 0.56 gives a note only.
Result: re-pins, each with its cause: hashes of gas examples, GA snapshots, the depth-rule catalogue, the app snapshots. `app-gate.yml` run if results move.

## Validation per PR

`cargo fmt --all --check`; clippy `-D warnings` on `ovd-core`, `ovd-format`; touched crates' tests with `--no-fail-fast`; `python3 tools/release_guard.py` and `--history`; `tools/check_fixture_hashes.py`; `drawing-crosscheck` green; `bash tools/gates.sh` for phases 4 and 5. Run cargo through `~/Dev/_cargo-target/ovd-x-run 992`.

## Open Questions

1. Order: phase 1 ships depth-from-tank opt-in first, so the default flip (phase 5) is last and the only large re-pin.
2. Does phase 4 wait for #935 so the search screens damage?
3. Ocean default (0 m) is a proposal.
