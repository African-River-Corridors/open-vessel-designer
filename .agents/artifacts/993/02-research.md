---
ticket: 993
phase: research
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Research: arrangement style "tanks in hold, flat deck"

## Built practice

On built ATB LNG barges the Type C tanks sit below deck and only the domes show. Depth follows the tank OD: about OD + double bottom + 0.3 m. The wheelhouse is on the tug, so the sightline is easy. A self-propelled barge has a fixed deckhouse, which sees over the domes only.

## Code today

- `ArrangementStyle` has two keys, `container_block` and `deckhouse` (`aft_arrangement.rs`, `ARRANGEMENT_STYLES`). Both style the **stern**. `compute` reads the style once, to drop accommodation units from the deck for `deckhouse`.
- `AftArrangementInputs.sightline_obstruction_m` is "the height above keel the eye must clear". `None` means nothing to see over. `wheelhouse_eye_min_m` is the fixed-house eye floor. Both can take a dome-top height.
- Gas depth is an input to the search (`gas_arrangement.rs`, `base_inputs`); the default is 0.55 × beam (#992). Tank dome height and diameter are inputs, or come from the dome rule.
- The damage model works from spaces and permeabilities (`damage_cases.rs`, `damage_stability.rs`). The governing case is a side breach across the hold bulkhead between the tank rows, flooding both holds. A flat deck adds buoyancy only if it is watertight. Here it is the main deck, so the hold is closed and the deck is tight by definition.
- Air draft falls: only the domes stand above the deck.

## Gaps

- No hold space, no deck or hatch element, no dome trunk in the model.
- No style key ties depth to tank OD.
