<!-- unreleased -->
- Calibration (issue #788, spec 0014 D10, D19): the ATB Formation allowance is fitted. `calibration/atb.yaml` holds four reference ATBs, now with the 4,000 m3 LNG bunker ATB (3,800 kW, 10 kn; Baird Maritime vessel review, 11 May 2021, via `docs/research/atb-gas-tug-barge.md` sec. 4). `python3 tools/atb_calibration.py <ovd> --write` runs each as an `atb` Design with its tug typed and the default resistance method (`barge_form_factor`), fits `formation_resistance_allowance` on the three ATBs that publish a speed, and writes `calibration/atb_results.md`, the fit block of `calibration/atb.yaml` and `crates/ovd-format/src/atb_calibration.json`. The fitted value is 2.842, status `estimate`: the allowance carries every margin the published installed power holds. In-sample residuals -37.9 % to +38.8 %; the held-out ATB (power published, no speed; run at an estimated 12 kn) +63.3 %. All four cases are outside the method's range, so their speeds are indicative. Two LNG ATB designs are listed and not run (no speed; no power). The Defaults Profile `formation_resistance_allowance` is 2.842 for an ATB (was 1.05), with the soft limit raised from 2.0 to 3.0. A file saved before reads 1.0 and a file that stores the input keeps its value, so no stored hash moves. Each ATB's results carry the fit, each residual, their range and the method's range check on the Option's Equivalent Hull in the unhashed `calibration.formation` label.

<!-- design-impact -->
### ticket/788-atb-calibration (2026-11-01)

Between `main bc33044` and `this branch 9e0d6ef5`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep atb notch 100x20 | `formation.notes[1]` | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 687 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 878 kW, for an installed power of 824 kW. | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 2233 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 2853 kW, for an installed power of 2679 kW. | changed |
| sweep atb notch 100x20 | `formation.environments[0].power_at_service_kw` | 687.01752591 | 2232.72856279 | +225 % |
| sweep atb notch 100x20 | `formation.service_margins.still_water_power_kw` | 687.01752591 | 2232.72856279 | +225 % |
| sweep atb notch 100x20 | `formation.tug.installed_power_kw` | 824.421031091 | 2679.27427534 | +225 % |
| sweep atb notch 100x20 | `formation.service_margins.power_with_margins_kw` | 877.855727551 | 2852.93094134 | +225 % |
| sweep atb notch 100x20 | `formation.resistance_allowance` | 1.05 | 2.842 | +171 % |
| sweep atb notch 100x20 | `formation.tug.bollard_pull_t` | 25.61736269 | 56.2056306503 | +119 % |
| sweep atb notch 100x20 | `formation.tug.machinery_t` | 89.150267385 | 108.412563178 | +21.6 % |
| sweep atb notch 100x20 | `formation.tug.lightship_t` | 253.855160089 | 273.117455882 | +7.59 % |
| sweep atb notch 100x20 | `formation.equivalent_hull.block_coefficient` | 0.785585961841 | 0.787068132955 | +0.189 % |
