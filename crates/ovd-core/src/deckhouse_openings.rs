//! The openings of a deckhouse stern (spec 0011 D17, issue #912): entrances, openable windows,
//! wheelhouse doors and windows, and air inlets, each with a position, a face, a kind and a space.
//! The engine places a default layout, or takes a typed list; the Measures of the Rule Trace read
//! each opening's distance from the deckhouse front, whether it faces the cargo area and whether it
//! stands in the zone IGC 3.2.4.1 keeps clear of openings.
//!
//! Source read: IGC Code, MSC.370(93), paragraphs 3.2.4.1, 3.2.4.2 and 3.2.5 (the rule-source text
//! `igc/MSC.370-93.txt`), and MSC.411(97) (which replaces 3.2.5 and takes out its A-0
//! sentence for wheelhouse windows, so no fire class is given here); IGC 1.2.31 (L is the load line
//! length); ICLL 1966 as amended by the 1988 Protocol, Regulation 3(1)(a) (`icll/MSC.143-77.txt`):
//! L is 96 % of the total length on a waterline at 85 % of the least moulded depth, or the length
//! from the fore side of the stem to the axis of the rudder stock on that waterline, if that is
//! greater. **Only the first reading is worked**: the rudder-stock alternative is not modelled, so L
//! can be a little under the Convention's where the stock is further forward than the stern end.
//!
//! IGC 3.2.4.1: entrances, air inlets and openings to accommodation, service and machinery spaces
//! and control stations must not face the cargo area. They go on the end bulkhead that does not face
//! the cargo area, or on the outboard sides at a distance of at least 4 % of L, not less than 3 m,
//! from the end of the deckhouse that faces the cargo area, and that distance need not exceed 5 m.
//! IGC 3.2.4.2: windows facing the cargo area and on the sides within that distance are fixed, but
//! wheelhouse windows may open and wheelhouse doors may stand within the limits if the wheelhouse can
//! be made gas and vapour tight rapidly and efficiently.
//!
//! The layout is an **estimate** of where a designer puts them (the positions, the door height and
//! the count are not read from a source); the paragraphs above are what the Rules (issue #913) check.
//! Concept and preliminary design only (ADR 0011).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// 4 % of L: IGC 3.2.4.1.
pub const ZONE_FRACTION: f64 = 0.04;
/// The least the zone is, and the most it need be (m): IGC 3.2.4.1 ("not less than 3 m", "need not
/// exceed 5 m").
pub const ZONE_MIN_M: f64 = 3.0;
pub const ZONE_MAX_M: f64 = 5.0;
/// The centre of a door above the deck of its tier (m). ESTIMATE: a 2.0 m door on a 0.0 m sill.
pub const DOOR_CENTRE_ABOVE_DECK_M: f64 = 1.0;
/// The centre of an air inlet or intake above the deck of its tier (m). ESTIMATE: at three quarters
/// of the tier height it clears the weather deck spray.
pub const INLET_CENTRE_FRACTION: f64 = 0.75;
/// The centre of a wheelhouse window above the wheelhouse floor (m). ESTIMATE: standing eye height
/// less the sill, the middle of a window 1.2 m deep.
pub const WHEELHOUSE_WINDOW_CENTRE_M: f64 = 1.5;
/// A tolerance on a distance measured against the edge of the zone (m): a door placed exactly at the
/// zone length is outside the zone, whatever the rounding of the sum that put it there.
const EDGE_EPS_M: f64 = 1e-9;

/// The length of the zone from the deckhouse front, in which no entrance, air inlet or opening may
/// stand on a side (IGC 3.2.4.1): 4 % of L, at least 3 m and at most 5 m.
pub fn zone_length_m(l_icll_m: f64) -> f64 {
    let z = ZONE_FRACTION * l_icll_m;
    if z.is_nan() || z < ZONE_MIN_M {
        // Also a length that is not a number: the least the zone is.
        ZONE_MIN_M
    } else if z > ZONE_MAX_M {
        ZONE_MAX_M
    } else {
        z
    }
}

/// The face of the deckhouse an opening is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// The end that faces the cargo area.
    Front,
    /// An outboard side.
    Side,
    /// The end bulkhead that does not face the cargo area.
    Aft,
}

impl Face {
    pub const fn key(self) -> &'static str {
        match self {
            Face::Front => "front",
            Face::Side => "side",
            Face::Aft => "aft",
        }
    }
    pub fn from_key(key: &str) -> Option<Face> {
        [Face::Front, Face::Side, Face::Aft]
            .into_iter()
            .find(|f| f.key() == key)
    }
}

/// What the opening is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A door or an entrance.
    Entrance,
    /// A window or a sidescuttle (fixed or openable, by `openable`).
    Window,
    /// An air inlet or outlet of a space.
    AirInlet,
    /// An air intake of the engines, when they do not take air from the engine room.
    AirIntake,
}

impl Kind {
    pub const fn key(self) -> &'static str {
        match self {
            Kind::Entrance => "entrance",
            Kind::Window => "window",
            Kind::AirInlet => "air_inlet",
            Kind::AirIntake => "air_intake",
        }
    }
    pub fn from_key(key: &str) -> Option<Kind> {
        [Kind::Entrance, Kind::Window, Kind::AirInlet, Kind::AirIntake]
            .into_iter()
            .find(|k| k.key() == key)
    }
}

/// The space the opening leads to or serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Accommodation,
    Service,
    Machinery,
    ControlStation,
    Wheelhouse,
}

impl Space {
    pub const fn key(self) -> &'static str {
        match self {
            Space::Accommodation => "accommodation",
            Space::Service => "service",
            Space::Machinery => "machinery",
            Space::ControlStation => "control_station",
            Space::Wheelhouse => "wheelhouse",
        }
    }
    pub fn from_key(key: &str) -> Option<Space> {
        [
            Space::Accommodation,
            Space::Service,
            Space::Machinery,
            Space::ControlStation,
            Space::Wheelhouse,
        ]
        .into_iter()
        .find(|s| s.key() == key)
    }
}

/// One opening as given (a typed row) or as placed by default. x is metres forward of the stern, y
/// metres to starboard of the centreline, z metres above the keel.
#[derive(Debug, Clone, PartialEq)]
pub struct Opening {
    pub id: String,
    pub kind: Kind,
    pub space: Space,
    pub face: Face,
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
    /// A window that can be opened, or a door (always openable). A fixed window is `false`.
    pub openable: bool,
    /// Designed so that the wheelhouse can be made gas and vapour tight rapidly (IGC 3.2.4.2).
    pub gas_tight: bool,
    /// The tier it is on, counted from 1 at the main deck; 0 for the wheelhouse.
    pub tier: i64,
}

/// An opening with what the Measures read of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub opening: Opening,
    /// Metres from the deckhouse front, measured along the deckhouse (0 on the front itself).
    pub distance_from_front_m: f64,
    /// It is on the end that faces the cargo area.
    pub faces_cargo_area: bool,
    /// It is on the front, or on a side nearer the front than the zone length (IGC 3.2.4.1).
    pub in_zone: bool,
}

/// What the layout is worked from: the deckhouse and the wheelhouse, and the load line length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub x_aft_m: f64,
    pub x_fwd_m: f64,
    pub width_m: f64,
    /// The floor of the first tier (m above the keel).
    pub base_m: f64,
    pub tier_height_m: f64,
    pub tiers: i64,
    /// The wheelhouse: aft end, forward end, width and floor (m above the keel). `None`: none.
    pub wheelhouse: Option<(f64, f64, f64, f64)>,
    /// L of IGC 1.2.31 (ICLL Regulation 3(1)): 96 % of the waterline length at 85 % of the depth.
    pub l_icll_m: f64,
}

/// The openings of a deckhouse, with how they came.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub l_icll_m: f64,
    pub zone_m: f64,
    /// A typed list stood in for the default layout.
    pub typed: bool,
    pub openings: Vec<Placed>,
    pub basis: String,
}

/// The distance of an opening from the deckhouse front, and whether it is in the zone.
fn measure(o: &Opening, frame: &Frame, zone_m: f64) -> (f64, bool) {
    let distance = match o.face {
        Face::Front => 0.0,
        // The aft end bulkhead and the sides: along the deckhouse from its front to the opening.
        Face::Side | Face::Aft => {
            let d = frame.x_fwd_m - o.x_m;
            if d > 0.0 { d } else { 0.0 }
        }
    };
    let in_zone = match o.face {
        Face::Front => true,
        Face::Side => distance < zone_m - EDGE_EPS_M,
        // The end bulkhead that does not face the cargo area: no distance is needed (IGC 3.2.4.1).
        Face::Aft => false,
    };
    (distance, in_zone)
}

/// The default layout of a deckhouse stern: accommodation and machinery doors and the air inlets on
/// the aft end (which does not face the cargo area, so no distance is needed there), one
/// accommodation entrance for each tier, wheelhouse doors on the sides marked gas-tight, wheelhouse
/// side windows that open and can be made gas-tight, and a fixed window in the wheelhouse front. No
/// opening stands on the deckhouse front, and none on a side of a tier. ESTIMATES: the positions,
/// the door height, the count.
pub fn default_layout(frame: &Frame) -> Vec<Opening> {
    let mut v: Vec<Opening> = Vec::new();
    let w = frame.width_m;
    let tiers = if frame.tiers > 0 { frame.tiers } else { 1 };
    // Each tier: an accommodation entrance on the aft end, to port of the centreline.
    for t in 1..=tiers {
        let deck = frame.base_m + (t - 1) as f64 * frame.tier_height_m;
        v.push(Opening {
            id: alloc::format!("accommodation entrance, tier {t}"),
            kind: Kind::Entrance,
            space: Space::Accommodation,
            face: Face::Aft,
            x_m: frame.x_aft_m,
            y_m: -w / 4.0,
            z_m: deck + DOOR_CENTRE_ABOVE_DECK_M,
            openable: true,
            gas_tight: false,
            tier: t,
        });
    }
    let deck1 = frame.base_m;
    // The engine room entrance, tier 1, to starboard.
    v.push(Opening {
        id: "machinery entrance".to_string(),
        kind: Kind::Entrance,
        space: Space::Machinery,
        face: Face::Aft,
        x_m: frame.x_aft_m,
        y_m: w / 4.0,
        z_m: deck1 + DOOR_CENTRE_ABOVE_DECK_M,
        openable: true,
        gas_tight: false,
        tier: 1,
    });
    // The engine room air inlet, and the engine air intake, on the aft end, tier 1.
    v.push(Opening {
        id: "engine room air inlet".to_string(),
        kind: Kind::AirInlet,
        space: Space::Machinery,
        face: Face::Aft,
        x_m: frame.x_aft_m,
        y_m: w / 3.0,
        z_m: deck1 + INLET_CENTRE_FRACTION * frame.tier_height_m,
        openable: true,
        gas_tight: false,
        tier: 1,
    });
    v.push(Opening {
        id: "engine air intake".to_string(),
        kind: Kind::AirIntake,
        space: Space::Machinery,
        face: Face::Aft,
        x_m: frame.x_aft_m,
        y_m: -w / 3.0,
        z_m: deck1 + INLET_CENTRE_FRACTION * frame.tier_height_m,
        openable: true,
        gas_tight: false,
        tier: 1,
    });
    // The accommodation air inlet, on the aft end of the top tier.
    let top_deck = frame.base_m + (tiers - 1) as f64 * frame.tier_height_m;
    v.push(Opening {
        id: "accommodation air inlet".to_string(),
        kind: Kind::AirInlet,
        space: Space::Accommodation,
        face: Face::Aft,
        x_m: frame.x_aft_m,
        y_m: 0.0,
        z_m: top_deck + INLET_CENTRE_FRACTION * frame.tier_height_m,
        openable: true,
        gas_tight: false,
        tier: tiers,
    });
    if let Some((x0, x1, ww, floor)) = frame.wheelhouse {
        let mid = 0.5 * (x0 + x1);
        for (side, y) in [("port", -ww / 2.0), ("starboard", ww / 2.0)] {
            // IGC 3.2.4.2: a wheelhouse door may stand where the sides are kept clear, if the
            // wheelhouse can be made gas-tight rapidly: it is designed so, and marked so.
            v.push(Opening {
                id: alloc::format!("wheelhouse door, {side}"),
                kind: Kind::Entrance,
                space: Space::Wheelhouse,
                face: Face::Side,
                x_m: mid,
                y_m: y,
                z_m: floor + DOOR_CENTRE_ABOVE_DECK_M,
                openable: true,
                gas_tight: true,
                tier: 0,
            });
            v.push(Opening {
                id: alloc::format!("wheelhouse window, {side}"),
                kind: Kind::Window,
                space: Space::Wheelhouse,
                face: Face::Side,
                x_m: mid,
                y_m: y,
                z_m: floor + WHEELHOUSE_WINDOW_CENTRE_M,
                openable: true,
                gas_tight: true,
                tier: 0,
            });
        }
        v.push(Opening {
            id: "wheelhouse window, front".to_string(),
            kind: Kind::Window,
            space: Space::Wheelhouse,
            face: Face::Front,
            x_m: x1,
            y_m: 0.0,
            z_m: floor + WHEELHOUSE_WINDOW_CENTRE_M,
            openable: false,
            gas_tight: false,
            tier: 0,
        });
    }
    v
}

/// The openings of a deckhouse: the typed list when there is one (it stands in for the default
/// layout, wholly), else the default layout; each with its distance from the front, whether it faces
/// the cargo area and whether it is in the zone.
pub fn place(frame: &Frame, typed: Option<&[Opening]>) -> Placement {
    let zone_m = zone_length_m(frame.l_icll_m);
    let (list, is_typed): (Vec<Opening>, bool) = match typed {
        Some(t) => (t.to_vec(), true),
        None => (default_layout(frame), false),
    };
    let openings = list
        .into_iter()
        .map(|o| {
            let (distance, in_zone) = measure(&o, frame, zone_m);
            Placed {
                faces_cargo_area: o.face == Face::Front,
                distance_from_front_m: distance,
                in_zone,
                opening: o,
            }
        })
        .collect();
    Placement {
        l_icll_m: frame.l_icll_m,
        zone_m,
        typed: is_typed,
        openings,
        basis: if is_typed {
            "The typed list of openings (the `deckhouse_openings` input) stands in for the default layout, wholly. The distance, the face to the cargo area and the zone follow from each opening's face and x. IGC 3.2.4.1, 3.2.4.2 (MSC.370(93)); L by ICLL Regulation 3(1)(a) at 85 % of the depth, without the rudder-stock alternative.".to_string()
        } else {
            "Default layout (ESTIMATE: positions, door height and count are not from a source): accommodation and machinery doors and air inlets on the aft end, no opening on the front, wheelhouse doors on the sides designed gas-tight (IGC 3.2.4.2). The zone is 4 % of L, at least 3 m and at most 5 m, from the deckhouse front (IGC 3.2.4.1); L by ICLL Regulation 3(1)(a) at 85 % of the depth, without the rudder-stock alternative.".to_string()
        },
    }
}

/// The deckhouse and wheelhouse of a vessel's stern block as a [`Frame`], with L from the hull. `None`
/// for a vessel with no deckhouse.
pub fn frame_of(
    aft: &crate::aft_arrangement::AftArrangementResult,
    hull: &crate::sections::SectionsTable,
    depth_m: f64,
) -> Option<Frame> {
    let d = aft.deckhouse.as_ref()?;
    // ICLL Regulation 3(1)(a): 96 % of the length on a waterline at 85 % of the least moulded depth.
    let l = 0.96
        * crate::sections::immersed(hull, &crate::sections::Waterplane::upright(0.85 * depth_m))
            .l_wl_m;
    Some(Frame {
        x_aft_m: d.x_aft_m,
        x_fwd_m: d.x_fwd_m,
        width_m: d.width_m,
        base_m: d.base_m,
        tier_height_m: d.tier_height_m,
        tiers: d.tiers,
        wheelhouse: Some((
            aft.wheelhouse_x0_m,
            aft.wheelhouse_x1_m,
            aft.wheelhouse_width_m,
            aft.wheelhouse_floor_lowered_m,
        )),
        l_icll_m: l,
    })
}
