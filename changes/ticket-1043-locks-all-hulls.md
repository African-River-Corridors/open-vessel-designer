<!-- unreleased -->
- A typed `lock_length_m` or `lock_width_m` the Formation does not fit blocks the Design on every hull (issue #1043): `ovd` 3.4.0 adds `ovd/waterway/within-lock-length` and `ovd/waterway/within-lock-width`, one blocking Rule per Waterway Limit check, so the Rule Trace names the lock as the cause. `arrangement.feasible` is the search's answer about the tanks, and the lock rows carry the block. New Designs take `ovd` 3.4.0; a saved file keeps its version. No lock, or a lock that fits: no change.

<!-- design-impact -->
### ticket-1043-locks-all-hulls (2026-11-10)

One new case moves, and the cause is design call C1, the withdrawn gas-only flip of PR #1074: 1 of 291 case designs, `sweep optional lock_length_m gas` (a case this branch adds). Before, the lock set `arrangement.feasible` false and added a failing "Feasible arrangement" check; now `arrangement.feasible` is true, as with no lock, and the lock's own check still fails. No stored fixture or example hash moves. The two new Rule rows sit in the unhashed `rule_trace`, so the sweep cannot see them; the done-tests (`lock_rule_rows`, `lock_fits_unchanged`) carry that proof.

Between `main 87adb92` and `this branch 4f7080bd`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional lock_length_m gas | `arrangement.feasible` | False | True | changed |
| sweep optional lock_length_m gas | `checks[31].message` | The arrangement fails a lock limit of the route: within lock length. | only methane (LNG) vapour may fuel an engine in a machinery space of category A (IGC Code 16.1); another cargo gas needs the Administration's acceptance (16.9.1) | changed |
| sweep optional lock_length_m gas | `checks[31].name` | Feasible arrangement | cargo as fuel | changed |
| sweep optional lock_length_m gas | `checks[31].severity` | error | warning | changed |
| sweep optional lock_length_m gas | `checks[32].message` | only methane (LNG) vapour may fuel an engine in a machinery space of category A (IGC Code 16.1); another cargo gas needs the Administration's acceptance (16.9.1) | absent | changed |
| sweep optional lock_length_m gas | `checks[32].name` | cargo as fuel | absent | changed |
| sweep optional lock_length_m gas | `checks[32].ok` | False | absent | changed |
| sweep optional lock_length_m gas | `checks[32].severity` | warning | absent | changed |
