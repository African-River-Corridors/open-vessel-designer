---
ticket: 913
phase: outline
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: a63ca819db7bb8a6a163350eb256d3767e3c7334
---

# Outline: igc-2g 4.0.0, IGC 3.2.4.1 and 3.2.4.2 on placed openings

Starts after #912 merges. First the vocabulary the rows need (no verdict changes), then the new Rule Set version in its own PR, as the ticket asks, then the evaluation cases.

## Implementation Overview

- [ ] PR 1: object-scoped opening inputs and `opening.clear_of_cargo_face`; no verdict changes
- [ ] PR 2: `igc-2g` 4.0.0 (the design change)
- [ ] PR 3: evaluation cases for the boundaries

## PR 1: vocabulary

- `ovd-format/src/rule_set.rs`: declare `opening_kind`, `opening_space`, `opening_face`, `opening_openable`, `opening_gas_tight` as `InputScope::Object`, value lists from #912's enums and `yes`/`no`.
- `ovd-core/src/measured.rs`: `object_input` answers them for `ObjectType::Opening`.
- `ovd-core/src/rule_trace.rs`: Measure `opening.clear_of_cargo_face` (Bool, `Unverified`).
- Test that fails today (core, `Label: property`): `Measured::object_input("opening", i, "opening_space")` gives the placed opening's space for the default layout; `clear_of_cargo_face` is the negation of `faces_cargo_area` for every opening.
- Test (`Label: property`): `zone_length_m(L)` equals the `clamp` shape at b 0.04, lo 3, hi 5 for L in 0..200 m (one source for the zone).
- `ovd rules --measures` lists the new Measure; results hashes unchanged.

## PR 2: `igc-2g` 4.0.0

- New `crates/ovd-format/rules/igc-2g/4.0.0.json`: 3.3.0 less the four `igc/3.2.4.1/*` input rows, plus the five rows of the design table, `verified: false`, `since: 4.0.0`, each citing IGC MSC.370(93) 3.2.4.1 or 3.2.4.2 with page and edition as the existing rows do.
- Register it in `rule_set.rs`. No change to `RELEASED.json`.
- **Test that fails today** (`crates/ovd-format/tests/igc_openings.rs`): the 140 × 29 m example as a new coastal deckhouse Option; the trace has `igc/3.2.4.1/side-opening-setback-placed` rows, one per side entrance, all `pass`, and no `igc/3.2.4.1/opening-setback` row. Today the new id does not exist.
- Test: a typed opening list with an accommodation door on the front fails `no-opening-faces-cargo`; the governing row names that door (`Label: sourced - IGC 3.2.4.1`).
- Test: a coastal container-block Option has the new rows `not evaluated`, none `pass`.
- `ovd rules --provenance` passes; the fast gate's version-class check accepts a major.
- Design impact: `python3 tools/design_impact.py changelog`; the fragment lists the replaced rows; refresh examples (`tools/refresh_examples.py`) and fixtures whose traces change. No hull number moves; if one does, stop and ask.

## PR 3: evaluation cases

New case kind `openings` in `evaluation/`: a frame (L, deckhouse front) and a typed opening list, with an oracle `oracle/openings_oracle.py` that works the verdict from the IGC text and shares no code with the engine.

| Case | L (m) | Opening | Expected |
|---|---|---|---|
| side entrance at 4.99 m | 140 | accommodation door, side | fail |
| side entrance at 5.00 m | 140 | same | pass |
| side entrance at 2.99 m | 70 | same | fail |
| side entrance at 3.00 m | 70 | same | pass |
| front door | any | accommodation door, front | fail |
| wheelhouse side door in zone, gas-tight | 140 | wheelhouse door, side, 1.0 m | pass |
| same, not gas-tight | 140 | same | fail |

- Each `Label: sourced - IGC 3.2.4.1, 3.2.4.2`. `python3 evaluation/run.py --check` passes; Rule mutation reaches each new row.

## Validation per PR

`cargo fmt --all --check`; clippy `-D warnings` on `ovd-core`, `ovd-format`; touched crates' tests `--no-fail-fast`; `python3 tools/release_guard.py` and `--history`; `tools/check_fixture_hashes.py`; `bash tools/gates.sh` on PR 2.

## Open Questions

1. Does IGC 3.2.4.2 also allow a gas-tight wheelhouse door on the front? The design reads "no" (stricter).
2. Keep the old typed-input rows for container-block Options (Applicability on `arrangement_style`), or drop them as the ticket says?
3. The #898 gaps (airlock, conning arc) stay gaps: file follow-ups, or leave them on #898?
