<!-- unreleased -->
- IGC opening setback on the deckhouse, no setback void (issue #776, spec 0011 D17, D18). A gas hull keeps no empty length for the opening setback: the stern block starts at the hold's aft bulkhead, and the former 2.00 m service space becomes cargo zone. The 0.20 m end space is a hold end clearance inside the hold, not a cofferdam. A hold cofferdam is a core option that replaces the gastight A-60 bulkhead (IGC 3.1.2), never adds to it. Rule Set `igc-2g` 3.1.0 adds `igc/3.2.4.1/opening-setback`, `igc/3.2.4.1/air-inlet-setback` and `igc/3.1.2/hold-machinery-segregation`. The defaults of `entrance_distance_m` and `engine_room_vent_inlet_distance_m` are now 5 m, the most IGC 3.2.4.1 asks. `opening_setback_basis` no longer changes a result. The GA labels the END CLEARANCE; there is no SERVICE SPACE.

<!-- design-impact -->
### ticket/776-igc-opening-setback (2026-11-02)

Between `main bb6a163` and `this branch ff65ac1d`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[12].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.46 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.55 m of 6.00 m | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[14].message` | intact GM 4.82 m >= 0.15 m at the reported cargo 3057 t, draught 3.80 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 4.79 m >= 0.15 m at the reported cargo 3057 t, draught 3.83 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[15].message` | barge form factor at the service speed: Fn 0.169, Cp 0.845, L/B 5.20, B/T 4.30; outside its range: Fn 0.17 (range 0.00 to 0.14), L/B 5.20 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.169, Cp 0.846, L/B 5.21, B/T 4.27; outside its range: Fn 0.17 (range 0.00 to 0.14), L/B 5.21 (range 7.59 to 16.29) | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[17].message` | GM 4.80 m (fluid), max GZ 1.30 m at 25° | GM 4.76 m (fluid), max GZ 1.30 m at 25° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[18].message` | GM 4.81 m (fluid), max GZ 1.28 m at 25° | GM 4.77 m (fluid), max GZ 1.29 m at 25° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[19].message` | GM 15.80 m (fluid), max GZ 4.16 m at 31° | GM 15.47 m (fluid), max GZ 4.13 m at 31° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[20].message` | GM 15.95 m (fluid), max GZ 4.15 m at 30° | GM 15.60 m (fluid), max GZ 4.13 m at 31° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[3].message` | freeboard 2.02 m >= required 0.3 m | freeboard 1.99 m >= required 0.3 m | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[12].kcg_m` | absent | 3.3 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[12].label` | absent | engine-room steel | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[12].lcg_m` | absent | 15.7636120126 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[12].mass_t` | absent | 33.7413072095 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[13].kcg_m` | absent | 0.8 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[13].label` | absent | engine seats | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[13].lcg_m` | absent | 15.7636120126 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[13].mass_t` | absent | 0.973747503982 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.engine_room_rate_basis` | absent | Schneekluth and Bertram, Ship Design for Efficiency and Economy, sec. 5, Table 5.2, p. 164: deckhouse coefficient C_DH, layer I, Fo/Fu = 1.0, taken for the engine room (edition not shown on the pages read) | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.engine_room_rate_status` | absent | unverified estimate | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.engine_room_rate_t_per_m3` | absent | 0.057 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.engine_room_steel_t` | absent | 33.7413072095 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.engine_room_volume_m3` | absent | 591.952758061 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.prime_movers.count` | absent | 2 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.prime_movers.kw_each` | absent | 575.779570032 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.prime_movers.rpm` | absent | 1800 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.prime_movers.rpm_status` | absent | estimate | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.seats_basis` | absent | Schneekluth and Bertram, sec. 5.1, p. 158: W = 27 P_B / ((n + 250) (15 + P_B / 1000)) t per engine | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.seats_t` | absent | 0.973747503982 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.machinery_steel.watson_cross_check` | absent | none | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[12].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.27 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.36 m of 6.00 m | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[14].message` | intact GM 3.81 m >= 0.15 m at the reported cargo 5935 t, draught 4.29 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 3.82 m >= 0.15 m at the reported cargo 5896 t, draught 4.29 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[15].message` | barge form factor at the service speed: Fn 0.138, Cp 0.905, L/B 7.83, B/T 3.79; outside its range: Fn 0.14 (range 0.00 to 0.14), Cp 0.91 (range 0.77 to 0.88), B/T 3.79 (range 4.05 to 8.12) | barge form factor at the service speed: Fn 0.138, Cp 0.906, L/B 7.83, B/T 3.79; outside its range: Fn 0.14 (range 0.00 to 0.14), Cp 0.91 (range 0.77 to 0.88), B/T 3.79 (range 4.05 to 8.12) | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[17].message` | GM 3.79 m (fluid), max GZ 0.84 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.79 m (fluid), max GZ 0.86 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[18].message` | GM 3.79 m (fluid), max GZ 0.84 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.79 m (fluid), max GZ 0.85 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[19].message` | GM 16.34 m (fluid), max GZ 4.12 m at 29° | GM 16.06 m (fluid), max GZ 4.10 m at 30° | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[20].message` | GM 16.49 m (fluid), max GZ 4.12 m at 29° | GM 16.20 m (fluid), max GZ 4.10 m at 29° | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[3].message` | freeboard 1.46 m >= required 0.3 m | freeboard 1.44 m >= required 0.3 m | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[12].kcg_m` | absent | 3.3 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[12].label` | absent | engine-room steel | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[12].lcg_m` | absent | 15.9336110575 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[12].mass_t` | absent | 35.3207683351 | changed |
| ... | 3428052 more changed quantities | | | |
