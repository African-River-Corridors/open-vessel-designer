---
ticket: 919
phase: design
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Design: insulation as explicit inputs

## Summary of the Request

Make conductivity and density inputs with defaults by insulation type and cargo, each with status and source; one thickness everywhere; heat leak and mass read the inputs; results carry the U-value and heat leak per tank; the Python API takes the same inputs.

## Current State

Thickness is an input; conductivity is a constant; density is a hidden 60 kg/m³; the type changes nothing; four paths fall back to 350 mm (research §1).

## Desired End State

- Inputs `tank_insulation_conductivity_w_per_mk` and `tank_insulation_density_kg_m3`, defaulted by type and cargo, each `estimate` with #918's source.
- No path reads a thickness that is not the input.
- `boil_off` gains `u_value_w_per_m2k` and `heat_leak_per_tank_kw`; holding time reads the same conductivity.
- `simulate` and the Python API take the two new keys.

## Scope

In: the two inputs, the type deciding them, the 350 mm paths, U-value and per-tank leak in results, `simulate`.
Out: thickness against boil-off and capacity tables (#920); the solve of thickness and pressure (#875, which should read these inputs); ellipsoid head area (open question 3); new insulation types such as PIR or vacuum perlite (open question 4).

## Design Questions

### Q1. How are the defaults keyed?

- **A. By cargo, the same rows as the type (chosen).** The profile's tables key on containment, cargo and carriage (`TABLE_KEYS`), and the type already defaults by cargo there. Conductivity and density get tables with the same rows (LNG → panel values; cold LPG → pu_foam values; pressurised LPG → none, 0). A designer who types a type other than the default gets a warning: "k and ρ are the defaults for <default type>; type the values for <typed type>".
- B. Add `tank_insulation_type` as a table key. Clean, but a fifth key (after #926 adds `service_area`) for one input pair, and every table's `by` list changes again.
- C. Constants per type in Rust. Hides the source from the Gas view; the ticket asks for inputs.

A keeps the table machinery as it is. B is the upgrade if more types arrive (open question 4).

### Q2. What values?

Status `estimate` for all, as #918 found no maker data.

| Type | k (W/m.K) | ρ (kg/m³) | Source |
|---|---|---|---|
| `panel`, `pu_foam` | 0.026 | **60 today**, 96 proposed | Lee et al. 2021 Table 3 (k); 96 is k's basis |
| `none` | n/a | 0 | no insulation |

Density is the one value that moves designs (+80 t on the example at 96). The design keeps 60 in the plumbing PR and moves to 96 in its own PR, so the maintainer can approve the move apart from the plumbing (open question 2).

### Q3. One thickness

- Core: `ga_adapter.rs:130,238` read the input's thickness (`inp`), not `unwrap_or(350.0)`; `models.rs:222` default becomes 0 with the comment "set by the caller"; fix the stale comment at `gas_arrangement.rs:1264`.
- `DEFAULT_TANK_INSULATION_MM` is renamed `LEGACY_SIMULATE_INSULATION_MM` and is read only by `simulate` for a key left out, as `tank_wall_of` keeps 25 mm (`simulate.rs:773`). This keeps the Python API's pinned results (`simulate.rs:1204`). Open question 1 asks whether to drop that too.

### Q4. Where the heat leak reads k

`TankDesign` gains `insulation_conductivity_w_per_mk` and `insulation_density_kg_m3`. `BargeCapacityResult` exposes both. `ovd-format/src/gas_fuel.rs` and `holding_time.rs` read them from the result, not from the constant; the constant stays only as the profile default's documented value.

### Q5. U-value

`U = Q / (A_outer · ΔT)` per tank, with `A_outer` the cylinder plus two heads on the outer radius, the same geometry as the heat leak. Reported with its basis ("on the outer area; domes, saddles, pipes out").

## Proposed End State Architecture

```
profile (type, k, ρ, t by cargo) → inputs → params → TankDesign{t, k, ρ}
  → tanks.rs mass (ρ, t) → capacity result {k, ρ, t}
  → gas_fuel / holding_time heat leak (k) → boil_off {U, Q per tank, Q total}
```

## What moves in design impact

PR "plumbing": nothing (same values). PR "density": insulation mass, lightship, draught, cargo, GT on every insulated Type C Option. The 350 mm paths: none on `run` (it always passes the thickness); GA geometry only where a result lacked the thickness.

## Risks

- `simulate.rs` is shared with lane B1 and #930: that PR waits for #930 or messages B1.
- #875 solves thickness; it must read k from the input, not the constant. Flag on #875.
- #918's sources are secondary; values stay `estimate`.

## Patterns to Follow

`default_by_table` rows with `status`, `source`, `basis` (`builtin_profile.json:505-560`); `simulate`'s key-left-out convention (`simulate.rs:773-799`); `Label: sourced` for the hand check.
