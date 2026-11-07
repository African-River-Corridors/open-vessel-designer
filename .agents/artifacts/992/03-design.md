---
ticket: 992
phase: design
date: 2026-11-07
status: draft (maintainer decisions folded in 2026-11-07)
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 3559dc191479a633bd06b48bceb772cf020a1e25
---

# Design: gas depth with larger tanks standing above the deck under a weather shield

## Summary of the Request

Prefer a larger tank OD on a shallower hull. Let the tank tops stand above the deck under a weather shield, instead of buying full moulded depth.

## Current State

Depth is 0.55 x beam for gas. The 140 x 29 m coastal LNG reference design gets 15.95 m, deeper than any of the 8 reference carriers. Depth grows with the tank when the tank grows. There is no shield.

## Maintainer decisions (2026-11-07)

- Coastal default: tank tops up to **2.0 m** above the main deck, under a weather shield ("use 2m for now as a default on coastal").
- River-only default: the tanks may stand up to their widest point above deck. The main deck sits at the tank axis, so the height above deck is up to OD/2.
- **One input covers #992 and #993.** The height of the tank top above the main deck. 0 m is tanks in hold (#993's pattern). #993 is superseded by this input.
- Question to answer: do full-hold barges have one build standard over the whole depth, or is the structure strong for some height and then only watertight? See "Structure above the strength deck" below.

## Desired End State

A gas hull whose depth follows the tank, less the height the tank is allowed to stand above the deck, and never less than freeboard needs. At 0 m the deck is flat and the tanks are in the hold, with domes through the deck. Above 0 m a modelled shield covers the tank tops. Sightline, damage and air draft are checked with the shield in.

## Structure above the strength deck

**Question.** Is the barge built to one standard over its full depth, or is it structural to some height and then only watertight?

**Marks:** *sourced* = text I read; *inference* = my reading; *unverified* = a search summary only, not the text.

**Short answer.** The hull girder runs up to the strength deck. Above it, a structure counts towards longitudinal strength only if it is built as part of the girder (an effective trunk, or continuous coamings with under-deck support). Anything else, a shield or a weathertight cover, is weight and windage only. A full hold is *structural and watertight up to the main deck*, not "structural low and watertight high". The split happens at the main deck, and it is a split between hull and appendage.

1. **Depth is measured to the strength deck.** ABS Steel Barges 2024, 3-1-1/7: depth D is "measured amidships from the molded baseline to the molded line of the strength deck plating at the side". *Sourced.* The strength deck is the deck that forms the top of the effective hull girder (ABS Steel Vessels, 3-1-1; *unverified*, search summary).
2. **What counts in the section modulus.** 3-2-1/3.3: deck plating (strength deck and other effective decks), shell, inner bottom, girders, and longitudinal bulkheads count, "provided they are continuous or effectively developed throughout the midship 0.4L and gradually tapered beyond". *Sourced.*
3. **A trunk can count.** 3-2-3/3: a trunk is an effective part of the hull girder if it runs at least the midship 0.6L and is at least 0.6B wide. Trunk side: not less than 8.5 mm (30 m barges) to 11 mm (225 m and up), interpolated. Trunk top within 0.4L: not less than the hull girder needs, and not less than the deck-plate formula. Buckling must be checked. The modulus to deck is then taken to the molded deck line plus the height of the effective trunk (3-2-1/3.3). *Sourced.*
4. **Continuous coamings can count.** 3-2-1/3.3: strength-deck longitudinal hatch coamings longer than 0.14L, supported by under-deck longitudinal bulkheads or deep girders, may be included. The section modulus to the coaming top uses a distance capped at yt = y(0.9 + 0.2x/B). *Sourced.*
5. **A shield or a weathertight cover does not count.** It is not continuous steel in the girder, so it adds no modulus. Local rules still apply to what is exposed. ABS 3-2-9/5.1: coaming height 600 mm in Position 1, 450 mm in Position 2 for tarpaulin-and-batten hatches, lower with gasketed steel covers. *Sourced.* Plate and stiffener sizes for a shield against green water are set by the class, per exposure position. I did not read a rule for a free-standing shield over tanks. *Gap.*
6. **ICLL reg. 36 and the load line.** A trunk gets freeboard credit only if it is as strong as a superstructure, at least 0.6B wide and 0.6L long (reg. 36). A taller trunk gets no extra credit above the standard height (reg. 33: 2.3 m at 125 m and up). A shield that is not weathertight gets none (regs. 3(13), 35(5)). *Sourced from the vault research note, which read MSC.143(77).* ES-TRIN 4.02(3) caps the height in the inland credit formula at 0.36 m, so a tall river trunk earns almost nothing. *Sourced, same note.*
7. **Tank supports are hull structure.** The cradles and chocks of a Type C tank pass the tank loads into the hull girder and the local structure. They must be strong from the tank to the hull, whatever lies above (IGC Code ch. 4, Type C and supports; load cases in ABS Guide for Liquefied Gas Carriers with Independent Tanks and the ABS Type C tank guidance note). *Inference.* I could not open IGC 4.20 and the search suggests 4.20.3 is about testing, so I cite no paragraph for the loads.
8. **Practice.** Ocean membrane LNG carriers use a trunk deck above the upper deck, with a transition structure at the deckhouse end (Korean study of trunk-deck transition structures; *unverified*, search summary). A trunk deck there is part of the hull girder. For the inland LNG barge, a Polish concept study lists three layouts: tanks on the main deck, partly below deck, and in the hull. It says deck tanks increase overall height and can restrict bridge passage, and partly below deck "ensures a lower height overall" (Kaup et al., Scientific Journals of the Maritime University of Szczecin, 2017). *Sourced.* Built coastal ATBs keep the tanks below deck (vault research note). River barges with a 3 m depth carry tanks mostly above deck (vault research note, estimates).

**How OVD models it.**
- **Structural depth is separate from the trunk or shield.** Depth D is to the main deck, which is the strength deck and the freeboard deck. Hull steel, hull girder and freeboard use D only.
- **The shield is weight and windage only.** No hull-girder credit and no freeboard credit. No buoyancy unless flagged watertight, and then only as the design says in D2.
- **A trunk is out of scope for now.** If it is added later, it is a separate object. It earns hull-girder credit only if it meets the ABS 3-2-3/3 test (0.6L, 0.6B, side and top plating) and freeboard credit only if it meets ICLL reg. 36. Until then, the model gives no credit and says so in the output.
- **Cradles are hull steel.** The tank-support weight is in lightship as hull structure, whatever the height above deck.
- **Class decides the scantlings.** OVD does not size shield plating. It reports the shield as an estimate of weight per m2 and flags "class to confirm".

## Decisions

### D1. The input

One input: **`tank_top_above_deck_m`**, the height of the top of the tank shell above the main deck at its highest point, in metres.

- **0 m** means tanks in hold. The deck is flat, tank domes go through it. This is #993's pattern.
- **Above 0 m** means tanks stand above the deck. Whether a shield is fitted is a second input (D3).
- The input is optional. Unset takes the default by Service Area from the Defaults Profile (below).

| Service Area | Default height above deck | Shield default | Basis |
|---|---|---|---|
| Coastal | 2.0 m | On | Maintainer, 2026-11-07. No built coastal gas barge is known to stand tanks above deck, so there is no calibration case. Mark as a default to revisit. |
| River-only | up to the tank's widest point: OD/2 | Off | Maintainer, 2026-11-07. The deck sits at the tank axis. |
| Ocean | 0 m (hold) | Off | **My proposal, not the maintainer's.** Built ocean ATBs hide the tanks (vault research note). To confirm. |

The Defaults Profile stores each as a rule: `{ "metres": 2.0 }` or `{ "od_fraction": 0.5 }`. A copy of the profile may edit them. A river-only value is "up to" OD/2: the search may pick less if depth, freeboard or sightline need it.

### D1a. Depth from the tank, less the standing height

- Tank-driven depth `D_tank` = double bottom + support height + tank OD + 0.3 m deck allowance (the numbers from #993 D2: OD + double bottom + 0.3 m; each is an input with its source named).
- For a standing height `h` above 0: `D = D_tank - 0.3 - h`. The tank top is `h` above the deck. For `h = 0`: `D = D_tank`, so the tank top sits 0.3 m below the deck underside and the dome goes through.
- Floors: `D >= draught + required freeboard + structure margin`, and the tank bottom stays above the deck's lowest allowed line. `h` is capped at OD (the deck would sit under the tank bottom). The result never falls below what freeboard needs; at 140 m that is not close (about 2.8 m required against 7 m available, vault research note, inference), so the tank sets the depth.
- Depth becomes an output of `fit`, not an input, when a height is set. An explicit depth in the Option still wins and `h` is then checked against it.
- The old ratio (0.55 x beam; D/B band 0.49 to 0.56) stays as a note only.

### D1b. Superseding #993

The following parts of #993 move into this design:
- The 0 m end of the input (hold, flat deck, depth from OD).
- **Dome trunks through the deck.** At 0 m the domes pass through gasketed openings. They are small access openings under ICLL 27(2)(b), not a trunk. Coaming height from ABS 3-2-9/5.1 where needed.
- **Air draft.** The highest point is the max of dome top, shield top and tank top, for every `h`.
- **Sightline.** The obstruction is the depth plus the highest of dome, tank top and shield. At 0 m that is the dome only (#993 D3).
- **Hold and damage.** At 0 m the hold is a closed space split by the bulkhead between tank rows; a side breach floods both. At `h > 0` the tank part below the deck is the same hold; the part above is not buoyant (D4).
- **Weight lines.** Flat deck, hatch covers, dome coamings, cradles. Each unit weight is named as an estimate until sourced.
- **Drawings.** One profile, midship section and plan, with the shield outline when fitted.
- **Stowage key.** #993's `tank_stowage` key is not added. The one input is the height. `hold` is 0, `deck` is the legacy case.

### D2. The weather shield model

| Item | Proposal |
|---|---|
| Weight | kg/m2 of plan area, default from a light steel or aluminium panel; counted in lightship, centroid at shield mid-height |
| Windage area | Projected side and front area above deck, added to the existing wind heel area |
| Sightline obstruction | Shield height above deck, along its length, enters the sightline check as an obstruction |
| Buoyancy | None unless the shield is flagged watertight; no credit in damage or intact curves |
| Load line and freeboard | A non-watertight shield is not a superstructure: no effective-length credit, freeboard from the deck at side |
| Drawing | Dashed outline in profile and plan, label "weather shield, not watertight" |

Options: model it as an item in the existing deck-items list (small change, reuses weight, windage and drawing paths), or as a new object. **Recommend the deck-items path**, with a new `shield` kind carrying the flags above.

### D3. The shield input

- **`weather_shield.enabled`**: default by Service Area (D1 table). Only has effect when `tank_top_above_deck_m > 0`.
- **`weather_shield.height_above_deck_m`**: default `tank_top_above_deck_m` + 0.3 m clearance.
- **`weather_shield.watertight`**: false. **`weather_shield.mass_per_m2`**: default from a light steel or aluminium panel, named as an estimate.
- At 0 m there is no shield; the key is ignored and a note says so.

### D4. Effects and checks

- **Damage:** shield gives no buoyancy, so the side-breach result must not improve. Add a check that a shield-on and a shield-off run give the same intact-buoyancy volume when not watertight. A lower depth must still pass the IGC 2.7 cases; the search (#935) is the gate.
- **Sightline:** shield height raises the obstruction. Add it to the check from #989 with the 1.65 m eye height from #990. Report the eye height needed for the shield and the tank top.
- **Air draft:** report the highest point (dome, shield or tank). Add a check that air draft is the max of these, and a note when the shield or tank, not the dome, sets it.
- **Checks to add:** tank top above deck <= limit; shield weight in lightship; windage in wind heel; freeboard from deck, not shield; D/B inside the reference band (note only).

### D5. Re-pins expected

- Gas defaults: the new depth for the 140 x 29 m coastal LNG reference design and any example that omits depth. The cause of each move: depth falls by the standing height and rises with the larger tank; lightship moves with depth, the shield and the cradles; freeboard, damage and sightline results follow from depth. State each cause in the PR.
- Fixture hashes of gas examples, the GA drawing snapshots, and the depth-rule catalogue (regenerated by `tools/gen_depth_rule.py`).
- A Rule Set version bump only if Rule data changes; this design adds none.

## Scope

In: the single input and its defaults, depth rule, shield object, the checks, drawing, dome trunks and the 0 m hold case. Out: a structural trunk object, tank-top structure and shield strength, non-gas cargoes, hold ventilation and inerting.

## Open Questions

1. Is 0 m the right Ocean default (my proposal)?
2. Does a river-only value of OD/2 need a limit for sightline or stability when the barge is wide (the search will screen it)?
3. Should D/B outside 0.49 to 0.56 be a warning or silent? (Proposed: note only.)
4. Does class accept a free-standing shield over tank tops? I found no rule. Get it from the class society for the first real design.
