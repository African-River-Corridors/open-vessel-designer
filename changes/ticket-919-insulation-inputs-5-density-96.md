<!-- unreleased -->
- Design change (issue #919): the default insulation density of the type C tanks is 96 kg/m3 for the insulation panel and polyurethane foam (the Defaults Profile table of `tank_insulation_density_kg_m3`; was 60 kg/m3, carried over from the original engine with no source). 96 kg/m3 is the foam the conductivity (0.026 W/m.K, Lee et al., J. Mar. Sci. Eng. 2021, 9, 36, Table 3) is for; the maintainer decided it on #919. The 140 x 29 m coastal LNG reference design's insulation goes from 133.8 t to 214.2 t (about +80 t), and its lightship and draught move with it, the cargo does not (table below). A file saved before the input keeps 60 kg/m3. A pressurised LPG tank has no insulation and does not move.

<!-- design-impact -->
### ticket/919-insulation-inputs-5-density-96 (2026-11-07)

Between `main 669689c` and `this branch 85075cb1`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep default depth gas 90x14 | `checks[22].message` | freeboard 5.34 m at the carried draught 2.36 m >= required 0.3 m | freeboard 5.32 m at the carried draught 2.38 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[23].message` | GM 3.67 m (fluid), max GZ 1.84 m at 37° | GM 3.62 m (fluid), max GZ 1.83 m at 37° | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 3.68 m (fluid), max GZ 1.82 m at 36° | GM 3.63 m (fluid), max GZ 1.81 m at 37° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 5.33 m (fluid), max GZ 1.76 m at 25° | GM 5.24 m (fluid), max GZ 1.75 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 7.11 m (fluid), max GZ 2.63 m at 34° | GM 5.27 m (fluid), max GZ 1.74 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[27].message` | barge form factor at the service speed: Fn 0.186, Cp 0.964, L/B 5.56, B/T 5.87; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 5.56 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.186, Cp 0.964, L/B 5.57, B/T 5.83; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 5.57 (range 7.59 to 16.29) | changed |
| sweep default depth gas 90x14 | `checks[3].message` | freeboard 5.34 m >= required 0.3 m | freeboard 5.32 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.00 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.99 m of 6.00 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[22].message` | freeboard 3.21 m at the carried draught 1.79 m >= required 0.3 m | freeboard 3.20 m at the carried draught 1.80 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[23].message` | GM 3.57 m (fluid), max GZ 1.65 m at 33° | GM 3.52 m (fluid), max GZ 1.64 m at 33° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[24].message` | GM 3.59 m (fluid), max GZ 1.64 m at 33° | GM 3.54 m (fluid), max GZ 1.63 m at 33° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[25].message` | GM 6.93 m (fluid), max GZ 2.00 m at 26° | GM 6.80 m (fluid), max GZ 1.98 m at 27° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[26].message` | GM 7.02 m (fluid), max GZ 1.99 m at 26° | GM 6.88 m (fluid), max GZ 1.98 m at 26° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[27].message` | barge form factor at the service speed: Fn 0.182, Cp 0.971, L/B 7.18, B/T 6.31; outside its range: Fn 0.18 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 7.18 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.182, Cp 0.970, L/B 7.18, B/T 6.26; outside its range: Fn 0.18 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 7.18 (range 7.59 to 16.29) | changed |
| sweep downflooding opening gas 90x11.4 | `checks[30].message` | boil-off 124 kg/h <= gas use 125 kg/h at the service speed | boil-off 124 kg/h <= gas use 126 kg/h at the service speed | changed |
| sweep downflooding opening gas 90x11.4 | `checks[3].message` | freeboard 3.21 m >= required 0.3 m | freeboard 3.20 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[0].criteria[1].name` | Area 0–31.9° | Area 0–31.7° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[0].criteria[2].name` | Area 30–31.9° | Area 30–31.7° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].criteria[1].name` | Area 0–32.1° | Area 0–31.9° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].criteria[2].name` | Area 30–32.1° | Area 30–31.9° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].criteria[1].name` | Area 0–31.9° | Area 0–31.7° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].criteria[2].name` | Area 30–31.9° | Area 30–31.7° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[1].criteria[1].name` | Area 0–32.1° | Area 0–31.9° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[1].criteria[2].name` | Area 30–32.1° | Area 30–31.9° | changed |
| sweep gas cargo LNG | `checks[22].message` | freeboard 3.53 m at the carried draught 1.97 m >= required 0.3 m | freeboard 3.52 m at the carried draught 1.98 m >= required 0.3 m | changed |
| sweep gas cargo LNG | `checks[23].message` | GM 5.45 m (fluid), max GZ 2.25 m at 31° | GM 5.39 m (fluid), max GZ 2.23 m at 31° | changed |
| sweep gas cargo LNG | `checks[24].message` | GM 5.48 m (fluid), max GZ 2.24 m at 31° | GM 5.41 m (fluid), max GZ 2.22 m at 31° | changed |
| sweep gas cargo LNG | `checks[25].message` | GM 6.89 m (fluid), max GZ 2.09 m at 25° | GM 6.79 m (fluid), max GZ 2.08 m at 25° | changed |
| sweep gas cargo LNG | `checks[26].message` | GM 6.86 m (fluid), max GZ 2.09 m at 25° | GM 6.76 m (fluid), max GZ 2.08 m at 25° | changed |
| sweep gas cargo LNG | `checks[27].message` | barge form factor at the service speed: Fn 0.164, Cp 0.967, L/B 5.78, B/T 7.06; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 5.78 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.164, Cp 0.967, L/B 5.79, B/T 7.01; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 5.79 (range 7.59 to 16.29) | changed |
| sweep gas cargo LNG | `checks[3].message` | freeboard 3.53 m >= required 0.3 m | freeboard 3.52 m >= required 0.3 m | changed |
| sweep gas cargo LNG pushed | `checks[13].message` | freeboard 2.69 m at the carried draught 3.31 m >= required 0.3 m | freeboard 2.67 m at the carried draught 3.33 m >= required 0.3 m | changed |
| sweep gas cargo LNG pushed | `checks[14].message` | GM 1.50 m (fluid), max GZ 0.85 m at 28° | GM 1.47 m (fluid), max GZ 0.83 m at 28° | changed |
| sweep gas cargo LNG pushed | `checks[15].message` | GM 1.50 m (fluid), max GZ 0.85 m at 28° | GM 1.47 m (fluid), max GZ 0.83 m at 28° | changed |
| sweep gas cargo LNG pushed | `checks[16].message` | GM 7.12 m (fluid), max GZ 2.66 m at 34° | GM 6.93 m (fluid), max GZ 2.62 m at 34° | changed |
| sweep gas cargo LNG pushed | `checks[17].message` | GM 7.12 m (fluid), max GZ 2.66 m at 34° | GM 6.93 m (fluid), max GZ 2.62 m at 34° | changed |
| sweep gas cargo LNG pushed | `checks[18].message` | GM 9.88 m (fluid), max GZ 2.35 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 647 t of ballast (the Ballast, departure condition meets every criterion) | GM 9.52 m (fluid), max GZ 2.31 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 647 t of ballast (the Ballast, departure condition meets every criterion) | changed |
| sweep gas cargo LNG pushed | `checks[19].message` | barge form factor at the service speed: Fn 0.177, Cp 0.969, L/B 6.12, B/T 4.23; outside its range: Fn 0.18 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 6.12 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.177, Cp 0.969, L/B 6.12, B/T 4.20; outside its range: Fn 0.18 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88), L/B 6.12 (range 7.59 to 16.29) | changed |
| sweep gas cargo LNG pushed | `checks[3].message` | freeboard 2.69 m >= required 0.3 m | freeboard 2.67 m >= required 0.3 m | changed |
| sweep gas cargo LNG pushed | `formation.notes[1]` | Coupling force between the pusher and the pushed craft: 494 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 967 kW, L_S 15.2 m taken as the pusher length, B_S 8.0 m). | Coupling force between the pusher and the pushed craft: 496 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 971 kW, L_S 15.3 m taken as the pusher length, B_S 8.1 m). | changed |
| ... | 25523 more changed quantities | | | |
