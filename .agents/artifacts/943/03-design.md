---
ticket: 943
phase: design
date: 2026-11-05
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: d938baecabcccca7a35c90ef1cba8ae04ccac535
---

# Design: deckhouse tiers sized from what they hold

## Summary of the Request

Stop sizing every tier to the whole machinery-space footprint. Room tiers hold the rooms; the wheelhouse sits on top at the height the sightline needs; mass, cost and tonnage follow the smaller house.

## Current State

Every tier is the full footprint (`aft_arrangement.rs:1077`). Sight tiers are full footprint too (`:1098-1117`). The 140 × 29 m river-only sweep case has 4 × 488.6 m² = 1,954 m² for 63.4 m² of rooms.

## Desired End State

- The house is a stack of tiers, each with its own plan: room tiers sized from the rooms table plus an allowance; sight tiers only as large as the wheelhouse needs to stand on and be reached.
- The wheelhouse front stays at the machinery space's forward end, so the eye position and the eye height needed are unchanged.
- Area, volume, steel, cost and GT are sums over the tiers. Lightship falls.

## Scope

In: tier shapes, the wheelhouse support, mass/cost/tonnage, results JSON, the #912 frame.
Out: the GA drawing (#783); placing openings (#912); a sourced service-space floor (an estimate here).

## Design Questions

### Q1. What shape does the house take?

- **A. Stepped house, front-aligned (chosen).** Room tiers keep the full width and the front face at the machinery space's forward end; their length is `floor_needed / width`, cut from the aft end. Above them, sight tiers are a trunk: the wheelhouse plan plus a stair, under the wheelhouse. The IGC 3.2.4 front stays where it is, so the opening zone does not move.
- B. One smaller footprint for every tier: sized for the room floor spread over all tiers. Keeps a uniform frame for #912, but sight tiers still carry empty floor.
- C. Wheelhouse on a column or mast, no sight tiers. Lightest, but a column of 5-8 m has no mass basis in the tree, and it changes the wheelhouse type (D4).

A is the smallest house that holds what it must. B is the fallback.

### Q2. What does a room tier hold?

`floor_needed = rooms total + stair per extra tier + service allowance`. The rooms table counts one stair (`rooms.rs:440-446`); add one per room tier above the first. The service allowance (casing, workshop, stores) has no source: an **estimate** constant, flagged in the results as `size_status: estimate`. Room tiers: `n = ceil(floor_needed / full_tier_area)`; floor spread evenly, so each room tier's length is `floor_needed / (n × width)`, not less than the wheelhouse length.

### Q3. What does a sight tier hold?

Plan = wheelhouse length × wheelhouse width (defaults 3.60 × 6.40 m, `aft_arrangement.rs:35-36`) plus one stair. Count as today: the tiers needed to put the floor at `eye − eye_above_floor`, less the room tiers.

### Q4. How is the shape carried?

`Deckhouse` gains `tiers_plan: Vec<TierPlan { x_aft_m, x_fwd_m, width_m, base_m, kind: Rooms | Access }>`. `area_m2`, `volume_m3` become sums; `tier_area_m2` is kept as the first tier's area so the JSON key stays. Steel `kcg` is the volume-weighted centre of the tiers, not the mid-height of a box.

### Q5. How does #912 read it?

`deckhouse_openings::Frame` takes the per-tier aft face: each tier's entrance on its own aft face. The front face is the same for every tier under A, so zone measurements do not change. Coordinate with the #912 builder; whichever merges second adapts.

## Proposed End State Architecture

```
size_deckhouse → floor_needed → room tiers (front-aligned, shortened)
              → eye floor     → access tiers (wheelhouse plan + stair)
Deckhouse { tiers_plan, area = Σ, volume = Σ } → mass, cost, GT, Frame (#912)
```

## What moves in design impact

Every deckhouse Option: deckhouse area and volume, deckhouse steel, lightship, centre of gravity, light draught, GT (and so crew and rooms through the GT loop), cost. Cargo can rise where lightship bound it. The river-only 140 × 29 m case falls from 1,954 m² to about (63.4 + allowance) + 3 × ~26 m². Container-block Options do not move.

## Risks

- The light waterline moves with lightship, so the second sightline pass may need one more access tier (`barge_capacity.rs:1439-1460`). Access tiers are small, so the effect is small; the test pins eye height.
- GT feeds the crew and rooms (`run.rs:888-899`): the rooms loop may settle on a different berth count. The existing settle warning covers it.
- An unsourced service allowance replaces an unsourced full footprint. It is still an estimate, now visible.
- #912 is in flight on the same struct: merge-order conflict.

## Patterns to Follow

Estimates as named constants with a basis string (`deckhouse_sizes`, `aft_arrangement.rs:83-106`); mass items with centres (`:688-706`); seam tests via `.vdesign` (`tests/arrangement_style.rs`).
