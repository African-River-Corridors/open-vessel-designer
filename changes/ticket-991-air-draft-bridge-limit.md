<!-- unreleased -->
- Air draft report and bridge clearance (issue #991): the results carry an `air_draft_report` (outside the results hash) with the air draft light and laden and the highest point named (`tank_dome`, `hatch_coaming`, `deck`, `stern_block`, `deckhouse` or `vent_mast`). A new input, `bridge_clearance_m` (the clearance under the lowest fixed bridge on the route; none by default), gives a warning, never a block, when the light air draft (ballasted where ballast is carried) is over it; the warning names the highest point and the source of the limit. A file saved before has no bridge clearance. `air_draft_limit_m` is unchanged: it is a design limit and still blocks. Saved Designs and the fixtures store the new input as null.

<!-- design-impact -->
### ticket/991-air-draft-bridge-limit (2026-11-07)

Between `main 970fd84` and `this branch 230cbb49`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional bridge_clearance_m | `warnings[2]` | absent | The light air draft 10.45 m (laden 9.60 m) is over the bridge limit 9.10 m by 1.35 m (typed input bridge_clearance_m). The highest point is the vent mast exit, 11.58 m above the keel. The vessel cannot pass under a bridge of that clearance. The wheelhouse of a tug is not counted for a pushed or ATB design. | changed |
