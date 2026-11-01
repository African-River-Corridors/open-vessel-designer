<!-- unreleased -->
- Hull steel (issue #741, spec 0013): the gas hull's panel model builds from a `HullStructureSpec` assembled from the Vessel Model (its Sections Table, decks, wing ballast Spaces, Tank Layout and `structure` block), not from the gas arrangement directly. The lightship, which the capacity calculation weighs before the Vessel Model exists, builds the same spec from the converged arrangement; a test holds the two equal. The panels lay out on the `structure` block's frame and longitudinal spacing. Members are named by OCX `functionType` values (OCX schema v3.2.0, e.g. "DECK: Strength deck"), with a `region` ("midship 0.4L", "ends", "cargo zone", "end zones") where two members share one. Scantlings stay the `abs-steel-barges` Rule Set's. No result moves: the default spacings are today's, and the rule length stays 0.96 x the length overall (the lightship's raked box carries no rake); the rule's own length on the Sections Table is shorter and moves with the lightship switch (spec 0013, T7).

<!-- design-impact -->
### ticket/741-hull-structure-spec (2026-11-01)

Between `main 567f829` and `this branch 7b1c21b6`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional frame_spacing_m | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.55 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.57 m of 6.00 m | changed |
| sweep optional frame_spacing_m | `checks[22].message` | freeboard 3.26 m at the carried draught 2.74 m >= required 0.3 m | freeboard 3.30 m at the carried draught 2.70 m >= required 0.3 m | changed |
| sweep optional frame_spacing_m | `checks[23].message` | GM 1.54 m (fluid), max GZ 1.12 m at 36° | GM 1.56 m (fluid), max GZ 1.12 m at 36° | changed |
| sweep optional frame_spacing_m | `checks[24].message` | GM 1.53 m (fluid), max GZ 1.10 m at 36° | GM 1.56 m (fluid), max GZ 1.11 m at 36° | changed |
| sweep optional frame_spacing_m | `checks[25].message` | GM 3.93 m (fluid), max GZ 1.51 m at 29° | GM 4.08 m (fluid), max GZ 1.52 m at 28° | changed |
| sweep optional frame_spacing_m | `checks[26].message` | GM 3.96 m (fluid), max GZ 1.50 m at 29° | GM 4.10 m (fluid), max GZ 1.51 m at 28° | changed |
| sweep optional frame_spacing_m | `checks[27].message` | barge form factor at the service speed: Fn 0.163, Cp 0.964, L/B 8.92, B/T 4.14; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | barge form factor at the service speed: Fn 0.163, Cp 0.965, L/B 8.91, B/T 4.20; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | changed |
| sweep optional frame_spacing_m | `checks[3].message` | freeboard 3.26 m >= required 0.3 m | freeboard 3.30 m >= required 0.3 m | changed |
| sweep optional longitudinal_spacing_mm | `checks[22].message` | freeboard 3.26 m at the carried draught 2.74 m >= required 0.3 m | freeboard 3.27 m at the carried draught 2.73 m >= required 0.3 m | changed |
| sweep optional longitudinal_spacing_mm | `checks[24].message` | GM 1.53 m (fluid), max GZ 1.10 m at 36° | GM 1.53 m (fluid), max GZ 1.11 m at 36° | changed |
| sweep optional longitudinal_spacing_mm | `checks[25].message` | GM 3.93 m (fluid), max GZ 1.51 m at 29° | GM 3.95 m (fluid), max GZ 1.51 m at 29° | changed |
| sweep optional longitudinal_spacing_mm | `checks[26].message` | GM 3.96 m (fluid), max GZ 1.50 m at 29° | GM 3.97 m (fluid), max GZ 1.50 m at 29° | changed |
| sweep optional longitudinal_spacing_mm | `checks[3].message` | freeboard 3.26 m >= required 0.3 m | freeboard 3.27 m >= required 0.3 m | changed |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[1][1]` | 0.0692031725093 | 0.0717459279455 | +3.67 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[1][1]` | 0.0692031725093 | 0.0717459279455 | +3.67 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[2][1]` | 0.138498470745 | 0.143586185346 | +3.67 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[2][1]` | 0.138498470745 | 0.143586185346 | +3.67 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[3][1]` | 0.207978367593 | 0.215615458006 | +3.67 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[3][1]` | 0.207978367593 | 0.215615458006 | +3.67 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].criteria[5].value` | 3.95898316867 | 4.1043405984 | +3.67 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gm_fluid_m` | 3.95898316867 | 4.1043405984 | +3.67 % |
| sweep optional frame_spacing_m | `stability.conditions[3].criteria[5].value` | 3.95898316867 | 4.1043405984 | +3.67 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gm_fluid_m` | 3.95898316867 | 4.1043405984 | +3.67 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[4][1]` | 0.277736032627 | 0.287929146859 | +3.67 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[4][1]` | 0.277736032627 | 0.287929146859 | +3.67 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[5][1]` | 0.347865686385 | 0.360623731621 | +3.67 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[5][1]` | 0.347865686385 | 0.360623731621 | +3.67 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[6][1]` | 0.41846296185 | 0.433797141836 | +3.66 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[6][1]` | 0.41846296185 | 0.433797141836 | +3.66 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gm_solid_m` | 3.99545391159 | 4.14181237937 | +3.66 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gm_solid_m` | 3.99545391159 | 4.14181237937 | +3.66 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[7][1]` | 0.489625275647 | 0.507549137895 | +3.66 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[7][1]` | 0.489625275647 | 0.507549137895 | +3.66 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[8][1]` | 0.56145221159 | 0.581981704727 | +3.66 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[8][1]` | 0.56145221159 | 0.581981704727 | +3.66 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[9][1]` | 0.634045919347 | 0.657199461001 | +3.65 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[9][1]` | 0.634045919347 | 0.657199461001 | +3.65 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[10][1]` | 0.707511531156 | 0.733310086863 | +3.65 % |
| sweep optional frame_spacing_m | `stability.conditions[3].gz[10][1]` | 0.707511531156 | 0.733310086863 | +3.65 % |
| sweep optional frame_spacing_m | `loading_conditions.conditions[3].gz[11][1]` | 0.781957599742 | 0.810424773416 | +3.64 % |
| ... | 1375 more changed quantities | | | |
