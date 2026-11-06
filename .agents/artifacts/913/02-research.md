---
ticket: 913
phase: research
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Research: igc-2g 4.0.0, IGC 3.2.4.1 and 3.2.4.2 on placed openings

## Summary

Today four `igc/3.2.4.1/*` rows read typed numbers: a front flag and three distances, each against a clamp of 4 % of the length overall, 3 to 5 m. #912 adds an `opening` object type with per-opening Measures (distance from the front, faces the cargo area, in the zone, openable, gas-tight, kind, space, face) and two Option Measures (`deckhouse.icll_length`, `deckhouse.opening_zone`).

The kernel already supports what the new rows need: one row per object with a governing (worst) row, formula operands that read an Option Measure from an object Rule, and object-scoped Applicability inputs (today only `loading_condition_kind`). What is missing is the object-scoped inputs for openings, and a Bool Measure whose `true` means "not facing the cargo area". The default stack takes the highest built-in version, so registering 4.0.0 points the coastal stack at it.

## Detailed Findings

### 1. Today's rows

| Rule (3.3.0) | Measure | Limit | Line |
|---|---|---|---|
| `igc/3.2.4.1/front-openings` | `openings.front_clear` (from the typed `deckhouse_front_opening`) | check | `igc-2g/3.3.0.json:1223` |
| `igc/3.2.4.1/side-opening-distance` | `input.deckhouse_side_opening_distance_m` | clamp(0.04 × LOA, 3, 5) | `:1251` |
| `igc/3.2.4.1/opening-setback` | `input.entrance_distance_m` | same | `:1290` |
| `igc/3.2.4.1/air-inlet-setback` | `input.engine_room_vent_inlet_distance_m` | same | `:1329` |

- The limit uses `input.length_overall_m`, not L of IGC 1.2.31 (ICLL length).
- Defaults: `builtin_profile.json:1134,1139,1153,1167` (all "the engine does not place openings").
- `lr-inland-gas/2.0.0.json:216,249,282` read `input.entrance_distance_m` and `input.engine_room_vent_inlet_distance_m` too, so the inputs stay.
- `openings.front_clear` is defined at `rule_trace.rs:603` and fed from the input at `ovd-format/src/trace.rs:739-742`.

### 2. What #912 adds (branch, commit `1fcb2a4e`)

- `ovd-core/src/deckhouse_openings.rs`: `Opening { id, kind, space, face, x, y, z, openable, gas_tight, tier }`; `Placed { distance_from_front_m, faces_cargo_area, in_zone }`; `zone_length_m` (clamp 0.04 L, 3, 5); `default_layout`: tier entrances, machinery entrance, inlets on the aft face; wheelhouse doors and openable windows on the sides, marked gas-tight; a fixed wheelhouse front window.
- Kinds: `entrance`, `window`, `air_inlet`, `air_intake`. Spaces: `accommodation`, `service`, `machinery`, `control_station`, `wheelhouse`. Faces: `front`, `side`, `aft`.
- `measured.rs`: `Measured.openings: Option<Placement>`; `has` is true for `opening.*`, `deckhouse.*` only when openings exist, so a hull with no deckhouse gets `not evaluated` rows.
- `rule_trace.rs`: `ObjectType::Opening` ("opening") and the Measures listed above, all `Unverified`.
- `frame_of` works L as 0.96 × the waterline length at 85 % of the depth (ICLL Reg. 3(1)(a)), without the rudder-stock alternative.

### 3. Object-scoped inputs

- Kernel: an Applicability condition on an object input is read per object; an object that does not meet it gets `not applicable` (`docs/specs/0004-rules-engine.md:341-344`; `arc-rules/src/model.rs:148-155`).
- OVD declares one: `loading_condition_kind` (`ovd-format/src/rule_set.rs:198-205`), read by `Measured::object_input` (`ovd-core/src/measured.rs:244-249`).
- Design inputs come from `trace.rs:442` (`applicability_values`); `arrangement_style` is a Choice input, so a Rule can apply to one style.

### 4. Operands and governing rows

- Formula operands may be an Option Measure on an object Rule: the loader accepts a Measure whose object is the design object (`arc-rules/src/load.rs:1011-1045`). So `clamp(x = deckhouse.icll_length)` works on an `opening` Rule.
- The evaluator returns one row per Rule per object and marks the governing row of each Requirement (`arc-rules/src/eval.rs:6`, `mark_governing` `:692`). That is "the worst opening is the row value".
- A `check` Rule passes on `true` (`model.rs:67`). #912 gives `faces_cargo_area` (true is the failure), so a negated Measure is needed for a check.

### 5. Versions and stacks

- Removing a Rule, or changing a result-changing field, is a major version (`0004-rules-engine.md:319-323`). Only `igc-2g` 1.0.0, 1.1.0 and 2.0.0 are released (`RELEASED.json`).
- The default stack takes "each set at its highest built-in version" (`builtin_profile.json`, `rule_sets.basis`, coastal gas stack at `:256`). Registering `rules/igc-2g/4.0.0.json` in `rule_set.rs:110-139` is enough.

### 6. The #898 gaps

- `igc/3.6.1/airlock` and the two spacing rows (`3.3.0.json:1493-1585`): gap "the model places no doors and no airlock". #912 places doors but no airlock, so they stay gaps.
- `solas/V.22.1.3/conning-arc` (`solas-v22/1.1.0.json:122`): needs window arcs from the conning position; #912 places window centres only. It stays a gap.

### 7. Rule text (library `igc/MSC.370-93.txt`)

- 3.2.4.1 (`:1494-1499`): openings to accommodation, service, machinery spaces and control stations shall not face the cargo area; on the far end bulkhead, or on the sides at ≥ 4 % of L, ≥ 3 m, need not exceed 5 m.
- 3.2.4.2 (`:1501-1505`): windows facing the cargo area or in that side zone fixed; wheelhouse windows may open and wheelhouse doors may be within the limits if the wheelhouse can be made gas-tight rapidly.
- 1.2.17 (`:446`): the wheelhouse ("main navigating equipment") is a control station. 1.2.31 (`:552`): L is the ICLL length.
- 3.2.5 (`:1514`): fire class of those windows; not in this ticket.

### 8. Evaluation harness

`evaluation/README.md`: cases by kind (`box`, `stern`, liquid, tank), each with an independent oracle under `oracle/`, and boundary mutants. No openings kind exists.

## Code References

Key files, not exhaustive: `rules/igc-2g/3.3.0.json:1223-1370`, `rule_set.rs:110-205`, `trace.rs:442,739`, `measured.rs:244`, `rule_trace.rs:603`, `arc-rules/src/{model,load,eval}.rs`, #912's `deckhouse_openings.rs`, `evaluation/`.

## Architecture Documentation

Rules are data; Measures are code. A Rule reads one Measure on one object type and may filter objects through object-scoped inputs. Limits are shapes with coefficients in the file. The trace gives one row per object and marks the worst.

## Open Questions

1. 3.2.4.2 lets wheelhouse doors stand "within the above limits". Does that also allow a wheelhouse door on the front (facing the cargo area), or only on the sides in the zone?
2. A coastal gas Option with a container-block stern has no placed openings: its rows become `not evaluated`. Is that acceptable, or should the old input rows stay for that style?
