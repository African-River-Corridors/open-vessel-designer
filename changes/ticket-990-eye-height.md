<!-- unreleased -->
- Wheelhouse eye height (issue #990): the helmsman's eye of a tug over its barges is 1.65 m above the wheelhouse floor, as ES-TRIN 2025/1 Art. 7.02(3) states (`ES_TRIN_EYE_ABOVE_FLOOR_M`); before it read 1.50 m, an estimate carried over from the original engine. A new Option already took 1.65 m for its own wheelhouse from the Defaults Profile. The engine default for a missing input, and the value a file saved before the input existed keeps, stay 1.50 m (ADR 0007), so those files do not move. The pushed and ATB fixtures' stored results hashes are re-stamped.

<!-- design-impact -->
### ticket/990-eye-height (2026-11-07)

Between `main 3559dc1` and `this branch de5df87d`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration meb-dcb23-ptich | `formation.notes[3]` | The tug's view over the barges needs 9.09 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 1935.1 m, more than 250 m (ES-TRIN 7.02(2)). | The tug's view over the barges needs 8.94 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 1458.4 m, more than 250 m (ES-TRIN 7.02(2)). | changed |
| sweep atb notch 100x20 | `formation.notes[2]` | The tug's view over the barges needs 6.21 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 272.6 m, more than 250 m (ES-TRIN 7.02(2)). | The tug's view over the barges needs 6.06 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 255.8 m, more than 250 m (ES-TRIN 7.02(2)). | changed |
| sweep gas cargo LNG pushed | `formation.notes[3]` | The tug's view over the barges needs 16.91 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | The tug's view over the barges needs 16.76 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | changed |
| sweep gas cargo LNG_ethane pushed | `formation.notes[3]` | The tug's view over the barges needs 16.92 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | The tug's view over the barges needs 16.77 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | changed |
| sweep gas cargo LPG_butane pushed | `formation.notes[3]` | The tug's view over the barges needs 13.25 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | The tug's view over the barges needs 13.10 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | changed |
| sweep gas cargo LPG_mix pushed | `formation.notes[3]` | The tug's view over the barges needs 13.25 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | The tug's view over the barges needs 13.10 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | changed |
| sweep gas cargo LPG_propane pushed | `formation.notes[3]` | The tug's view over the barges needs 14.68 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | The tug's view over the barges needs 14.53 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | changed |
| sweep gas channel 60x10 | `formation.notes[3]` | The tug's view over the barges needs 9.58 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | The tug's view over the barges needs 9.43 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | changed |
| sweep gas channel 90x14 | `formation.notes[3]` | The tug's view over the barges needs 16.92 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | The tug's view over the barges needs 16.77 m of wheelhouse lift; the tug gives 6.00 m. At that height the barges hide the water ahead (ES-TRIN 7.02(2): at most 250 m). | changed |
| sweep hull shipd bulbs 80 m | `formation.notes[3]` | The tug's view over the barges needs 6.31 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 287.6 m, more than 250 m (ES-TRIN 7.02(2)). | The tug's view over the barges needs 6.16 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 268.3 m, more than 250 m (ES-TRIN 7.02(2)). | changed |
| sweep gas channel 60x10 | `formation.sightline.blind_distance_laden_m` | 28719.5644766 | 3237.86319653 | -88.7 % |
| calibration meb-dcb23-ptich | `formation.sightline.blind_distance_light_m` | 1935.09115235 | 1458.44199288 | -24.6 % |
| calibration danube-barge-77 | `formation.sightline.lift_needed_m` | 2.12835010377 | 1.97835010377 | -7.05 % |
| calibration danube-barge-77 | `formation.sightline.lift_used_m` | 2.12835010377 | 1.97835010377 | -7.05 % |
| sweep hull shipd bulbs 80 m | `formation.sightline.blind_distance_light_m` | 287.550313707 | 268.309179102 | -6.69 % |
| sweep crude pushed 60x10 | `formation.sightline.lift_needed_m` | 2.38570800347 | 2.23570800347 | -6.29 % |
| sweep crude pushed 60x10 | `formation.sightline.lift_used_m` | 2.38570800347 | 2.23570800347 | -6.29 % |
| sweep atb notch 100x20 | `formation.sightline.blind_distance_light_m` | 272.587856781 | 255.770727292 | -6.17 % |
| calibration amaggi-box-2000 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x12-imt205 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x12-pb2012 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x12-rci250 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x13-cby252 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x13-cby400 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x13-cby433 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration canal-box-200x35x13-pb2352 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration cargill-amazon-jumbo | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration danube-barge-77 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration dsb-holdbarge-1997-76.5 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration europa-iib-1635ou | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration europa-iib-rdb06 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration hidrovia-lhg-3200 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration jeffboat-am2100 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration mcdonough-195x35x12 | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration meb-dcb23-ptich | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration meb-dcb24-pola | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| calibration meb-rdb12-samarskaya | `formation.sightline.eye_lowered_above_water_m` | 2.5 | 2.65 | +6 % |
| ... | 379 more changed quantities | | | |
