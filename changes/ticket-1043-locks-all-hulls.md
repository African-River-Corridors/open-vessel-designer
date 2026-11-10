<!-- unreleased -->
- A typed `lock_length_m` or `lock_width_m` the Formation does not fit blocks the Design on every hull (issue #1043): `ovd` 3.4.0 adds `ovd/waterway/within-lock-length` and `ovd/waterway/within-lock-width`, one blocking Rule per Waterway Limit check, so the Rule Trace names the lock as the cause. `arrangement.feasible` is the search's answer about the tanks, and the lock rows carry the block. New Designs take `ovd` 3.4.0; a saved file keeps its version. No lock, or a lock that fits: no change.

<!-- design-impact -->
### ticket-1043-locks-all-hulls (2026-11-10)

No design change. The sweep compares hashed keys only, and the two new Rule rows sit in the unhashed `rule_trace`, so it cannot see them. A new case, `lock_length_m gas` (LPG propane, Type C tanks, self-propelled, a 100 m lock), shows the gas hull gives the same hashed results. The done-tests (`lock_rule_rows`, `lock_fits_unchanged`) carry the proof that the rows fail, pass and stay not applicable as they should.
