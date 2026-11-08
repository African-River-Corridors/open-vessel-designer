<!-- unreleased -->
- Arrangement search (issue #1031): the damage refinement of a (number across, number along) group stopped short of the largest size that passes. It built each size it tried from a side clearance worked out from its moving upper bound, not from the failing candidate's own. After the first size that failed, every later size came out as another diameter, was not offered, and pulled the upper bound down to the lower size untested. The 140 x 29 m coastal LNG reference design (tank top 2 m above the deck, two along) kept a 1-across 16.00 m tank (7,969 t) while a 2-across 11.41 m tank (8,064 t) passes damage. Each size is now built from the failing candidate's clearance. Two changes keep the damage-run limit (24 + 1) where the cargo is: a group is bisected once, and the groups are refined after the numbers along that carry more cargo have been screened, the largest first. The bisect still resolves the diameter to 0.1 m, so two searches with different size grids can end one step apart. Where the damage-run limit is hit, `limit_note` says so, as before. Damage runs on the four test hulls (140 x 29, 89 x 16, 110 x 12, 80 x 12 m, coastal, two along) were 8, 10, 11 and 11 and are now 10, 10, 11 and 12 (searched across, answer's run included); with the number along searched, 9, 17, 20 and 22 before and 8, 20, 18 and 19 now. No run reaches the limit of 24 + 1.

<!-- design-impact -->
### ticket/1031-refine-stops-short (2026-11-08)

Between `main b763c7d` and `this branch 399d9092`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas coastal-and-river 110x11.4 | `checks[14].message` | blind distance ahead of the bow 198.9 m <= 220 m, light (limit from the Rule Sets of the stack: solas/V.22.1.1/blind-zone-light); lift used 0.00 m of 0.00 m | blind distance ahead of the bow 215.4 m <= 220 m, light (limit from the Rule Sets of the stack: solas/V.22.1.1/blind-zone-light); lift used 0.00 m of 0.00 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[22].message` | tank length 19.04 m / inner diameter 8.77 m = 2.17 <= 7.0 | tank length 19.04 m / inner diameter 8.96 m = 2.13 <= 7.0 | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[23].message` | freeboard 3.27 m at the carried draught 2.73 m >= required 0.3 m | freeboard 3.20 m at the carried draught 2.80 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[24].message` | GM 1.43 m (fluid), max GZ 1.08 m at 36° | GM 1.33 m (fluid), max GZ 1.02 m at 36° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[25].message` | GM 1.42 m (fluid), max GZ 1.07 m at 36° | GM 1.32 m (fluid), max GZ 1.02 m at 35° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[26].message` | GM 5.04 m (fluid), max GZ 1.56 m at 25° | GM 4.92 m (fluid), max GZ 1.53 m at 25° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[28].message` | barge form factor at the service speed: Fn 0.163, Cp 0.964, L/B 8.92, B/T 4.15; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | barge form factor at the service speed: Fn 0.163, Cp 0.963, L/B 8.94, B/T 4.04; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), B/T 4.04 (range 4.05 to 8.12) | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[31].message` | boil-off 246 kg/h > gas use 140 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 251 kg/h > gas use 140 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[3].message` | freeboard 3.27 m >= required 0.3 m | freeboard 3.20 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[1].scrap_t` | 0.667986571217 | 2.61086157122 | +291 % |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[1].scrap_m2` | 8.50938307283 | 33.2593830728 | +291 % |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[8].scrap_m2` | 52.3757240027 | 26.002304893 | -50.4 % |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[8].scrap_t` | 35.9755754243 | 20.0035731542 | -44.4 % |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[8].bought_m2` | 144 | 108 | -25 % |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[8].count` | 4 | 3 | -25 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].capacity_m3` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[6].capacity_t` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].capacity_m3` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[7].capacity_t` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[2].capacity_m3` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[2].capacity_t` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[3].capacity_m3` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[3].capacity_t` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[4].capacity_m3` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[4].capacity_t` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].capacity_m3` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[5].capacity_t` | 47.579126395 | 35.743001395 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[0].capacity_m3` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[0].capacity_t` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[1].capacity_m3` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[1].capacity_t` | 45.3179760103 | 34.0443510103 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[1].net_m2` | 99.4906169272 | 74.7406169272 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[7].ordered_t` | 7.81001342878 | 5.86713842878 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[7].rule_net_t` | 7.81001342878 | 5.86713842878 | -24.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[60][1]` | 0.399594416532 | 0.338442567123 | -15.3 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[60][1]` | 0.399594416532 | 0.338442567123 | -15.3 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[60][1]` | 0.413707112207 | 0.352253950653 | -14.9 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[60][1]` | 0.413707112207 | 0.352253950653 | -14.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[3].trim_m` | 0.693715744786 | 0.590861856695 | -14.8 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[3].trim_m` | 0.693715744786 | 0.590861856695 | -14.8 % |
| ... | 764 more changed quantities | | | |
