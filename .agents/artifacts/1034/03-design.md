---
ticket: 1034
phase: design
date: 2026-11-08
status: approved (the maintainer's gate, 2026-11-09)
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a85c10002f3e7e752f587d5ac0e745834532a76f
---

# Design: river draught and air-draft limits, and the cost of the damage search

## Summary of the Request

The maintainer asked (2026-11-08): "I think I want to have this also allowing draught and air draft to be a required constraint on the river runs? How long do the damage runs take? I want to understand this better and get the level between search and time correct."

## Current State

- `draught_limit_m` and `air_draft_limit_m` are optional, with no default. When typed, both act in the gas search: the air-draft limit drops a candidate; the draught limit sets the least depth and caps the carried draught.
- A river-only gas design stands its tanks up to OD/2 above the deck. The search keeps the **tallest share that passes** (#992 P5, PR #1027). With no limits nothing bounds the hull from above.
- `bridge_clearance_m` (#991) gives a warning only.
- Run time: 1 to 22 CPU s per design today (research, section 6).

## What the measurements say (short)

| Question | Answer (measured on `a85c1000`, release) |
|---|---|
| One IGC 2.7 damage run | 0.05 to 0.15 s wall on 10 workers; 0.23 to 1.0 s on one worker. 74 to 375 cases. |
| Damage runs per design | 4 (140 x 29 m, 2 along), 5 (140 x 29 m, along searched), 16 (89 x 16 m), 1 on each river design (the final ADN/ES-TRIN run, about 0.03 s). |
| Whole run, CPU s | 140 x 29 m: 11 to 14. 89 x 16 m: 17 to 22. 100 x 20 m river: 10 to 14. 110 x 11.4 m river: 0.7 to 1.1. |
| Share of a run spent on damage cases | About 10 % on the coastal designs; under 1 % on the river designs. |
| What most of a run is | The intact stability stage of the search (GZ curves for every loading condition of every candidate it checks). |
| Why the river barge went 2.5 s to 10 s | Share 0.5 fails, so the stability stage runs on every candidate at that share, and the share search runs twice (depth probe and run pass). Not damage: a river-only run screens no IGC damage. |
| Cap 24 against 12 and 48 | 48 changes nothing (no design used more than 16). 12 makes the 89 x 16 m design infeasible. |
| Slowest case | An infeasible design: 15 to 56 CPU s, because every share's stability stage runs in full. |

So the "level between search and time" is mostly a choice about **how many shares the intact search tries**, not about damage runs.

## Desired End State

- A river-only Option cannot be made without an answer for the draught limit and the air-draft limit. Saved files give the same results as before.
- With the limits in force, the search picks the share that carries the most cargo, not the tallest, at a run time close to today's.
- Two speed-ups with no change of results cut the share-search time.

## Decisions

### D1. How a river-only Option gets its limits

Options:
- **A. Required answer, no default. Recommended.** For a river-only Option, both limits must be answered: a number (typed, or filled by a Vessel Class), or "no limit" typed on purpose. Same pattern as the Service Area (#925).
- B. A default per Service Area (one number for all rivers). Rejected: the CEMT air drafts alone span 3.0 to 9.1 m (ECMT Resolution 92/2, Table 1). A wrong default would block or pass a design without a word. The `bridge_clearance_m` basis already says no Service Area can have a default.
- C. A default per route. Rejected for now: OVD has no route object. A Vessel Class is the nearest thing, and it already fills both limits.
- D. A prompt in the app only. Rejected: the CLI and Python would still run with no limit.

How A works on each path:
- **New Option** (`ovd new`, `ovd fill --as-new`, a new Option in the app, Python `simulate`): the profile gives no value for river-only. With neither typed nor class-filled, the Option is incomplete. A run refuses with one message that names both inputs and the three fixes: type a value, pick a Vessel Class, or type "no limit".
- **CLI:** `ovd new` and `ovd fill` get `--draught-limit <m|none>` and `--air-draft-limit <m|none>`, written as typed into every river-only Option that has no answer. `none` writes a typed `null`.
- **Python:** `simulate(p)` with a river-only Option and a limit key left out raises `TypeError` naming it, as for any required input. A key given as `None` is a typed "no limit". `calibration.py` adds `"air_draft_limit_m": None` to keep its runs.
- **App:** the two fields show with no value and a "no limit" choice, as the Service Area field does. The Option shows "incomplete" until both are answered.
- **Saved files: nothing moves.** A file that stores a limit as `null` with source `default` was written before this change (after it, `fill` never writes a default `null` for river-only). Such a file runs as before, with the same results and hash. `validate`, `run` and the app give a **validation** warning, not a results warning, so the hash does not move: "This river-only Option has no draught limit (or air-draft limit). Its hull is not bounded. Set one with `ovd fill --draught-limit`."
- **A typed "no limit"** runs, with a validation warning that the hull is not bounded by that limit.
- **Coastal-and-river:** not required, as today. When typed, the limits act the same way. Reason: the coastal default stands tanks at a fixed 2.0 m, so no share search runs deep. The design draught is usually set by the port, not the river. See Q2.
- **Scope across Cargos:** the rule applies to every river-only Option, not gas only (see Q1). The limits mean the same thing for every vessel on a river, and one rule is easier to follow.

### D1b. `bridge_clearance_m` against `air_draft_limit_m`

Options:
- **A. Keep both; only the air-draft limit is required. Recommended.** The limit is a design limit: it blocks and steers the search. The bridge clearance stays an optional route fact that warns. A new validation warning when both are typed and the limit is above the clearance: "the air-draft limit lets the vessel stand higher than the lowest bridge given".
- B. Fill the limit from the bridge clearance when the limit is not typed. Rejected: it mixes a block and a warning. A designer who types a bridge clearance to see a warning would suddenly get a block.
- C. Merge the two into one input with a "block or warn" switch. Rejected: it changes saved files and #991's results.

### D2. How the limits steer the search, and the share rule

The limits already act (research, section 1). No new mechanism is needed: the air-draft check drops tall candidates, and the draught limit sets the least depth and caps the draught. What changes is the rule that picks the share.

Options:
- A. Keep "tallest share that passes". Cheapest. Leaves cargo on the table: on the 100 x 20 m barge under limits A it gives 1,328 t where 1,559 t fits.
- B. "Most cargo" over every share. Unsafe without limits: it takes the 100 x 20 m barge to the hold on a 17.40 m hull. With limits it is safe, but it costs the most (about 16 to 31 CPU s on the 100 x 20 m barge, because the lowest shares are often infeasible and infeasible shares are the slowest).
- **C. "Climb": start at the tallest share that passes, then go down one share at a time while the cargo rises by at least 1 %; keep the last share that rose. Recommended.** It finds the peak in every case measured, at one or two more share searches.

The 100 x 20 m river LNG barge under each rule (limits from CEMT VIa, ECMT Resolution 92/2, Table 1):

| Limits (draught / air draft) | A: tallest that passes | B: most cargo, every share | C: climb |
|---|---|---|---|
| none | 0.375: 2,351 t, 6.16 m deep, AD 10.32 m | hold: 4,559 t, **17.40 m deep**, AD 18.32 m | 0.375: 2,351 t |
| 4.5 m / 9.1 m (top of VIa draught; 9.1 m is also the UNECE coastal-link bridge height, unverified) | 0.5: 1,328 t, 4.80 m, AD 8.38 m | 0.375: **1,559 t**, 5.22 m, AD 8.85 m | 0.375: **1,559 t** |
| 3.5 m / 9.1 m | 0.375: 1,559 t | 0.375: 1,559 t | 0.375: 1,559 t |
| 2.5 m / 7.0 m (bottom of both VIa ranges) | infeasible | infeasible | infeasible |

The 110 x 11.4 m river tanker picks share 0.5 under all three rules with limits A or B (862 t and 870 t).

Why C:
- With limits in force, the peak cargo is at the passing share or the one below it in every limited case measured. C finds it.
- Even with no limits, C stopped at 2,351 t on the 100 x 20 m barge: the cargo fell at share 0.25 before the deep shares. So C is safe on a saved file with no limits too. This is a finding on two designs, not a proof: the limits from D1 are the real guard.
- The 1 % step stops a climb that buys a much deeper hull for a few tonnes (the 110 m tanker gains 0 t from 0.375 to 0.25 and to the hold).
- Ties go to the taller share (less depth, less steel), as today.
- The damage screen (coastal-and-river with a typed share) keeps its one budget of 24 runs over all shares the climb visits.

What the user sees: the summary line names the rule ("share 0.375, the most cargo within the limits; 0.5 carries 1,328 t").

### D3. Search breadth against run time

Options for the maintainer (CPU s measured or estimated, research section 6):

| Option | 100 x 20 m river, limits A | 100 x 20 m, no limits | 110 m river, limits A | Coastal designs |
|---|---|---|---|---|
| Today: tallest that passes | 1.0 (1,328 t) | 12.2 | 0.9 | 11 to 22 |
| **Climb (D2 C) plus the probe memo (D4). Recommended.** | about 2 to 4 (1,559 t) | about 8 to 15 | about 1 to 2 | 11 to 22 |
| Every share | about 16 (1,559 t) | about 22 (4,559 t, 17.4 m deep) | about 7.6 | 11 to 22 |

- The damage-run cap stays at **24**. Recommended. It does not bind on any design measured. 12 makes the 89 x 16 m coastal design infeasible (it needs 16). 48 gives the same answers as 24 and only adds time where a design would need more than 24, which none did.
- The coastal run time is mostly the intact stability stage of the wider search the damage screen brings (tanks along, refinement). The damage cases themselves are about 1 to 2 s.

### D4. Speed-ups with no change of results

- **D4a. The depth probe hands its share answer to the run pass. Recommended, in scope.** Today both run `share_stage_one` in full. Estimate: about half of a river run's share-search time (the profile puts 49 % in the probe). Results byte-equal.
- **D4b. `ovd run` reruns an Option without damage** when its stored hash is at the unscreened stage, which is what `fill` writes. This adds 4 to 7 s to a coastal `ovd run` from the CLI. **Recommended: a separate issue.** Fixing it touches the hash stages of #935.
- **D4c. Infeasible designs are the slowest** (15 to 56 CPU s), because every share runs its stability stage on every candidate. **Recommended: a separate issue.** With required limits more designs will be infeasible on a first try. One idea: stop the stability stage of a share after its cheap-checked best cannot beat the closest answer so far.

### D5. River runs keep 2 tanks along

A river-only run does not search 3 or 4 along: the choice is tied to the IGC damage screen (#935). Under tight limits (2.5 m / 7.0 m) both river designs fail length over diameter with 2 along. **Recommended: a separate issue** to let a river-only run choose tanks along on the intact checks. Out of scope here.

## Gate outcome

The maintainer approved this design at the gate on 2026-11-09. The answers below **override** the text above where they differ. In particular, the text above lets a designer type "no limit" (D1, Q3). The gate said no: there is no "no limit" for a river-only Option.

1. **Q1.** Every river-only Option, with any Cargo, must answer both limits.
2. **Q2.** Coastal-and-river Options: the limits stay optional, as today.
3. **Q3.** No "no limit". A river-only Option must carry a number, typed or filled by a Vessel Class. So there is no `none` value for `--draught-limit` or `--air-draft-limit`, and no "no limit" choice in the app. Python `simulate` raises `TypeError` for a missing or `None` limit on a river-only Option. `calibration.py` and the river-only tests get real numbers. Saved files are unchanged: a file that stores a limit as `null` with source `default` still runs, with the same results and hash, and a validation warning (not a results warning).
4. **Q4.** The climb rule (Phase 3, a later PR).
5. **Q5.** The damage-run cap stays 24.
6. **Q6.** Bridge clearance stays a warning. A new warning fires when the air-draft limit is above a typed bridge clearance.
7. **Q7.** The follow-ups are separate issues: #1049, #1050, #1051 and #1052.
8. **The wheelhouse lift (answer of 2026-11-09, after Phase 2 made the air-draft limit required).** An existing rule makes a deckhouse wheelhouse elevating whenever a limit is set, so every river-only deckhouse design would have lifted its wheelhouse, even under a loose 40 m limit (the 140 x 29 m LNG air draft fell from 25.8 m to 18.0 m). **The wheelhouse lifts only when a fixed wheelhouse would break the air-draft limit.** Where the fixed wheelhouse meets the limit, it stays fixed and the design does not move. The engine runs the fixed design first and runs the lifting one only when the fixed one fails `air_draft_within_limit`. A limit equal to the fixed air draft keeps it fixed. The run's `wheelhouse_basis` is `limit_not_binding` for a fixed wheelhouse under a limit, `air_draft_limit` for one lifted by the limit. The tests no longer type `wheelhouse_max_lift_m = 0` to hide the lift.

## Risks

- **Tests and examples that build river-only Options without limits** (`damage_budget.rs`, `tank_top_above_deck.rs`, `mesh.rs`, `adn_river_stack.rs` and others) will get the "incomplete" error. Each needs the limits typed, or typed "no limit". With typed "no limit" their values do not move.
- **The climb moves river-only answers** where the peak is below the tallest passing share (the 100 x 20 m barge under limits A). Saved files with a stored share or default limits re-run the same search, so their answers can move too: every moved design is named through `tools/design_impact.py changelog`.
- **Timing figures are noisy.** The machine was shared (load average 7 to 70). CPU seconds are the main figure; wall times moved up to 3 times. The PR bodies must re-measure on the merged tree.
- **#1033 (default depth) is in build at the same time.** It may change the coastal and the river depths. Whoever lands second re-measures the tables.

## Open Questions for the maintainer

1. **Q1. Which river-only Options need the two limits?** (a) Every river-only Option, any Cargo. **Recommended:** one rule, and the limits mean the same for every river vessel. (b) River-only gas only, where the depth follows the tank.
2. **Q2. Coastal-and-river designs?** (a) Optional, as today. **Recommended:** their tanks stand at a fixed 2.0 m, so no share search goes deep. (b) Required too, for the river leg.
3. **Q3. May a designer type "no limit"?** (a) Yes, typed on purpose, with a warning that the hull is not bounded. **Recommended:** it keeps scripts and the calibration set working. (b) No, a river-only Option must carry a number.
4. **Q4. Which share rule?** (a) Climb: from the tallest passing share, down while cargo rises 1 % or more. **Recommended:** 1,559 t instead of 1,328 t on the 100 x 20 m barge, at about 2 to 4 CPU s. (b) Keep tallest that passes. (c) Most cargo over every share (about 16 to 31 CPU s).
5. **Q5. The damage-run cap?** (a) Keep 24. **Recommended:** no design uses more than 16; 12 breaks the 89 x 16 m design. (b) 48. (c) 12.
6. **Q6. The bridge clearance?** (a) Keep it a warning; require only the air-draft limit; warn when the limit is above a typed clearance. **Recommended.** (b) Fill the limit from the clearance when the limit is missing.
7. **Q7. Follow-up issues?** (a) Open three: the CLI double run (D4b), slow infeasible runs (D4c), and tanks along for river runs (D5). **Recommended.** (b) Fold any of them into this work.
