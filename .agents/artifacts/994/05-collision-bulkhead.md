---
ticket: 994
phase: design
date: 2026-11-10
status: draft, for the maintainer
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 87adb923b5dc5d90086fb4e3a35adf57ed1aa57f
---

# Design: the forward house stands aft of the collision bulkhead (ES-TRIN 3.03)

## Summary of the request

The maintainer decided on 2026-11-10: rework the `wheelhouse_forward` style so the forward house stands
aft of the collision bulkhead plane. ES-TRIN 2025/1 Article 3.03(2) allows no accommodation, and no
installation needed for the vessel's safety, ahead of that plane; only the anchor gear is exempt. The
deck ahead of the bulkhead is the foredeck for the anchor and mooring gear. OVD must show the cargo
length this costs.

PR #1075 (branch `agent/lane-f994b`), which put the house over the raked bow with 1.8 m of clear deck
ahead, is held and will not merge. The design doc 03 chose D1 A, "the wheelhouse over the forepeak". The
forepeak is the compartment ahead of the collision bulkhead, so D1 A conflicts with 3.03(2). This note
replaces D1 A.

## 1. The rule

### What ES-TRIN says (Edition 2025/1, signed English text, cesni.eu; the copy the `es-trin-2025` Rule
Set hashes)

- **1.01, 4.16** "length" or L: the maximum length of the hull in m, excluding rudder and bowsprit.
- **1.01, 4.18** "length of waterline" or L_WL: the length of the hull in m, measured at the maximum
  draught.
- **1.01, 4.25** "forward perpendicular": the vertical line at the forward point of the intersection of
  the hull with the maximum draught line.
- **3.03(1)(a)**: a collision bulkhead at a suitable distance from the bow so the laden vessel stays
  buoyant with 100 mm of residual safety clearance when the compartment ahead of it floods. As a general
  rule this is met when the bulkhead stands between 0.04 L and 0.04 L + 2 m aft of the forward
  perpendicular in the plane of maximum draught. Beyond 0.04 L + 2 m it must be proved by calculation.
  The distance may be reduced to 0.03 L, proved by a calculation with the compartment ahead and the
  adjacent ones flooded.
- **3.03(2)**: no accommodation or installations needed for the safety of the vessel and its operation
  may be located ahead of the plane of the collision bulkhead. This does not apply to anchor gear.
- **19.02(3)** (passenger vessels) takes L_WL in place of L. Not OVD's case.
- **21.01(4)**: a pusher may do without the collision bulkhead. **21.02(2)(a)**: a ship-borne lighter
  up to 40 m may do without it if its front face bears 2.5 times the load. Not modelled: both are
  outside the self-propelled gas barge this style serves.

### Where OVD puts the collision bulkhead today

- The structure model only. `bulkhead_xs` in `crates/ovd-core/src/hull_plates.rs` (the doc comment at
  line 380) puts it midway along the bow zone: `(cargo_zone_fwd_m + loa_m) / 2`. The shell model
  (`crates/ovd-core/src/hull_shell.rs`, `bulkheads`) uses the same point, and the panel model
  (`hull_panels.rs`) counts it among the `holds_along + 3` bulkheads.
- The arrangement, the drawings and the Rules have no plane. The MARPOL row `marpol/19.7/collision-bulkhead`
  says so in its gap: "the model has no collision bulkhead position".
- Today's `wheelhouse_forward` (main) puts the house at `loa - 3 m - 8 m`, so its front is 3 m from
  the deck end; PR #1075 put the front 1.8 m from the deck end on the 110 m barge.

Against 3.03(1)(a), on the three barges of this note (self-propelled LNG, `raked_box`, default rake):

| Barge | L (m) | Rake (m) | Max draught (m) | Forward perpendicular, x (m) | 0.04 L plane, x (m) | Today's midway bulkhead, x (m) | Today's house front, x (m) |
|---|---|---|---|---|---|---|---|
| 70 x 10 x 4, river | 70 | 6.4 | 3.7 | 69.52 | 66.72 | 66.70 | 67.00 |
| 110 x 11.4 x 6, river | 110 | 9.6 | 5.7 | 109.52 | 105.12 | 105.10 | 107.00 |
| 140 x 29 x 12, coastal | 140 | 10.0 | 11.7 | 139.75 | 134.15 | 134.90 | 137.00 |

x is metres forward of the stern, as the house site reports it. The midway bulkhead happens to sit near
0.04 L because the default rake is 1.6 x depth and L / depth is near 20 on these hulls; it is not a rule.
The house front stands 1.9 m (110 m), 0.3 m (70 m) and 2.85 m (140 m) ahead of the plane today.

The forward perpendicular on a raked bow: the keel ends `rake` aft of the bow and rises `rise` to the
deck at the bow (`rise` is the moulded depth unless typed, `bow_rake_rise_m`). At the maximum draught
T_max the hull meets the water at

    x_fp = loa - rake x max(0, 1 - T_max / rise)

For T_max at or above the rise, and for no rake, x_fp = loa. The rise is the hull's `bow_rake_rise`
in `crates/ovd-core/src/hull.rs` (`rake_waterplane_length`); this is the same geometry.

### Decisions

**D1. The plane is an output of every design, with the distance an input.**

- `collision_bulkhead_m`: the distance of the collision bulkhead aft of the forward perpendicular, in
  the plane of maximum draught (m). None (the default) gives 0.04 L, the bottom of the 3.03(1)(a) band,
  which keeps the most cargo. A typed value is checked by the Rule rows of section 2 and never clamped.
- L is the hull's length overall. ES-TRIN 1.01, 4.16 reads L as the maximum hull length without rudder
  and bowsprit; OVD's `length_overall_m` is the hull, with no appendages modelled.
- The plane of maximum draught is the hull's deepest draught, `moulded_depth_m - required freeboard`
  (what the results report as `draught_max_m` before a route limit). A route's draught limit limits the
  loading, not the certificate draught, so it does not move the plane. (A call for the maintainer; made
  here and logged.)
- Reported in `geometry`: `forward_perpendicular_x_m`, `collision_bulkhead_from_fp_m`,
  `collision_bulkhead_x_m`, for every Arrangement Style and every hull form the capacity chain builds.
  A pushed barge reports them too (its hull has a bow); the house rule does not apply to it.

**D2. The forward house: front on the plane, never ahead of it.**

- x_fwd = min(plane, loa - BOW_CLEAR_M). The 3 m clear deck constant stays as the floor for the
  anchor gear; the plane sets a longer foredeck on every barge here (3.28 m, 4.88 m, 5.85 m).
- x_aft = x_fwd - LENGTH_M (8 m). The cargo zone ends at or aft of x_aft: `bow_void_m =
  max(rake + fwd_sep, loa - x_aft)`, and `cargo_length_lost_m = bow_void_m - (rake + fwd_sep)`.
- The lost length is a reported number, with the warning reworded: "The forward house stands aft of the
  collision bulkhead plane: the house takes {lost} m of cargo length." It keeps the words "takes" and
  "of cargo length" so the existing test in `wheelhouse_forward.rs` (a short forepeak) still passes.
- A long rake (20 m typed on the 110 m barge) moves the house aft from the stem to the plane and costs no
  cargo length; a short typed rake costs more than today, because the plane, not the rake, sets the house.

**D3. The structure model keeps its midway bulkhead in this build; a follow-up moves it onto the plane.**

One bulkhead position is right. But moving the panel and shell bulkhead changes the steel mass of every
design by a little (the bulkhead's section changes along the rake), so every case in the design-impact
sweep, every fixture, the website examples and `examples/` move and re-pin. That is a second build with
its own design-impact table, not a side effect of the house. The coordinator should open it as its own
issue: "the structure model takes the collision bulkhead from the plane". Until then the two positions
differ by 0.02 m (110 m), 0.02 m (70 m) and 0.75 m (140 m) on the default rake. (A call for the
maintainer; made here and logged.)

**D4. What a coastal-and-river Option does.**

- `es-trin-2025` is in both default stacks (`builtin_profile.json`, `rule_sets.default_stacks`), so the
  3.03 rows of section 2 judge a coastal Option as they judge a river one.
- **SOLAS II-1/12** was checked. The text is not in the rule sources (`~/Dev/ovcd-rule-sources/solas`
  holds MSC.269(85) only, the IS Code amendments). Secondary sources (Gard, "Collision bulkhead: very
  important valve"; Marine Insight, "What is Collision Bulkhead in Ships?") give the bulkhead not less
  than 0.05 L or 10 m, whichever is less, and not more than 0.08 L or 0.05 L + 3 m, whichever is greater,
  from the forward perpendicular. A regulation Rule needs its paragraph read in the source edition, so
  this is a named gap, not a row, until the SOLAS II-1 text joins the sources. On the 110 m barge the
  SOLAS minimum would be 5.5 m against ES-TRIN's 4.4 m.
- **IGC Code 2.4.4** (MSC.370(93), annex p. 29): "Cargo tanks shall not be located forward of the
  collision bulkhead." The IGC Code does not place the bulkhead; it takes the ship's. This becomes an
  evaluated row in `igc-2g` (section 2). It has teeth: a short typed rake on a coastal gas design ends
  the cargo zone ahead of the plane and fails it; the forward house, whose aft face ends the cargo zone,
  passes it.
- **ICLL** (MSC.143(77), Regulation 3(2)) defines the load-line forward perpendicular on the waterline
  of the load-line length, not on the maximum draught. OVD keeps ES-TRIN's definition for the plane; the
  load-line L of `deckhouse.icll_length` is a different measure and is not touched.
- No SOLAS or IGC rule was found that bars a wheelhouse ahead of the collision bulkhead on a cargo ship.
  The 3.03(2) row is the only house check, in both Service Areas.

**D5. Rule rows: new versions, nothing edited.** `es-trin-2025` 2.2.0 and `igc-2g` 4.2.0 (minor: rows
added only; spec 0004, "Version numbers"). The default stack takes the highest built-in version, so new
Options get the rows; a stored file keeps its pinned versions. The MLC row
`mlc/A3.1.6(c)/sleeping-room-location` stays as it is: the rooms table still places no room by deck or
position, and the forward house holds no sleeping room (D2 A of doc 03), so the 3.03(2) row covers the
house and the MLC row keeps its gap.

**D6. The hazard-study flag is unchanged.** The reasons of D5 (doc 03) stand; the plane adds none.

## 2. The check

Measures (`crates/ovd-core/src/rule_trace.rs`, with `measured.rs` and `ovd-format/src/trace.rs`
registering them like `wheelhouse.wing_sees_hull_side`):

| Measure | Value | Method |
|---|---|---|
| `arrangement.house_aft_of_collision_bulkhead` | Bool | Whether every house on deck (the forward house of `wheelhouse_forward`; the deckhouse or container block of the other styles) has its forward face at or aft of the collision bulkhead plane. Only an Option with a stern block has a value |
| `arrangement.collision_bulkhead_within_band` | Bool | Whether the distance from the forward perpendicular lies in [0.04 L, 0.04 L + 2 m] |
| `arrangement.collision_bulkhead_at_least_0_03_l` | Bool | Whether that distance is at least 0.03 L |
| `arrangement.cargo_aft_of_collision_bulkhead` | Bool | Whether the cargo zone's forward end is at or aft of the plane |

The builder may give the band and minimum as `formula` rows on a Number Measure
(`arrangement.collision_bulkhead_from_fp`, m) if a limit shape fits; the Bool form is the floor.

Rows (ids are fixed by the done-tests):

| Row | Set, version | Kind, Measure | Consequence | Applies when |
|---|---|---|---|---|
| `es-trin/3.03(2)/no-accommodation-ahead-of-collision-bulkhead` | es-trin-2025 2.2.0 | check, house_aft_of_collision_bulkhead | block | formation_kind self_propelled |
| `es-trin/3.03(1)(a)/collision-bulkhead-position` | es-trin-2025 2.2.0 | check, within_band | warn (outside the band a flooding calculation must prove it; OVD makes none) | always |
| `es-trin/3.03(1)(a)/collision-bulkhead-minimum` | es-trin-2025 2.2.0 | check, at_least_0_03_l | block | always |
| `igc/2.4.4/no-cargo-tank-forward-of-collision-bulkhead` | igc-2g 4.2.0 | check, cargo_aft_of_collision_bulkhead | block | always (the set is the coastal gas stack) |

Source fields: ES-TRIN rows cite "ES_TRIN_2025_signed_en.pdf, Chapter 3, p. 14" and Chapter 1, p. 5 for
the definitions; the IGC row cites "MSC.370-93.pdf, annex p. 29" with the edition string of the other
`igc-2g` rows. `codified.review` is the new build issue. `verified: false`.

The 3.03(2) row passes by construction for the forward house (D2 sets the front on the plane) and for
the aft styles (their houses stand on the stern block). It is still the row that proves it, as
`ovd/aft-block/wheelhouse-fits-beam` proves a fit. The failing path is tested in `ovd-core`: a
`ForwardHouseSite` placed ahead of the plane fails the stern block's check "house aft of collision
bulkhead" (`AftArrangementInputs.collision_bulkhead_x_m`, `AftArrangementResult.house_aft_of_collision_bulkhead`).

## 3. The cost

Worked from today's results with the formulas of D1 and D2 (`draught_max_m`, the default rake, the
0.2 m forward end-bulkhead space); the build's tests print the real values. Cargo zone in m.

| Barge | Deckhouse style | `wheelhouse_forward` today (main) | PR #1075 (held) | This design: house aft of the plane | Cost against deckhouse | Foredeck ahead of the house |
|---|---|---|---|---|---|---|
| 70 x 10 x 4, river | 45.368 | 40.768 (4.60 lost) | 43.968 (1.40 lost) | 40.688 (4.68 lost) | 10.3 % | 3.28 m |
| 110 x 11.4 x 6, river | 82.168 | 80.768 (1.40 lost) | 82.168 (0 lost) | 79.088 (3.08 lost) | 3.7 % | 4.88 m |
| 140 x 29 x 12, coastal | 111.768 | 110.768 (1.00 lost) | 111.768 (0 lost) | 108.118 (3.65 lost) | 3.3 % | 5.85 m |

The cost is the house length plus the foredeck less what the bow void already gave: on the 110 m barge
8 m + 4.88 m - 9.8 m. A typed `collision_bulkhead_m` above 0.04 L costs the same again, metre for
metre. The 70 m barge loses a tenth of its cargo zone: the style suits the longer hulls.

## 4. What to keep from PR #1075

- **Keep, reused**: its test helpers (`results_of`, `house`, `lost`, `cargo`, `rake`, `fwd_sep`,
  `says_house_takes_cargo`) are the shape of the new `collision_bulkhead.rs`; and its test
  `the_other_styles_keep_their_results` became `the_other_styles_keep_their_cargo_zone`.
- **Drop**: the code change (the clear deck clamped to what the bow leaves) and the tests
  `the_default_rake_holds_house_and_clear_deck_and_no_cargo_length_is_lost`,
  `the_house_lies_within_the_rake_and_forward_separation_and_the_clear_deck_ends_at_the_stem`,
  `a_long_rake_keeps_the_full_clear_deck_ahead_of_the_house` and
  `a_typed_rake_too_short_for_the_house_itself_takes_cargo_length`: each asserts the house at the stem.
  Its fragment `changes/agent-lane-f994b.md` goes with it.
- The existing tests in `crates/ovd-format/tests/wheelhouse_forward.rs` and `forward_house_rules.rs`
  stand: the long-rake equality with the deckhouse holds (no length lost), the short-forepeak test keeps
  its words, and the vent, opening and blind-distance rows do not read the house's x.

## Done-tests

`.done-test/994` (the coordinator must update its `until:` line when the new build issue opens):

- `crates/ovd-core/tests/collision_bulkhead.rs`: the forward perpendicular, the default distance, the
  failing and passing check. Fails on main at compile time: the module and the two fields are new.
- `crates/ovd-format/tests/collision_bulkhead.rs`: the plane keys for every style, a typed distance,
  the house on the plane, the cost, a long rake, the other styles' snapshot. Five of six fail on main.
- `crates/ovd-format/tests/collision_bulkhead_rules.rs`: the four rows, the band and the minimum,
  the IGC row on a coastal design and not on a river one, the MLC gap. Six of eight fail on main.

## Out of scope

The structure model's bulkhead (D3, a follow-up issue); SOLAS II-1/12 until its text is in the sources;
the flooding calculation of 3.03(1)(a); the plane on the drawings (goes with the structure follow-up);
the pusher and lighter exemptions.
