---
ticket: 935
phase: design
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Design: damage survival inside the arrangement search

## Summary of the Request

Screen candidates by damage in the costly stage, refine the diameter in 0.1 m steps, reuse the winner's damage result, and say "not screened" when damage is skipped.

## Current State

The search's trace has no damage cases, so `igc/2.7*` rows are `not evaluated` (`measured.rs:481`). Damage runs once afterwards (`simulate.rs:3249`). The 140 × 29 m deckhouse example gets 13.28 m and 103 failing damage rows.

## Desired End State

- `fit` returns the largest tank whose trace has no failing blocking row, damage rows included.
- Each rejected candidate in `fit_tried` names the Rule id, the condition and case, and the stage (final or intermediate).
- The simulate run reuses the winner's verdicts; no case runs twice.
- With damage skipped, the arrangement says "not screened for damage" and the verdict line claims no pass.

## Scope

In: IGC 2.7 screen; ADN 9.3.1.15 and ES-TRIN 28.03 through the same seam; 0.1 m refinement; reuse; "not screened" flag.
Out: hold flooding severity (#914 lead); #915 wording; the `--no-damage` switch (#923).

## Design Questions

### Q1. Where does the damage loop run during the search?

- **A. Move the verdict loop into `ovd-core` (chosen).** A new `ovd_core::gas_damage` holds table reading (`igc_damage_data`, `permeability_table`) and the case loop, returning a typed `GasDamageRun` (verdicts, per-case records, governing case). `ovd-format/damage.rs` keeps only the JSON rendering. `passes_all` calls it. Core already holds the physics and `arc-rules` is `no_std`, so this is a move, not new code. The Python bridge gets the same behaviour for free (no drift, #932).
- B. A screen callback in `FitOptions`, supplied by `ovd-format`. Puts a closure in a `Clone + Default` struct; other callers of `fit` (Python, `smallest_viable_loa`) silently skip damage.
- C. Re-run `fit` in `simulate`, excluding the failed tank, until damage passes. Whole searches again; the Python path drifts.

### Q2. In what order are candidates screened?

- **Chosen:** damage only after a candidate passes stability and the non-damage trace, in the existing descending order. The first that also passes damage wins, as today.
- Risk: every stability-passing candidate between 13.28 m and the winner runs damage. On the example that may be more than the ticket's target of 3. Mitigation: memo the verdicts by (depth, tanks across, tank OD to 1 mm), and stop the walk for a (depth, tanks across) pair once its first damage pass is found. If the count exceeds 3, the maintainer decides (open question 1).

### Q3. How is the 0.1 m refinement expressed?

After a winner, for the winner's (depth, tanks across), try tank OD from the next larger grid candidate down in 0.1 m steps. Build each through `side_clearance_override_m`: with n across, OD = (B − 2·cs − (n−1)·gap)/n, so a 0.1 m OD step is a 0.05·n m clearance step. First pass from the top wins. No bisection: the pass is non-monotonic (#914, measured), because tank breach, cargo loss and auto ballast change in steps with OD (research §4a). Lesser and local damage (IGC 2.3.2) are not generated (`damage_cases.rs:41`): a known gap, kept.

### Q4. How is "not screened" carried?

A `damage_screen: Screen` on `FitOptions` with `Run` (default when the stack evaluates a damage Rule) and `Skip`. `ArrangementFit` gets `damage_screened: bool`; `results.arrangement.damage_screened` (hashed only when false, so existing hashes keep their bytes when true). The switch itself comes from #923; until it lands, `Skip` is reachable from tests only.

## Proposed End State Architecture

```
before: search → passes_all(stability + trace without damage) → fit → simulate runs damage
after:  search → passes_all(stability + trace) → gas_damage::run (core) → trace with damage
        → refine 0.1 m → fit (carries GasDamageRun) → simulate renders JSON from it
```

## What moves in design impact

Every gas Option whose stack evaluates IGC 2.7 and whose auto tank failed damage: tank OD, cargo volume and t, draught, lightship, GT, and the hashes of fixtures, website examples and `examples/`. The 140 × 29 m example is expected to fall from 13.28 m to about 11.1 m. Options on `igc-2g` 2.0.0 or below do not move.

## Risks

- Run time: damage is the slowest part of a run (#921). The memo and the per-pair stop bound it; the PR reports before and after.
- Equivalence: the search must run damage on the same conditions and cargo mass as `simulate` does today, or reuse gives different rows. The reuse step asserts byte-equal damage blocks against a recompute in a test.
- `no_std`: `serde_json` stays in `ovd-format`.
- A hull that passes nothing: `closest_to_passing` must count damage failures too, or the "closest" answer looks better than it is.

## Patterns to Follow

Seam tests through a `.vdesign` built in code (`tests/arrangement_style.rs:14-40`); snapshot labels and re-pinning (`docs/agents/build-process.md`); changelog fragment from `tools/design_impact.py changelog`.
