<!-- unreleased -->
- A typed `lock_length_m` or `lock_width_m` the Formation does not fit blocks the Design on every hull (issue #1043): `ovd` 3.4.0 adds `ovd/waterway/within-lock-length` and `ovd/waterway/within-lock-width`, one blocking Rule per Waterway Limit check, so the Rule Trace names the lock as the cause. `arrangement.feasible` is the search's answer about the tanks, and the lock rows carry the block. New Designs take `ovd` 3.4.0; a saved file keeps its version. No lock, or a lock that fits: no change.

<!-- design-impact -->
### ticket-1043-locks-all-hulls (2026-11-10)

One new case moves, and the cause is the withdrawn flip of PR #1074: 1 of 291 case designs, `sweep optional lock_length_m gas` (LPG propane, Type C tanks, self-propelled, a 100 m lock, a case this branch adds). Before, the lock flipped `arrangement.feasible` to false and added a failing "Feasible arrangement" check. Now `arrangement.feasible` is true, as with no lock, and the "Feasible arrangement" check is gone; the lock's own check still fails. No stored fixture or example hash moves. The two new Rule rows sit in the unhashed `rule_trace`, so the sweep cannot see them; the done-tests (`lock_rule_rows`, `lock_fits_unchanged`) carry the proof.
