<!-- unreleased -->
- Arrangement search (issue #935): on a stack that evaluates IGC 2.7 damage survival (`igc-2g` 3.0.0 and later), the search runs the damage cases on each candidate that passes the cheap checks and stability, from the most cargo down, and the first that also passes damage wins. A candidate turned down by damage names the Rule, the condition and the case in the list of arrangements tried. On the 140 x 29 x 12 m coastal LNG reference design a cap of 12 m or 13 m now gives 11.5 m tanks (it gave 12.0 m with 4 failing damage rows and 13.0 m with 24). A run that skips damage does not screen. The damage progress hook now reports one count per damage run (the search's runs, then the answer's). The size limit is provisional until #1009 (the free surface of the cargo in a breached tank) lands.

<!-- design-impact -->
### ticket/935-damage-search-2 (2026-11-07)

Between `main 72eb5da` and `this branch 20b411b9`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas coastal-and-river 110x11.4 | `checks[14].message` | blind distance ahead of the bow 154.6 m <= 220 m, light (limit from the Rule Sets of the stack: solas/V.22.1.1/blind-zone-light); lift used 0.00 m of 0.00 m | blind distance ahead of the bow 158.5 m <= 220 m, light (limit from the Rule Sets of the stack: solas/V.22.1.1/blind-zone-light); lift used 0.00 m of 0.00 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[22].message` | tank length 40.08 m / inner diameter 9.62 m = 4.17 <= 7.0 | tank length 40.08 m / inner diameter 6.62 m = 6.05 <= 7.0 | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[23].message` | freeboard 2.59 m at the carried draught 3.41 m >= required 0.3 m | freeboard 3.87 m at the carried draught 2.13 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[24].message` | GM 0.80 m (fluid), max GZ 0.64 m at 34° | GM 2.73 m (fluid), max GZ 1.57 m at 39° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[25].message` | GM 0.79 m (fluid), max GZ 0.63 m at 34° | GM 2.74 m (fluid), max GZ 1.56 m at 39° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[26].message` | GM 4.79 m (fluid), max GZ 1.64 m at 28° | GM 6.58 m (fluid), max GZ 1.86 m at 25° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[27].message` | GM 4.83 m (fluid), max GZ 1.63 m at 27° | GM 6.66 m (fluid), max GZ 1.86 m at 25° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[28].message` | barge form factor at the service speed: Fn 0.161, Cp 0.956, L/B 9.08, B/T 3.33; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), B/T 3.33 (range 4.05 to 8.12) | barge form factor at the service speed: Fn 0.164, Cp 0.972, L/B 8.78, B/T 5.31; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88) | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[31].message` | boil-off 285 kg/h > gas use 146 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 199 kg/h > gas use 135 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[3].message` | freeboard 2.59 m >= required 0.3 m | freeboard 3.87 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[9].message` | 4 tier(s): 2 for rooms (97.1 m2 each, holding the 172.6 m2 of accommodation floor) and 2 for access (24.6 m2 each, lifting the wheelhouse eye over the cargo); over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | 2 tier(s) for rooms (97.1 m2 each, holding the 172.6 m2 of accommodation floor); over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[0].kind` | double_bottom_ballast | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[0].name` | DB1P | WB1P | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[1].kind` | double_bottom_ballast | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[1].name` | DB1S | WB1S | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[2].kind` | double_bottom_ballast | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[2].name` | DB2P | WB2P | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[3].kind` | double_bottom_ballast | wing_ballast | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.ballast_tanks[3].name` | DB2S | WB2S | changed |
| sweep gas coastal-and-river 110x11.4 | `stability.lowest_gm_condition` | Full load, arrival | Full load, departure | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[10].region` | cargo zone | end zones | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[11].member` | WEB_FRAME: Side | WEB_FRAME: Deck transverse frame | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[11].region` | end zones |  | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[12].member` | WEB_FRAME: Deck transverse frame | TRANSVERSAL_BULKHEAD | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[13].member` | TRANSVERSAL_BULKHEAD | SHELL | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[13].region` |  | end plating | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].centre_m[0]` | 0 | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].centre_m[1]` | 0 | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].centre_m[2]` | 5.85 | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].items` | 1 | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].member` | SHELL | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].ordered_t` | 0.3087405 | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].region` | end plating | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[14].rule_net_t` | 0.3087405 | absent | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[6].member` | SHELL: Inner bottom shell | SHELL: Inner side shell | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[7].member` | LONGITUDINAL: Centerline girder | DECK: Platform deck | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[8].member` | LONGITUDINAL: Side girder | WEB_FRAME: General web-frame | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[9].member` | WEB_FRAME: Floor frame | WEB_FRAME: Side | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.groups.members[9].region` |  | cargo zone | changed |
| sweep gas coastal-and-river 110x11.4 | `structure.purchase.plates[8].bought_m2` | absent | 288 | changed |
| ... | 904 more changed quantities | | | |
