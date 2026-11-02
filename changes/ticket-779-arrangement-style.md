<!-- unreleased -->
- Arrangement Style and the deckhouse (issue #779, spec 0011 D3, D4, D6): a new input `arrangement_style` picks `container_block` (today's stern block, exactly; the value of every file saved before and of a river-only Option) or `deckhouse` (the default of a new coastal Option). A deckhouse stern has a deckhouse of tiers over the machinery space that holds the accommodation (aft only; the 40 ft accommodation units leave the deck), and a wheelhouse on its top: fixed, with tiers added until its eye clears the cargo and the Blind Distance limit, or elevating when an air-draft limit is set. The deckhouse and wheelhouse steel is volume x 0.057 t/m3 (Schneekluth and Bertram Table 5.2, an unverified estimate; no outfit item, no source rate was found), the deckhouse floor area prices the deckhouse cost line in place of the wheelhouse and accommodation lump, and its volume joins the gross tonnage. The tier height, side deck, tier limit and the accommodation floor each tier holds (the floor of the 40 ft accommodation units, until issue #780 sizes the rooms) are estimates. The GA and the 3D view do not draw the deckhouse yet (issue #783). A coastal deckhouse sizes its fixed wheelhouse to the stricter of ES-TRIN 7.02(2) (250 m) and SOLAS V/22.1.1 (min(2 L, 500 m)), in two passes (keel waterline, then the trimmed light waterline). A deckhouse keeps half of the wheelhouse and accommodation cost lump for the wheelhouse and navigation (an estimate). The deckhouse carries no outfit mass until the calibration set (issue #782), so lightship is under-stated and GM over-stated (a results warning says so). Stored results of the container block do not move.

<!-- design-impact -->
### ticket/779-arrangement-style (2026-11-02)

Between `main e8c9e15` and `this branch a6624f88`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep crude coastal service 90x14 | `checks[10].message` | 2 x 1.71 m engines abreast need 6.41 m across a 14.00 m beam | deckhouse 18.60 m over the machinery space x 12.40 m wide (the 14.00 m beam less a side deck each side) | changed |
| sweep crude coastal service 90x14 | `checks[10].name` | machinery fits beam | deckhouse has a footprint | changed |
| sweep crude coastal service 90x14 | `checks[11].message` | engine 1.59 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 4.50 m | 1 tier(s) of 230.6 m2 hold the 29.7 m2 of accommodation floor and, for a fixed wheelhouse, lift the eye over the cargo; over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | changed |
| sweep crude coastal service 90x14 | `checks[11].name` | machinery fits under deck | deckhouse tier count | changed |
| sweep crude coastal service 90x14 | `checks[12].message` | blind distance ahead of the bow 0.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 0.00 m of 6.00 m | 2 x 1.71 m engines abreast need 6.41 m across a 14.00 m beam | changed |
| sweep crude coastal service 90x14 | `checks[12].name` | wheelhouse sees over cargo | machinery fits beam | changed |
| sweep crude coastal service 90x14 | `checks[13].message` | the hull floats within 10 degrees of trim (every condition) | engine 1.59 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 4.50 m | changed |
| sweep crude coastal service 90x14 | `checks[13].name` | trim in range | machinery fits under deck | changed |
| sweep crude coastal service 90x14 | `checks[13].severity` | error | warning | changed |
| sweep crude coastal service 90x14 | `checks[14].message` | barge form factor at the service speed: Fn 0.176, Cp 0.840, L/B 6.22, B/T 5.15; outside its range: Fn 0.18 (range 0.00 to 0.14), L/B 6.22 (range 7.59 to 16.29) | blind distance ahead of the bow 0.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 0.00 m of 0.00 m | changed |
| sweep crude coastal service 90x14 | `checks[14].name` | resistance method in range[Deep water] | wheelhouse sees over cargo | changed |
| sweep crude coastal service 90x14 | `checks[14].ok` | False | True | changed |
| sweep crude coastal service 90x14 | `checks[14].severity` | warning | error | changed |
| sweep crude coastal service 90x14 | `checks[15].message` | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | the hull floats within 10 degrees of trim (every condition) | changed |
| sweep crude coastal service 90x14 | `checks[15].name` | coefficients are first order | trim in range | changed |
| sweep crude coastal service 90x14 | `checks[15].severity` | warning | error | changed |
| sweep crude coastal service 90x14 | `checks[16].message` | GM 5.29 m (fluid), max GZ 1.69 m at 26° | barge form factor at the service speed: Fn 0.176, Cp 0.839, L/B 6.22, B/T 5.21; outside its range: Fn 0.18 (range 0.00 to 0.14), L/B 6.22 (range 7.59 to 16.29) | changed |
| sweep crude coastal service 90x14 | `checks[16].name` | Stability — full load, departure | resistance method in range[Deep water] | changed |
| sweep crude coastal service 90x14 | `checks[16].ok` | True | False | changed |
| sweep crude coastal service 90x14 | `checks[17].message` | GM 5.31 m (fluid), max GZ 1.68 m at 26° | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | changed |
| sweep crude coastal service 90x14 | `checks[17].name` | Stability — full load, arrival | coefficients are first order | changed |
| sweep crude coastal service 90x14 | `checks[18].message` | GM 16.33 m (fluid), max GZ 3.72 m at 27° | GM 5.89 m (fluid), max GZ 1.96 m at 28° | changed |
| sweep crude coastal service 90x14 | `checks[18].name` | Stability — ballast, departure | Stability — full load, departure | changed |
| sweep crude coastal service 90x14 | `checks[19].message` | GM 16.60 m (fluid), max GZ 3.72 m at 27° | GM 5.92 m (fluid), max GZ 1.95 m at 28° | changed |
| sweep crude coastal service 90x14 | `checks[19].name` | Stability — ballast, arrival | Stability — full load, arrival | changed |
| sweep crude coastal service 90x14 | `checks[20].message` | GM 5.42 m (fluid), max GZ 1.77 m at 26° | GM 16.11 m (fluid), max GZ 3.69 m at 27° | changed |
| sweep crude coastal service 90x14 | `checks[20].name` | Stability — slack cargo tanks, worst case | Stability — ballast, departure | changed |
| sweep crude coastal service 90x14 | `checks[21].message` | absent | GM 16.37 m (fluid), max GZ 3.69 m at 27° | changed |
| sweep crude coastal service 90x14 | `checks[21].name` | absent | Stability — ballast, arrival | changed |
| sweep crude coastal service 90x14 | `checks[21].ok` | absent | True | changed |
| sweep crude coastal service 90x14 | `checks[21].severity` | absent | warning | changed |
| sweep crude coastal service 90x14 | `checks[22].message` | absent | GM 5.47 m (fluid), max GZ 1.79 m at 26° | changed |
| sweep crude coastal service 90x14 | `checks[22].name` | absent | Stability — slack cargo tanks, worst case | changed |
| sweep crude coastal service 90x14 | `checks[22].ok` | absent | True | changed |
| sweep crude coastal service 90x14 | `checks[22].severity` | absent | warning | changed |
| sweep crude coastal service 90x14 | `checks[4].message` | cargo zone 52.37 m = LOA 90.0 - bow void 7.20 m - stern void 30.43 m | cargo zone 51.21 m = LOA 90.0 - bow void 7.20 m - stern void 31.59 m | changed |
| sweep crude coastal service 90x14 | `checks[7].message` | 3 x 40 ft units abreast need 10.91 m across a 14.00 m beam | 2 x 40 ft units abreast need 7.58 m across a 14.00 m beam | changed |
| sweep crude coastal service 90x14 | `checks[8].message` | 3 units in 1 row(s) of 3; a second row adds 12.99 m to the stern block and comes straight off the cargo zone | 2 units in 1 row(s) of 2; a second row adds 12.99 m to the stern block and comes straight off the cargo zone | changed |
| sweep crude coastal service 90x14 | `stability.lowest_gm_condition` | Full load, departure | Slack cargo tanks, worst case | changed |
| sweep crude coastal service 90x14 | `stern_block_mass.deckhouse_steel.outfit` | absent | not in the mass: no outfit rate was found in a source read (spec 0011 D7); the calibration set (D11) gives one | changed |
| ... | 3069 more changed quantities | | | |
