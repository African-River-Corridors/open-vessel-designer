---
ticket: 993
phase: design
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Design: arrangement style "tanks in hold, flat deck"

## Summary of the Request

Add a third style after built ATB gas barges: Type C tanks in the hold, domes through a flat main deck, a fixed deckhouse aft.

## Desired End State

`arrangement_style = tanks_in_hold` gives a closed hold, a flat deck, dome trunks, a depth that follows the tank OD, a sightline over the domes, and damage cases that flood the hold. It composes with #992.

## Decisions

### D1. The style key

- A: `tanks_in_hold` as a third `ArrangementStyle`.
- B: a separate input (`tank_stowage: hold | deck`), crossed with the stern style.

**Recommend B for the model, A as the user key.** The stern (deckhouse) and the stowage are two axes. #992 (weather shield) is the same axis as stowage. Keep `arrangement_style` for the stern only; add `tank_stowage = hold | deck | shielded`. `tanks_in_hold` then means `deckhouse` stern + `hold`. Differs from `deckhouse`: `deckhouse` says nothing about where the tanks sit.

### D2. Depth rule

- A: keep 0.55 × beam.
- B: depth = OD + double bottom + 0.3 m (built practice), solved in `fit`.
- C: B as the floor; the user may raise it.

**Recommend C.** Depth becomes an output of the tank search. The 0.3 m, and the double-bottom height from the rule set, are inputs with sources named. On the 140 × 29 m coastal LNG reference design this gives a depth set by the tank, not the beam.

### D3. Sightline: dome tops only

- A: obstruction = dome top above keel.
- B: obstruction = dome top plus the trunk cover.

**Recommend A.** Set `sightline_obstruction_m` to depth + dome height (the deck is flat, so nothing else rises). Keep `sightline_margin_m`. Cargo hold hatches are flush, so they add none.

### D4. Hold, deck and hatch

- A: the hold is one space, the deck a plane.
- B: two holds split by the bulkhead between tank rows; the main deck is the hold's watertight top; dome trunks are penetrations with a coaming; hatches (if any) are flush and weathertight.

**Recommend B.** The damage case is a side breach across the bulkhead between the tank rows, flooding both holds. The model must have that bulkhead and both holds as damage spaces, with tank permeability set by tank volume. The deck gives no buoyancy credit beyond the closed hold volume.

### D5. Weight and lightship

- A: reuse box-hull steel with a deck factor.
- B: add lines: flat main deck plate, hatch covers, dome trunks and coamings, cradles and chocks.

**Recommend B**, with every unit weight named as an estimate until sourced. Double bottom and hold bulkhead weights come from the existing hull steel; avoid counting them twice.

### D6. Drawings

- A: reuse the deck-stowed section.
- B: a new profile and section: flat deck, domes standing above, hold bulkhead, cradles.

**Recommend B.** One profile, one midship section, one plan. Dome tops carry the air-draft dimension.

### D7. Re-pins

- A: re-pin everything.
- B: none move by default; `tank_stowage` defaults to `deck`, so existing fixtures stay byte-equal. New fixtures only for the new style.

**Recommend B.** Run `tools/check_fixture_hashes.py` to prove no move.

## Composition with #992

#992 is the other end: larger tanks on less depth, tops standing above the deck under a weather shield. Both are values of one `tank_stowage` input, set by how far the tank stands above the deck: `hold` = 0 m (full depth, domes only); `shielded` = tank top above deck, shield over it; `deck` = today. One search handles all three: depth = f(OD, standing height), where the standing height is 0 for `hold`. The sightline and air draft take the same standing height, and the damage model takes the same hold-or-open flag. So #992 and #993 share one seam; build the seam once, in the first of the two to land.

## Out of Scope

Cargo handling gear, a hold ventilation design, hold inerting, and tank fixing loads.
