---
ticket: 992
phase: research
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Research: gas depth with larger tanks, less depth and a weather shield

## Summary

Gas depth is a fixed ratio of beam: 0.55 (`builtin_profile.json`, `depth_to_beam_ratio.default_by_cargo_group.gas`; table in `catalogue/depth_rule.md`). The ratio comes from 8 gas carriers. Seven are sea-going and none is wider than 23.6 m. On the 29 m beam it gives 15.95 m. Real Type C barges run D/B 0.49 to 0.56.

The maintainer's concern is right: a tank-first depth grows with tank OD, and the hull follows. A shallower hull with the tank tops above deck breaks that link. The model has no shield and no "tank above deck" amount today.

## Detailed Findings

1. **Default.** `vdesign.rs` reads `depth_to_beam_ratio` by cargo group when an Option omits depth, and fails loudly if the group is missing. A copy of the profile may edit it.
2. **Search.** `gas_arrangement.rs` steps tank OD, tanks across and side clearance. Depth is an input to each candidate, not a result. There is no term for a tank top above deck.
3. **Damage.** The governing case is the side breach across the hold bulkhead. Depth and freeboard do not help it. A tank above deck under a shield gives no buoyancy unless the shield is watertight. Damage in the search is #935.
4. **Sightline.** A taller tank above deck needs more eye height. The limit is moving per Service Area (#989), and the eye height goes to 1.65 m (#990). The check lives in `wheelhouse.rs`.
5. **Air draft.** The dome sets it: about 13.6 m light on the 140 x 29 m coastal LNG reference design. A shield over the tank tops sits below the dome, so it should not raise air draft. A tank above deck raises the dome only if the tank, not the dome, is the highest point.
6. **Freeboard.** A non-watertight shield is not a superstructure and has no buoyancy credit. Freeboard reads the deck at the side, so lower depth means less freeboard for the same draught. This must be checked against the load-line rule data.
7. **Gaps.** Windage area and shield weight have no model. The drawing has no shield symbol. Hashes of gas examples will move.
