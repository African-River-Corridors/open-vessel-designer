# Examples and reference designs follow today's rules

Status: **accepted**, 2026-11-05. Issue #910. The maintainer decided on 2026-11-05.

## Context

OVD is still being built, and many Rules change at once. Today parts of the repo work to keep old
results: an example pins old Rule Set versions, and a gas file with no stored Rule Sets is read with
older versions (`trace::legacy_rule_sets`). Pinning hid a real failure: on today's Rules the
140 x 29 m example fails IGC 3.2.4.1 and is under the IGC 4.23.1.2 minimum.

We are in a build process. We calibrate as we go. Rebuilding old designs is not a goal.

## Decision

1. **An example or fixture that fails today's Rules is a bug in the example.** Fix the design (change
   the inputs until it passes). Do not pin the Rule Set stack
   to an older version to keep it passing.
2. **Examples and reference designs track today's Rules.** `examples/`, the website examples and the
   fixtures that stand for a design are regenerated on the current built-in Rule Sets, with the
   Rule Sets they store at their latest version.
3. **Every move shows.** A design that moves appears in the design-impact table of the pull request,
   and the pull request body gives a cause for each moved case. A moved design with no cause is a
   defect in the pull request.
4. **What stays: format migration.** Opening an old file still works. A reader fills a missing field
   with a defined value, a renamed field is read under its new name, and a file of an older version
   is read as the version it is. Reproducing the old results of that file is not a goal.
5. **Released Rule Set versions do not change.** This ADR does not touch the rule in the build
   process: a released file is never edited, and a change is a new version
   (`crates/ovd-format/rules/RELEASED.json`).

## Follow-up (not changed here)

Code that still keeps old results. Each is a separate issue, and each one that goes needs its own
design-impact table.

- `trace::legacy_rule_sets` and `trace::legacy_version`: a gas file with no stored Rule Sets is read
  with the older `is2008-a` 1.0.0 and `igc-2g` 1.1.0 versions.
- The "a file saved before reads X" defaults: about 110 comments and branches in
  `crates/ovd-format/src` (for example in `vdesign.rs`, `objects.rs`, `crew.rs`, `cost.rs`) keep the
  value a file saved before the input read. Most are format migration and stay. Those that exist
  only to hold a result stable are the follow-up.
- `simulate::unstored_rules`: its note about results "pinned to the original Python".
- Examples that pin Rule Set versions: the 140 x 29 m example stores `igc-2g` 2.0.0 and the
  sets beside it as `source: default`. Re-baselining it is its own ticket, because it moves results.

## Consequences

- A reviewer can ask of any example: does it pass today's Rules? If not, fix the example.
- A pull request that regenerates examples needs no migration shim to keep old numbers.
- Calibration still decides which numbers are right. This ADR says only that old numbers are not
  kept for their own sake.
