---
ticket: 919
phase: research
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Research: insulation as explicit inputs

## Summary

Thickness is the only insulation input that reaches the numbers. Conductivity is a core constant (0.026 W/m.K) read by the boil-off and the holding time in `ovd-format`. Density is 60 kg/m³ in the core's input default and never set from a Design. The type is recorded and shown, and changes nothing. Four code paths fall back to 350 mm while the profile gives 330 mm for LNG.

The #918 research (comment of 2026-11-05) hand-worked the example's tank: U = 0.0759 W/m²K on the outer area, 26.54 kW per tank, 746.85 kg/h for four tanks. The engine gives the same to the digit. #918 found no maker data: every published value is secondary.

## Detailed Findings

### 1. Where each value enters

| Value | Today | Where |
|---|---|---|
| Thickness | input `tank_insulation_thickness_mm`, LNG 330 mm estimate | `builtin_profile.json:533`; `TankDesign.insulation_thickness_mm` |
| Thickness fallback | 350 mm | `barge_capacity.rs:118` (`DEFAULT_TANK_INSULATION_MM`, used `:189` and `simulate.rs:886`); `ga_adapter.rs:130,238` (`unwrap_or(350.0)`); `models.rs:222`; stale comment `gas_arrangement.rs:1264` |
| Conductivity | constant 0.026 W/m.K, PU foam 96 kg/m³ at about 215 K | `ovd-core/src/gas_fuel.rs:69-73`; read at `ovd-format/src/gas_fuel.rs:109,137` and `holding_time.rs:126,226` |
| Density | 60 kg/m³ | `models.rs:120,223`; mass at `tanks.rs:298-308`; printed `eval.rs:1695` |
| Type | input `tank_insulation_type`: `unspecified`, `none`, `pu_foam`, `panel` | `vdesign.rs:201`; `builtin_profile.json:505` (LNG `panel`); shown only (`simulate.rs:1128`) |

### 2. The heat leak

`tank_heat_leak_kw` (`ovd-core/src/gas_fuel.rs:88-102`): cylinder by the log-mean formula; two heads as one sphere of the same radii; domes, saddles, pipes out. ΔT from 45 °C (IGC 7.2, `:67`). `ovd-format/src/gas_fuel.rs:100-140` builds the `boil_off` block: heat leak for all tanks, boil-off, conductivity, no U-value and no per-tank figure. Holding time uses the same function (`holding_time.rs:115-130`).

### 3. `simulate` keeps the original engine for a key left out

`tank_wall_of` and `dome_of` (`simulate.rs:773-799`) document the convention: a key left out keeps the old fixed value; `null` asks for the rule. The insulation thickness follows it (`:883-887`, 350 mm). `simulate.rs:1204` says the Python API results "pinned to the original Python" do not move.

### 4. #918's values (secondary sources only)

- Conductivity: PU foam 0.0248-0.0266 W/m.K at 200-230 K (Lee et al., J. Mar. Sci. Eng. 2021, 9, 36, Table 3; cited in the code); foam without vacuum 0.035-0.055; vacuum perlite 0.001-0.016.
- Density: PU foam 96 kg/m³ (the basis of the conductivity); perlite 40-300 kg/m³.
- Thickness: 330 mm (one secondary report, panel); 300 mm perlite in vacuum for small fuel tanks.
- At 96 kg/m³ the example's insulation mass rises from 133.8 t to 214 t (+80 t).
- Boil-off 0.27-0.29 %/day, about twice a large carrier's 0.15 %/day basis; nothing in OVD says why.
- Sphere heads overstate a 2:1 ellipsoid head's area by about 45 % (hand geometry).

### 5. Hand check (from #918)

OD 11.0 m, steel OD 10.34 m, 330 mm, cylinder 37.904 m, k 0.026, ΔT 207 K: cylinder 20.72 kW, heads 5.83 kW, 26.54 kW per tank, U 0.0759 W/m²K on 1,690 m². Engine: 106.17 kW / 4 = 26.54 kW; 746.85 kg/h.

### 6. Tests and examples

`crates/ovd-format/tests/type_c_defaults.rs` (profile defaults), gas fixtures under `tests/fixtures/gas_*.vdesign`, `examples/gas-140x29-coastal-lng/`. The `boil_off` block is in the results.

## Code References

Key files, not exhaustive: listed in the table of §1, plus `ovd-format/src/gas_fuel.rs:100-140`, `holding_time.rs:115-130,226`, `simulate.rs:773-887`.

## Architecture Documentation

Inputs → Defaults Profile tables (by containment, cargo, carriage) → params → `TankDesign` (core) → tank geometry and mass; heat leak and holding time are worked in `ovd-format` from core functions.

## Open Questions

1. Should a Python `simulate` call that leaves the thickness out keep 350 mm (the original-engine convention), or take the profile default?
2. Which default density for `panel`: today's 60 kg/m³ or 96 kg/m³ (the conductivity's basis, +80 t on the example)?
3. Is the ellipsoid-head area of the heat leak in #919 or in #920?
