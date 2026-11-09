---
ticket: 1034
phase: research
date: 2026-11-08
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a85c10002f3e7e752f587d5ac0e745834532a76f
---

# Research: river draught and air-draft limits, and the cost of the damage search

## Summary

- The two limits exist and already act in the gas search. The **air-draft limit** is a cheap check: a candidate whose light (or ballasted light) air draft is over it is dropped. The **draught limit** floors the depth (depth >= limit + least freeboard + margin) and caps the carried draught. Neither has a default, so a river-only run with nothing typed has neither.
- With realistic river limits the deep-hull problem goes away. The air-draft limit is what stops it. On the 100 x 20 m river LNG barge, "most cargo" with no limits picks tanks in the hold on a **17.40 m** hull. With a 9.1 m air-draft limit the hold case fails, and the best share carries 1,559 t on a 5.22 m hull.
- **The damage runs are not what makes runs slow.** One IGC 2.7 damage run takes 0.05 to 0.15 s of wall time on 10 workers (0.23 to 1.0 s on one). Most of a run is the intact stability stage of the search (GZ curves for each loading condition of each candidate).
- **Why the river barge went from about 2.5 s to about 10 s with #1027.** A river-only run screens no IGC damage. Its damage run is the final ADN/ES-TRIN one, 48 cases in about 0.03 s. The time goes to the share search: share 0.5 fails the intact checks, so the stability stage runs on every candidate at that share. And the whole share search runs twice: once in the depth probe and once in the run pass. PR #1027's text blamed the damage screen; the test `damage_budget.rs` is right.
- The damage-run cap of 24 binds on none of the four designs. The most any design uses is 16 (the 89 x 16 m coastal design). A cap of 12 cuts that design short and gives an infeasible answer.

## Detailed Findings

### 1. What the limits do today

- Inputs: `draught_limit_m`, `air_draft_limit_m`, `bridge_clearance_m`, all `default: null`, group "Waterway limits" (`vdesign.rs`, `INPUTS`, near line 3298; `builtin_profile.json`, near lines 1019, 1026 and 1048).
- Draught limit, in a gas run: it joins the list of draught limits with the channel limit (`simulate.rs`, near line 3337). With a standing tank height, the least external limit goes to the search as `draught_limit_m` (`FitOptions`). There it floors the depth: `target_depth` takes the tank-driven depth, but not less than limit + least freeboard + margin (`gas_arrangement.rs`, near line 1643). Each candidate's draught cap is the less of the limit and its own depth less the freeboard (`search`, near line 1903). The ranking is the tonnage carried at that cap (`ranked_cargo`).
- Air-draft limit: `air_draft_within_limit` is a capacity check on the light, or the ballasted light, condition (`barge_capacity.rs`, near line 2153). It is in `r.checks`, so it is one of the cheap checks of the search (`cheap_checks`). A candidate over it never reaches stability.
- Bridge clearance (#991): a warning only, never a block, on the light air draft. Its basis says no Service Area has a default, as the lowest bridge differs from one waterway to the next. `air_draft_limit_m` is "a design limit and blocks" (`changes/ticket-991-air-draft-bridge-limit.md`).
- A Vessel Class fills both limits, marked `class`: the top of the class's draught range and the bottom of its air-draft range (`class_input.rs`, module note; `vessel_classes.rs`, `draught_limit_m`, `air_draft_limit_m`). A typed limit above the class's value is a warning.

### 2. What "required" means in the code

- `InputSpec.required` (`vdesign.rs`, near line 2011). The required inputs are cargo, hull form, Formation, length, beam and the Service Area.
- The Service Area (#925) is the precedent. It has no default. `run`, `validate`, `fill`, the app and Python `simulate` all give one message that names the values and the fix (`service_area_required`, near line 1607). `ovd fill --service-area` and `ovd new --service-area` set it. The app draws the field with nothing chosen (`LeftPanel.svelte`, `InputField.svelte`, `UNSET`). A file saved without one became an error until set (`changes/ticket-925-service-area-required.md`).
- Saved files: "A stored input is never replaced by a profile value, so a later profile change never changes a saved file" (`vdesign.rs`, module note). A saved file stores the limits as `null` with source `default`. So a rule that reads a stored `null` from before can tell it apart from a value written after the change: after it, `fill` would no longer write a default `null` for a river-only Option.
- Results hash: `warnings` is inside the results hash (`run.rs`, `UNHASHED_KEYS` does not list it). A new warning in the results would move every hash. A validation warning (the `Report` of `check`/`fill`, printed on stderr, shown in the app's messages) does not.
- Python: `simulate(p)` builds one Option the way `ovd fill --as-new` does; a required input left out raises `TypeError` naming it (`src/ovd/simulate.py`). `calibration.py` already passes `draught_limit_m` for its river ships, not `air_draft_limit_m` (near line 800).

### 3. Published limit values

- ECMT Resolution 92/2 (CEMT/CM(92)6/FINAL), Table 1, as stored in `catalogue/vessel_classes.yaml` (confidence "high"). For the classes nearest the two river designs:
  - CEMT Va (motor vessel, 95 to 110 m x 11.4 m): draught 2.5 to 2.8 m; air draft 5.25 to 9.1 m.
  - CEMT VIa (pushed convoy, 95 to 110 m x 22.8 m): draught 2.5 to 4.5 m; air draft 7.0 to 9.1 m.
  - A class fill gives the top of the draught range and the bottom of the air-draft range: VIa gives 4.5 m and 7.0 m.
- UNECE Inventory of Main Standards and Parameters of the E Waterway Network (ECE/TRANS/SC.3/144): 5.25, 7.00 and 9.10 m under bridges, and 9.10 m on routes linked to the coast. **Unverified** (a secondary summary, as the `bridge_clearance_m` basis says).
- No Service Area default can be right for every river: the CEMT air drafts alone span 3.0 to 9.1 m.

### 4. The share search (#992 P5, PR #1027)

- `STANDING_SHARE_STEPS` = 1.0, 0.75, 0.5, 0.25, 0 of the limit. With the river-only limit 0.5 the shares are 0.5, 0.375, 0.25, 0.125 and 0 (tanks in hold).
- `fit_with`: stage 1 searches the shares from the top on the intact checks, with no damage, and stops at the first that passes (`share_stage_one`). Stage 2 screens that share by damage, and the next one down if it fails. So the rule is "the tallest share that passes".
- `simulate_once`: with a standing height it first runs a depth probe (`probe_depth`), which runs `share_stage_one` again, then the run pass, then up to two more passes if the depth moved (`DEPTH_SETTLE_RUNS` = 3).
- A river-only gas run takes the ADN stack (`adn-gas`), not `igc-2g`. `igc_damage_set` is then `None`, so the search screens no damage and does not choose the number of tanks along (`tanks_along_to_try` needs the IGC screen). It stays at 2 along.

### 5. The two rules on the river designs, with limits

Method: `ovd run --json` (release build of `a85c1000`) on fresh `ovd fill --as-new` designs, with the share limit typed at each step so each share is seen. A typed share is a limit, so a share that fails shows the next one down. Limit sets: none; **A** = 4.5 m / 9.1 m (top of CEMT VIa draught, top of its air draft and the UNECE coastal-link figure); **B** = 3.5 m / 9.1 m (inside the VIa draught range); **C** = 2.5 m / 7.0 m (bottom of both VIa ranges). Cargo is tonnes at the draught cap; AD is the light air draft.

100 x 20 m river-only LNG barge, self-propelled:

| Limits | Share | Feasible | Depth (m) | Tanks | OD (m) | Cargo (t) | AD light (m) | Highest point |
|---|---|---|---|---|---|---|---|---|
| none | 0.5 | no (shows 0.375) | | | | | | |
| none | 0.375 | yes | 6.16 | 2 x 2 | 8.10 | 2,351 | 10.32 | vent mast |
| none | 0.25 | yes | 7.17 | 2 x 2 | 8.10 | 2,345 | 10.34 | vent mast |
| none | 0.125 | yes | 12.47 | 1 x 2 | 13.00 | 3,055 | 15.53 | vent mast |
| none | 0.10 (not a step) | yes | 12.80 | 1 x 2 | 13.00 | 3,055 | 15.49 | vent mast |
| none | 0 (hold) | yes | 17.40 | 1 x 2 | 16.00 | 4,559 | 18.32 | stern block |
| A | 0.5 | yes | 4.80 | 2 x 2 | 6.10 | 1,328 | 8.38 | vent mast |
| A | 0.375 | yes | 5.22 | 2 x 2 | 6.60 | 1,559 | 8.85 | vent mast |
| A | 0.25 | yes | 6.05 | 2 x 2 | 6.60 | 1,527 | 8.89 | vent mast |
| A | 0.125 | yes | 6.44 | 2 x 2 | 6.10 | 1,281 | 8.81 | stern block |
| A | 0 (hold) | **no**: air draft 9.79 > 9.10 | 7.50 | 2 x 2 | 6.10 | | 9.79 | stern block |
| B | 0.5 | no (shows 0.375) | | | | | | |
| B | 0.375 | yes | 5.22 | 2 x 2 | 6.60 | 1,559 | 8.85 | vent mast |
| B | 0.25, 0.125 | yes | 6.05, 6.44 | 2 x 2 | 6.60, 6.10 | 1,527, 1,281 | 8.89, 8.81 | |
| B | 0 (hold) | no (air draft 9.80) | | | | | | |
| C | every share | **no**: tank length over diameter (35.3 m / 3.40 m = 10.4 > 7.0) | | 3 x 2 | 3.4 to 4.1 | | | |

110 x 11.4 m river-only LNG tanker, self-propelled (CEMT Va size):

| Limits | 0.5 | 0.375 | 0.25 | 0.125 | 0 (hold) |
|---|---|---|---|---|---|
| none | 1,163 t, 4.90 m, AD 9.65 | **1,461 t**, 6.47 m, AD 10.72 | 1,461 t, 7.55 m | 1,461 t, 8.62 m | 1,461 t, 10.00 m |
| A | **862 t**, 4.80 m, AD 8.70 | 854 t | 838 t | 834 t | no (2 x 2 x 3.8 m closest) |
| B | **870 t**, 4.40 m, AD 8.68 | 854 t | 838 t | 834 t | no |
| C | no feasible share (tank length over diameter) | | | | |

What this shows:
- With no limits, "most cargo" is unsafe: it takes the 100 x 20 m barge to the hold on a 17.40 m hull (4,559 t, air draft 18.3 m).
- With the A or B limits, the deepest hulls fail the air-draft check, and "most cargo" picks a share inside the limits. On the 100 x 20 m barge under A it is share 0.375: 1,559 t against 1,328 t for "tallest that passes" (+17 %), on a 5.22 m hull against 4.80 m. Under B both rules give the same answer. On the 110 m tanker both rules give the same answer under A and B.
- Cargo rises to a peak as the share falls, then falls. On every limited case the peak is the passing share or the one below it.
- At the draught limits tested, the cargo is bound by tank volume, not by draught (draught 1.8 to 2.5 m). The draught limit acts by setting the least depth (4.80 m under A). The air-draft limit is the upper bound.
- The C limits (2.5 m, 7.0 m) give no feasible design on either hull. The tanks become small and long, and with 2 along fixed they fail length over diameter. A river-only run never tries 3 or 4 along (section 4).
- The vent mast is the highest point on most river answers. `vent_mast_lowerable` (#822) exists and takes it out of the air draft.

### 6. Measured run time

Machine: Apple, 10 cores, shared with other build lanes (load average 7 to 70 during the runs). Wall time moved by up to 3 times between repeats, so CPU seconds (user) are the main figure. Each figure is the range of 2 to 4 runs. Tools: `ovd run --json` with `/usr/bin/time -p` on designs with no stored hash; a throwaway harness (not committed) that calls `run_option_with_damage`, counts `damage_runs_started`, logs the damage progress hook with times, and sets `set_damage_run_cap` and `set_damage_workers`.

**One IGC 2.7 damage run, on its own** (from the progress log; cases vary with the arrangement):

| Design | Cases per run | Wall, 10 workers | Wall, 1 worker |
|---|---|---|---|
| 140 x 29 m coastal LNG reference | 74 to 150 | 0.07 to 0.15 s | 0.23 to 0.91 s |
| 89 x 16 m coastal LNG | 78 to 375 | 0.05 to 0.73 s | 0.35 to 1.02 s |
| 100 x 20 m river LNG (ADN/ES-TRIN, final run only) | 48 | about 0.03 s | |

**Whole runs, the current search** (fresh fill; cap 24):

| Design | Answer | Damage runs | CPU s, damage on | CPU s, `--no-damage` | Best wall s |
|---|---|---|---|---|---|
| 140 x 29 m coastal LNG reference, tanks along searched | 2 x 3, 12.0 m, 11.60 m deep, 8,319 t | 5 | 11.4 to 14.4 | 5.1 to 5.6 | 9.2 |
| Same, 2 along typed | 2 x 2, 11.5 m, 11.10 m deep, 8,144 t | 4 | 10.6 to 12.1 | 4.7 to 4.8 | 9.3 |
| 89 x 16 m coastal LNG | 1 x 3, 10.94 m, 9.68 m deep, 1,625 t | 16 | 17.2 to 22.3 | 3.5 to 3.7 | 12.5 |
| 100 x 20 m river LNG | 2 x 2, 8.1 m, share 0.375, 2,351 t | 1 (final, ADN/ES-TRIN) | 10.4 to 13.9 | 9.1 to 9.9 | 10.3 |
| 110 x 11.4 m river LNG | 1 x 2, 7.6 m, share 0.5, 1,163 t | 1 (final) | 0.7 to 1.1 | 0.5 to 0.7 | 0.7 |

On the coastal designs the IGC case work is about 1 to 2 s of a 10 to 20 s run. The rest of the gap between damage on and off is the wider search the screen brings: tanks along 2, 3 and 4, the refinement steps, and the stability stage of each candidate it screens.

**Where the time goes** (`sample`, 1 ms, release):
- 100 x 20 m river barge: `probe_depth` 49 % and the run pass's `fit_with` 48 %; both are almost all `share_stage_one`. About 95 % of the samples are under `stability_assessment` (IS Code loading conditions, `gz_at`, `sections::cut`). The final damage run is under 1 %.
- 140 x 29 m coastal reference: on the main thread, `fit_once` 75 %, `stability_assessment` 54 %, `screen_damage` plus the final damage stages 12 %. Worker threads add about 3.3 CPU s of damage cases.

**Why the river barge went from about 2.5 s to about 10 s** (CPU s, damage on):

| Run | CPU s |
|---|---|
| Tank top typed at 3.04 m (no share search; one share) | 2.4 |
| Share limit typed 0.375 (the first share passes) | 1.8 to 3.4 |
| Share limit 0.5, the default (0.5 fails, then 0.375) | 10.4 to 13.9 |
| Share search off (depth 11.0 m by the ratio, as before #1027, on today's main) | 4.8 |

The extra 7 to 10 s is the failing share 0.5. With no candidate passing, the stability stage runs on every candidate that passed the cheap checks. The probe and the run pass each do this. PR #1027 said: "It is the damage screen of #1020 screening the several candidates the new pick needs". That is not right for a river-only run: there is no IGC screen.

**Infeasible runs are the slowest case.** Under the C limits every share fails, so every share's stability stage runs in full, in each pass: 42 to 56 s on the 100 x 20 m barge and 15 to 33 s on the 110 m tanker.

**The search options on the river designs** (CPU s). "Every share" and "climb" are estimates: the sum of the measured one-share runs (each carries its own run overhead, so the sum is high).

| Design, limits | Current (tallest that passes) | Climb: most cargo, stop when cargo stops rising | Every share searched |
|---|---|---|---|
| 100 x 20, none | 12.2 (2,351 t) | about 15.5 (2,351 t) | about 22 (4,559 t, 17.4 m deep) |
| 100 x 20, A | 1.0 (1,328 t) | about 3.7 (**1,559 t**) | about 16 (1,559 t) |
| 100 x 20, B | 16.1 (1,559 t) | about 19.3 (1,559 t) | about 31 (1,559 t) |
| 110 x 11.4, none | 0.8 (1,163 t) | about 3.6 (1,461 t) | about 10 (1,461 t) |
| 110 x 11.4, A | 0.9 (862 t) | about 1.8 (862 t) | about 7.6 (862 t) |
| 110 x 11.4, B | 0.7 (870 t) | about 1.4 (870 t) | about 6.6 (870 t) |

**The damage-run cap** (coastal designs; the river designs run one damage run whatever the cap):

| Design | Cap 12 | Cap 24 (today) | Cap 48 |
|---|---|---|---|
| 140 x 29 m, along searched | 5 runs, same answer | 5 runs | 5 runs, same answer |
| 140 x 29 m, 2 along | 4 runs, same answer | 4 runs | 4 runs, same answer |
| 89 x 16 m | 13 runs, stops at the limit: **infeasible**, 1 x 3, 10.68 m deep, 1,912 t | 16 runs, feasible, 1,625 t | 16 runs, same as 24 |

### 7. Two costs outside the search

- **`ovd run` runs twice** on a file whose hash `fill` stamped at the unscreened stage. With damage on, `run_design_with_damage` reruns the Option without damage to compare hashes (`run.rs`, near line 1403). On the 140 x 29 m reference, `ovd run` took 16 to 21 s on the stamped file and 10 to 14 s with no stored hash. `fill --as-new` itself runs the Option once to stamp the hash (about 4 to 8 s).
- **The depth probe repeats the share search.** The probe and the run pass each run `share_stage_one` in full. Passing the probe's stage-one answer to the run pass would remove about half the share-search time (estimate from the profile, not measured).

## Code References

- `crates/ovd-core/src/gas_arrangement.rs`: `DEFAULT_DAMAGE_RUN_CAP` (line 72), `STANDING_SHARE_STEPS` (line 83), `target_depth` (line 1643), `search` (line 1827), `fit_with` (line 2489), `share_stage_one` (line 2540), `share_levels` (line 2609), `tanks_along_to_try` (line 2466).
- `crates/ovd-format/src/simulate.rs`: `simulate_once` and the depth probe (line 1487), draught limits and `FitOptions` (lines 3337 to 3560).
- `crates/ovd-format/src/vdesign.rs`: module note (lines 1 to 24), `service_area_required` (line 1607), `INPUTS` limits (lines 3298 to 3317), `Missing` (line 4126), the required-input branch (line 4903).
- `crates/ovd-format/src/class_input.rs`, `crates/ovd-core/src/vessel_classes.rs`, `catalogue/vessel_classes.yaml`.
- `crates/ovd-format/src/run.rs`: `UNHASHED_KEYS` (line 405), `run_design_with_damage` (line 1379).
- `crates/ovd-format/src/damage.rs`: `set_damage_run_cap`, `set_damage_workers`, `DamageProgress`.
- Tests: `crates/ovd-format/tests/damage_budget.rs` (river barge screens no IGC damage), `tank_top_above_deck.rs` (`the_river_barge_is_feasible_because_the_search_takes_a_lower_share_than_half`).

## Open Questions

Carried to the design: the scope of "required" (river-only only, or every Service Area; gas only, or every Cargo), whether a typed "no limit" is allowed, the share rule, and the search breadth.
