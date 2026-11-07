---
ticket: 994
phase: research-questions
date: 2026-11-06
status: draft
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Research Questions: arrangement style "wheelhouse forward"

## Key Context Pointers

- Ticket #994 (lists the rules). Style enum: `crates/ovd-core/src/aft_arrangement.rs` (`ArrangementStyle`, `ARRANGEMENT_STYLES`).
- Sightline: `crates/ovd-core/src/wheelhouse.rs` (`Obstruction`, `Sightline`, blind distance); inputs `wheelhouse_eye_above_floor_m`, `sightline_margin_m` in `aft_arrangement.rs`.
- Openings: `crates/ovd-core/src/deckhouse_openings.rs` (`Face`, `default_layout`, `zone_length_m`, `place_for_vessel`).
- Rule text: ADN 9.3.1.17.1; IGC 3.2.4.1, 3.2.4.2, 3.2.5, 8.2.10.3, 8.2.11.1, 4.28.2.1; SOLAS V/22.1.3, 22.1.4, 22.1.6; ES-TRIN 7.02(3); ICLL 39.

## Questions

1. Which code assumes the wheelhouse is aft (stern block, `Face` names, sightline eye position)?
2. Does the sightline already ignore obstructions behind the eye? (Ticket says yes.) What does a forward eye see over?
3. Can `deckhouse_openings` express "the deckhouse faces aft toward the cargo"? Which `Face` values exist?
4. Where do the style and Service Area gate each other today?
5. What do weight, trim and the drawings read from the stern block?
6. Which Rule rows exist for 3.2.4, 3.2.5, 8.2.10.3 and 8.2.11.1, and which need new rows?
