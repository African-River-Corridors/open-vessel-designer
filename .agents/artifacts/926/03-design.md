---
ticket: 926
phase: design
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Design: tank design pressure from the Service Area

## Summary of the Request

Coastal Type C tanks default to their IGC 4.23.1.2 minimum. River-only keeps the profile value and gains a non-blocking reference row. The results say how the pressure was set.

## Current State

LNG defaults to 0.40 MPa on both Service Areas (`builtin_profile.json:346`). A coastal LNG tank of the example warns at 0.40 against Po 0.415 MPa. A river-only twin has no pressure check (its stack has no `igc-2g`).

## Desired End State

- Coastal, untyped: each tank takes Po. Typed higher: kept. Typed lower: the IGC row fails (`warn` now, `block` after #927).
- River-only: 0.40 MPa (LNG); an `ovd` 3.2.0 row "IGC 4.23.1.2 minimum, for reference" shows Po beside it and never blocks.
- Results: `tank.design_pressure_basis` = `typed`, `default` or `igc_4_23_1_2_minimum`.
- Changing the Service Area in a session moves a defaulted pressure with it.

## Scope

In: the profile default, the Python `simulate` default, the reference row, the basis field, evaluation cases.
Out: making the IGC row block (#927); solving pressure with insulation (#875); the Service Area as a required input (#925).

## Design Questions

### Q1. How does the default follow the Service Area?

- **A. Add `service_area` as a fourth table key (chosen).** `TABLE_KEYS` gains `service_area`; the pressure table gets a first row `when: { service_area: coastal-and-river, cargo_containment: type_c_tanks }` → `null`, and the existing rows serve river-only. The session's refill (`session.rs:1190-1208`) then moves a defaulted pressure when the Service Area changes, with no new code path. #875 (C6, a floor by Service Area) needs the same key.
- B. Use the existing `default_by_service_area` (`vdesign.rs:64-82`), which already refills on a change of Service Area (`session.rs:1174-1189`). It takes one key per input, so it cannot also follow the Cargo and carriage (LNG 0.40, pressurised LPG, refrigerated LPG `null`) on river-only. Making it two-key, or chaining it into the table, is a second mechanism doing the table's job.
- C. A code rule in `fill`: coastal + defaulted pressure → `null`. A default chosen outside the profile, which the session would not refill.
- D. Engine-side: on coastal, treat a defaulted 0.40 as "IGC minimum". The file then says 0.40 while the engine uses Po: the file lies.

A adds the key to the mechanism that already handles a multi-key default; the existing session code refills it on a change of Service Area once it is a table key. Cost of A: the other 10 tables change their `by` list (mechanical; their rows do not name `service_area`, so they match any). The loader accepts an old three-key `by` list as "any Service Area", so a user profile still loads.

### Q2. Where does the river-only reference row live?

In `ovd` 3.2.0 (minor: adds one Rule), `applies_when: { service_area: [river-only], cargo_containment: [type_c_tanks] }`, the same measure and limit as the IGC row, `consequence: note`, `origin: regulation` citing IGC 4.23.1.2 with a note citing ADN 9.3.1.23.1 (the authority or class decides; no number). `verified: false`. On coastal it is `not applicable`, so the IGC row is the only one there.

### Q3. How do the results show the basis?

A `design_pressure_basis` key in the tank block, from the flag and the input's source. Not a new `Source` variant: `Source` is file format (ADR 0007) and "IGC minimum" is a value rule, not a provenance. The app's input panel shows the basis from the profile hit, as it does for fully refrigerated LPG today.

### Q4. Python `simulate`

With #925 the Service Area is required there. A coastal call with no pressure key takes the IGC minimum; river-only keeps 0.40. This touches `simulate.rs:870-880`, shared with lane B1: message B1 before the PR.

## Proposed End State Architecture

```
profile table (service_area, containment, cargo, carriage) → tank_design_pressure_mpa (null on coastal)
  → params → TankDesign.design_pressure_igc_minimum → Po per tank → wall, mass
trace: coastal → igc/4.23.1.2 (warn/block); river-only → ovd reference row (note)
```

## What moves in design impact

Every coastal Type C Option on a defaulted pressure: design pressure (0.40 → Po, e.g. 0.415 MPa on the example), wall, tank mass, lightship, draught, cargo. River-only Options: no number moves; one new `note` row. Fully refrigerated LPG: no change.

## Risks

- Po depends on tank size, and the search sizes the tank: a larger tank gets a thicker wall. No loop: Po needs no wall (`tanks.rs:150`).
- #935 moves the same Options (damage screen). The two design-impact tables must be regenerated on whichever merges second.
- A fourth table key doubles the combinations the coverage check walks; it is a small enum, so the cost is small.

## Patterns to Follow

The fully refrigerated LPG `null` row (`builtin_profile.json:378-383`) for basis text; the IGC row for the reference row's limit; `evaluation/` boundary cases with an independent oracle.
