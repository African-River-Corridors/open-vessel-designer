---
ticket: 926
phase: research
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Research: tank design pressure from the Service Area

## Summary

The engine already works the IGC 4.23.1.2 minimum per tank when `tank_design_pressure_mpa` is `null` (#577). Today only fully refrigerated LPG defaults to `null`; LNG defaults to 0.40 MPa on every Service Area. The profile's default tables are keyed on three fixed keys (containment, cargo, LPG carriage condition); the Service Area is not one of them, and the loader rejects any other `by` list. A separate mechanism, `default_by_service_area`, keys an input on the Service Area alone, and does refill on a change of Service Area; it takes one key only. The session refills table defaults when a table key changes, so adding the Service Area as a key would make a defaulted pressure follow a change of Service Area.

The IGC row is `warn` today (#927 makes it `block`). A `note` consequence reports a failed test as `warn` and never blocks. `ovd` is at 3.1.0; 3.2.0 is free on `main`.

## Detailed Findings

### 1. `null` → IGC minimum

- Core: `tanks.rs:150-174`: with `design_pressure_igc_minimum`, Po = 0.2 + `igc_dynamic_pressure_term_mpa(σm, OD, overall length, ρ/1000)`; else the typed value.
- Term: `pressure_vessel.rs:222-232`: A = 0.00185 (σm/55)², C = max(D, 0.75 D, 0.45 l), A·C·ρr^1.5.
- `run`: a `null` is passed through to params (`run.rs:286-297`).
- `simulate`: `design_pressure_by_rule` (`simulate.rs:893-897`) sets the flag; a key left out takes `DEFAULT_TANK_DESIGN_PRESSURE_MPA` (`:870-880`).
- `eval` (Python bridge): `gas_arrangement.rs:193-205` (ovd-format) maps `null` to the flag.

### 2. Profile tables

- `tank_design_pressure_mpa` (`builtin_profile.json:346`): rows by Cargo and carriage; LNG 0.4 `sourced` (owners' spec sheets, 4 bar); fully refrigerated LPG `null` with an IGC basis (`:378-383`).
- `TABLE_KEYS = ["cargo_containment", "cargo", "lpg_carriage_condition"]` (`vdesign.rs:224`); the loader requires `by == TABLE_KEYS` (`profile.rs:68-73`); 11 inputs use tables; `table_combinations` (`vdesign.rs:232`) enumerates every combination for the coverage check.
- Session: when a table key changes, every input still on `Source::Default` among `TABLE_DEFAULTS` is dropped and refilled (`session.rs:1190-1208`).
- **A per-Service-Area mechanism exists: `default_by_service_area`.** It is a keyed default (`KEYED_DEFAULTS`, `vdesign.rs:64-82`): one input, one key, one value per key value. Today `slop_share`, `arrangement_style`, `navigation_hours_per_day` and `manning_rule_set` use it (`builtin_profile.json:655,1368,1381,1386`). When the Service Area changes, `drop_dependent_defaults` drops every keyed input still on `Source::Default` keyed on it, and the next fill gives the new value (`session.rs:1174-1189`). So those defaults do refill.
- Its limit: one key per input. An input is either a table default or a keyed default, not both (`session.rs:286-300`). The pressure default depends on the Cargo and carriage (LNG 0.40, pressurised LPG by its own row, refrigerated LPG `null`) as well as the Service Area, so a single-key `default_by_service_area` cannot hold it.
- `Source` has `Typed`, `Default`, `Class`, `Reference` (`vdesign.rs:3712-3731`). No "rule" source; the basis text of the hit carries how a value was chosen (`profile.rs` `TableHit`).

### 3. The Rules

- `igc/4.23.1.2/min-design-pressure` (`igc-2g/3.3.0.json:988-1024`): per tank, `tank.design_pressure ≥ at-least(0.2 + tank.dynamic_pressure_term, lo 0.2)`, `warn`, `verified: false`.
- `failed()` maps `warn` and `note` to outcome `warn` (`arc-rules/src/eval.rs:394-399`). A `note` Rule never blocks.
- `ovd` versions: 3.0.0, 3.1.0 (`rule_set.rs:177-181`). Rules can apply by `service_area` (a Choice input, `trace.rs:442`).
- River-only gas stacks carry `adn-gas`, not `igc-2g` (`builtin_profile.json` `default_stacks`), so a river-only LNG tank has no pressure check today.

### 4. Rule text

- IGC 4.23.1.2 (library `igc/MSC.370-93.txt:3265-3290`): Po = 0.2 + A C (ρr)^1.5 MPa; A = 0.00185 (σm/ΔσA)², ΔσA 55 N/mm² for steel.
- ADN 9.3.1.23.1 (`adn/adn.txt:34630`): tanks comply with the pressure-vessel provisions of the competent authority or a recognised classification society; no number. 9.3.1.24.1 (`:34643`): pressure control (for example a design pressure safe over the holding time).

### 5. Hand-worked Po, 140 × 29 m example, Option 1

From `examples/gas-140x29-coastal-lng/results.json`: OD 11.0 m, overall tank length 48.904 m, 9Ni, σm 230 MPa, LNG ρ 450 kg/m³.

- A = 0.00185 × (230/55)² = 0.032353
- C = max(11.0, 8.25, 0.45 × 48.904 = 22.007) = 22.007
- ρr^1.5 = 0.45^1.5 = 0.30187
- Po = 0.2 + 0.032353 × 22.007 × 0.30187 = **0.4149 MPa**

The stored trace gives limit 0.414920 for each tank (warn at 0.40). Hand and engine agree.

### 6. Readers that move with the pressure

Wall thickness (`tanks.rs:176-200`), tank and dome mass, lightship, draught, cargo. On the example, 0.40 → 0.415 MPa is about +4 % pressure.

## Code References

Key files, not exhaustive: `tanks.rs:150-200`, `pressure_vessel.rs:222`, `run.rs:286-297`, `simulate.rs:862-897`, `ovd-format/src/gas_arrangement.rs:193-205`, `profile.rs:56-140`, `vdesign.rs:224-260`, `session.rs:1190-1208`, `igc-2g/3.3.0.json:988`, `ovd/3.1.0.json`.

## Architecture Documentation

Defaults live in the Defaults Profile, chosen by table rows keyed on fixed context keys; the engine reads params; Rules are data. A `null` input means "work it out by rule".

## Open Questions

1. Python `simulate` with no pressure key takes 0.40 MPa whatever the Service Area. After #925 the Service Area is required there too; should a coastal call with the key left out take the IGC minimum?
2. Do user profiles with a three-key `by` list exist outside the repo? If so, a fourth key needs a migration.
