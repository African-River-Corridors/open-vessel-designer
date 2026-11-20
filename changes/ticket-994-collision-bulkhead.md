<!-- unreleased -->
- The forward house stands aft of the collision bulkhead (issue #994, rework). Every result reports the forward perpendicular and the collision bulkhead plane under `geometry` (`forward_perpendicular_x_m`, `collision_bulkhead_from_fp_m`, `collision_bulkhead_x_m`). A new input `collision_bulkhead_m` (none gives 0.04 L, ES-TRIN 2025/1 Article 3.03(1)(a)) sets its distance. The `wheelhouse_forward` house has its front at the plane, or 3 m inside the stem if that is further aft, and the cargo zone ends at its aft face; the cargo length this costs is reported, and the warning says why. A new check, "house aft of collision bulkhead", and four Rule rows in two new Rule Set versions: `es-trin-2025` 2.2.0 (three rows) and `igc-2g` 4.2.0 (one row). The new `geometry` keys change every stored results hash, so the fixtures, the website example and `examples/` are re-stamped. The 80 x 9 m forward-house design in `damage_refine_bounds` carries 405 t, not 475 t: its cargo zone is 0.68 m shorter, its depth takes three passes, and the last pass has one damage run left. The arrangement search no longer stops its walk at the damage-run limit: it goes on with the verdicts an earlier depth pass kept, which cost no run (it took a 5.06 m tank, 294 t, before this fix). Why the table below moves every case: each change is an addition. The three `geometry` keys are new in all 291 cases; the three ES-TRIN 3.03 Rule rows are new in every Rule Trace, and every other row's Rule Set reads `es-trin-2025@2.2.0` (was 2.1.0); the check "house aft of collision bulkhead" is new in the 188 cases that report an arrangement; one gas case that already had no feasible arrangement lists that check among its failures. No cargo, mass, stability or cost figure moves in any case.

<!-- design-impact -->
### ticket/994-collision-bulkhead (2026-11-20)

Between `main ef36c07` and `this branch 875adec7`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration amaggi-box-2000 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration amaggi-box-2000 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x25x12-pb2230 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x25x12-pb2230 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x25x12-pb2230 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x12-cby222 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x12-cby222 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x12-cby222 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x12-imt201 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x12-imt201 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x12-imt201 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x12-imt205 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x12-imt205 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x12-imt205 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x12-pb2012 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x12-pb2012 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x12-pb2012 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x12-rci250 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x12-rci250 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x12-rci250 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x13-cby252 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x13-cby252 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x13-cby252 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x13-cby400 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x13-cby400 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x13-cby400 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x13-cby433 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x13-cby433 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x13-cby433 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration canal-box-200x35x13-pb2352 | `geometry.collision_bulkhead_from_fp_m` | absent | 2.4384 | changed |
| calibration canal-box-200x35x13-pb2352 | `geometry.collision_bulkhead_x_m` | absent | 58.0416 | changed |
| calibration canal-box-200x35x13-pb2352 | `geometry.forward_perpendicular_x_m` | absent | 60.48 | changed |
| calibration cargill-amazon-jumbo | `geometry.collision_bulkhead_from_fp_m` | absent | 2.44 | changed |
| calibration cargill-amazon-jumbo | `geometry.collision_bulkhead_x_m` | absent | 58.08 | changed |
| calibration cargill-amazon-jumbo | `geometry.forward_perpendicular_x_m` | absent | 60.52 | changed |
| calibration danube-barge-77 | `geometry.collision_bulkhead_from_fp_m` | absent | 3.08 | changed |
| calibration danube-barge-77 | `geometry.collision_bulkhead_x_m` | absent | 73.44 | changed |
| calibration danube-barge-77 | `geometry.forward_perpendicular_x_m` | absent | 76.52 | changed |
| calibration dsb-holdbarge-1997-76.5 | `geometry.collision_bulkhead_from_fp_m` | absent | 3.06 | changed |
| ... | 10697 more changed quantities | | | |
