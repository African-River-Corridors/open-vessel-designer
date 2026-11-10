<!-- unreleased -->
- River-only Options need a number for the draught limit and the air-draft limit (issue #1034): typed, or filled by a Vessel Class. There is no "no limit" for them. A new Option, `ovd fill --as-new`, `ovd run`, `ovd validate`, the app and Python `simulate` give one message that names the limit that is not set (`simulate` raises `TypeError`, also for a key given as `None`). `ovd new` and `ovd fill` take `--draught-limit <m>` and `--air-draft-limit <m>`, in metres above 0. Plain `ovd fill` of a hand-written file gives the same message for a limit that is left out, or typed or bare `null`. Only a limit stored as `null` marked `default` (a file saved before this rule) keeps its results and hash, with one validation warning. The app shows the two fields with no value and no "none" choice for a river-only Option. An air-draft limit above a typed bridge clearance gives a warning.
- A deckhouse wheelhouse lifts only when a fixed one would break the air-draft limit (issue #1034, the maintainer's answer of 2026-11-09). A typed limit no longer makes it elevating by itself: where the fixed wheelhouse meets the limit, it stays fixed. The rule has no route condition, so it holds for coastal-and-river Options too. The run's `wheelhouse_basis` reads `limit_not_binding` for a fixed wheelhouse under a limit and `air_draft_limit` for one the limit lifted. Moves against main as shipped:
  - A river-only deckhouse design under a loose limit stays where main with no limit put it. "sweep gas river-only deckhouse 140x29" (limit 40 m) keeps its fixed wheelhouse, light air draft 24.29 m; the sweep set had given main a 40 m limit, which lifted it to 16.52 m. That is a move of the sweep, not of a main design.
  - A coastal-and-river deckhouse design with a typed loose limit now stays fixed. 140 x 29 x 9 m LNG, limit 40 m: main lifts the wheelhouse (6,783 t, 1 tier); this branch keeps it fixed (10,987 t, 5 tiers, a "deckhouse tier count" warning, still feasible). It is the same design as main with no limit.
  - A typed `wheelhouse_max_lift_m` of 0 with a limit now runs the two-pass tier sizing. 140 x 29 x 12 m LNG, limit 22 m: 7,630 t on main, 8,351 t here (3 tiers both). At 40 m nothing moves.
  - A gas design with its moulded depth left out and a draught limit takes its depth from the limit, and the limit is now required. "sweep default depth gas 90x14" (90 x 14 m LNG, limit 6.0 m): depth 6.3 m set by freeboard, 930.7 t; main with no limit: 5.2 m set by tank, 963.6 t. This is a real Phase 2 effect and it moves for any limit number. The dry-bulk and liquid default-depth designs keep their results; only the air-draft check and its flags are new.
  - "sweep optional deckhouse_openings" loses its lift column (lift allowed 0.00 m, was 6.00 m, used 0.00 m both times) and its figures move in the third digit.

<!-- design-impact -->
### ticket/1034-p2-required-limits (2026-11-10)

Between `main 6b39849` and `this branch 1ae9234c`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas river-only deckhouse 140x29 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 5.49 m of 6.00 m | blind distance ahead of the bow 200.8 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 0.00 m of 0.00 m | changed |
| sweep gas river-only deckhouse 140x29 | `checks[17].message` | light air draft 16.52 m <= limit 40.00 m (laden is 13.97 m and is NOT the governing case) | light air draft 24.29 m <= limit 40.00 m (laden is 21.53 m and is NOT the governing case) | changed |
| sweep gas river-only deckhouse 140x29 | `checks[22].message` | tank length 54.88 m / inner diameter 11.41 m = 4.81 <= 7.0 | tank length 54.88 m / inner diameter 11.91 m = 4.61 <= 7.0 | changed |
| sweep gas river-only deckhouse 140x29 | `checks[23].message` | freeboard 7.97 m at the carried draught 4.03 m >= required 0.3 m | freeboard 7.73 m at the carried draught 4.27 m >= required 0.3 m | changed |
| sweep gas river-only deckhouse 140x29 | `checks[24].message` | GM 13.44 m (fluid), max GZ 5.79 m at 35° | GM 12.47 m (fluid), max GZ 5.60 m at 35° | changed |
| sweep gas river-only deckhouse 140x29 | `checks[25].message` | GM 13.47 m (fluid), max GZ 5.79 m at 35° | GM 12.49 m (fluid), max GZ 5.60 m at 35° | changed |
| sweep gas river-only deckhouse 140x29 | `checks[26].message` | GM 25.45 m (fluid), max GZ 7.57 m at 35° | GM 25.15 m (fluid), max GZ 7.52 m at 35° | changed |
| sweep gas river-only deckhouse 140x29 | `checks[27].message` | GM 25.57 m (fluid), max GZ 7.57 m at 35° | GM 25.26 m (fluid), max GZ 7.51 m at 35° | changed |
| sweep gas river-only deckhouse 140x29 | `checks[28].message` | barge form factor at the service speed: Fn 0.147, Cp 0.970, L/B 4.33, B/T 7.16; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 4.33 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.146, Cp 0.968, L/B 4.34, B/T 6.77; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 4.34 (range 7.59 to 16.29) | changed |
| sweep gas river-only deckhouse 140x29 | `checks[31].message` | boil-off 925 kg/h > gas use 487 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 965 kg/h > gas use 486 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas river-only deckhouse 140x29 | `checks[3].message` | freeboard 7.97 m >= required 0.3 m | freeboard 7.73 m >= required 0.3 m | changed |
| sweep gas river-only deckhouse 140x29 | `checks[9].message` | 1 tier(s) for rooms (98.6 m2 each, holding the 63.4 m2 of accommodation floor); over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | 4 tier(s): 1 for rooms (98.6 m2 each, holding the 63.4 m2 of accommodation floor) and 3 for access (24.6 m2 each, lifting the wheelhouse eye over the cargo); over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[10].label` | electric motors | bow thruster | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[11].label` | bow thruster | small items | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[12].label` | small items | piping and cabling | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[13].label` | piping and cabling | engine-room steel | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[14].label` | engine-room steel | engine seats | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[15].kcg_m` | 1.10333333333 | absent | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[15].label` | engine seats | absent | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[15].lcg_m` | 11.616 | absent | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[15].mass_t` | 3.2 | absent | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[2].label` | lift column | gensets | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[3].label` | gensets | gas valve units | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[4].label` | gas valve units | gas heater | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[5].label` | gas heater | gas compressor | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[6].label` | gas compressor | propulsors | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[7].label` | propulsors | switchboards | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[8].label` | switchboards | frequency converters | changed |
| sweep gas river-only deckhouse 140x29 | `stern_block_mass.items[9].label` | frequency converters | electric motors | changed |
| sweep gas river-only deckhouse 140x29 | `structure.purchase.sections[9].section` | T 400x9.5+180x18 | T 400x9.5+130x13 | changed |
| sweep optional deckhouse_openings | `checks[14].message` | blind distance ahead of the bow 156.4 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 0.00 m of 6.00 m | blind distance ahead of the bow 154.6 m <= 250 m, light (limit from the Rule Sets of the stack: es-trin/7.02(2)/blind-zone-unladen); lift used 0.00 m of 0.00 m | changed |
| sweep optional deckhouse_openings | `checks[15].message` | light air draft 9.60 m <= limit 40.00 m (laden is 6.15 m and is NOT the governing case) | light air draft 9.61 m <= limit 40.00 m (laden is 6.15 m and is NOT the governing case) | changed |
| sweep optional deckhouse_openings | `checks[16].message` | no trim balances the load: Laden: LCG 54.42 m is forward of the furthest LCB the hull reaches (47.98 m) | no trim balances the load: Laden: LCG 54.46 m is forward of the furthest LCB the hull reaches (48.02 m) | changed |
| sweep optional deckhouse_openings | `checks[17].message` | intact GM 3.61 m >= 0.15 m at the reported cargo 3821 t, draught 4.20 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 3.61 m >= 0.15 m at the reported cargo 3826 t, draught 4.20 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep optional deckhouse_openings | `checks[22].message` | GM 17.68 m (fluid), max GZ 3.81 m at 27° | GM 17.81 m (fluid), max GZ 3.83 m at 27° | changed |
| sweep optional deckhouse_openings | `checks[23].message` | GM 18.04 m (fluid), max GZ 3.81 m at 26° | GM 18.18 m (fluid), max GZ 3.83 m at 26° | changed |
| sweep optional deckhouse_openings | `stern_block_mass.items[10].label` | small items | piping and cabling | changed |
| sweep optional deckhouse_openings | `stern_block_mass.items[11].label` | piping and cabling | engine-room steel | changed |
| sweep optional deckhouse_openings | `stern_block_mass.items[12].label` | engine-room steel | engine seats | changed |
| sweep optional deckhouse_openings | `stern_block_mass.items[13].kcg_m` | 0.8 | absent | changed |
| ... | 1222 more changed quantities | | | |
