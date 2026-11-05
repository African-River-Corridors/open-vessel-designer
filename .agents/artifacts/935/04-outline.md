---
ticket: 935
phase: outline
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Outline: damage survival inside the arrangement search

Move the gas damage loop into `ovd-core` first with no change in results, then call it from the search, then refine and reuse. Each step is one PR to `main`. No Rule Set data changes, so no new Rule Set version.

## Implementation Overview

- [ ] PR 1: the damage loop runs in `ovd-core`, results byte-equal
- [ ] PR 2: the search screens by IGC 2.7 (the design change)
- [ ] PR 3: ADN 9.3.1.15 and ES-TRIN 28.03 screened the same way
- [ ] PR 4: 0.1 m refinement, and reuse of the winner's run
- [ ] PR 5: "not screened" when damage is skipped (after #923)

## PR 1: the damage loop runs in `ovd-core`

New `crates/ovd-core/src/gas_damage.rs`: `igc_damage_data`, `permeability_table`, `GasRun`, `run(&GasRun, spaces, hull, conditions, density) -> GasDamageRun` with typed per-case records and the governing case. `ovd-format/src/damage.rs` keeps the public names as thin wrappers and renders the JSON block from `GasDamageRun`.

- Test that fails today: `ovd-core` unit test `gas_damage_runs_in_core` calls `ovd_core::gas_damage::run` on a small two-tank hull and asserts the verdict names and `ok` flags (`Label: property`). It does not compile on `main`.
- Byte-equal: `tools/check_fixture_hashes.py` and the example `results.json` unchanged (no fragment table needed; fragment says "no design change").
- `cargo build -p ovd-core --no-default-features` proves `no_std`.

## PR 2: the search screens by IGC 2.7

`passes_all` builds the candidate's IS Code conditions with weights (keep them from `stability_assessment`), assembles the vessel model, runs `gas_damage::run` when the `igc-2g` set evaluates `igc/2.7/damage-survival`, and puts the verdicts in `Measured.damage`. `candidate_measured` takes the verdicts. Memo by (depth, tanks across, OD to 1 mm). `failing_checks` names each damage failure as `rule id: condition: case (stage)`.

- **Test that fails today** (`crates/ovd-format/tests/damage_screen.rs`): the 140 × 29 m example as a new coastal deckhouse Option, auto search; assert no `fail` among `igc/2.7*` rows and tank OD ≤ 11.1 m. Today: 13.28 m, 103 fails. `Label: snapshot` for the OD the code prints after the change.
- Test: the same file with `igc-2g` pinned to 2.0.0 gives today's tank (13.28 m) (`Label: snapshot`, printed on `main`).
- Test: every `fit_tried` entry rejected by damage has a reason holding a Rule id starting `igc/2.7`, a case name and the stage.
- Test: a counter in `ArrangementFit` (`damage_runs`); assert it is reported. PR body gives the count and run time before and after on the example.
- Design impact: `python3 tools/design_impact.py changelog`; regenerate every fixture, website example and `examples/`; name each moved Option in the PR body. Re-pin moved snapshots by running the code.

## PR 3: ADN and ES-TRIN damage in the screen

Move `gas_box_damage_verdicts` the same way; screen when `adn-gas` evaluates `adn/9.3.1.15.1/damage-stability`, or `es-trin-2025` evaluates `es-trin/28.03(4)/damage-stability` and LOA > 110 m (`simulate.rs:3273-3320`).

- Test that fails today: a gas Option on a stack with `adn-gas`, sized so the auto tank fails an ADN damage row on `main`; assert no `fail` among `adn/9.3.1.15*` rows. (The builder finds the hull by scan; the scan is pasted in the PR.)
- If no such hull is found, the PR lists ADN and ES-TRIN as gaps instead (ticket item 2 allows this).

## PR 4: refinement and reuse

Refine per design Q3. `ArrangementFit` carries `damage: Option<GasDamageRun>`; `simulate.rs:3249` uses it instead of running again.

- Test that fails today: the example's OD is not on the 0.5 m grid (expect 11.1 m, per #914's scan) (`Label: snapshot`).
- Test: the damage block from the reused run equals a fresh run on the same answer, byte for byte.
- Test: `damage_runs` ≤ 3 on the example, or the PR states the count and asks.

## PR 5: "not screened"

After #923 merges: `FitOptions.damage_screen = Skip` when damage is skipped; `results.arrangement.damage_screened = false`.

- Test that fails today: `ovd run --no-damage` on the example; results say "not screened for damage" and no "meets every evaluated blocking Rule".

## Validation per PR

`cargo fmt --all --check`; clippy `-D warnings` on `ovd-core`, `ovd-format`; the touched crates' tests with `--no-fail-fast`; `python3 tools/release_guard.py` and `--history`; `tools/check_fixture_hashes.py`; `bash tools/gates.sh` on PR 2 (many crates move).

## Open Questions

1. If the plain descending walk needs more than 3 damage runs on the example, is a larger count acceptable, or should the search stop and report?
2. PR 3: is a gap list for ADN and ES-TRIN acceptable if no test hull shows a difference?
3. PR 5 waits for #923. Ship PRs 1-4 first?
