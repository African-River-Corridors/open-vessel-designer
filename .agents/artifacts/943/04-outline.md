---
ticket: 943
phase: outline
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Outline: deckhouse tiers sized from what they hold

Add a per-tier plan to the deckhouse with no change in numbers first, then change the sizing in one design-change PR, then adapt the readers that need the shape. No Rule Set data changes.

## Implementation Overview

- [ ] PR 1: `Deckhouse` carries a per-tier plan; results unchanged
- [ ] PR 2: room tiers and access tiers sized from what they hold (the design change)
- [ ] PR 3: #912 openings and the app read the per-tier plan

## PR 1: a per-tier plan, numbers unchanged

`aft_arrangement.rs`: add `TierPlan` and `Deckhouse.tiers_plan`; today's sizing fills it with `tiers` identical full-footprint entries. `area_m2` and `volume_m3` are computed as sums. Results JSON adds `deckhouse.tiers_plan` (unhashed, so no hash moves).

- Test that fails today (core unit test, `Label: property`): `tiers_plan.len() == tiers` and the sum of plan areas equals `area_m2`. Does not compile on `main`.
- `tools/check_fixture_hashes.py` unchanged.

## PR 2: tiers sized from what they hold

`size_deckhouse` per design Q2-Q3. New estimate constants in `deckhouse_sizes`: `SERVICE_ALLOWANCE_M2` (value and basis string for the maintainer to approve) and `ACCESS_STAIR_M2` (from the `rooms.rs` stair row, 1.6 m²). Steel `kcg` from the tiers' volume centre.

- **Test that fails today** (`crates/ovd-format/tests/deckhouse_tiers.rs`, `Label: snapshot`): the sweep case "gas river-only deckhouse 140x29" built in code; assert `deckhouse.area_m2 < 4 × 488.6`. Today 1,954.4.
- Test (`Label: property`): the room tiers' floor ≥ the rooms table `total_m2`, on the river-only case and the small coastal case (`design_impact.py:558`).
- Test (`Label: property`): the wheelhouse front x and the eye height above the keel equal those of `main` on the river-only case (values printed on `main` and pasted, `Label: snapshot`), and the sightline row passes.
- Test (`Label: property`): a container-block Option's results hash is unchanged.
- Design impact: `python3 tools/design_impact.py changelog`; each moved case has a cause in the fragment; lightship falls on every river-only deckhouse case (assert in the PR body table). `python3 tools/refresh_examples.py`; re-pin moved snapshots by running the code.

## PR 3: readers of the shape

- `deckhouse_openings::Frame` (#912) takes each tier's aft face. Test that fails before: a stepped house puts tier 2's entrance at tier 2's aft face, not tier 1's (`Label: property`). If #912 merges after PR 2, this moves into #912's own PR instead.
- App `sternCrew.ts` shows room and access tiers. Test: an e2e or unit check that the Stern & crew view lists both counts.
- GA drawing stays with #783; the per-tier plan is the input it will read.

## Validation per PR

`cargo fmt --all --check`; clippy `-D warnings` on `ovd-core`, `ovd-format`; touched crates' tests `--no-fail-fast`; `python3 tools/release_guard.py` and `--history`; `tools/check_fixture_hashes.py`; `bash tools/gates.sh` on PR 2.

## Open Questions

1. The service allowance on the main-deck tier: what value, and is an estimate acceptable until calibration (#782)?
2. If a lighter house makes the second sightline pass need one more access tier, is "eye height unchanged" still the test, or "sightline still met"?
3. Fallback B (one smaller footprint for all tiers) if the stepped house is too much for #912 and #783 now?
