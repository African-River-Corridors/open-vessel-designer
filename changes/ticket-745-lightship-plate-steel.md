<!-- unreleased -->
- Hull steel drives the lightship (issue #745, spec 0013 T7, D9, D16): a gas hull's lightship and cost take the plate model's ordered weight plus its allowance, the plate model laid out on the hull's Sections Table and its hold bulkheads where the Vessel Model puts them. The panel table is now the plate model rolled up, one row per member group at the group's ordered weight. The scantlings take the rule length of ABS 3-1-1/3.1 on the hull's own sections. On the 140 × 29 m worked example the lightship falls 266 t (Option A), 267 t (B) and 385 t (C); draught, stability, cost and power move with it; tanks and cargo do not. The lightship residuals of `calibration/hull_steel_cases.yaml` are re-reported, not tuned.
- Steel list storage (issue #745): the results keep the steel summary in `structure` (groups, totals, allowance, scrap, plates bought), in the results hash. The per-item list moves beside it as `steel_list`, outside the hash like `structure_geometry`. `ovd run --json` leaves both out unless `--steel-list` is given, so a committed results file stores the summary and its hash covers what it stores. `ovd export --format steel-csv` still writes the full list. In the table below, the `structure.steel_list` rows (3,256,915 of the 3,343,272 changed quantities) are the list leaving `structure`, not a design change. On 116 of 261 cases, all gas hulls, the rest follow the lighter hull: draught, hydrostatics, ballast plans, stability, sightline, the default propeller sized to the light draught (issue #355) and with it installed power, gas use and the stern block mass. One check changes outcome: the default-depth 90 × 14 m gas sweep case's boil-off now exceeds its gas use at the service speed.

<!-- design-impact -->
### ticket/745-lightship-plate-steel (2026-11-02)

Between `main aacda4a` and `this branch c6a0b63a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.54 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.55 m of 6.00 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 3.04 m at the carried draught 2.66 m >= required 0.3 m | freeboard 3.07 m at the carried draught 2.63 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 6.13 m (fluid), max GZ 2.56 m at 29° | GM 6.21 m (fluid), max GZ 2.59 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 6.14 m (fluid), max GZ 2.55 m at 29° | GM 6.23 m (fluid), max GZ 2.58 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.88 m (fluid), max GZ 3.49 m at 27° | GM 13.30 m (fluid), max GZ 3.51 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.99 m (fluid), max GZ 3.49 m at 27° | GM 13.42 m (fluid), max GZ 3.50 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[27].message` | barge form factor at the service speed: Fn 0.192, Cp 0.951, L/B 4.98, B/T 6.08; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 4.98 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.192, Cp 0.952, L/B 4.97, B/T 6.17; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 4.97 (range 7.59 to 16.29) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 3.04 m >= required 0.3 m | freeboard 3.07 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].bought_m2` | 864 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].count` | 24 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].net_m2` | 585.826402621 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].ordered_mm` | 21 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].scrap_m2` | 278.173597379 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].scrap_t` | 45.8569175279 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].stock_m[0]` | 3 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[8].stock_m[1]` | 12 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].bought_m2` | 180 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].count` | 5 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].net_m2` | 121.476540689 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].ordered_mm` | 34.5 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].scrap_m2` | 58.5234593109 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].scrap_t` | 15.8496158679 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].stock_m[0]` | 3 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.purchase.plates[9].stock_m[1]` | 12 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].area_m2` | 36 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].basis` | rule | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].centre_m[0]` | 32.4 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].centre_m[1]` | 1.5 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].centre_m[2]` | 0 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].edition` | ABS Rules for Building and Classing Steel Barges, January 2026 (Part 3 January 2026, Part 5 July 2025) | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].grade` | ordinary | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].id` | PL1 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].kind` | plate | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].length_m` | 12 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].member` | SHELL: Bottom shell | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].ordered_mm` | 11 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].ordered_t` | 3.1086 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].paragraph` | 3-2-2/3.5.1 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].pieces` | 1 | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `structure.steel_list[0].region` | midship 0.4L | absent | changed |
| ... | 3343232 more changed quantities | | | |
