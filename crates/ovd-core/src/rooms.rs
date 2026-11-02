//! The accommodation rooms of a deckhouse (spec 0011 D5, section 2; issue #780), sized from the
//! berths the crew model gives.
//!
//! Source read: ES-TRIN 2025/1, Chapter 15, Articles 15.02, 15.03, 15.04 and 15.07 (the PDF text in
//! the rule sources). **MLC 2006 Standard A3.1 is not in the rule sources, so it is not applied**:
//! a hull whose Service Area reaches sea is sized to the ES-TRIN rooms and the result says so; the
//! deckhouse keeps the interim floor as its lower bound there (see `aft_arrangement`). RPN 18.02(4)
//! (one cabin each when rest is taken underway in operating mode B) is cited from
//! `docs/research/manning.md`; its text is not in the rule sources either.
//!
//! Each row says how its area came: `sourced` (a figure in the paragraph), `derived` (worked from
//! figures in the paragraph, the working named in the note) or `gap` (the paragraph requires the
//! room or fixture but gives no area: counted, area 0, never invented). Concept and preliminary
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
/// Clear width of a door, at least (m): ES-TRIN 15.02(11)(a). Taken as the least corridor width: a
/// corridor narrower than its doors would not pass a person through them.
pub const CORRIDOR_WIDTH_M: f64 = 0.60;

const ES: &str = "ES-TRIN 2025/1 ";

/// What the rooms are sized from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomsBasis {
    /// Berths: the crew, the owner margin and the spare berths (crew model, D16).
    pub berths: u32,
    /// The Service Area reaches sea: MLC A3.1 would apply, and is not applied (text not read).
    pub reaches_sea: bool,
    /// Inland operating mode B (RPN 18.01): rest may be taken underway, one cabin each (18.02(4)).
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
    /// `sourced`, `derived` or `gap`.
    pub basis: &'static str,
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
    /// MLC A3.1 was applied. Always `false` until its text is in the rule sources.
    pub mlc_applied: bool,
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
        note: note.to_string(),
    }
}

/// Size the rooms from the berths. No berths: no rooms.
pub fn size_rooms(b: &RoomsBasis) -> Rooms {
    let n = b.berths;
    let mut rows: Vec<Room> = Vec::new();
    if n > 0 {
        // One cabin per berth: ES-TRIN 15.02(10) allows two to a cabin, but one each is the cautious
        // reading, and RPN 18.02(4) requires it in mode B when rest is taken underway.
        let berth_floor = BERTH_LENGTH_M * BERTH_WIDTH_M;
        let cabin = (CABIN_VOLUME_M3 / HEADROOM_M)
            .max(CABIN_AIR_FIRST_M3 / HEADROOM_M)
            .max(berth_floor);
        let mut cabin_row = row(
            "cabin",
            "Sleeping cabin (one berth)",
            n,
            cabin,
            "15.02(9), 15.02(10), 15.02(6), 15.07(1)",
            "derived",
            "The largest of 7 m3 / 2.00 m headroom, 5 m3 of air / 2.00 m, and the 2.00 x 0.90 m berth. \
             Furniture volume is not added. One cabin each: the cautious reading of 15.02(10), which allows two.",
        );
        if b.mode_b {
            cabin_row.clause.push_str(
                "; RPN 18.02(4) via docs/research/manning.md (one cabin each in mode B with rest underway; the RPN text is not in the rule sources)",
            );
        }
        rows.push(cabin_row);
        let communal = (COMMUNAL_PER_PERSON_M2 * n as f64).max(COMMUNAL_MIN_M2);
        rows.push(row(
            "communal",
            "Communal living room and mess",
            1,
            communal,
            "15.02(8), 15.02(7), 15.04(3)",
            "derived",
            "2 m2 per person, and 8 m2 in all, of free floor not counting furniture (15.02(8)), so a lower bound: furniture is not added, so the room is larger. One sitting; partitioned from the cabins (15.02(7)).",
        ));
        let toilets = ceil_div(n, PERSONS_PER_TOILET);
        rows.push(row(
            "toilet",
            "Toilet compartment",
            toilets,
            TOILET_M2,
            "15.03(1)(a), 15.03(3)",
            "sourced",
            "One per six crew, 1 m2 each.",
        ));
        rows.push(row(
            "washbasin",
            "Washbasin",
            ceil_div(n, PERSONS_PER_WASHBASIN),
            0.0,
            "15.03(1)(b)",
            "gap",
            "One per four crew; the text gives no floor area.",
        ));
        rows.push(row(
            "shower",
            "Shower or bath",
            ceil_div(n, PERSONS_PER_SHOWER),
            0.0,
            "15.03(1)(c)",
            "gap",
            "One per six crew; the text gives no floor area.",
        ));
        let front = cabin / BERTH_LENGTH_M;
        let bays = ceil_div(n, 2);
        rows.push(row(
            "corridor",
            "Corridor",
            1,
            CORRIDOR_WIDTH_M * bays as f64 * front,
            "15.02(11)(a), 15.02(4)",
            "derived",
            "0.60 m wide (the clear width of a door) along cabins on both sides: two cabins per bay, \
             each cabin front 1.75 m (the cabin floor over the 2.00 m berth length).",
        ));
        rows.push(row(
            "galley",
            "Galley",
            1,
            0.0,
            "15.04(2)",
            "gap",
            "A cooker, sink, water, refrigerator, storage and working space; the text gives no floor area.",
        ));
        rows.push(row(
            "work_clothes",
            "Work-clothes store and drying room",
            1,
            0.0,
            "15.07(2)",
            "gap",
            "Required, not in the sleeping quarters; the text gives no floor area.",
        ));
        rows.push(row(
            "stairs",
            "Stairways between tiers",
            0,
            0.0,
            "15.02(12)",
            "gap",
            "At least 0.60 m wide, tread 0.15 m; the number depends on the tier count, and no floor area is given.",
        ));
    }
    let total_m2 = rows.iter().map(|r| r.area_m2).sum();
    Rooms {
        berths: n,
        rows,
        total_m2,
        reaches_sea: b.reaches_sea,
        mlc_applied: false,
        rule_note: if b.reaches_sea {
            "The Service Area reaches sea, so MLC 2006 Standard A3.1 applies; its text is not in the rule sources, so it is NOT applied. The rooms are sized to ES-TRIN 2025/1 Chapter 15, and the deckhouse keeps the interim floor as its lower bound.".to_string()
        } else {
            "River-only: ES-TRIN 2025/1 Chapter 15 (MLC 2006 does not reach a ship exclusively in inland waters).".to_string()
        },
    }
}
