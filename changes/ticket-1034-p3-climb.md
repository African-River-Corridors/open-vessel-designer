<!-- unreleased -->
- The river share of the tank diameter now climbs to the most cargo (issue #1034, the maintainer's decision of 2026-11-09). With the share searched, the search starts at the tallest share that passes the intact checks, goes down one share at a time while the next share passes and its cargo rises by 1 % or more of the cargo in hand, and keeps the last share that rose. A tie, a smaller gain, a fall or an infeasible share stops it, so the taller share (less depth, less steel) wins. Damage screens the share kept, as before, under the same budget of 24 runs. `ovd run` says "the most cargo within the limits" on the share line, and names the tallest passing share and its cargo when the climb moved the share; the results carry `arrangement.share_climb_from` only then. River-only designs whose tallest passing share is not the one with the most cargo move; coastal-and-river designs and designs with a typed standing height do not search the share and do not move. In the design-impact sweep one case moves: "sweep default depth gas 90x14".

<!-- design-impact -->
### ticket/1034-p3-climb (2026-11-10)

Between `main 6b39849` and `this branch 0bc7bbd0`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep default depth gas 90x14 | `arrangement.share_climb_from.cargo_t` | absent | 963.599833045 | changed |
| sweep default depth gas 90x14 | `arrangement.share_climb_from.share` | absent | 0.5 | changed |
| sweep default depth gas 90x14 | `checks[12].message` | genset 2.89 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.20 m | genset 2.89 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 9.50 m | changed |
| sweep default depth gas 90x14 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 5.86 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 5.67 m of 6.00 m | changed |
| sweep default depth gas 90x14 | `checks[17].message` | light air draft 10.53 m <= limit 40.00 m (laden is 9.69 m and is NOT the governing case) | light air draft 13.69 m <= limit 40.00 m (laden is 12.13 m and is NOT the governing case) | changed |
| sweep default depth gas 90x14 | `checks[22].message` | tank length 27.74 m / inner diameter 7.52 m = 3.69 <= 7.0 | tank length 26.90 m / inner diameter 10.52 m = 2.56 <= 7.0 | changed |
| sweep default depth gas 90x14 | `checks[23].message` | freeboard 3.30 m at the carried draught 1.90 m >= required 0.3 m | freeboard 6.54 m at the carried draught 2.96 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 5.99 m (fluid), max GZ 2.37 m at 30° | GM 2.08 m (fluid), max GZ 1.35 m at 42° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 6.02 m (fluid), max GZ 2.36 m at 30° | GM 2.07 m (fluid), max GZ 1.33 m at 42° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 12.88 m (fluid), max GZ 3.05 m at 25° | GM 6.07 m (fluid), max GZ 2.29 m at 31° | changed |
| sweep default depth gas 90x14 | `checks[27].message` | GM 13.03 m (fluid), max GZ 3.05 m at 25° | GM 6.10 m (fluid), max GZ 2.28 m at 31° | changed |
| sweep default depth gas 90x14 | `checks[28].message` | barge form factor at the service speed: Fn 0.180, Cp 0.962, L/B 5.92, B/T 7.32; outside its range: Fn 0.18 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 5.92 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.180, Cp 0.924, L/B 5.93, B/T 4.70; outside its range: Fn 0.18 (range 0.00 to 0.14), Cp 0.92 (range 0.77 to 0.88), L/B 5.93 (range 7.59 to 16.29) | changed |
| sweep default depth gas 90x14 | `checks[31].message` | boil-off 155 kg/h > gas use 154 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 208 kg/h > gas use 118 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep default depth gas 90x14 | `checks[3].message` | freeboard 3.30 m >= required 0.3 m | freeboard 6.54 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[4].message` | cargo zone 57.49 m = LOA 90.0 - bow void 8.52 m - stern void 23.99 m | cargo zone 55.81 m = LOA 90.0 - bow void 10.20 m - stern void 23.99 m | changed |
| sweep default depth gas 90x14 | `checks[5].message` | 2 x 1.35 m thrusters need 4.50 m across a 14.00 m beam | 2 x 2.03 m thrusters need 5.85 m across a 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 5.85 m of 14.00 m beam | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].fill` | absent | 1 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].free_surface_moment_tm` | absent | 0 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].lcg_m` | absent | 37.944 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].mass_t` | absent | 117.1968 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].name` | absent | DB1P | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].tcg_m` | absent | -3.5 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[0].vcg_m` | absent | 0.3 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].fill` | absent | 1 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].free_surface_moment_tm` | absent | 0 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].lcg_m` | absent | 37.944 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].mass_t` | absent | 117.1968 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].name` | absent | DB1S | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].tcg_m` | absent | 3.5 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[1].vcg_m` | absent | 0.3 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].fill` | absent | 1 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].free_surface_moment_tm` | absent | 0 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].lcg_m` | absent | 65.848 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].mass_t` | absent | 117.1968 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].name` | absent | DB2P | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].tcg_m` | absent | -3.5 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[2].vcg_m` | absent | 0.3 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[3].fill` | absent | 1 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[2].ballast[3].free_surface_moment_tm` | absent | 0 | changed |
| ... | 2556 more changed quantities | | | |
