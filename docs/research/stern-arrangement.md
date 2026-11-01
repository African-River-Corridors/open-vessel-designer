# Research note: stern arrangement — engine room, wheelhouse and accommodation (OVD #734)

**Research line.** *Searched:* the rule texts (IGC 2014 as amended by MSC.411(97), IGF 2015, SOLAS II-2
and V/22 as adopted by MSC.99(73), MLC 2006 Standard A3.1, ES-TRIN 2025/1, CCNR crew rules RPN 2026,
ABS Steel Barges 2024 Part 5 Ch 3, IACS Rec. 95), weight-estimation methods (Watson, Aasen & Bjørhovde
SAWE 3505), cost studies (DNV for SEA\LNG 2019, SCHOTTEL 2024, ICCT 2024), and published particulars of
small LNG carriers and bunker vessels. *Exists:* clear, numeric rules for gas separation, airlocks,
escape, fire integrity, bridge visibility and crew accommodation; no published rule or regression for
engine-room length per kW; no public price per kW for gensets, azimuths or the gas fuel system.
*Reusing:* OVD's own spec 0007 machinery model (Hekkenberg rates, engine table, layout method, Blind
Distance), the rule-set engine (spec 0004), the outfit layer (ADR 0021) and the GA and 3D pipelines
(specs 0003, 0009). *Building because:* OVD's stern block is laid out from deck containers and a lifting
wheelhouse — an inland pusher arrangement — and nothing in it sizes rooms from crew, rules or the gas
code.

Concept and preliminary design only, never approval (ADR 0011). Status words: **sourced** (primary text
read), **class copy** (a class society's or IACS text), **secondary** (quoted by another source),
**derived** (arithmetic on sourced figures), **estimate**, **not found**.

## 1. What OVD does today (0.3.x, `origin/main` 5a0fb99a)

**In the lightship** (with `stern_block_mass_included`, the 140 × 29 m example, #722): 356 t —
two ducted azimuth units 222.3 t, two 1,200 kWe gas gensets 40.2 t, bow thruster 24.6 t, motors,
converters and switchboards 17.6 t, piping and cabling 7.3 t, small items 1.7 t, wheelhouse, lift
column and two accommodation units 42.3 t. Plus the 4 % piping-and-machinery factor on steel and tanks.

**In the cost:** a propulsion line (installed kW × an example rate whose basis names thrusters, drives
and switchboard: $1.72 M on the example), the piping-and-machinery mass line, and a
`wheelhouse_accom_lump`.

**In the arrangement** (`aft_arrangement.rs`, `machinery.rs`): below deck, aft to forward — aft peak
with steering gear 2.5 m, drive (thruster) room, engine room, genset room (gas gensets N+1 abreast),
gas valve unit room; the drive room and engine room drawn 1.4 × long for unmodelled rooms (fitted on 23
built ships, #365). On deck: 40 ft ISO high-cube units (gas: 2 accommodation + 1 battery; dry bulk: 1 +
2), and a 6.4 × 3.6 × 3.4 m wheelhouse on a lift column (max lift 6 m).

**Missing** (#722): a genset cost line; machinery steel (seats, engine-room structure and casing,
funnel and exhaust); the gas fuel supply system (fuel-gas handling, a gas combustion unit, any
reliquefaction). A pushed or towed barge has no machinery by design.

### Where it is "river pusher" shaped

| Today | Why it is inland/pusher practice | What a coastal self-propelled gas hull needs |
|---|---|---|
| Accommodation = N × 40 ft containers on deck | Container living units are an inland/pusher fit-out; no crew number drives the count | Cabins sized from a crew number and MLC A3.1 (coastal) or ES-TRIN 15 (river) |
| A battery container on a gas-fuelled hull (gas default 1 unit) | Inland battery-swap practice | No battery unless the Power Source has one |
| Lifting wheelhouse on a column, 6.4 × 3.6 m, eye 1.50 m (legacy) | Rhine/pusher practice for bridge clearance | A fixed bridge on a deckhouse unless an air-draft limit forces lift; SOLAS V/22 field of view; IACS Rec. 95 room sizes |
| Opening setback 2.00 m (ADN 9.3.1.17.4 / inland rule book 3.3.7) | Inland tanker rule | IGC 3.2.4.1: 4 % of L, ≥ 3 m, need not exceed 5 m — **5 m at L = 140 m**; OVD's basis page 05 lists 3.2.4.1 as "not modelled" |
| No airlock, escape trunk, steering-gear access, funnel | Small inland stern block | IGC 3.6 airlocks; SOLAS II-2/13.4.2 two escapes, one in an 800 × 800 mm protected enclosure; SOLAS II-1/29.13 steering gear compartment |
| No fire-integrity boundaries | — | SOLAS II-2/9 tables 9.7/9.8: A-60 between Cat A machinery and accommodation; IGC 3.1.2 A-60 or cofferdam between hold spaces and machinery; II-2/9.2.4.2.5 A-60 accommodation front |
| Wheelhouse 8 t, sourced on one ship | — | Deckhouse steel and outfit by volume and deck area |

## 2. Rules for the stern arrangement

### 2.1 Gas cargo separation (IGC 2014, MSC.370(93); 3.2.5 as amended by MSC.411(97)) — sourced

| Clause | Requirement |
|---|---|
| 3.1.1 | Hold spaces segregated from machinery, accommodation, service spaces, control stations; hold spaces **forward of** Category A machinery spaces (other layouts only after a SOLAS II-2/17 review) |
| 3.1.2 | No secondary barrier (Type C): segregation by cofferdam, oil fuel tank, or a gastight all-welded **A-60** bulkhead; A-0 if the adjoining space has no ignition source |
| 3.2.1 | No accommodation, service space or control station within the cargo area |
| 3.2.3 | No door from a non-hazardous to a hazardous area, except an airlock to service spaces forward of the cargo area when accommodation is aft |
| 3.2.4.1 | Entrances, air inlets and openings to accommodation, service, machinery spaces and control stations shall not face the cargo area; on the end bulkhead not facing cargo or the outboard side, at least **4 % of L, ≥ 3 m, need not exceed 5 m** from the end facing the cargo area |
| 3.2.4.2 | Windows facing cargo and on the sides within that distance: fixed. Wheelhouse windows and doors may be in it if the wheelhouse can be made gastight quickly |
| 3.2.5 | Those windows and sidescuttles **A-60**, except the wheelhouse windows (MSC.411(97) removed the A-0 sentence for wheelhouse windows) |
| 3.5.4, 3.5.6 | Deck access to non-hazardous spaces outside hazardous areas unless by airlock; no below-deck access hazardous → non-hazardous |
| 3.6.1, 3.6.7 | Airlock: two self-closing gastight steel doors **1.5–2.5 m apart**, overpressure ventilation, sill ≥ 300 mm |
| 16.1, 16.4.1, 16.4.6 | Only methane boil-off may be burnt in a Cat A machinery space; gas pipes not through accommodation, service or control spaces; double-wall or 30 ach ducting; **one master gas valve per consumer space, located in the cargo area** |

**IGF does not apply** to a ship under IGC (IGF Preamble): a gas carrier burning its own cargo follows IGC
Ch. 16. IGF's spaces (fuel preparation room, tank connection space, gas-safe vs ESD machinery space, airlock
≥ 1.5 m²) are useful design guidance only — or binding if a flag treats a separate fuel tank under IGF.

**Gap:** ADN Part 9 (Type G tank vessels) for EU inland service was not re-read here; OVD already cites
ADN 9.3.1.17.4 for the 2.00 m setback.

### 2.2 Machinery spaces: fire integrity and escape (SOLAS II-2, MSC.99(73) text) — sourced

- Tankers (and, by IGC Ch. 11, gas carriers — **confirm in IGC 11**) use method IC. Table 9.7: Cat A
  machinery to control station, corridor, accommodation, stairway, low-risk service, high-risk service
  **A-60**; to other machinery A-0. Table 9.8 decks: the same pattern.
- II-2/9.2.4.2.5: accommodation fronts facing cargo **A-60**, and the sides for 3 m back.
- II-2/13.4.2.1: **two means of escape** from each Cat A space; one a ladder in a protected enclosure,
  **internal size ≥ 800 × 800 mm**, emergency lit; or one ladder plus a lower-level door to a safe route.
  13.4.2.2: < 1,000 GT may waive one; a steering gear space with the emergency steering position needs a
  second escape unless it has direct deck access. 13.4.2.3: other machinery spaces two routes unless
  occasionally entered or travel ≤ 5 m.
- IACS UI SC 277 (secondary): an enclosure may end in an 800 × 800 mm hatch to open deck.

### 2.3 Steering gear

- SOLAS II-1/29 (secondary, class circular quoting the text): main gear 35°–35°, 35°→30° in ≤ 28 s;
  auxiliary 15°–15° in ≤ 60 s at half speed or 7 kn; power-operated above 120 mm / 230 mm stock.
  29.13: compartment readily accessible and **as far as practicable separated from machinery spaces**.
- ES-TRIN 6.01–6.04 (sourced): design to 15° list; second drive in service within 5 s; two power
  sources for two powered drives. ES-TRIN sets performance through Ch. 5 trials, not a hard-over time.
- An azimuth drive steers by the unit: no rudder or tiller room, but the unit's steering motors sit in
  the drive room. OVD's 2.5 m aft peak "with the steering gear" is an estimate fitted to built stern
  blocks, not a rule.

### 2.4 Wheelhouse and visibility

| Source | Requirement | Status |
|---|---|---|
| SOLAS V/22.1.1 (≥ 45 m) | Blind zone ahead ≤ 2 L or 500 m, 10° either side, all draughts, trims, deck cargo | sourced |
| V/22.1.2–.1.5 | Blind sectors ≤ 10° each, ≤ 20° total; conning arc ≥ 225°; bridge wings 225°; steering position ±60° | sourced |
| V/22.1.8–.1.9 | Window upper edge lets a **1.80 m** eye (≥ 1.60 m allowed) see the horizon; front windows inclined 10–25° | sourced |
| IACS Rec. 95 | Window lower edge ≤ 1.0 m, upper edge ≥ 2.0 m above floor; deckhead ≥ 2.25 m; seated eye 1.40 m; console passages 0.7–1.0 m | class copy |
| ES-TRIN 7.02 | Blind distance ≤ 250 m (unladen, half supplies); field of view ≥ 240° (≥ 140° forward); eye 1.65 m; upper edge for 1.80 m eye 10° above horizontal | sourced |
| ES-TRIN 7.12, 1.01/4.29 | Elevating wheelhouse: stability, access in any position, emergency lowering; fixed height with all mobile installations lowered | sourced |
| RheinSchPV §1.07 | Laden blind distance ≤ 350 m (500 m with radar and video) | sourced (in 0007) |

OVD has V/22.1.1 and ES-TRIN 7.02(2) Blind Distance (#354, #355). Not modelled: the arcs, bridge
wings, window geometry, room size.

### 2.5 Accommodation and crew

| Source | Requirement | Status |
|---|---|---|
| MLC 2006 Art. II.1(i) | Applies to seagoing ships; excludes ships **exclusively** in inland or sheltered waters. A coastal-and-river hull is in scope | sourced |
| MLC A3.1.6 | Headroom ≥ 2.03 m; sleeping rooms above the load line, amidships or aft; **no direct openings** into sleeping rooms from cargo or machinery spaces; those bulkheads steel and gastight | sourced |
| MLC A3.1.9 | One cabin per seafarer (exemption < 3,000 GT); berth ≥ 1.98 × 0.80 m; single-berth floor **4.5 m² (< 3,000 GT), 5.5 m² (3,000–10,000 GT), 7 m² (≥ 10,000 GT)**; officers without day room 7.5 / 8.5 / 10 m²; day room for master, chief engineer, chief officer | sourced |
| MLC A3.1.10–.12 | Mess rooms near the galley (no area figure); sanitary 1 toilet, basin and shower per 6; hospital at ≥ 15 seafarers on voyages > 3 days (coastal relaxation) | sourced |
| ES-TRIN 15.02–15.03 | Headroom ≥ 2.00 m; no per-person cabin area; cabin ≥ 7 m³, 5 m³ first + 3 m³ each further occupant, ≤ 2 per cabin; communal ≥ 2 m²/person and ≥ 8 m²; 1 toilet per 6, basin per 4, shower per 6 | sourced |
| CCNR RPN 2026 Art. 19.02 | Motor vessel L > 86 m (top band): about 3 crew (mode A1, 14 h) to 5 (mode B, 24 h) | sourced (table alignment to check) |
| IMO A.1047(27) | Safe manning set by the flag from principles; no numbers | sourced |
| Reference | One 7,500 m³ LNG bunker vessel with crew 14 | secondary — verify |

**Gross tonnage** sets the MLC bands; OVD computes no GT today. GT = K1·V (ITC 1969 Annex I) is a
closed form on the enclosed volume — **derived** method; it needs the deckhouse volume this spec adds.

### 2.6 Stern types per OVD vessel type

| OVD Formation | Stern arrangement | Rule basis | Owner |
|---|---|---|---|
| `self_propelled` (gas, liquid, dry, container) | Integrated stern: machinery below deck aft, deckhouse with accommodation and wheelhouse over it | IGC 3 (gas), SOLAS II-1/II-2, MLC (coastal) / ES-TRIN 15 (river) | **spec 0011** |
| `pushed_barge`, `pushed_convoy` — the pusher | A tug hull: its own stern arrangement | ES-TRIN; RPN 19.03 convoy crews | 0011 model, called by 0007 Formation |
| `atb` — the tug | A tug hull, wheelhouse sightline over the laden barge | ABS Barges 5-3-1 (two vessels) | 0011 model; barge geometry from **spec 0014** |
| `atb` — the barge | Notch, no machinery | ABS Barges 5-3-1/11.3 (no notch dimensions published) | **spec 0014** |
| `towed_barge` | No machinery; possibly a deck generator | — | none |

## 3. Machinery weight, machinery steel and cost data

| Item | Value | Status |
|---|---|---|
| Watson remainder | Wrem = Cm · MCR^0.70 t; Cm 0.69 cargo, 0.72 tankers, 0.83 ferries | secondary (Aalto, USP lecture notes citing Watson 1998) |
| Watson diesel-electric total | Wm = 0.72 · MCR^0.78 t (MCR = all genset kW); 2,400 kW → ≈ 312 t | secondary; derived |
| Steel in way of engine room, superstructure, wheelhouse | W = k · V per block; k by yard regression, unpublished | sourced (form only), SAWE 3505 (2010) |
| Accommodation outfit | W = k · A (accommodation deck area); 4.4 % of lightship on one AHT | sourced (form only) |
| Seats, casing, funnel as separate items | — | not found (inside Watson's remainder) |
| Deckhouse steel per m³ | — | not found |
| LNG engine capital cost | ≈ 400–1,500 USD/kW (chart read) | sourced, DNV for SEA\LNG 2019 Fig. 6-5 |
| Fuel-gas process system cost | "Not given in available literature" | sourced (DNV names the gap) |
| Diesel-electric + 2 azimuths vs diesel-direct CPP | 126 % of capital cost (2,400 kW case) | sourced, SCHOTTEL 2024 |
| Genset or azimuth price per kW | — | not found |
| GVU, GCU, reliquefaction mass or cost (5–20k m³) | — | not found |
| Engine-room length vs kW | — | not found (0007 open gap, #365) |

**Reading.** OVD's 356 t on 2 × 1,200 kWe sits **above** Watson's 312 t for a whole diesel-electric
plant, because the ducted azimuth rate (1.5 D³) carries 222 t. So the machinery *items* are not
light; what is missing is steel and the gas system, not engines. (Inference: Watson's total is a
cross-check, not a replacement, and adding it on top would double-count.)

## 4. Reference vessels (self-propelled gas, size class)

| Vessel | Size | Machinery | Arrangement | Status |
|---|---|---|---|---|
| Gas Agility (2020) | 135.5 × 24 m, 18,600 m³ membrane | diesel-electric + DF genset, 2 azimuths, full reliquefaction | aft deckhouse | sourced (trade press) |
| Coralius (2017) | 99.6 × 17.95 m, 5,800 m³ Type C | 1 × 6L34DF 3,000 kW mechanical, CPP; aux 2 × 375 kW LNG + 475 kW diesel | aft deckhouse | secondary |
| Kairos (2018) | 7,500 m³, 2 × Type C | 4-stroke DF; fuel integrated with cargo | aft deckhouse; crew 14 | secondary |
| Avenir Accolade | 123.4 × 18.6 m, 7,500 m³ Type C | DF | aft deckhouse | secondary |
| Q-LNG 4000 + tug (2021) | barge 98.7 × 19.5 m, 4,000 m³ | ATB; tug 2 × 1,900 kW, Z-drives; barge 5 × 450 kW gensets for cargo | barge: no accommodation for propulsion crew | sourced (trade press) |

All published self-propelled examples put a **fixed aft deckhouse** over the engine room; none uses
container accommodation or an elevating wheelhouse. (Inference from five vessels' published GAs and
photos; a measured calibration set is a build ticket.)

## 5. Named gaps

1. ADN Part 9 Type G for EU inland service (not re-read).
2. IGC Ch. 11: which SOLAS II-2 tables apply to a gas carrier.
3. SOLAS II-1/29.13 primary text; MSC/Circ.982 window heights; ISO 8468 (paywalled).
4. Engine-room length per kW; deckhouse steel per m³; outfit per m² — calibrate from scaled GAs.
5. Prices per kW for gensets, azimuths and the fuel-gas system — example rates only, marked so.
6. Later amendments to SOLAS II-2/9 and /13, IGF and MLC not checked.

## Sources

- IGC Code, MSC.370(93): https://wwwcdn.imo.org/localresources/en/KnowledgeCentre/IndexofIMOResolutions/MSCResolutions/MSC.370(93).pdf ; MSC.411(97) via https://www.mardep.gov.hk/en/msnote/pdf/msin1733anx1.pdf
- IGF Code, MSC.391(95): https://wwwcdn.imo.org/localresources/en/KnowledgeCentre/IndexofIMOResolutions/MSCResolutions/MSC.391(95).pdf
- SOLAS II-2 and V, MSC.99(73): https://wwwcdn.imo.org/localresources/en/KnowledgeCentre/IndexofIMOResolutions/MSCResolutions/MSC.99(73).pdf
- SOLAS II-1/29 quoted in IRClass Tech. Circ. 022/2014: https://www.irclass.org/media/1585/early-implementation-of-the-amendments-to-solas-regulation-ii-i29.pdf
- IACS Rec. 95 Rev.1 (2022): https://iacs.s3.af-south-1.amazonaws.com/wp-content/uploads/2022/05/20154015/rec95rev1.pdf
- MLC 2006: https://www.ilo.org/sites/default/files/wcmsp5/groups/public/@ed_norm/@normes/documents/normativeinstrument/wcms_090250.pdf
- ES-TRIN 2025/1: https://cesni.eu/wp-content/uploads/2024/11/ES_TRIN_2025_signed_en.pdf
- CCNR RPN 01.01.2026: https://www.ccr-zkr.org/files/documents/reglementSTF/RPN_01012026_en.pdf
- ABS Steel Barges 2024: https://cloudqa.eagle.org/content/dam/eagle/rules-and-guides/current/special_service/10-rules-for-building-and-classing-steel-barges-2024/10-barge-rules-jan24.pdf
- DNV / ABS on UI SC 277: https://www.dnv.com/news/2025/emergency-escape-from-machinery-spaces-of-category-a/
- Aasen & Bjørhovde, SAWE 3505 (2010): https://shipweight.com/wp-content/uploads/2024/04/SAWE%202010%20-%20Paper%203505%20Early%20Stage%20Weight%20and%20CoG%20Estimation%20using%20Parametric%20Formulas%20and%20Regression%20on%20Historical%20Data.pdf
- DNV for SEA\LNG (2019): https://sea-lng.org/wp-content/uploads/2020/04/Alternative-Marine-Fuels-Study_final_report_25.09.19.pdf
- SCHOTTEL (2024): https://63congreso.ingenierosnavales.com/wp-content/uploads/2024/05/29.This-presentation-compares-a-vessel-design-with-a-conventional_compressed.pdf
- ICCT for MARAD (2024): https://theicct.org/wp-content/uploads/2024/03/ID-98-%E2%80%93-MARAD-report_final2.pdf
- Gas Agility: https://www.bairdmaritime.com/shipping/tankers/gas/vessel-review-gas-agility-worlds-largest-lng-bunkering-vessel-boasts-innovative-cargo-handling-and-transfer-systems
- Q-LNG 4000: https://www.bairdmaritime.com/work-boat-world/tug-and-salvage-world/inland-tug-operations/vessel-review-q-ocean-service-q-lng-4000-first-us-flagged-offshore-lng-bunkering-atb-to-serve-southeastern-coastal-clients

## Round 3: setback and double counting (2026-10-31)

Asked by the maintainer in round 2 (D17 note): check whether voids and cofferdams are counted twice. Read on
OVD `origin/main` (0bab67f9 era, fetched 2026-10-31).

**What OVD does today.**
- `gas_separation.rs:118–124`: the aft separation = end space (`cofferdam_m`) + the 2.00 m setback
  (issue #381, measured from the cargo area). The service space = aft separation − end space = 2.00 m.
- `gas_separation.rs:24–35`: the "cofferdam" default is the hold end space of the inland type G rule
  (0.20 m for domed Type C heads, 0.50 m for plane ends; ADN 9.3.1.11.3(a)). It is a tank-end
  clearance, not an IGC 3.1.2 cofferdam.
- `vessel_model.rs:280–293`: both become full-beam, full-depth Spaces aft of the cargo zone: `CD1C`
  (kind Cofferdam) and `SV1C` (kind Void, the 2.00 m service space). The machinery spaces start aft
  of them. So the setback is modelled as **empty hull length**.

**What IGC 3.2.4.1 asks** (row 3.2.4.1 above): entrances, air inlets and openings of accommodation,
service and machinery spaces and control stations must not face the cargo area; they sit on the end
bulkhead not facing cargo or on the outboard side, at least 4 % of L (≥ 3 m, need not exceed 5 m)
**from the end of the superstructure or deckhouse facing the cargo area**. It places openings on
the deckhouse. It does not ask for an empty length of hull. (Inference from the paragraph text;
confirm against the IGC Code text before the Rule is coded.)

**Double counts if the IGC value were added the way OVD adds the ADN 2.00 m:**
1. The 3–5 m would become a void (`SV1C`) in front of the deckhouse. IGC measures it inside the
   deckhouse, so the length would be lost twice: once as void, once as the deckhouse's closed front.
   This is the "about 3 m of cargo length lost" in round 2: an overcount.
2. With D18 (gastight A-60 bulkhead between hold and machinery), the hold end space `CD1C` stays a
   tank-end clearance inside the hold. If the cofferdam option is picked, it **replaces** the A-60
   bulkhead; it is not added to it. Calling the end space a "Cofferdam" invites adding a second one.
3. The spec 0012 void strips (`WV…`, PR #770) lie outboard of the holds, above the wing tanks. They
   do not overlap the aft separation. No double count there.

**Recommendation for the IGC Rule Set (run in the first GA, D17).** On an IGC hull: no service-space
void; the machinery and deckhouse start at the hold's aft A-60 bulkhead (plus the tank-end clearance
inside the hold); the Rule `igc/3.2.4.1/opening-setback` places the doors, vents and air inlets at
least max(3, min(5, 0.04 L)) m aft of the deckhouse front, and checks it; the end space is named as a
hold end clearance, not a cofferdam; a cofferdam appears only when D18's option is chosen, in place of
the A-60 bulkhead. River-only and old files keep the ADN 2.00 m service space as today. Expected
effect (inference): a new IGC Option gains about 2 m of cargo length against today, instead of
losing about 3 m.

## Round 3: D19 estimate basis (2026-10-31)

No deckhouse or engine-room steel rate per m³ was found in a source read so far. Watson's residual
cannot stand in: OVD's machinery items (356 t) already exceed Watson's whole diesel-electric total
(312 t) for 2 × 1,200 kWe (§ above). Options for the unverified estimate: a rate from a textbook
weight method (Watson; Schneekluth & Bertram), cited by the build lane before it ships; or one
reference ship's GA (Kairos, 7,500 m³) measured now. Either is marked "unverified estimate" in the
rule trace until the calibration ticket replaces it. Not yet researched on the web.
