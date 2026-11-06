<!-- unreleased -->
- Insulation as explicit inputs (issue #919): the tank insulation's thermal conductivity (`tank_insulation_conductivity_w_per_mk`) and density (`tank_insulation_density_kg_m3`) are inputs of the type C tanks, with Defaults Profile tables by Cargo and carriage condition (estimates: conductivity from Lee et al., J. Mar. Sci. Eng. 2021, 9, 36, Table 3; density 60 kg/m3 as before). The heat leak, the boil-off and the holding time read the conductivity; the insulation mass reads the density. A file saved before them reads 0.026 W/m.K and 60 kg/m3, marked default, so its results do not move. A typed insulation type other than the Cargo's default, with either value still the default, gets a soft warning. No existing design moves; the design-impact rows below are the two new sweep cases (a typed 0.052 W/m.K and a typed 120 kg/m3 on an LNG design), which the base cannot run.

<!-- design-impact -->
### ticket/919-insulation-inputs (2026-11-06)

Between `main 4f9c946` and `this branch 99f7fd11`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional tank_insulation_conductivity_w_per_mk | `checks[30].message` | boil-off 193 kg/h > gas use 141 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 385 kg/h > gas use 141 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.78 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.77 m of 6.00 m | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[22].message` | freeboard 3.69 m at the carried draught 2.31 m >= required 0.3 m | freeboard 3.66 m at the carried draught 2.34 m >= required 0.3 m | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[23].message` | GM 2.02 m (fluid), max GZ 1.27 m at 37° | GM 1.96 m (fluid), max GZ 1.26 m at 37° | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[24].message` | GM 2.02 m (fluid), max GZ 1.26 m at 37° | GM 1.96 m (fluid), max GZ 1.24 m at 37° | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[25].message` | GM 5.19 m (fluid), max GZ 1.67 m at 26° | GM 4.99 m (fluid), max GZ 1.64 m at 27° | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[26].message` | GM 5.24 m (fluid), max GZ 1.66 m at 26° | GM 5.03 m (fluid), max GZ 1.63 m at 26° | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[27].message` | barge form factor at the service speed: Fn 0.164, Cp 0.969, L/B 8.82, B/T 4.90; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88) | barge form factor at the service speed: Fn 0.164, Cp 0.969, L/B 8.83, B/T 4.84; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88) | changed |
| sweep optional tank_insulation_density_kg_m3 | `checks[3].message` | freeboard 3.69 m >= required 0.3 m | freeboard 3.66 m >= required 0.3 m | changed |
| sweep optional tank_insulation_thickness_mm | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.72 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.73 m of 6.00 m | changed |
| sweep optional tank_insulation_thickness_mm | `checks[22].message` | freeboard 3.33 m at the carried draught 2.67 m >= required 0.3 m | freeboard 3.35 m at the carried draught 2.65 m >= required 0.3 m | changed |
| sweep optional tank_insulation_thickness_mm | `checks[23].message` | GM 1.40 m (fluid), max GZ 1.05 m at 36° | GM 1.43 m (fluid), max GZ 1.06 m at 36° | changed |
| sweep optional tank_insulation_thickness_mm | `checks[24].message` | GM 1.40 m (fluid), max GZ 1.04 m at 36° | GM 1.42 m (fluid), max GZ 1.05 m at 36° | changed |
| sweep optional tank_insulation_thickness_mm | `checks[25].message` | GM 3.63 m (fluid), max GZ 1.42 m at 29° | GM 3.71 m (fluid), max GZ 1.44 m at 29° | changed |
| sweep optional tank_insulation_thickness_mm | `checks[26].message` | GM 3.65 m (fluid), max GZ 1.41 m at 29° | GM 3.73 m (fluid), max GZ 1.43 m at 28° | changed |
| sweep optional tank_insulation_thickness_mm | `checks[27].message` | barge form factor at the service speed: Fn 0.163, Cp 0.965, L/B 8.91, B/T 4.23; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | barge form factor at the service speed: Fn 0.163, Cp 0.965, L/B 8.90, B/T 4.26; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.97 (range 0.77 to 0.88) | changed |
| sweep optional tank_insulation_thickness_mm | `checks[3].message` | freeboard 3.33 m >= required 0.3 m | freeboard 3.35 m >= required 0.3 m | changed |
| sweep optional tank_insulation_conductivity_w_per_mk | `boil_off.boil_off_kg_per_h` | 192.52894703 | 385.05789406 | +100 % |
| sweep optional tank_insulation_conductivity_w_per_mk | `boil_off.conductivity_w_per_mk` | 0.026 | 0.052 | +100 % |
| sweep optional tank_insulation_conductivity_w_per_mk | `boil_off.heat_leak_kw` | 27.3695942282 | 54.7391884564 | +100 % |
| sweep optional tank_insulation_thickness_mm | `loading_conditions.conditions[3].trim_m` | -0.23105710504 | -0.254055650534 | -9.95 % |
| sweep optional tank_insulation_thickness_mm | `stability.conditions[3].trim_m` | -0.23105710504 | -0.254055650534 | -9.95 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[3].trim_m` | -0.493075960514 | -0.454656255212 | +7.79 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[3].trim_m` | -0.493075960514 | -0.454656255212 | +7.79 % |
| sweep optional tank_insulation_thickness_mm | `loading_conditions.conditions[2].trim_m` | -0.344885671394 | -0.368028743441 | -6.71 % |
| sweep optional tank_insulation_thickness_mm | `stability.conditions[2].trim_m` | -0.344885671394 | -0.368028743441 | -6.71 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[2].trim_m` | -0.609147265904 | -0.57049143461 | +6.35 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[2].trim_m` | -0.609147265904 | -0.57049143461 | +6.35 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[0].trim_m` | 0.528243258035 | 0.559984755364 | +6.01 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[0].trim_m` | 0.528243258035 | 0.559984755364 | +6.01 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[2].draught_fwd_m` | 1.0089006165 | 1.06031734393 | +5.1 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[2].draught_fwd_m` | 1.0089006165 | 1.06031734393 | +5.1 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[1].trim_m` | 0.636789620629 | 0.668314639188 | +4.95 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[1].trim_m` | 0.636789620629 | 0.668314639188 | +4.95 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[3].draught_fwd_m` | 1.05095211389 | 1.10223118851 | +4.88 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[3].draught_fwd_m` | 1.05095211389 | 1.10223118851 | +4.88 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[3].criteria[5].value` | 5.24232466832 | 5.03336507649 | -3.99 % |
| sweep optional tank_insulation_density_kg_m3 | `loading_conditions.conditions[3].gm_fluid_m` | 5.24232466832 | 5.03336507649 | -3.99 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[3].criteria[5].value` | 5.24232466832 | 5.03336507649 | -3.99 % |
| sweep optional tank_insulation_density_kg_m3 | `stability.conditions[3].gm_fluid_m` | 5.24232466832 | 5.03336507649 | -3.99 % |
| ... | 1383 more changed quantities | | | |
