---
ticket: 913
phase: design
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Design: igc-2g 4.0.0, IGC 3.2.4.1 and 3.2.4.2 on placed openings

## Summary of the Request

Replace the four typed-input rows of IGC 3.2.4.1 with rows on #912's placed openings, add 3.2.4.2, use L of ICLL, ship as `igc-2g` 4.0.0, and prove the boundaries in the evaluation harness.

## Current State

Four rows read typed numbers against a clamp on the length overall (`igc-2g/3.3.0.json:1223-1370`). A design passes on a number nobody drew.

## Desired End State

- On a deckhouse stern, one row per opening per Rule; the worst opening governs.
- Rows read placed geometry and the ICLL length.
- A hull with no placed openings gets `not evaluated`, never `pass`.

## Scope

In: `igc-2g` 4.0.0; object-scoped inputs for openings; one new Bool Measure; evaluation cases; a note on the #898 gaps.
Out: fire class of windows (3.2.5); airlock and conning arc (they stay gaps, research §6); drawing the openings.

## Design Questions

### Q1. How does a Rule pick the openings it governs?

- **A. Object-scoped Applicability inputs (chosen).** Declare `opening_kind`, `opening_space`, `opening_face`, `opening_openable`, `opening_gas_tight` as object inputs, as `loading_condition_kind` is (`rule_set.rs:198`, `measured.rs:244`). The clause's scope (which spaces, which kinds, the wheelhouse exception) then lives in the Rule file, where a reviewer reads it beside the citation.
- B. Composite Measures in code (`opening.igc_3241_ok`). Fewer moving parts, but the clause logic hides in Rust and a Rule mutation cannot reach it.

### Q2. The rows

All `verified: false`, `since: 4.0.0`, limit `clamp(x = deckhouse.icll_length, b 0.04, lo 3, hi 5)` where a limit is used. Spaces S = accommodation, service, machinery, control_station.

| Rule id | Applies to openings with | Measure | Test |
|---|---|---|---|
| `igc/3.2.4.1/no-opening-faces-cargo` | kind ∈ {entrance, air_inlet, air_intake}, space ∈ S ∪ {wheelhouse} | `opening.clear_of_cargo_face` (new, = not facing) | check |
| `igc/3.2.4.1/side-opening-setback-placed` | kind = entrance, space ∈ S, face = side | `opening.distance_from_front` | ≥ clamp |
| `igc/3.2.4.1/air-inlet-setback-placed` | kind ∈ {air_inlet, air_intake}, face = side | same | ≥ clamp |
| `igc/3.2.4.2/windows-fixed` | kind = window, space ∈ S, openable = yes | same (front gives 0, so fails) | ≥ clamp |
| `igc/3.2.4.2/wheelhouse-gas-tight` | space = wheelhouse, kind ∈ {entrance, window}, face = side, openable = yes, gas_tight = no | same | ≥ clamp |

The wheelhouse is a control station (IGC 1.2.17), so a wheelhouse door facing the cargo area fails row 1; 3.2.4.2 relaxes only the side zone. This is the stricter reading (open question 1).

Aft-face openings are never filtered in: IGC 3.2.4.1 needs no distance on the far end bulkhead.

### Q3. One source for the zone

The rows compute the limit from the file. #912's `in_zone` and `deckhouse.opening_zone` stay as displayed values, not tested by a Rule, so the clamp lives in one tested place. A unit test asserts they agree.

### Q4. What happens to the old rows?

- **A. Drop all four (chosen, as the ticket says).** Major version. A coastal container-block Option then has `not evaluated` 3.2.4 rows: true, since nothing is placed.
- B. Keep them with Applicability `arrangement_style = container_block`. Old coastal files keep a (typed-number) check. Also major. Offered as open question 2.

`openings.front_clear` stays in the registry: `lr-inland-gas` does not read it, but older `igc-2g` versions do.

### Q5. Stack

Register `rules/igc-2g/4.0.0.json`; the coastal gas stack takes the highest built-in version, so no profile edit. River-only stacks do not carry `igc-2g`.

## Proposed End State Architecture

```
#912 Placement ──► Measured.openings ──► object inputs (kind, space, face, openable, gas_tight)
                                    └──► Measures (distance_from_front, clear_of_cargo_face, icll_length)
igc-2g 4.0.0 rows ──► one row per opening ──► governing = worst
```

## What moves in design impact

Verdicts only; no hull length moves (spec 0011 D17). Coastal deckhouse Options: four rows replaced by five per-opening rows; with the default layout all pass. Coastal container-block Options: four `pass` rows become `not evaluated`. The fragment lists both.

## Risks

- #912 is uncompiled; its Measure names may change before merge. This ticket starts only after #912 merges.
- L of ICLL without the rudder-stock alternative can be a little short; the zone is then at most 0.04 × the shortfall too small near 3 m and 5 m. The Measure's method says so.
- An object input on `bool` fields (`openable`, `gas_tight`) needs a value list `yes`/`no`; the kernel compares strings.

## Patterns to Follow

`loading_condition_kind` for object inputs; existing clamp rows for limits; `evaluation/oracle/stern_oracle.py` for an oracle that works verdicts from the text.
