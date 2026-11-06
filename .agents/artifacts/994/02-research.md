---
ticket: 994
phase: research
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Research: arrangement style "wheelhouse forward"

## What the code does today

- `ArrangementStyle` has two values, `ContainerBlock` and `Deckhouse`, with `key()`/`from_key()` and the list `ARRANGEMENT_STYLES = ["container_block", "deckhouse"]` (`aft_arrangement.rs:50-77`). Adding a style touches this list and every match on it.
- Both styles put the accommodation and wheelhouse at the stern. `AftDesign` holds the wheelhouse eye (`wheelhouse_eye_above_floor_m`, 1.50 m) and the margin (`sightline_margin_m`, 0.50 m); the max lift default is 6.00 m.
- `wheelhouse.rs` computes the sightline in the ship frame (x forward from the aft end). Each `Obstruction` has a forward edge and a top; the eye line over the top meets the water at `x_hit`; the Blind Distance is the largest `x_hit - x_bow`. Limits: ES-TRIN 7.02(2) 250 m; SOLAS V/22.1.1 two lengths or 500 m. The ticket says obstructions behind the eye are skipped. With the eye forward, the cargo tanks are behind it, so the lift needed falls to zero.
- `deckhouse_openings.rs` places entrances, windows, doors and air inlets by `Face`, and measures distance from the deckhouse front and whether the opening faces the cargo area against the IGC 3.2.4.1 zone (4 % of L, 3 to 5 m). It assumes the cargo area is forward of the deckhouse. A forward deckhouse has the cargo area aft, so "faces the cargo area" flips from the fore face to the aft face.
- Fire class: the module notes MSC.411(97) replaced 3.2.5 and dropped its A-0 sentence for wheelhouse windows, so no class is given today.

## Gaps to design

- No code reads the rear view, a second muster point, or an unmanned engine room.
- No row checks IGC 8.2.11.1 vent distance from openings, or 8.2.10.3 exit height.
- Seakeeping: bow accelerations are about 1.2 times the stern's (IGC 4.28.2.1); ICLL 39 bow height is not binding.
- Precedent: no gas-cargo vessel in service. Lloyd's Register gave approval in principle in 2025 to a forward-accommodation LNG carrier (HHI design). ADN 9.3.1.17.1 allows a forward wheelhouse.
