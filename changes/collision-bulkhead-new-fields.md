<!-- unreleased -->
- Design impact of #1101 (the forward house aft of the collision bulkhead): every case moves, but only by new output fields and a new check. Every result has three new fields (`geometry.forward_perpendicular_x_m`, `geometry.collision_bulkhead_from_fp_m`, `geometry.collision_bulkhead_x_m`), and every Design with a check list has the new check "house aft of collision bulkhead"; inserting it shifts the later checks one place, with no check changed by name except one message that now names it (an infeasible gas case, infeasible before and after). No feasibility, tank pick or cargo figure of an existing case moves. The three new `collision_bulkhead_m` cases (a 110 m river-only LNG barge with the forward wheelhouse) carry 0.8 to 5.1 % less cargo than on `main`, where the house sits over the bow.

<!-- design-impact -->
### collision-bulkhead-new-fields (2026-11-20)

Between `main 3271b02` and `this branch 3442460c`.

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
| ... | 13336 more changed quantities | | | |
