<!-- unreleased -->
- A typed `lock_length_m` or `lock_width_m` is a hard route limit, like `draught_limit_m` and `air_draft_limit_m` (issue #1043). How a failed lock check blocks the Design is in `ticket-1043-locks-all-hulls`.

<!-- design-impact -->
### ticket-1043-lock-feasibility (2026-11-10)

No design change: 290 case designs give identical results on `main 2b25ee7` and on this branch, and every fixture and example hash matches. Only a design that types a lock it does not fit moves, and none of the pinned designs types one.
