//! The accommodation rooms of a deckhouse (spec 0011 D5, section 2; issue #780), sized from the
//! berths the crew model gives.
//!
//! Source read: ES-TRIN 2025/1, Chapter 15, Articles 15.02, 15.03, 15.04 and 15.07 (the PDF text in
//! the rule sources), and, for a Service Area that reaches sea (issue #892), the Maritime Labour
//! Convention, 2006, as amended (ILO consolidated text including the 2022 amendments), Standard
//! A3.1 paragraphs 6(a), 9(a), (d), (e), (f), (k), (m), 11(c), (d) and 12, and Guideline B3.1.8(4).
//! Each room of a sea-reaching hull is sized to the greater of the two, and its row names the
//! governing source. **No exemption is applied by default**: not A3.1.9(a), (h) (ships under 3,000
//! GT), A3.1.9(m) (day rooms, under 3,000 GT), A3.1.12 (relaxation for coastal trade) nor A3.1.20
//! (ships under 200 GT). They are the competent authority's to grant, after consultation, so a
//! concept sizes without them. One cabin per crew member is OVD's own rule; RPN 18.02(4) (a
//! derogation for modes A1 and A2) is mentioned as context only (CCNR RPN, state 1 January 2026,
//! English unofficial translation).
//!
//! Each row says how its area came: `sourced` (a figure in the paragraph), `derived` (worked from
//! figures in the paragraph, the working named in the note) or `estimate` (the paragraph
//! requires the room or fixture but gives no area: a stated ESTIMATE, its working in the note, like the
//! tier height). Concept and preliminary
//! design only (ADR 0011).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Headroom in accommodation, at least (m): ES-TRIN 15.02(6).
pub const HEADROOM_M: f64 = 2.00;
/// Cubic capacity of a sleeping cabin, at least (m3): ES-TRIN 15.02(9).
pub const CABIN_VOLUME_M3: f64 = 7.0;
/// Airspace of a sleeping cabin for the first occupant, at least (m3): ES-TRIN 15.02(10).
pub const CABIN_AIR_FIRST_M3: f64 = 5.0;
/// Internal berth length and width, at least (m): ES-TRIN 15.07(1).
pub const BERTH_LENGTH_M: f64 = 2.00;
pub const BERTH_WIDTH_M: f64 = 0.90;
/// Free floor of the communal living quarters per person, and in total, at least (m2): ES-TRIN
/// 15.02(8).
pub const COMMUNAL_PER_PERSON_M2: f64 = 2.0;
pub const COMMUNAL_MIN_M2: f64 = 8.0;
/// Crew per toilet, per washbasin, per shower: ES-TRIN 15.03(1)(a), (b), (c).
pub const PERSONS_PER_TOILET: u32 = 6;
pub const PERSONS_PER_WASHBASIN: u32 = 4;
pub const PERSONS_PER_SHOWER: u32 = 6;
/// Floor space of a toilet compartment, at least (m2): ES-TRIN 15.03(3).
pub const TOILET_M2: f64 = 1.0;
/// ESTIMATES for what ES-TRIN requires but gives no area for. Each is worked from a plain footprint,
/// not read from a source, and is marked `estimate` in the rooms table.
/// Washbasin: 0.60 x 0.80 m of basin and standing room (m2).
pub const WASHBASIN_M2: f64 = 0.5;
/// Shower: a 0.90 x 0.90 m cubicle (m2).
pub const SHOWER_M2: f64 = 0.81;
/// Galley: a 2.4 m run of cooker, sink, refrigerator and worktop 0.60 m deep with a 1.0 m working
/// aisle (3.84 m2), and 1.2 m2 of stores, rounded (m2).
pub const GALLEY_M2: f64 = 5.0;
/// Work-clothes room: 2.0 m2 for hanging and drying, and 0.25 m2 a berth for a lockable locker per
/// crew member (15.07(1)) (m2).
pub const WORK_CLOTHES_BASE_M2: f64 = 2.0;
pub const WORK_CLOTHES_PER_BERTH_M2: f64 = 0.25;
/// One stairway: 0.60 m wide (15.02(12)(a)), 13 risers of 0.20 m for a 2.6 m tier at a 0.15 m
/// tread (15.02(12)(b)) is a 1.95 m run, and a 0.60 m landing: 0.60 x 2.55 = 1.53, rounded (m2).
pub const STAIRWAY_M2: f64 = 1.6;
/// Clear width of a door, at least (m): ES-TRIN 15.02(11)(a). Taken as the least corridor width: a
/// corridor narrower than its doors would not pass a person through them.
pub const CORRIDOR_WIDTH_M: f64 = 0.60;

const ES: &str = "ES-TRIN 2025/1 ";
const MLC: &str = "MLC 2006 as amended (2022 consolidated text) Standard A3.1 ";

/// Headroom where full and free movement is needed, at least (m), for a ship that reaches sea:
/// MLC 2006 A3.1.6(a) (203 cm).
pub const MLC_HEADROOM_M: f64 = 2.03;
/// Single-berth sleeping room floor of a ship that is not a passenger ship or special purpose ship,
/// at least (m2), for seafarers who are not officers: A3.1.9(f)(i), (ii), (iii), by gross tonnage
/// band (under 3,000; 3,000 to under 10,000; 10,000 and over).
pub const MLC_SLEEPING_M2: [f64; 3] = [4.5, 5.5, 7.0];
/// Sleeping room floor per officer where no private sitting room or day room is provided, at least
/// (m2): A3.1.9(k)(i), (ii), (iii), by the same bands.
pub const MLC_OFFICER_SLEEPING_M2: [f64; 3] = [7.5, 8.5, 10.0];
/// The gross tonnage bands of A3.1.9(f) and (k): 3,000 and 10,000.
pub const MLC_GT_EDGES: [u32; 2] = [3_000, 10_000];
/// Officers who need a sitting room or day room next to the sleeping room, A3.1.9(m): the master,
/// the chief engineer and the chief navigating officer.
pub const MLC_DAY_ROOM_OFFICERS: u32 = 3;
/// A day room: ESTIMATE, A3.1.9(m) gives no area. A 2.5 x 2.4 m room for a settee, a table and a
/// chair (m2).
pub const DAY_ROOM_M2: f64 = 6.0;
/// Seafarers from which a separate hospital is required, A3.1.12.
pub const MLC_HOSPITAL_SEAFARERS: u32 = 15;
/// Hospital room: ESTIMATE, A3.1.12 gives no area. One berth (2.0 x 0.9 m) with 0.9 m at its head
/// and 0.75 m of access each side: 2.9 x 2.4 m = 6.96, rounded (m2).
pub const HOSPITAL_M2: f64 = 7.0;

/// The MLC band of a gross tonnage (0: under 3,000; 1: under 10,000; 2: 10,000 and over). A gross
/// tonnage of nothing (not known) takes the largest band, the cautious one.
pub fn gt_band(gt: u32) -> usize {
    if gt == 0 || gt >= MLC_GT_EDGES[1] {
        2
    } else if gt >= MLC_GT_EDGES[0] {
        1
    } else {
        0
    }
}

/// What the rooms are sized from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomsBasis {
    /// Berths: the crew, the owner margin and the spare berths (crew model, D16).
    pub berths: u32,
    /// The Service Area reaches sea: MLC A3.1 applies (issue #892).
    pub reaches_sea: bool,
    /// Gross tonnage, whole units rounded down (the MLC bands have whole-number edges); 0 when it
    /// is not known (the largest band is taken). Read only where the Service Area reaches sea.
    pub gt: u32,
    /// Berths held by officers (deck officers and engineers of the crew). The rest are ratings, the
    /// cook, the owner margin and the spare berths, all sized as ratings.
    pub officers: u32,
    /// Seafarers carried (the crew, without the spare berths), for the hospital threshold.
    pub seafarers: u32,
    /// Inland operating mode B (RPN 18.01). It does not change the rooms: every berth has its own
    /// cabin in every mode. Kept so the results show the mode the crew was sized for.
    pub mode_b: bool,
}

/// One row of the rooms table.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Room {
    pub id: &'static str,
    pub name: &'static str,
    pub count: u32,
    pub area_each_m2: f64,
    pub area_m2: f64,
    /// The clause: document, edition and paragraph.
    pub clause: String,
    /// `sourced`, `derived` or `estimate`.
    pub basis: &'static str,
    /// The source that sets the area or the count: `es-trin`, `mlc` or `both` (the two agree).
    pub governing: &'static str,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Rooms {
    pub berths: u32,
    pub rows: Vec<Room>,
    /// The sum of the row areas (m2).
    pub total_m2: f64,
    pub reaches_sea: bool,
    /// MLC A3.1 was applied: the Service Area reaches sea.
    pub mlc_applied: bool,
    /// The headroom the rooms are sized for (m): ES-TRIN 2.00, MLC 2.03 at sea.
    pub headroom_m: f64,
    pub rule_note: String,
}

fn ceil_div(n: u32, d: u32) -> u32 {
    n.div_ceil(d)
}

fn row(
    id: &'static str,
    name: &'static str,
    count: u32,
    each: f64,
    clause: &str,
    basis: &'static str,
    note: &str,
) -> Room {
    Room {
        id,
        name,
        count,
        area_each_m2: each,
        area_m2: count as f64 * each,
        clause: alloc::format!("{ES}{clause}"),
        basis,
        governing: "es-trin",
        note: note.to_string(),
    }
}

fn governing_of(mlc: f64, es: f64) -> &'static str {
    if (mlc - es).abs() < 1e-9 {
        "both"
    } else if mlc > es {
        "mlc"
    } else {
        "es-trin"
    }
}

/// A row of a sea-reaching hull: the clause names MLC and, where it also applies, ES-TRIN.
#[allow(clippy::too_many_arguments)]
fn sea_row(
    id: &'static str,
    name: &'static str,
    count: u32,
    each: f64,
    clause: String,
    basis: &'static str,
    governing: &'static str,
    note: String,
) -> Room {
    Room {
        id,
        name,
        count,
        area_each_m2: each,
        area_m2: count as f64 * each,
        clause,
        basis,
        governing,
        note,
    }
}

/// Size the rooms from the berths. No berths: no rooms. A Service Area that reaches sea sizes each
/// room to the greater of MLC 2006 A3.1 and ES-TRIN 2025/1 Chapter 15 (issue #892).
pub fn size_rooms(b: &RoomsBasis) -> Rooms {
    let n = b.berths;
    let sea = b.reaches_sea;
    let band = gt_band(b.gt);
    let band_name = [
        "under 3,000 GT",
        "3,000 GT to under 10,000 GT",
        "10,000 GT and over",
    ][band];
    let gt_note = if b.gt == 0 {
        "gross tonnage not known: the largest band is taken".to_string()
    } else {
        alloc::format!("{} GT, band {band_name}", b.gt)
    };
    let mut rows: Vec<Room> = Vec::new();
    if n > 0 {
        // One cabin per berth in every mode: OVD's own cautious rule. ES-TRIN 15.02(10) allows two to a
        // cabin. RPN 18.02(4) is a derogation from (1) and (2) (modes A1 and A2) that allows rest
        // underway in a room assigned to one crew member; it is context, not the source of this rule.
        let berth_floor = BERTH_LENGTH_M * BERTH_WIDTH_M;
        let cabin = (CABIN_VOLUME_M3 / HEADROOM_M)
            .max(CABIN_AIR_FIRST_M3 / HEADROOM_M)
            .max(berth_floor);
        // Cabin areas of the hull in all (for the corridor): the river-only rule has one size.
        let mut cabin_area_sum = cabin * n as f64;
        if !sea {
            let cabin_row = row(
                "cabin",
                "Sleeping cabin (one berth)",
                n,
                cabin,
                "15.02(9), 15.02(10), 15.02(6), 15.07(1)",
                "derived",
                "The largest of 7 m3 / 2.00 m headroom, 5 m3 of air / 2.00 m, and the 2.00 x 0.90 m berth. \
                 Furniture volume is not added. One cabin per crew member in every mode: OVD's own cautious rule (15.02(10) allows two to a cabin). Context: CCNR RPN, state 1 January 2026 (English unofficial translation), 18.02(4) is a derogation from (1) and (2), modes A1 and A2, allowing rest underway in a room assigned to one crew member only; it names no mode B and is not the source of this rule.",
            );
            rows.push(cabin_row);
        } else {
            let o = b.officers.min(n);
            let day = o.min(MLC_DAY_ROOM_OFFICERS);
            let plain_officers = o - day;
            let ratings = n - o;
            let f = MLC_SLEEPING_M2[band];
            let k = MLC_OFFICER_SLEEPING_M2[band];
            let f_para = ["9(f)(i)", "9(f)(ii)", "9(f)(iii)"][band];
            let k_para = ["9(k)(i)", "9(k)(ii)", "9(k)(iii)"][band];
            let es_clause = "15.02(9), 15.02(10), 15.02(6), 15.07(1)";
            let common = alloc::format!(
                "ES-TRIN needs {cabin:.2} m2 (7 m3 / 2.00 m headroom, the largest of its three tests). MLC headroom is {MLC_HEADROOM_M:.2} m (A3.1.6(a)); a berth is at least 1.98 x 0.80 m (A3.1.9(e)), the 2.00 x 0.90 m of ES-TRIN 15.07(1) is larger. Floor includes berth, lockers and seat (Guideline B3.1.5(6)). An individual sleeping room for each seafarer (A3.1.9(a), (d)); the exemption for ships under 3,000 GT is not applied. Furniture to A3.1.9(n) and (o) is inside the floor. {gt_note}."
            );
            if ratings > 0 {
                let each = f.max(cabin);
                cabin_area_sum += (each - cabin) * ratings as f64;
                rows.push(sea_row(
                    "cabin",
                    "Sleeping cabin (rating, cook, margin or spare berth)",
                    ratings,
                    each,
                    alloc::format!("{MLC}{f_para}, 9(a), 9(d), 9(e), 6(a); {ES}{es_clause}"),
                    "sourced",
                    governing_of(f, cabin),
                    alloc::format!("MLC A3.1.9(f) single berth floor {f:.1} m2. {common}"),
                ));
            }
            if day > 0 {
                let each = f.max(cabin);
                cabin_area_sum += (each - cabin) * day as f64;
                rows.push(sea_row(
                    "cabin_senior",
                    "Sleeping cabin with day room (master, chief engineer, chief navigating officer)",
                    day,
                    each,
                    alloc::format!("{MLC}{f_para}, 9(m), 9(a), 9(e), 6(a); {ES}{es_clause}"),
                    "sourced",
                    governing_of(f, cabin),
                    alloc::format!("A3.1.9(m) gives each of the three an adjoining sitting room or day room, so the single berth floor {f:.1} m2 of A3.1.9(f) applies and not the officer floor of A3.1.9(k), which is for \"no private sitting room or day room\". {common}"),
                ));
                rows.push(sea_row(
                    "day_room",
                    "Day room (sitting room) beside a senior officer's cabin",
                    day,
                    DAY_ROOM_M2,
                    alloc::format!("{MLC}9(m)"),
                    "estimate",
                    "mlc",
                    "Required for the master, the chief engineer and the chief navigating officer. ESTIMATE: A3.1.9(m) gives no area; a 2.5 x 2.4 m room for a settee, a table and a chair, rounded to 6.0 m2. The exemption for ships under 3,000 GT is not applied.".to_string(),
                ));
            }
            if plain_officers > 0 {
                let each = k.max(cabin);
                cabin_area_sum += (each - cabin) * plain_officers as f64;
                rows.push(sea_row(
                    "cabin_officer",
                    "Sleeping cabin (officer, no day room)",
                    plain_officers,
                    each,
                    alloc::format!("{MLC}{k_para}, 9(a), 9(e), 6(a); {ES}{es_clause}"),
                    "sourced",
                    governing_of(k, cabin),
                    alloc::format!("MLC A3.1.9(k) officer floor {k:.1} m2 where no private sitting room or day room is provided. {common}"),
                ));
            }
        }
        let communal = (COMMUNAL_PER_PERSON_M2 * n as f64).max(COMMUNAL_MIN_M2);
        rows.push(row(
            "communal",
            "Communal living room and mess",
            1,
            communal,
            "15.02(8), 15.02(7), 15.04(3)",
            "derived",
            if sea {
                "2 m2 per person, and 8 m2 in all, of free floor not counting furniture (15.02(8)), so a lower bound: furniture is not added, so the room is larger. One sitting; partitioned from the cabins (15.02(7)). For a sea-reaching hull this is the mess of A3.1.10 (apart from the sleeping rooms, near the galley, adequate size); Guideline B3.1.6(3) asks 1.5 m2 per seat, which 2 m2 a person already exceeds, so ES-TRIN governs."
            } else {
                "2 m2 per person, and 8 m2 in all, of free floor not counting furniture (15.02(8)), so a lower bound: furniture is not added, so the room is larger. One sitting; partitioned from the cabins (15.02(7))."
            },
        ));
        let toilets = ceil_div(n, PERSONS_PER_TOILET);
        let mut toilet_row = row(
            "toilet",
            "Toilet compartment",
            toilets,
            TOILET_M2,
            "15.03(1)(a), 15.03(3)",
            "sourced",
            "One per six crew, 1 m2 each.",
        );
        if sea {
            toilet_row.clause = alloc::format!("{MLC}11(c); {ES}15.03(1)(a), 15.03(3)");
            toilet_row.governing = "both";
            toilet_row.note = "One per six persons (A3.1.11(c) and ES-TRIN 15.03(1)(a) agree), 1 m2 each (15.03(3)). A3.1.11(b) asks for sanitary facilities within easy access of the bridge and the machinery space: that is a placement of these toilets, not an added count; the exemption under 3,000 GT is not applied.".to_string();
        }
        rows.push(toilet_row);
        let es_basins = ceil_div(n, PERSONS_PER_WASHBASIN);
        let mut basin_row = row(
            "washbasin",
            "Washbasin",
            es_basins,
            WASHBASIN_M2,
            "15.03(1)(b)",
            "estimate",
            "One per four crew. ESTIMATE: the text gives no floor area; 0.60 x 0.80 m of basin and standing room, rounded to 0.5 m2.",
        );
        if sea {
            // A3.1.11(d): a washbasin in every sleeping room (every berth has its own room here).
            basin_row.count = n.max(es_basins);
            basin_row.area_m2 = basin_row.count as f64 * WASHBASIN_M2;
            basin_row.governing = governing_of(n as f64, es_basins as f64);
            basin_row.clause = alloc::format!("{MLC}11(d), 11(c), 11(f); {ES}15.03(1)(b)");
            basin_row.note = "One in every sleeping room, with hot and cold running fresh water (A3.1.11(d), (f)); ES-TRIN 15.03(1)(b) asks one per four crew, fewer. A3.1.11(c) asks one per six persons without personal facilities, which these satisfy. ESTIMATE: the text gives no floor area; 0.60 x 0.80 m of basin and standing room, rounded to 0.5 m2, added to the cabin floor.".to_string();
        }
        rows.push(basin_row);
        let mut shower_row = row(
            "shower",
            "Shower or bath",
            ceil_div(n, PERSONS_PER_SHOWER),
            SHOWER_M2,
            "15.03(1)(c)",
            "estimate",
            "One per six crew. ESTIMATE: the text gives no floor area; a 0.90 x 0.90 m cubicle.",
        );
        if sea {
            shower_row.clause = alloc::format!("{MLC}11(c), 11(f); {ES}15.03(1)(c)");
            shower_row.governing = "both";
            shower_row.note = "One tub or shower per six persons (A3.1.11(c) and ES-TRIN 15.03(1)(c) agree). ESTIMATE: the text gives no floor area; a 0.90 x 0.90 m cubicle.".to_string();
        }
        rows.push(shower_row);
        let front = if sea {
            cabin_area_sum / n as f64 / BERTH_LENGTH_M
        } else {
            cabin / BERTH_LENGTH_M
        };
        let bays = ceil_div(n, 2);
        rows.push(row(
            "corridor",
            "Corridor",
            1,
            CORRIDOR_WIDTH_M * bays as f64 * front,
            "15.02(11)(a), 15.02(4)",
            "derived",
            if sea {
                "0.60 m wide (the clear width of a door) along cabins on both sides: two cabins per bay, each cabin front the mean cabin floor over the 2.00 m berth length."
            } else {
                "0.60 m wide (the clear width of a door) along cabins on both sides: two cabins per bay, \
             each cabin front 1.75 m (the cabin floor over the 2.00 m berth length)."
            },
        ));
        rows.push(row(
            "galley",
            "Galley",
            1,
            GALLEY_M2,
            "15.04(2)",
            "estimate",
            "A cooker, sink, water, refrigerator, storage and working space. ESTIMATE: the text gives no floor area; a 2.4 m run of units 0.60 m deep with a 1.0 m aisle (3.84 m2) and 1.2 m2 of stores, rounded to 5.0 m2.",
        ));
        rows.push(row(
            "work_clothes",
            "Work-clothes store and drying room",
            1,
            WORK_CLOTHES_BASE_M2 + WORK_CLOTHES_PER_BERTH_M2 * n as f64,
            "15.07(2)",
            "estimate",
            "Required, not in the sleeping quarters. ESTIMATE: the text gives no floor area; 2.0 m2 for hanging and drying and 0.25 m2 a berth for the lockers of 15.07(1).",
        ));
        rows.push(row(
            "stairs",
            "Stairways between tiers",
            1,
            STAIRWAY_M2,
            "15.02(12)",
            "estimate",
            "At least 0.60 m wide, tread 0.15 m. ESTIMATE: no floor area is given; 13 risers of 0.20 m at a 0.15 m tread is a 1.95 m run plus a 0.60 m landing, 0.60 x 2.55 m, rounded to 1.6 m2. One stairway (to the wheelhouse); a further one for each tier above the first is not counted, as the rooms do not know the tier count.",
        ));
        if sea && b.seafarers >= MLC_HOSPITAL_SEAFARERS {
            rows.push(sea_row(
                "hospital",
                "Hospital",
                1,
                HOSPITAL_M2,
                alloc::format!("{MLC}12"),
                "estimate",
                "mlc",
                alloc::format!("Separate hospital accommodation for medical use only, for a ship with {MLC_HOSPITAL_SEAFARERS} or more seafarers on a voyage over three days (A3.1.12); {} are carried. The voyage length is not an input, so it is taken as over three days, and the relaxation for coastal trade is not applied. ESTIMATE: A3.1.12 gives no area; one berth (2.0 x 0.9 m) with 0.9 m at its head and 0.75 m of access each side, 2.9 x 2.4 m = 6.96 m2, rounded to 7.0 m2.", b.seafarers),
            ));
            rows.push(sea_row(
                "hospital_sanitary",
                "Hospital toilet, washbasin and shower",
                1,
                TOILET_M2 + WASHBASIN_M2 + SHOWER_M2,
                "MLC 2006 as amended (2022 consolidated text) Guideline B3.1.8(4)".to_string(),
                "estimate",
                "mlc",
                "For the exclusive use of the hospital's occupants: one toilet, one washbasin and one tub or shower (a Guideline, not a Standard: applied cautiously). 1.00 m2 toilet (as ES-TRIN 15.03(3)), 0.5 m2 washbasin and 0.81 m2 shower, the ESTIMATES above.".to_string(),
            ));
        }
    }
    let total_m2 = rows.iter().map(|r| r.area_m2).sum();
    Rooms {
        berths: n,
        rows,
        total_m2,
        reaches_sea: sea,
        mlc_applied: sea,
        headroom_m: if sea { MLC_HEADROOM_M } else { HEADROOM_M },
        rule_note: if sea {
            alloc::format!(
                "The Service Area reaches sea, so each room is sized to the greater of MLC 2006 Standard A3.1 (as amended, 2022 consolidated text) and ES-TRIN 2025/1 Chapter 15; the governing source is on each row. {gt_note}. No exemption is applied: not the under-3,000 GT ones of A3.1.9(a), (h), (m), 10(a), 11(b), 15, nor the under-200 GT ones of A3.1.20, nor the coastal-trade relaxation of A3.1.12; the competent authority grants them, so a concept sizes without them. Berths held by officers are sized to A3.1.9(k) or (m) and all others as ratings."
            )
        } else {
            "River-only: ES-TRIN 2025/1 Chapter 15 (MLC 2006 does not reach a ship exclusively in inland waters).".to_string()
        },
    }
}
