<!-- unreleased -->
- Arrangement search (issue #935): when the stack screens by IGC 2.7 damage survival and the run works damage out, the search chooses the number of tanks along from 2, 3 and 4 (the profile's number first), by the most cargo among the candidates that pass damage; equal cargo goes to the fewest tanks, then the fewest along. A typed `tanks_in_line` pins it. The first evaluation (damage not run) keeps the profile's number, and the run that works damage out says what it changed, number along included (`damage_screening.tanks_along`). The whole run works out at most 24 damage runs over all numbers along, every share of the tank diameter and every depth pass, plus one for the answer; a later depth pass reuses the verdicts of the candidates an earlier one screened, so it keeps the pick the earlier pass found when the budget is gone. On the 140 x 29 x 12 m coastal LNG reference design the pick is two across by three along (13.38 m tanks, 23,101 m3) in place of two along (11.53 m, 18,324 m3). The size limits are provisional until #1009 (the free surface of the cargo in a breached tank) lands.

<!-- design-impact -->
### ticket/935-tanks-along (2026-11-08)

Between `main bc28adf` and `this branch 106ceaa8`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas coastal-and-river 110x11.4 | `checks[14].message` | blind distance ahead of the bow 158.5 m <= 220 m, light (limit from the Rule Sets of the stack: solas/V.22.1.1/blind-zone-light); lift used 0.00 m of 0.00 m | blind distance ahead of the bow 198.9 m <= 220 m, light (limit from the Rule Sets of the stack: solas/V.22.1.1/blind-zone-light); lift used 0.00 m of 0.00 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[22].message` | tank length 40.08 m / inner diameter 6.62 m = 6.05 <= 7.0 | tank length 19.04 m / inner diameter 8.77 m = 2.17 <= 7.0 | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[23].message` | freeboard 3.87 m at the carried draught 2.13 m >= required 0.3 m | freeboard 3.27 m at the carried draught 2.73 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[24].message` | GM 2.73 m (fluid), max GZ 1.57 m at 39° | GM 1.43 m (fluid), max GZ 1.08 m at 36° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[25].message` | GM 2.74 m (fluid), max GZ 1.56 m at 39° | GM 1.42 m (fluid), max GZ 1.07 m at 36° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[26].message` | GM 6.58 m (fluid), max GZ 1.86 m at 25° | GM 5.04 m (fluid), max GZ 1.56 m at 25° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[27].message` | GM 6.66 m (fluid), max GZ 1.86 m at 25° | GM 3.61 m (fluid), max GZ 1.25 m at 25° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[28].message` | barge form factor at the service speed: Fn 0.164, Cp 0.972, L/B 8.78, B/T 5.31; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88) | barge form factor at the service speed: Fn 0.163, Cp 0.964, L/B 8.92, B/T 4.15; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[31].message` | boil-off 199 kg/h > gas use 135 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 246 kg/h > gas use 140 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[3].message` | freeboard 3.87 m >= required 0.3 m | freeboard 3.27 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[9].message` | 2 tier(s) for rooms (97.1 m2 each, holding the 172.6 m2 of accommodation floor); over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | 3 tier(s): 2 for rooms (97.1 m2 each, holding the 172.6 m2 of accommodation floor) and 1 for access (24.6 m2 each, lifting the wheelhouse eye over the cargo); over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[4].kind` | peak_ballast | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[4].name` | FP1C | WB3P | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[4].side` | centre | port | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].capacity_m3` | absent | 47.579126395 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].capacity_t` | absent | 47.579126395 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].kind` | absent | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].lcg_m` | absent | 69.637 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].name` | absent | WB3S | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].side` | absent | starboard | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].tcg_m` | absent | 5.51157080127 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].vcg_m` | absent | 3 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].capacity_m3` | absent | 45.3179760103 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].capacity_t` | absent | 45.3179760103 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].kind` | absent | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].lcg_m` | absent | 90.179 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].name` | absent | WB4P | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].side` | absent | port | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].tcg_m` | absent | -5.51157080127 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].vcg_m` | absent | 3 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].capacity_m3` | absent | 45.3179760103 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].capacity_t` | absent | 45.3179760103 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].kind` | absent | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].lcg_m` | absent | 90.179 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].name` | absent | WB4S | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].side` | absent | starboard | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].tcg_m` | absent | 5.51157080127 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].vcg_m` | absent | 3 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[8].capacity_m3` | absent | 342 | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[8].capacity_t` | absent | 342 | changed |
| ... | 931 more changed quantities | | | |
