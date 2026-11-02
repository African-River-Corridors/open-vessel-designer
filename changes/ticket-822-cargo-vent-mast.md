<!-- unreleased -->
- Cargo vent mast and the air drafts (issue #822): a gas hull now has a cargo PRV vent mast, placed by the engine at the forward end of the cargo zone, as far from the openings aft as the hull allows. Its exit stands, for a `coastal-and-river` Service Area, the greater of B/3 or 6 m above the weather deck (IGC 8.2.10.3) and 6 m above the working area (8.2.10.4: the weather shield top, or the tank top with no shield; the dome top is not a working area); for `river-only`, the greater of 2.00 m above the deck and the dome top plus `vent_stub_m` (ADN 9.3.1.22.3; its 1.00 m reduction clause is noted, not modelled). The mast stands at the forward end of the cargo area (where the bow rake begins) and keeps B or 25 m, whichever is less, from the nearest opening, air intake or outlet (IGC 8.2.11.1; the allowance for ships under 90 m is noted, not used), or 6.00 m from the accommodation (ADN); the results say whether it does (`vent_mast_distance_met`) and a warning says so when it does not. Two new inputs for gas hulls, `vent_stub_m` (1.0 m, a working value, no source) and `vent_mast_lowerable` (false). The river air drafts (`air_draft_loaded_m`, `air_draft_light_m`, `air_draft_ballast_m`, the river top that `air_draft_limit_m` is checked against at both check sites) rise to the exit unless the mast is lowerable; `air_draft_navigating_light_m` (the sea top, wheelhouse raised) always includes it. A lowerable mast adds a warning: no practice source was found for lowering cargo vent masts, and class or flag acceptance is needed. New result keys (hashed): `vent_mast_x_m`, `vent_exit_above_keel_m`, `vent_exit_above_deck_m`, `vent_exit_above_working_area_m`, `vent_working_area_top_m`, `vent_working_area`, `vent_exit_governed_by`, `vent_distance_to_openings_m`, `vent_distance_to_accommodation_m`, `vent_mast_distance_target_m`, `vent_mast_distance_met`, `vent_mast_lowerable`, `vent_mast_notes`. Four Measures for the Rule rows of #849 and #850: `vent.exit_above_deck`, `vent.exit_above_working_area`, `vent.distance_to_openings`, `vent.distance_to_accommodation`. The side elevation and the plan draw the mast with its exit height. No Rule row is added. Every gas Option's air draft and results hash moves; no other Cargo's result moves.
- Legacy gas files (issue #822): a file saved before has no `vent_stub_m` or `vent_mast_lowerable`; it reads 1.0 m and false with a note of its own. Unlike the other legacy inputs, these two do not keep the old results: the vent mast is now modelled, so a legacy gas file's air drafts and results hash move.
- Gas files whose stored results hash moved and was re-stamped from `ovd run --json` (issue #822): the fixtures `gas_igc_2g_exact` and `gas_igc_2g_self_propelled`; the website example `lrg-gas-89`; both files of `examples/gas-140x29-coastal-lng/` (3 and 23 Options); the legacy files `v2_gas_lng_before_292`, `v2_gas_lpg_before_292`, `v2_gas_lpg_before_313` and `v3_gas_before_739`. `v1_adn_g_self_propelled` keeps its old hash on purpose.
- Example (issue #822): `examples/gas-140x29-coastal-lng/` (results.json, ga/, trace.md, the README tables) is regenerated from the engine by `tools/refresh_gas_example.py`. It also moved for older reasons, because earlier changes had re-stamped hashes only: the stern void, lightship, draught, stability and cost figures changed for those reasons, not because of the vent mast.

<!-- design-impact -->
### ticket/822-cargo-vent-mast (2026-11-02)

Between `main fa05bd2` and `this branch d1549ccc`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_distance_to_accommodation_m` | absent | 63.988 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_distance_to_openings_m` | absent | 60.988 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_exit_above_deck_m` | absent | 4.43666666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_exit_above_keel_m` | absent | 10.1366666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_exit_above_working_area_m` | absent | 0.7 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_exit_governed_by` | absent | dome_top | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_mast_distance_met` | absent | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_mast_distance_target_m` | absent | 6 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_mast_lowerable` | absent | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_mast_notes[0]` | absent | ADN 9.3.1.22.3: the height of the outlet above the deck may be reduced where nothing stands, no work is done and signs mark a radius of 1.00 m round it; this is not modelled. | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_mast_x_m` | absent | 78.88 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_working_area` | absent | weather_shield | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.vent_working_area_top_m` | absent | 9.43666666667 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_distance_to_accommodation_m` | absent | 105.988 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_distance_to_openings_m` | absent | 102.988 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_exit_above_deck_m` | absent | 8.22761590787 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_exit_above_keel_m` | absent | 13.9276159079 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_exit_above_working_area_m` | absent | 1.22047980428 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_exit_governed_by` | absent | dome_top | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_mast_distance_met` | absent | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_mast_distance_target_m` | absent | 6 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_mast_lowerable` | absent | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_mast_notes[0]` | absent | ADN 9.3.1.22.3: the height of the outlet above the deck may be reduced where nothing stands, no work is done and signs mark a radius of 1.00 m round it; this is not modelled. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_mast_x_m` | absent | 120.88 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_working_area` | absent | weather_shield | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.vent_working_area_top_m` | absent | 12.7071361036 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_distance_to_accommodation_m` | absent | 105.988 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_distance_to_openings_m` | absent | 102.988 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_exit_above_deck_m` | absent | 4.93168378545 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_exit_above_keel_m` | absent | 10.6316837854 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_exit_above_working_area_m` | absent | 0.732865735208 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_exit_governed_by` | absent | dome_top | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_mast_distance_met` | absent | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_mast_distance_target_m` | absent | 6 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_mast_lowerable` | absent | False | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_mast_notes[0]` | absent | ADN 9.3.1.22.3: the height of the outlet above the deck may be reduced where nothing stands, no work is done and signs mark a radius of 1.00 m round it; this is not modelled. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_mast_x_m` | absent | 120.88 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_working_area` | absent | weather_shield | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.vent_working_area_top_m` | absent | 9.89881805024 | changed |
| sweep default depth gas 90x14 | `results.vent_distance_to_accommodation_m` | absent | 61.208 | changed |
| ... | 2788 more changed quantities | | | |
