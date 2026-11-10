<!-- unreleased -->
- A lock that the hull does not fit makes the arrangement infeasible (issue #1043). A typed `lock_length_m` or `lock_width_m` is a hard route limit, like `draught_limit_m` and `air_draft_limit_m`. When "within lock length" or "within lock width" fails, `arrangement.feasible` is false, the results add a failing "Feasible arrangement" check that names the lock check, and the Rule Trace row for the feasible arrangement fails with it. No lock typed, or a lock that fits: no change to the results or the hash.

<!-- design-impact -->
### ticket-1043-lock-feasibility (2026-11-10)

No design change: 290 case designs give identical results on `main 2b25ee7` and on this branch, and every fixture and example hash matches. Only a design that types a lock it does not fit moves, and none of the pinned designs types one.
