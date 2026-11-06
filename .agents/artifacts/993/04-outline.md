---
ticket: 993
phase: outline
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Outline: arrangement style "tanks in hold, flat deck"

Each phase is one PR to `main`. Failing test first. `tank_stowage` defaults to `deck`, so existing results stay byte-equal.

## Implementation Overview

- [ ] Phase 1: the `tank_stowage` input and the standing height
- [ ] Phase 2: depth follows the tank OD
- [ ] Phase 3: sightline and air draft
- [ ] Phase 4: hold, deck and trunk in damage
- [ ] Phase 5: weight and lightship
- [ ] Phase 6: drawings
- [ ] Phase 7: fixtures and docs

## Phase 1: the input

Files: `crates/ovd-core/src/gas_arrangement.rs`, `aft_arrangement.rs`, the Python and format input schemas.
- Test (fails today): `tank_stowage = "hold"` is accepted; an unknown key is an error naming the valid keys.
- Result: key parsed; `deck` unchanged; fixture hashes equal.

## Phase 2: depth

Files: `gas_arrangement.rs` (`fit`, `base_inputs`).
- Test: on the 140 × 29 m coastal LNG reference design with `hold`, depth = OD + double bottom + 0.3 m (`Label: snapshot`, printed by the code).
- Test: `deck` depth unchanged.
- Result: depth is an output of `fit` under `hold`.

## Phase 3: sightline and air draft

Files: `aft_arrangement.rs` (`sightline_obstruction_m`), the air-draft calc.
- Test: obstruction = depth + dome height; air draft under `hold` is below `deck` on the same tank.
- Result: the fixed deckhouse sees over the domes only.

## Phase 4: damage

Files: `compartments.rs`, `damage_cases.rs`, `gas_damage.rs` (after #935).
- Test: the side breach across the hold bulkhead floods both holds; no buoyancy credit from the deck above the closed hold.
- Result: the governing case is named in the output.

## Phase 5: weight

Files: `barge_capacity.rs`, cost and weight lines.
- Test: lightship rises by the named lines; no double count of bulkhead steel.

## Phase 6: drawings

Files: the profile, section and plan generators.
- Test: the SVG holds the hold bulkhead, the dome tops and the air-draft dimension.

## Phase 7: fixtures and docs

- New fixtures for `hold` only. `tools/check_fixture_hashes.py` shows no existing fixture moved. Docs and changelog name the style and its #992 link.
