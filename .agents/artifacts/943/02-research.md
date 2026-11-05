---
ticket: 943
phase: research
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Research: deckhouse tiers sized from what they hold

## Summary

`size_deckhouse` makes every tier the whole machinery-space footprint: the length from the stern to the machinery space's forward end, by the beam less two side decks. The tier count is the larger of "tiers to hold the rooms floor" and "tiers to lift a fixed wheelhouse's eye to the sightline". On the 140 × 29 m river-only sweep case the sightline wins: 4 tiers of 488.6 m² for 63.4 m² of rooms.

Area and volume are `tiers × tier_area` and `× tier height`. Steel is volume × 0.057 t/m³; cost is area × rate; tonnage adds the volume. The wheelhouse stands on the top tier at the deckhouse front, so its x and the eye height do not depend on the tier footprint. The #912 openings read one uniform footprint for all tiers.

## Detailed Findings

### 1. One footprint, two reasons for tiers

- Constants (all estimates): `TIER_HEIGHT_M` 2.6, `DECK_ALLOWANCE_M` 0.5, `SIDE_DECK_M` 0.8, `MAX_TIERS` 4 (warn only) (`aft_arrangement.rs:87-96`).
- `size_deckhouse` (`aft_arrangement.rs:1073`): width = beam − 2 × 0.8; length = `x_fwd` (the machinery space's forward end); `tier_area = length × width` (`:1075-1077`).
- Rooms tiers: `ceil(required / tier_area)`, at least 1 (`:1090-1094`). `required` is the rooms table total, or the interim 40 ft-unit floor (`:1078-1087`).
- Sight tiers: for a fixed wheelhouse, `ceil((eye − eye_above_floor − depth) / 2.6)` when larger (`:1098-1117`).
- Output: `area_m2 = tiers × tier_area`, `volume_m3 = × 2.6` (`:1128-1129`).

### 2. The rooms table

`rooms::size_rooms` (`rooms.rs:231`) returns rows and `total_m2` (`:152`): cabins, mess (`:340`), corridor (`:408`), galley (`:422`), work-clothes store (`:432`), one stairway (`:440-446`, "a further one for each tier above the first is not counted, as the rooms do not know the tier count"). No machinery casing, workshop, CO2 room or other service space is in it.

### 3. The wheelhouse and the eye

- On a deckhouse the wheelhouse's forward end is the deckhouse front (`aft_arrangement.rs:1290-1292`), and its floor is the top tier (`:1421-1425`).
- The eye minimum comes from the capacity chain: a probe stern, then `wheelhouse::eye_needed` from the wheelhouse front x, the obstructions and the light waterline, against the Blind Distance limit or the stricter SOLAS V/22.1 limit (`barge_capacity.rs:1578-1609`).
- Two passes: the first on the keel, the second on the first's trimmed light waterline; the second is kept only if it has fewer tiers and still passes (`barge_capacity.rs:1439-1460`). Lightship changes the light waterline, so a lighter house can change the eye needed.

### 4. Who reads the deckhouse

| Reader | Reads | Where |
|---|---|---|
| Mass | `volume_m3` × `DECKHOUSE_T_PER_M3` 0.057; wheelhouse box separately | `aft_arrangement.rs:688-706`; rate `machinery.rs:587` |
| Cost | `area_m2` × `deckhouse_rate_per_m2` | `ovd-format/src/cost.rs:140`; `barge_cost.rs:424` |
| Tonnage | `volume_m3` joins V | `tonnage.rs:116-120` |
| Headroom Rules | `tier_height_m − 0.5` | `measured.rs:143-152` |
| Results | `deckhouse` block (tiers, tier_area, area, volume) | `ovd-format/src/eval.rs:540-556`; rooms block `run.rs:985-996`; wheelhouse block `run.rs:1042` |
| App | `tiers` | `app/src/lib/sternCrew.ts:117,140`; `SternCrew.svelte:25` |
| Vessel model | wheelhouse fixed or lifting | `vessel_model.rs:760` |

### 5. The #912 openings assume a uniform footprint

On `ticket/912-deckhouse-openings` (commit `1fcb2a4e`, uncompiled), `deckhouse_openings::Frame` holds one `x_aft_m`, `x_fwd_m`, `width_m`, `tiers` and `tier_height_m`; `default_layout` puts each tier's entrance on the aft face at `frame.x_aft_m`; `frame_of` copies the `Deckhouse` fields. A stepped house needs a per-tier aft face.

### 6. Rule text

- ES-TRIN 2025/1 Article 7.02(2): the area of obstructed vision ahead, laden-free with half supplies, not over 250 m (library `es-trin/ES_TRIN_2025_signed_en.txt:1995-2001`).
- ES-TRIN Article 15.02, accommodation (`:5764`); MLC 2006 Standard A3.1 for a sea-reaching hull (`mlc-2006/`).
- IGC 3.2.4.1-3.2.4.2 (library `igc/MSC.370-93.txt`), openings, via #912/#913.
- Spec 0011 section 2 already says the rooms give "the deckhouse footprint and tier count" (`docs/specs/0011-stern-arrangement.md:36-37`). This ticket brings the code to the spec.

### 7. Tests and cases

- Sweep case "sweep gas river-only deckhouse 140x29" (`tools/design_impact.py:575-577`), small coastal case (`:558`).
- Seam tests: `crates/ovd-format/tests/arrangement_style.rs`, `crew.rs`; core tests of `size_deckhouse` in `aft_arrangement.rs` (#[cfg(test)]).

## Code References

Key files, not exhaustive: `aft_arrangement.rs` (`size_deckhouse`, mass items, wheelhouse placement), `rooms.rs`, `barge_capacity.rs:1439-1609`, `tonnage.rs`, `ovd-format/src/{cost,eval,run}.rs`, `app/src/lib/sternCrew.ts`.

## Architecture Documentation

The deckhouse is a struct on the stern result, computed in `ovd-core` (`no_std`), rendered in `ovd-format`. Area, volume and tiers flow into mass, cost and tonnage; mass feeds the light waterline, which feeds the sightline in the second pass.

## Open Questions

1. What floor does the main-deck tier need beyond the rooms table (engine casing, workshop, stores, CO2 room)? No source in the tree.
2. Does a lighter house change the eye needed on the sweep case through the light waterline? Not run tonight.
