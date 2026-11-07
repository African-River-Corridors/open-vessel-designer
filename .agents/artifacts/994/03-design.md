---
ticket: 994
phase: design
date: 2026-11-06
status: approved
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Design: arrangement style "wheelhouse forward"

## Summary of the Request

Add a fourth style, flagged for a hazard study. Every Service Area is allowed (maintainer's choice at the gate), with a warning on a coastal design. Machinery and (most) accommodation stay aft. IGC 3.2.4 is read strictly (maintainer's decision): no door on a face toward the cargo area.

## Decisions

### D1. Style key and where the wheelhouse sits

- A. Key `wheelhouse_forward`, wheelhouse over the forepeak (no cargo length lost). Recommended.
- B. Same key, wheelhouse takes cargo length forward (a deckhouse ahead of the first tank).
- C. Reuse `deckhouse` with a position flag. Hides a hazard-study style inside a normal one.

Recommend A. Add the key to `ARRANGEMENT_STYLES`. If the forepeak is too short for the house, fall back to B with a warning naming the cargo length lost.

### D2. What stays aft

- A. Machinery, gensets, fuel system and the main accommodation aft; forward house holds the wheelhouse, a watch room and a muster point. Recommended.
- B. All accommodation forward. Breaks the A-60 and escape reasoning; leaves machinery unattended with no berths near.

Recommend A. The crew is split across about 100 m of cargo deck. Record two escape routes, a second muster point, and an unmanned or lightly manned engine room as flagged inputs, not computed checks.

### D3. Sightline and rear view

- Eye forward: the eye sits in the forward house; `wheelhouse.rs` already skips obstructions behind the eye, so cargo gives no blind distance. Obstructions ahead of the eye (the bow, a mooring winch) still count.
- Rear view: SOLAS V/22.1.3 needs no view astern from the conning position, but the wings must see aft along the hull (22.1.4, 22.1.6). ES-TRIN 7.02(3) may need cameras.
- A. Wing check only; cameras reported as an open item. Recommended.
- B. Credit a camera as meeting 7.02(3). Needs a source for the camera standard; not read.

Recommend A. New Measure: a wing sees the full hull side (wing position vs beam). No camera credit until a source is read.

### D4. New Rule rows

- IGC 3.2.4.1/3.2.4.2: reuse `deckhouse_openings` with the cargo-facing face set to aft. Doors only on the fore face, or on the sides at least 4 % of L (3 to 5 m) back from the aft face. Wheelhouse windows may open only per 3.2.4.2.
- IGC 3.2.5: aft face A-60 as a reported requirement (MSC.411(97) text), not a computed fire design.
- IGC 8.2.11.1: vent mast 25 m from openings; 8.2.10.3: exit at B/3. New Measures: distance from each vent mast to each opening.
- A. Add the rows in a new Rule Set version. Recommended. B. Report as notes only (no pass/fail). Weaker; the ticket wants a flag, not a check.

Recommend A: Rule data change, so a new `igc-2g` minor version; older versions stay pinned. Rows cite paragraphs, no text copied.

### D5. Hazard-study flag

- A. `results.arrangement.hazard_study_required: true` with reasons (no gas-cargo vessel in service; split crew; rear view; bow acceleration), and a boxed line at the top of the report. Recommended.
- B. A warning in the list only. Easy to miss.

Recommend A. The verdict line may say rules met but never "approved"; it names the hazard study.

### D6. Service Areas

- A. River and estuary only; other Service Areas fail the style with a clear reason.
- B. Coastal allowed with a warning (bow acceleration about 1.2 times the stern's, IGC 4.28.2.1; ICLL 39 not binding).

**Maintainer's choice at the gate: all Service Areas allowed, with a warning on coastal designs (bow motions).** This overrides the first recommendation ("river and estuary only"). No Service Area fails the style; a design whose Service Area reaches sea gets the bow-motion warning, and the bow-motion reason joins the hazard-study reasons.

### D7. Weight, trim and drawings

- Move the house mass from the stern block to the bow; the stern block keeps machinery and accommodation mass. Reuse `stern_block_mass` with a split. Trim and the weight check read the new longitudinal centre. Drawings: a forward house on the profile and plan; a vent-mast distance dimension. Recommended: do weight first, drawings last.

### D8. Re-pins

- New style only; existing styles unchanged, so no fixture should move.
- A new Rule Set version moves nothing pinned to an older version. Run `tools/design_impact.py changelog` to prove no Option moved; re-pin only new snapshots by running the code.

## Maintainer's choices at the gate (2026-11-06)

- Service Areas: all allowed, with a warning on coastal designs (D6 above).
- Everything else follows the recommendations: D1 A (wheelhouse over the forepeak), D2 A (machinery and main accommodation aft), D3 A (eye forward, wing view-aft check, no camera credit), D4 A (new `igc-2g` rows for 3.2.4 on the aft face, 8.2.11.1 and 8.2.10.3 vent distances, a 3.2.5 A-60 note; new Rule Set versions follow the repo's immutability rules), D5 A (flag in results and report).
- IGC 3.2.4.2 is read strictly (standing decision): no door on a face toward the cargo area.

## Out of Scope

Camera standard, a computed escape analysis, fire design, coastal motions.
