---
ticket: 935
phase: research
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Research: damage survival inside the arrangement search

## Summary

The search builds every candidate (depth × tanks across × extra side clearance), runs cheap checks on all, then runs the costly stage from the most cargo down and keeps the first that passes. The costly stage is IS Code stability plus the Rule Trace. The trace's `Measured` has an empty damage list, so every `igc/2.7*` row reads `not evaluated` and cannot fail. Damage runs once, after the search, in `ovd-format`.

The numerical damage engine (cases, flooding, stability, stages) is already in `ovd-core`. Only the glue is in `ovd-format`: reading the damage tables from the Rule Set, the case loop, and the JSON block. A candidate already holds everything the loop needs. So moving the loop into `ovd-core` is a move of code, not new physics.

## Detailed Findings

### 1. The costly stage reads stability and the trace, never damage

- Grid: `DEFAULT_TANKS_WIDE` 1–3 and `DEFAULT_EXTRA_CLEARANCE_M` 0.5–2.0 m (`gas_arrangement.rs:66-67`); each extra is applied as `side_clearance_override_m = side + extra` (`:1375`).
- Costly loop: sort by ranked cargo, descending (`:1420`), call `passes_all` (`:1431`), keep the best, stop once cargo drops below the winner's (`fit` calls `search(.., false)`, `:1670`).
- `passes_all` (`:1059`): no Rule Sets → stability checks; with Rule Sets → `candidate_failures(&full_measured(c, o), o).is_empty()`.
- `full_measured` (`:1067`) → `candidate_measured` (`:276`), which sets checks, conditions, tank, mast and separation, then `..Measured::default()` (`:300`): `damage`, `adn_gas_damage`, `es_trin_damage` are empty.
- `Measured::has` returns true for `stability.damage*` (`measured.rs:481-487`), and with no cases the rows are `not evaluated` (missing). They never fail in the search.
- The answer's trace is rebuilt the same way (`:1710`), so `fit.feasible` also ignores damage; only the simulate trace sees it.

### 2. Damage runs after the search, in `ovd-format`

- `simulate.rs:3249-3272`: if the stack's `igc-2g` has `igc/2.7/damage-survival` evaluated and IS Code conditions exist, assemble the vessel model (`vessel_model::assemble`, `vessel_model.rs:276`) and call `damage::gas_damage_verdicts`.
- `simulate.rs:3273-3320`: the same for `adn-gas` (`adn/9.3.1.15.1/damage-stability`) and `es-trin-2025` (`es-trin/28.03(4)/damage-stability`, only when `l > 110`).
- `damage.rs:33` reads the IGC 2.3.1 extents from the set's `igc-damage-*` tables (`arc_rules::eval::table_value`, `no_std`). `damage.rs:134` builds cases with `igc_damage_cases` (`damage_cases.rs:550`) and runs `run_gas_damage` (`damage.rs:187`): per condition, per case, `gas_flooding` (`damage_cases.rs:728`), `assess` (`damage_stability.rs:712`), `intermediate_stages` (`:881`). It returns `Vec<DamageVerdict>` and an unhashed JSON block (spec 0012 D21).
- The only non-core dependencies in that loop are `serde_json` and `crate::eval::num` for the block.

### 3. A candidate holds what a damage run needs

- Hull: `r.hull_spec` → `hydrostatics::Hull::build` (already done in `is_code_cargo`, `gas_arrangement.rs:886`).
- Conditions: `is_code_conditions` (`:911`) returns `LoadedCondition`s; `stability_assessment` builds them and keeps `.condition`, dropping the weights the damage loop needs.
- Vessel model: `VesselModelInputs { capacity, result, hull, structure }` (`vessel_model.rs:35`); `FitOptions.structure` exists (`gas_arrangement.rs:139`).
- Rule Sets: `FitOptions.rules` (the selected sets, with tables).
- Damage needs IS Code conditions: `simulate.rs:3136-3147` fills `damaged_conditions` only on the `is_code` path. Without `is_code`, no damage runs today either.

### 4. Rule text

- IGC 2.7 survival, 2.7.1.1-.3 (any stage), 2.7.2.1 (final): library `igc/MSC.370-93.txt:1375-1400`.
- IGC 2.3.1 extents: `MSC.370-93.txt:1099`. IGC 2.3.2.1 (lesser damage, if worse, is assumed) and 2.3.2.2 (local damage to depth d): `:1130-1136`.
- **Known gap:** the engine does not generate the lesser damage of 2.3.2.1 or the local damage of 2.3.2.2 (`damage_cases.rs:41`). The screen inherits that gap; a design that passes may still fail a lesser or local case. This ticket does not close it.

### 4a. Why the pass can be non-monotonic in diameter, in the engine as it is

#914 measured it (11.1 m passes, 11.0 m fails 2 rows); lesser damage is not the cause, since it is not generated. The case set and the conditions change in steps with the arrangement, not smoothly:
- The box extents are fixed (IGC 2.3.1, from L and B), but the tank OD and side space move the compartment boundaries, so a case breaches a tank or not (`gas_flooding`, `damage_cases.rs:728`). A breached tank loses its cargo (`without_breached_cargo`); an intact tank in a flooded hold keeps its buoyancy (spec 0012 D7). Each is a jump in mass or buoyancy, not a smooth change.
- The cases are placed on bulkheads and just beyond them (`box_damage_cases`, `damage_cases.rs:569-578`); a bulkhead that moves with the tank length changes which windows exist.
- The IS Code conditions are rebuilt per arrangement, with automatic ballast plans (`is_code_conditions`, `gas_arrangement.rs:911`), and the ballast chosen changes in steps.
Which of these flips the 11.0/11.1 m pair was not traced (no runs tonight). These step changes are enough to rule out bisection: a pass at one OD says nothing about the OD 0.1 m smaller.
- ADN 9.3.1.15.1: `adn/adn.txt:34129`. ES-TRIN Article 28.03: `es-trin/ES_TRIN_2025_signed_en.txt` (index `:378`).

### 5. `--no-damage` does not exist yet

#923 is open. No `no_damage` switch on `main`. #935 item 5 depends on it.

### 6. Tests and examples

- Seam tests in `crates/ovd-format/tests/*.rs` build a `.vdesign` in code, `fill_as_new`, `run_option` (pattern: `tests/arrangement_style.rs:14-40`).
- Example `examples/gas-140x29-coastal-lng/` (vdesign, `results.json`, `trace.md`); its results hash is in the fixture-hash check.
- Rule data: only `igc-2g` 1.0.0, 1.1.0, 2.0.0 are released (`rules/RELEASED.json`); 3.x are unreleased. This ticket changes engine code, not rule data.

## Code References

Key files, not exhaustive: `gas_arrangement.rs` (search, `passes_all`, `fit`), `simulate.rs:3027-3460` (fit call, damage blocks, `Measured` for the trace), `damage.rs` (tables and case loop), `damage_cases.rs`, `damage_stability.rs`, `measured.rs` (`DamageVerdict` `:99`, `has` `:481`).

## Architecture Documentation

`ovd-core` is `no_std` + `alloc` (`lib.rs:8`); `arc-rules` is `no_std`. `ovd-format` reads files, builds JSON and calls core. Rule numbers live in Rule Set tables, not code. Results hashes cover hashed blocks only; the damage block is unhashed (D21), but the tank size is hashed.

## Open Questions

1. How many candidates pass cheap checks and stability between 13.28 m and 11.1 m on the example? That count is the damage-run count of a plain descending scan. Not measured (no builds tonight).
2. Does `simulate` pass the same cargo mass to `is_code_conditions` as the search ranks by? Both read the payload, but this must be checked when the result is reused.
