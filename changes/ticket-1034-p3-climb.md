<!-- unreleased -->
- The river share of the tank diameter now climbs to the most cargo (issue #1034, the maintainer's decision of 2026-11-09). With the share searched, the search starts at the tallest share that passes the intact checks, goes down one share at a time while the next share passes and its cargo rises by 1 % or more of the cargo in hand, and keeps the last share that rose. A tie, a smaller gain, a fall or an infeasible share stops it, so the taller share (less depth, less steel) wins. Damage screens the share kept, as before, under the same budget of 24 runs. `ovd run` says "the most cargo within the limits" on the share line, and names the tallest passing share and its cargo when the climb moved the share; the results carry `arrangement.share_climb_from` only then. River-only designs whose tallest passing share is not the one with the most cargo move; coastal-and-river designs and designs with a typed standing height do not search the share and do not move. In the design-impact sweep one case moves because of this change: "sweep default depth gas 90x14", from share 0.5 (930.66 t, 6.3 m deep) to share 0.25 (1,742.14 t, 9.5 m deep). The table below also lists the two deckhouse cases that Phase 2 (#1055) moves, because its base is `main` until #1055 merges.

<!-- design-impact -->
### ticket/1034-p3-climb (2026-11-10)

Between `main 6b39849` and `this branch dd81936e`.

The table is against `main`, so it lists the moves of Phase 2 (#1055: "sweep gas river-only deckhouse 140x29" and "sweep optional deckhouse_openings") as well as this change's move (the 90x14 case). The 90x14 case goes from share 0.5, 930.66 t, 6.3 m deep to share 0.25, 1,742.14 t, 9.5 m deep.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep default depth gas 90x14 | `arrangement.depth_set_by` | freeboard | tank | changed |
| sweep default depth gas 90x14 | `arrangement.share_climb_from.cargo_t` | absent | 930.658943234 | changed |
| sweep default depth gas 90x14 | `arrangement.share_climb_from.share` | absent | 0.5 | changed |
| sweep default depth gas 90x14 | `checks[12].message` | genset 2.89 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 6.30 m | genset 2.89 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 9.50 m | changed |
| sweep default depth gas 90x14 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 4.67 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 5.67 m of 6.00 m | changed |
| sweep default depth gas 90x14 | `checks[17].message` | light air draft 10.47 m <= limit 40.00 m (laden is 9.62 m and is NOT the governing case) | light air draft 13.67 m <= limit 40.00 m (laden is 12.04 m and is NOT the governing case) | changed |
| sweep default depth gas 90x14 | `checks[22].message` | tank length 26.90 m / inner diameter 7.52 m = 3.58 <= 7.0 | tank length 26.90 m / inner diameter 10.52 m = 2.56 <= 7.0 | changed |
| sweep default depth gas 90x14 | `checks[23].message` | freeboard 4.33 m at the carried draught 1.97 m >= required 0.3 m | freeboard 6.45 m at the carried draught 3.05 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 5.46 m (fluid), max GZ 2.29 m at 35° | GM 1.86 m (fluid), max GZ 1.36 m at 45° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 5.49 m (fluid), max GZ 2.28 m at 34° | GM 1.85 m (fluid), max GZ 1.34 m at 44° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 11.60 m (fluid), max GZ 2.81 m at 25° | GM 4.15 m (fluid), max GZ 1.47 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[27].message` | GM 6.87 m (fluid), max GZ 2.10 m at 25° | GM 5.81 m (fluid), max GZ 2.26 m at 32° | changed |
| sweep default depth gas 90x14 | `checks[28].message` | barge form factor at the service speed: Fn 0.185, Cp 0.967, L/B 5.64, B/T 7.05; outside its range: Fn 0.18 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 5.64 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.185, Cp 0.948, L/B 5.61, B/T 4.55; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 5.61 (range 7.59 to 16.29) | changed |
| sweep default depth gas 90x14 | `checks[31].message` | boil-off 151 kg/h > gas use 150 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 208 kg/h > gas use 135 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep default depth gas 90x14 | `checks[3].message` | freeboard 4.33 m >= required 0.3 m | freeboard 6.45 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[5].message` | 2 x 1.62 m thrusters need 5.04 m across a 14.00 m beam | 2 x 2.16 m thrusters need 6.12 m across a 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 5.04 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 6.12 m of 14.00 m beam | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].fill` | absent | 0.95 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].free_surface_moment_tm` | absent | 797.589333333 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].lcg_m` | absent | 37.944 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].mass_t` | absent | 111.33696 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].name` | absent | DB1P | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].tcg_m` | absent | -3.5 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].vcg_m` | absent | 0.285 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].fill` | absent | 0.95 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].free_surface_moment_tm` | absent | 797.589333333 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].lcg_m` | absent | 37.944 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].mass_t` | absent | 111.33696 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].name` | absent | DB1S | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].tcg_m` | absent | 3.5 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].vcg_m` | absent | 0.285 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].fill` | absent | 0.95 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].free_surface_moment_tm` | absent | 797.589333333 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].lcg_m` | absent | 65.848 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].mass_t` | absent | 111.33696 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].name` | absent | DB2P | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].tcg_m` | absent | -3.5 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].vcg_m` | absent | 0.285 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[3].fill` | absent | 0.95 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[3].free_surface_moment_tm` | absent | 797.589333333 | changed |
| ... | 2481 more changed quantities | | | |
