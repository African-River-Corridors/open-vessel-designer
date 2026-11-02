# Spec 0011: Stern arrangement — engine room, wheelhouse and accommodation

Status: **agreed with the maintainer, 2026-10-31** (three rounds; D1–D19; see §10). Issue: #734. Builds on
spec 0007 (#346, ADR 0017) and amends the rows of 0007 named in §7.
Research: `docs/research/stern-arrangement.md` and `docs/research/manning.md`.

Concept and preliminary design only, never approval (ADR 0011).

**Split with spec 0014 (ATB), agreed between lanes 2026-10-30:** 0014 owns the ATB barge notch (depth,
width, wing walls, coupler sockets, notch structure notes, the tug-in-notch fit, and the gas separation
between tug and barge), and amends `atb.rs`. 0011 owns the arrangement model of every self-propelled
hull's stern (engine room, steering gear, wheelhouse, accommodation, machinery mass, steel and cost). The
ATB tug is a self-propelled hull: 0014 calls 0011's model and does not define it. The tug's wheelhouse
height uses 0011's sightline model; 0014 supplies the barge obstructions and the coupled geometry.

## 1. Problem

OVD's stern block comes from a river pusher idea: 40 ft containers for accommodation and batteries, a
lifting wheelhouse on a column, and a machinery run whose length 0007 fits to built inland stern blocks.
A self-propelled coastal gas hull needs a stern laid out for its own duty: a crew housed to MLC, a fixed
bridge that meets SOLAS V/22, the IGC Ch. 3 separation from the cargo area, escape from the machinery
space, and the machinery steel and gas fuel system that the lightship and cost leave out today. The stern
drives lightship, centre of gravity, cost and the sightline Rule, so a weak stern weakens every result.

## 2. Solution

An **Arrangement Style** input picks how the stern is laid out:

- `container_block` — today's stern block, exactly (the legacy value, and the river-pusher option);
- `deckhouse` — a stern sized from rules: the machinery space from 0007, a steering gear space, and a
  deckhouse of tiers above the main deck holding cabins, mess, galley, stores and an airlock, with a
  fixed wheelhouse on top.

The `deckhouse` style derives, from a **Crew** number and the Service Area:

- the cabin, mess, sanitary and corridor areas (MLC A3.1 when the Service Area reaches sea; ES-TRIN 15
  when it is river-only), and so the deckhouse footprint and tier count;
- the wheelhouse floor height from the sightline Rules already in OVD (0007 Blind Distance), the
  wheelhouse size from IACS Rec. 95, and the air draft;
- the gas-code separation: openings set back 4 % of L, ≥ 3 m, ≤ 5 m (IGC 3.2.4.1); A-60 front;
  airlock (IGC 3.6);
- the escape routes from the machinery space (SOLAS II-2/13.4.2) as an arrangement item with a check;
- the mass and centres of the deckhouse steel, outfit and machinery steel;
- cost lines for gensets, machinery steel, the deckhouse and the gas fuel system.

## 3. Design tree (the decisions)

| # | Topic | Decision | Round |
|---|---|---|---|
| D1 | Extend 0007 or a new spec | **New spec 0011, listing the 0007 rows it amends** (§7) | R1 |
| D2 | Vessel types first | **All self-propelled hulls, gas first; pusher and ATB tugs in a later wave** | R1 |
| D3 | Arrangement model | **Both, by an Arrangement Style input:** `container_block` kept for river and old files; a rules-sized `deckhouse` the default for a new coastal Option | R1 |
| D4 | Wheelhouse | **Fixed on the deckhouse, unless an air-draft limit is set; then elevating** | R1 |
| D5 | Crew and accommodation rule | **Crew is an input; MLC A3.1 sizes cabins when the hull goes to sea, ES-TRIN 15 when river-only** | R1 |
| D5a | Default crew | **Manning is a Rule Set, like the class rules: one per regime; the crew model (§4.1) runs on the chosen set** | R2 |
| D6 | Accommodation location | **Aft only now; forward accommodation later** | R1 |
| D7 | Machinery steel method | **Deckhouse and engine-room blocks by volume × rate, plus seat and funnel allowances; Watson's machinery total reported as a cross-check only** | R1 |
| D8 | Gas fuel system scope | **Mass and cost allowances for gas valve units, heater and compressor; a gas combustion unit as an input; reliquefaction later** | R1 |
| D9 | Cost lines | **Item rates: gensets per kWe, propulsion per kW, deckhouse per m², gas system per kW, machinery steel per t** (example rates, marked so) | R1 |
| D10 | Outputs | **GA, rooms table, rule trace and 3D together** | R1 |
| D11 | Calibration | **Measure 6–10 small gas carriers and tugs from scaled published GAs (stern length, deckhouse volume, crew), held-out cases** | R1 |
| D12 | Phasing | **Wave 1 mass and cost lines; wave 2 the deckhouse model, rules, GA, rooms table, rule trace and 3D together; wave 3 tugs** (§8) | R2 |
| D13 | Crew model | **Manning Rule Sets built up like class rules (maintainer's note, round 2). `liberia-man-001` is the default for a coastal Option; inland `ccnr-rpn-19`; inland pushers RPN 19.03; seagoing ATB tugs Liberia by the tug's own GT and kW; a typed crew overrides.** Research ticket: compare the candidate manning sets (maintainer's note, round 3) and read US 46 CFR as a cross-check before wave 3 | R2–R3 |
| D14 | Power for the engineer band on a diesel- or gas-electric hull | **Total genset kW (the higher band; cautious)** | R2 |
| D15 | Gross tonnage | **Computed (ITC 1969, hull + deckhouse volume), typed override** | R2 |
| D16 | Spare berths above the crew | **2 (pilot and one supernumerary; estimate)** | R2 |
| D17 | Opening setback on an IGC hull | **Part of an IGC vessel-design Rule Set, run as part of the first GA. No setback void: deckhouse and engine room start at the hold's A-60 bulkhead; doors and vents at least 4 % L (3–5 m) aft of the deckhouse front, checked. The ADN 2.00 m void rule is dropped until an ADN Rule Set is built (maintainer's note, round 3); old files may be deleted and the calibration tests updated for the rule change** | R3 |
| D18 | Hold-to-machinery separation (IGC 3.1.2, Type C) | **Gastight A-60 bulkhead by default; cofferdam as an option, replacing the bulkhead, never added to it. The 0.20 m tank-end space is renamed a hold end clearance** | R2–R3 |
| D19 | Deckhouse and engine-room steel rate before calibration | **Ship an unverified estimate now: the build lane takes a textbook rate (Watson; Schneekluth & Bertram), cited, marked unverified until calibration** | R2–R3 |

## 4. Inputs

### 4.1 Crew model (decided R2: runs on a manning Rule Set)

From the manning research (all steps inference, each resting on a cited rule):

1. **Regime** from the Service Area picks the manning Rule Set: seagoing (coastal-and-river) →
   `liberia-man-001` by default (+ STCW + MLC); river-only → `ccnr-rpn-19`; inland pushers → RPN
   19.03; seagoing ATB tugs → `liberia-man-001` by the tug's own GT and kW.
   Inputs: genset kW total sets the engineer band on electric drives (D14).
2. **Inputs:** GT (computed, ITC 1969), propulsion kW, daily navigation hours, UMS yes/no, cargo.
3. **Bridge watches** from hours (STCW A-VIII/1 rest): > 12 h → three watchkeepers; master watch-free
   above 3,000 GT (Liberia MAN-001 §3.3).
4. **Deck officers** = master + watchkeepers, floored by the flag table.
5. **Engineers** by kW band and UMS (UK MSN 1868 Annex B; Liberia MAN-001).
6. **Ratings** by GT band (Liberia MAN-001); a hole in the band grid takes the higher neighbour.
7. **Cargo role** (gas): +1 cargo engineer as an owner option, not a rule.
8. **Cook:** seagoing and ≥ 10 → +1 qualified cook (MLC A3.2(5) as Liberia and Singapore apply it).
9. **Owner margin** (steward, ETO, cadets) reported apart from the minimum.
10. **Cross-check:** UK MSN 1868 is shown beside Liberia; a difference over one person is flagged;
    reference ships (one published crew: 14–15 on a 7,500 m³ gas carrier, secondary).
11. **Berths** = minimum + owner margin + 2 spare berths (D16). River mode B with rest underway: one cabin each
    (RPN 18.02(4)).

Worked: 140 × 29 m coastal LNG hull, 2.4 MW, 24 h, UMS → minimum **13** (4 deck officers, 2 engineers,
6 ratings, cook); planning 14–16. The same hull, 14 h daytime → 11–13 (needs flag agreement). An 85 m
inland Type G tanker, mode B → **4** (RPN 19.02).

### 4.2 Input table

| Input | Unit / choices | Legacy value | New Option default | Basis |
|---|---|---|---|---|
| `arrangement_style` | `container_block`, `deckhouse` | `container_block` | per D3 | decision |
| `crew` | persons, or none | — | none: the crew model (§4.1) | method; typed value overrides |
| `gross_tonnage` | GT, or none | — | none: computed, ITC 1969 (D15) | method; typed value overrides |
| `navigation_hours_per_day` | h | — | 24 coastal (R2); 14 river | estimate |
| `unmanned_machinery_space` | bool | — | true (R2: 24 h, UMS) | estimate |
| `manning_rule_set` | `liberia-man-001`, `uk-msn-1868`, `ccnr-rpn-19` | — | `liberia-man-001` coastal; `ccnr-rpn-19` river-only | Rule Sets (D13) |
| `wheelhouse_type` (0007) | `elevating`, `fixed` | `elevating` | per D4 | — |
| `air_draft_limit_m` | m, or none | none | none | route input |
| `gas_combustion_unit` | bool | — | per D8 | — |

## 5. Rules (spec 0004)

New or extended Rule Sets — each a draft until a second reader verifies it:

- `igc` — a vessel-design Rule Set run as part of the first GA (D17): `igc/3.2.4.1/opening-setback`
  (4 % L, ≥ 3 m, ≤ 5 m, measured on the deckhouse from its front; no hull void; the ADN 2.00 m void
  rule is dropped until an ADN Rule Set exists, D17), `igc/3.6.1/airlock`, `igc/3.1.2/hold-machinery-segregation` (evaluated; gastight A-60 bulkhead default, cofferdam an option, D18).
- Manning (D13) — `liberia-man-001` (coastal default; seagoing ATB tugs), `uk-msn-1868` (cross-check), `ccnr-rpn-19` (inland default; 19.03 pushers).
- `solas-ii-2` — `13.4.2.1/two-escapes-cat-a`, `9.2.4.2.5/a60-front`.
- `mlc-2006` (new family) — `A3.1.9(f)/cabin-area`, `A3.1.6(a)/headroom`, `A3.1.6(c)/sleeping-room-location`,
  `A3.1.11/sanitary`; applies when the Service Area reaches sea.
- `es-trin-2025` — `15.02/accommodation`, `15.03/sanitary` for river-only.
- `solas-v22` — keep 22.1.1; add `22.1.3/conning-arc` as a gap until window geometry exists.

## 6. Mass, centres and cost

Per D7–D9 and D19. Today's wheelhouse 8 t and lift column 5 t (one ship) and the accommodation
0.17 t/m³ stay for `container_block`. Deckhouse and engine-room steel use a cited textbook rate, marked
an unverified estimate in the rule trace until the calibration set (D11) replaces it.

## 7. Amendments to spec 0007

- "Machinery space and mass": the deck items row ("Deck units and the wheelhouse as today") becomes
  per Arrangement Style.
- Mass-item table: wheelhouse, lift column and accommodation rows apply to `container_block` only.
- New items: deckhouse steel, outfit, machinery steel, gas fuel system.
- Cost: the propulsion rate's basis is split; gensets get their own line.

## 8. Phasing and build tickets

Round 2 reconciled the three waves with 3D beside the GA (D10, D12):

- **Wave 1 — missing mass and cost:** genset cost line; machinery steel allowances (unverified textbook
  rate, D19); gas fuel system allowances; item cost rates; the IGC Rule Set with the 3.2.4.1 setback in
  the first GA, the setback void and ADN 2.00 m rule removed, the end space renamed, old files deleted
  and calibration tests updated (D17, D18).
- **Research ticket (before wave 3):** compare the candidate manning Rule Sets and read US 46 CFR.
- **Wave 2 — the deckhouse:** Arrangement Style, crew model and GT, rooms sized from MLC / ES-TRIN,
  fixed wheelhouse from the sightline, IGC/SOLAS/MLC rules, the calibration set, and the GA, rooms
  table, rule trace and 3D together.
- **Wave 3 — tugs:** the same model for pusher and ATB tugs (spec 0014 calls it).

Tickets: sub-issues of #734, in wave order.

## 9. Out of scope

Class approval items; detailed structural design of the deckhouse; ventilation and HVAC sizing; noise;
the ATB barge notch (spec 0014); damage stability of stern spaces (spec 0012); ocean-only service.

## 10. Decision log

- Round 1 (2026-10-31): every recommendation taken except D10 (3D with the GA in the first build) and
  D5a (default crew: the maintainer asked for a synthesis of the manning rules; `docs/research/manning.md`).
- Round 2 (2026-10-31): every recommendation taken except D13 (manning as Rule Sets, built like class
  rules; Liberia the coastal default), D17 (the setback in an IGC Rule Set run in the first GA; check
  that voids and cofferdams are not counted twice) and D19 (ship an unverified estimate now).
- Round 3 (2026-10-31): the double-count check found the setback modelled as empty hull length
  (`docs/research/stern-arrangement.md`, "Round 3"). Every recommendation taken except the setback: no
  void, and the ADN rule dropped until its Rule Set is built. The maintainer asked for research
  comparing the manning Rule Sets (a ticket).
- Confirmed 2026-10-31.
