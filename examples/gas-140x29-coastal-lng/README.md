# 140 x 29 m coastal LNG gas barge: review run

**Concept and preliminary design only. Not verified. Never for approval.** This folder is a worked example for an expert review of the tool, first run on 2026-10-29 and re-run with `ovd` on 2026-10-30 (the rename to OVD, spec 0010). The re-run takes in the propulsor layout changes since the first run (#694, #702, #708): the thruster and propulsor positions in the drawings and the results hashes moved; no figure in the tables below did. Re-run again after #722: the gas-hull lightship and cost now take the hull steel from the panel model (structural members with ABS barge minimum scantlings), so the lightship, draught, stability and cost figures moved. Re-run again after #740: the hull steel scantlings take the rule length of ABS 3-1-1/3.1 (96 % of the length on a waterline at 85 % of the depth) in place of the length overall, so the lightship, draught, stability and cost figures moved again (the tank, cargo, installed power, boil-off and gas use rows did not; the power curve moved by under 0.2 %). Re-run again after #745: the lightship and cost take the plate model's ordered steel (the plates laid out on the hull's Sections Table, the raked bow included, and the rule length on it), so the lightship fell by 266 to 385 t and the draught, stability, cost and power figures moved (the tank and cargo rows did not). Re-run again after #773, #774, #775/#851 and #776 (the genset cost line, the machinery steel allowances, the gas fuel system mass and cost allowances, and the IGC opening setback on the deckhouse): most rows moved, among them the stern void, cargo zone, cargo volume and mass, lightship, displacement, draught, freeboard, air draft, stability, power, boil-off and gas use. After #855 (the deckhouse model) only the cost and gross tonnage in `results.json` moved. The results, `trace.md`, the drawings and this README were regenerated together from the engine at that point (#857). Re-run again after #822: the gas hull has a cargo vent mast, so the air drafts rise to its exit and every results hash moved. `tools/refresh_examples.py` writes `results.json`, `ga/`, `trace.md` and the tables below from the engine. The numbers are what the tool prints; nothing here is a design.

Examples track today's Rules: if this design fails a Rule, the design is fixed, not the Rule Set stack pinned (ADR 0025).

## The design

Self-propelled gas barge, IGC Type C tanks, LNG, 140.0 m x 29.0 m, raked box hull, Service Area `coastal-and-river`, tanks at most 12.0 m outer diameter, two tanks along the hull, maximum speed 12 kn. Every other input is the Defaults Profile value (`source: default` in the file). Three Options:

- **A basis**: moulded depth 12.0 m; end spaces 0.60 m aft and 0.50 m forward (the former core defaults, kept as typed inputs: maintainer triage on #280, 2026-10-29).
- **B profile**: as A with the Defaults Profile end spaces (0.20 m, the type G rule value).
- **C deep**: as A at the default gas depth (0.55 x beam = 15.95 m).

`min_freeboard_m` is typed at 1.81 m in every Option. Without it `fit` fails the ICLL tabular-freeboard Rule for every coastal Design, because that Rule reads the safety clearance as the freeboard (finding filed on the register, #121). The river-only Service Area gives the same tanks, cargo and draught as A (sensitivity M), but its vent mast is lower (ADN 9.3.1.22.3, not IGC 8.2.10), so its air drafts and its results hash differ.

## Results

| Quantity | A basis | B profile | C deep |
|---|---:|---:|---:|
| Moulded depth (m) | 12.00 | 12.00 | 15.95 |
| Tanks across x along | 2 x 2 | 2 x 2 | 2 x 2 |
| Tank outer diameter, to insulation (m) | 11.00 | 11.00 | 12.00 |
| Side clearance per side (m) | 3.00 | 3.00 | 2.00 |
| Cargo zone (m) | 99.8 | 100.5 | 93.5 |
| Bow void (m) | 19.7 | 19.4 | 26.0 |
| Stern void (m) | 20.5 | 20.1 | 20.5 |
| Cargo volume at 0.98 fill (m3) | 14,682 | 14,797 | 16,284 |
| Cargo (t) | 6,607 | 6,659 | 7,328 |
| Lightship (t) | 5,084 | 5,092 | 5,688 |
| Displacement, laden (t) | 11,726 | 11,786 | 13,054 |
| Draught, laden (m) | 3.55 | 3.57 | 4.31 |
| Draught cap (depth less freeboard) (m) | 10.19 | 10.19 | 14.14 |
| Freeboard, laden (m) | 8.45 | 8.43 | 11.64 |
| Air draft, laden (m) | 18.11 | 18.09 | 21.31 |
| Air draft, light (m) | 20.09 | 20.08 | 23.68 |
| GM fluid, full load departure (m) | 16.12 | 16.04 | 12.32 |
| Max GZ, full load departure (m) | 6.18 | 6.17 | 5.58 |
| GM fluid, ballast departure (m) | 25.35 | 25.26 | 20.91 |
| Installed power (design speed 9.7 kn, margin 1.2) (kW) | 1,717 | 1,717 | 2,022 |
| Service speed, deep water (kn) | 11.4 | 11.4 | 12.0 |
| Power at the service speed (kW) | 1,431 | 1,431 | 1,626 |
| Boil-off (IGC 7.2 ambient) (kg/h) | 747 | 752 | 764 |
| Gas use at the service speed (kg/h) | 261 | 261 | 291 |
| Checks | boil-off within gas use | boil-off within gas use | boil-off within gas use |
| Results hash | a6d0aca8294f | 1f73394c4d7d | 8673c19b596b |

Power curve, deep water (kW at kn): 8 kn 642; 9 kn 907; 10 kn 1,240; 11 kn 1,651; 12 kn 2,148 (A basis).

Every Option meets every evaluated blocking Rule. Four blocking Rules are not evaluated by the tool: IGC 2.7 damage survival, the IS Code weather criterion, the inland tank-capacity table and the hull-design clause. One warning check is not met in every Option: the boil-off exceeds the gas use at the service speed, so the excess needs reliquefaction, pressure accumulation or a gas combustion unit (IGC 7.1).

## What moves the number (sensitivities file)

| Option | Change from A | Tank OD (m) | Cargo (m3) | Cargo (t) | Draught (m) |
|---|---|---:|---:|---:|---:|
| I lift 9 m | wheelhouse lift cap 6 to 9 m | 12.00 | 17,529 | 7,888 | 3.95 |
| J bow 12 m | bow rake typed 12 m (default 1.6 x depth = 19.2 m) | 11.00 | 15,862 | 7,138 | 3.58 |
| K both | both of the above | 12.00 | 18,948 | 8,527 | 3.99 |
| L depth 14 | as K at depth 14 m | 12.00 | 18,948 | 8,527 | 4.18 |
| M river | Service Area river-only (same tanks as A, lower vent mast) | 11.00 | 14,682 | 6,607 | 3.55 |

The river blind-distance Rule (ES-TRIN 7.02(2), 250 m) with the 6 m lift cap is what keeps the tank at 11 m instead of the 12 m cap on a 12 m hull. The bow rake default takes 19.2 m of the 140 m. Together they are the difference between 14,682 and 18,948 m3.

## Reproduce

```
ovd run examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign
ovd run --json examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign > results.json
ovd draw --out ga examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign
```

`results.json` holds the steel summary of each Option (`structure`: weight per member group, scrap, plates bought). The per-item steel list is not stored: `ovd run --json --steel-list` adds it, and `ovd export --format steel-csv --option "A basis" -o steel.csv examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign` writes it as CSV.

`trace.md` lists every Rule of Option A with its outcome, value, limit, status and citation (from `rule_trace` in `results.json`). `ga/` holds the three GA views of each Option as PDF and SVG.

## Findings from this run

Filed on the register (#121): the ICLL freeboard Measure; `ovd draw` and Option names over 15 characters; the bow rake default and the river sightline Rule on a coastal hull. See the register for the ticket numbers.
