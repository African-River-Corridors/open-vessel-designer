<!-- unreleased -->
- IGC opening setback on the deckhouse, no setback void (issue #776, spec 0011 D17, D18). A gas hull keeps no empty length for the opening setback: the stern block starts at the hold's aft bulkhead, and the former 2.00 m service space becomes cargo zone. The 0.20 m end space is a hold end clearance inside the hold, not a cofferdam. A hold cofferdam is a core option that replaces the gastight A-60 bulkhead (IGC 3.1.2), never adds to it. Rule Set `igc-2g` 3.1.0 adds `igc/3.2.4.1/opening-setback`, `igc/3.2.4.1/air-inlet-setback` and `igc/3.1.2/hold-machinery-segregation`. The defaults of `entrance_distance_m` and `engine_room_vent_inlet_distance_m` are now 5 m, the most IGC 3.2.4.1 asks. `opening_setback_basis` no longer changes a result. The GA labels the END CLEARANCE; there is no SERVICE SPACE.

<!-- design-impact -->
### ticket/776-igc-opening-setback (2026-11-02)

Between `main 85f8ef9` and `this branch b402b66a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[15].message` | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | hold segregated from the stern block by a gastight A-60 bulkhead (IGC 3.1.2) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[15].name` | opening setback | hold machinery segregation | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[21].message` | tank length 27.29 m / inner diameter 6.80 m = 4.02 <= 7.0 | tank length 28.29 m / inner diameter 6.80 m = 4.16 <= 7.0 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 3.04 m at the carried draught 2.66 m >= required 0.3 m | freeboard 2.98 m at the carried draught 2.72 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 6.14 m (fluid), max GZ 2.58 m at 29° | GM 5.98 m (fluid), max GZ 2.54 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 6.16 m (fluid), max GZ 2.57 m at 29° | GM 5.99 m (fluid), max GZ 2.53 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.98 m (fluid), max GZ 3.50 m at 27° | GM 12.80 m (fluid), max GZ 3.48 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 13.09 m (fluid), max GZ 3.49 m at 27° | GM 12.91 m (fluid), max GZ 3.47 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[27].message` | barge form factor at the service speed: Fn 0.192, Cp 0.951, L/B 4.97, B/T 6.09; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 4.97 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.191, Cp 0.950, L/B 4.99, B/T 5.95; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 4.99 (range 7.59 to 16.29) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 3.04 m >= required 0.3 m | freeboard 2.98 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[4].message` | cargo zone 56.59 m = LOA 88.0 - bow void 9.32 m - stern void 22.09 m | cargo zone 58.59 m = LOA 88.0 - bow void 9.32 m - stern void 20.09 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[5].message` | 2 x 1.49 m thrusters need 4.77 m across a 16.30 m beam | 2 x 1.55 m thrusters need 4.91 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 4.77 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 4.91 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | blind distance ahead of the bow over 250 m, light (ES-TRIN 7.02(2)): lift needed 9.91 m of 6.00 m | blind distance ahead of the bow over 250 m, light (ES-TRIN 7.02(2)): lift needed 9.88 m of 6.00 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[16].message` | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | hold segregated from the stern block by a gastight A-60 bulkhead (IGC 3.1.2) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[16].name` | opening setback | hold machinery segregation | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[22].message` | tank length 48.29 m / inner diameter 10.05 m = 4.80 <= 7.0 | tank length 49.29 m / inner diameter 10.04 m = 4.91 <= 7.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[23].message` | freeboard 2.27 m at the carried draught 3.43 m >= required 0.3 m | freeboard 2.23 m at the carried draught 3.47 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[24].message` | GM 3.56 m (fluid), max GZ 1.33 m at 25° | GM 3.50 m (fluid), max GZ 1.30 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[25].message` | GM 3.56 m (fluid), max GZ 1.34 m at 25° | GM 3.50 m (fluid), max GZ 1.30 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 8.58 m (fluid), max GZ 2.87 m at 29° | GM 8.51 m (fluid), max GZ 2.86 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 8.62 m (fluid), max GZ 2.86 m at 29° | GM 8.55 m (fluid), max GZ 2.85 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[28].message` | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.69, B/T 4.73; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.70, B/T 4.68; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | freeboard 2.27 m >= required 0.3 m | freeboard 2.23 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].message` | cargo zone 98.59 m = LOA 130.0 - bow void 9.32 m - stern void 22.09 m | cargo zone 100.59 m = LOA 130.0 - bow void 9.32 m - stern void 20.09 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].message` | 2 x 2.03 m thrusters need 5.85 m across a 16.30 m beam | 2 x 2.09 m thrusters need 5.99 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].message` | 2 of 2 requested thrusters fit; 2 would need 5.85 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 5.99 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `stability.lowest_gm_condition` | Full load, departure | Full load, arrival | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[1]` | The blind distance needs 9.91 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | The blind distance needs 9.88 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[15].message` | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | hold segregated from the stern block by a gastight A-60 bulkhead (IGC 3.1.2) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[15].name` | opening setback | hold machinery segregation | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[21].message` | tank length 48.29 m / inner diameter 7.20 m = 6.71 <= 7.0 | tank length 49.29 m / inner diameter 7.20 m = 6.85 <= 7.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[22].message` | freeboard 2.27 m at the carried draught 3.43 m >= required 0.3 m | freeboard 2.24 m at the carried draught 3.46 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[23].message` | GM 5.21 m (fluid), max GZ 1.94 m at 27° | GM 5.15 m (fluid), max GZ 1.91 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[24].message` | GM 5.22 m (fluid), max GZ 1.94 m at 27° | GM 5.16 m (fluid), max GZ 1.91 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[25].message` | GM 10.51 m (fluid), max GZ 3.53 m at 30° | GM 10.48 m (fluid), max GZ 3.52 m at 30° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[26].message` | GM 10.56 m (fluid), max GZ 3.53 m at 30° | GM 10.54 m (fluid), max GZ 3.52 m at 30° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[27].message` | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.29, B/T 5.00; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 7.29 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.29, B/T 4.95; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 7.29 (range 7.59 to 16.29) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[3].message` | freeboard 2.27 m >= required 0.3 m | freeboard 2.24 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[4].message` | cargo zone 98.59 m = LOA 130.0 - bow void 9.32 m - stern void 22.09 m | cargo zone 100.59 m = LOA 130.0 - bow void 9.32 m - stern void 20.09 m | changed |
| ... | 107839 more changed quantities | | | |
