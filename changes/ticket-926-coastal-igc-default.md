<!-- unreleased -->
- Defaults Profile (issue #926): on a coastal Service Area the default tank design pressure of an LNG or LNG and ethane Type C tank is the IGC 4.23.1.2 minimum of each tank (`null`, as for fully refrigerated LPG), not 0.40 MPa. A river-only Design keeps 0.40 MPa. A typed pressure is kept; a saved file that stores a pressure keeps it. A defaulted pressure follows the Service Area when it changes in a session.
- Results (issue #926): `drawing.tank.design_pressure_basis` says how the pressure was set: `typed`, `default` or `igc_4_23_1_2_minimum`. The key is outside the results hash.

<!-- design-impact -->
### ticket/926-coastal-igc-default (2026-11-06)

Between `main 5ee9e06` and `this branch dc7aed83`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas coastal-and-river 110x11.4 | `checks[14].message` | blind distance ahead of the bow 153.9 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 0.00 m of 0.00 m | blind distance ahead of the bow 154.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 0.00 m of 0.00 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[23].message` | freeboard 2.55 m at the carried draught 3.45 m >= required 0.3 m | freeboard 2.56 m at the carried draught 3.44 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[24].message` | GM 0.66 m (fluid), max GZ 0.55 m at 34° | GM 0.67 m (fluid), max GZ 0.55 m at 34° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[25].message` | GM 0.65 m (fluid), max GZ 0.54 m at 34° | GM 0.66 m (fluid), max GZ 0.55 m at 34° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[26].message` | GM 4.29 m (fluid), max GZ 1.45 m at 27° | GM 4.34 m (fluid), max GZ 1.46 m at 27° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[27].message` | GM 4.32 m (fluid), max GZ 1.44 m at 26° | GM 4.37 m (fluid), max GZ 1.45 m at 26° | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[28].message` | barge form factor at the service speed: Fn 0.161, Cp 0.956, L/B 9.09, B/T 3.29; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), B/T 3.29 (range 4.05 to 8.12) | barge form factor at the service speed: Fn 0.161, Cp 0.956, L/B 9.09, B/T 3.30; outside its range: Fn 0.16 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), B/T 3.30 (range 4.05 to 8.12) | changed |
| sweep gas coastal-and-river 110x11.4 | `checks[3].message` | freeboard 2.55 m >= required 0.3 m | freeboard 2.56 m >= required 0.3 m | changed |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[56][1]` | 0.00754692586604 | 0.0141602060329 | +87.6 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[56][1]` | 0.00754692586604 | 0.0141602060329 | +87.6 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[56][1]` | 0.0165217213693 | 0.0231622300366 | +40.2 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[56][1]` | 0.0165217213693 | 0.0231622300366 | +40.2 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[57][1]` | -0.0198260632711 | -0.0132381428353 | +33.2 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[57][1]` | -0.0198260632711 | -0.0132381428353 | +33.2 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[57][1]` | -0.0291391489482 | -0.0225786002629 | +22.5 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[57][1]` | -0.0291391489482 | -0.0225786002629 | +22.5 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[55][1]` | 0.0437195857693 | 0.0503822673382 | +15.2 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[55][1]` | 0.0437195857693 | 0.0503822673382 | +15.2 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[55][1]` | 0.0523559337831 | 0.0590457059127 | +12.8 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[55][1]` | 0.0523559337831 | 0.0590457059127 | +12.8 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[58][1]` | -0.0566407370272 | -0.0501086224229 | +11.5 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[58][1]` | -0.0566407370272 | -0.0501086224229 | +11.5 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[58][1]` | -0.066291670923 | -0.0597870756464 | +9.81 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[58][1]` | -0.066291670923 | -0.0597870756464 | +9.81 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[54][1]` | 0.0793288844212 | 0.0860375221126 | +8.46 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[54][1]` | 0.0793288844212 | 0.0860375221126 | +8.46 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[54][1]` | 0.0876269283223 | 0.0943625250089 | +7.69 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[54][1]` | 0.0876269283223 | 0.0943625250089 | +7.69 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[59][1]` | -0.0938782790604 | -0.0874050869141 | +6.9 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[59][1]` | -0.0938782790604 | -0.0874050869141 | +6.9 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[59][1]` | -0.103866349113 | -0.0974208272861 | +6.21 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[59][1]` | -0.103866349113 | -0.0974208272861 | +6.21 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[53][1]` | 0.114321554541 | 0.121072579944 | +5.91 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[53][1]` | 0.114321554541 | 0.121072579944 | +5.91 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[53][1]` | 0.12228175618 | 0.129059616539 | +5.54 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[53][1]` | 0.12228175618 | 0.129059616539 | +5.54 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[0].gz[60][1]` | -0.131497065549 | -0.125085817027 | +4.88 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[0].gz[60][1]` | -0.131497065549 | -0.125085817027 | +4.88 % |
| sweep gas coastal-and-river 110x11.4 | `loading_conditions.conditions[1].gz[52][1]` | 0.148640618678 | 0.155430331313 | +4.57 % |
| sweep gas coastal-and-river 110x11.4 | `stability.conditions[1].gz[52][1]` | 0.148640618678 | 0.155430331313 | +4.57 % |
| ... | 780 more changed quantities | | | |
