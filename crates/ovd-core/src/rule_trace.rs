//! OVD's side of the rules engine (spec 0004, ADR 0013, ADR 0019): the object types, the Measure
//! registry and the formula shapes it registers with the domain-free kernel, `arc-rules`. The
//! evaluator and the Rule Trace are the kernel's ([`arc_rules::eval`]); they are re-exported here,
//! and [`evaluate`] and [`failures`] pass OVD's formula shapes.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub use arc_rules::domain::{ArgDef, FormulaShape, MeasureInfo, MethodStatus, ValueKind};
pub use arc_rules::eval::{
    Failure, MeasureSource, MeasureValue, Outcome, ROW_SHAPES, Trace, TraceRow, UsedValue, applies,
    applies_to_object, holds, margin, row_shape, stricter, table_value,
};

use crate::rules::RuleSet;

/// The type of object a Measure is computed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Option,
    Hull,
    /// A Type C gas cargo tank.
    Tank,
    LoadingCondition,
    /// A liquid bulk cargo tank (spec 0006): one object type per tank role, so a Rule on cargo tanks
    /// never reaches a ballast or slop tank.
    CargoTank,
    BallastTank,
    SlopTank,
    /// One assumed damage of a liquid bulk hull (issue #341): the flooded compartments of an extent.
    DamageCase,
    /// One assumed damage of MARPOL Annex I regulation 28 (issue #341): its extents reach the cargo tanks.
    MarpolDamageCase,
    /// The mean oil outflow of a liquid bulk hull (MARPOL Annex I regulation 23, issue #341): one object.
    OilOutflow,
    /// One assumed damage of ADN 2025 9.3.1.15 on a gas hull (spec 0012 D3, issue #763).
    AdnGasDamageCase,
    /// One assumed damage of ES-TRIN 2025/1 28.03(4) on a craft over 110 m (spec 0012 D2, issue #763).
    EsTrinDamageCase,
    /// One assumed damage of NORMAM-202 5.23.8 on a liquid bulk hull (spec 0012 D2, issue #763).
    NormamDamageCase,
    /// One opening of a deckhouse stern: an entrance, a window or an air inlet (issue #912).
    Opening,
}

impl ObjectType {
    pub const ALL: [ObjectType; 14] = [
        ObjectType::Option,
        ObjectType::Hull,
        ObjectType::Tank,
        ObjectType::LoadingCondition,
        ObjectType::CargoTank,
        ObjectType::BallastTank,
        ObjectType::SlopTank,
        ObjectType::DamageCase,
        ObjectType::MarpolDamageCase,
        ObjectType::OilOutflow,
        ObjectType::AdnGasDamageCase,
        ObjectType::EsTrinDamageCase,
        ObjectType::NormamDamageCase,
        ObjectType::Opening,
    ];

    /// The object type of a key, or `None` for a key OVD does not declare.
    pub fn from_key(key: &str) -> Option<ObjectType> {
        ObjectType::ALL.into_iter().find(|o| o.key() == key)
    }

    pub const fn key(self) -> &'static str {
        match self {
            ObjectType::Option => "option",
            ObjectType::Hull => "hull",
            ObjectType::Tank => "tank",
            ObjectType::LoadingCondition => "loading-condition",
            ObjectType::CargoTank => "cargo-tank",
            ObjectType::BallastTank => "ballast-tank",
            ObjectType::SlopTank => "slop-tank",
            ObjectType::DamageCase => "damage-case",
            ObjectType::MarpolDamageCase => "marpol-damage-case",
            ObjectType::OilOutflow => "oil-outflow",
            ObjectType::AdnGasDamageCase => "adn-gas-damage-case",
            ObjectType::EsTrinDamageCase => "es-trin-damage-case",
            ObjectType::NormamDamageCase => "normam-damage-case",
            ObjectType::Opening => "opening",
        }
    }
}

/// One entry of the Measure registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasureDef {
    pub id: &'static str,
    pub unit: &'static str,
    pub object: ObjectType,
    pub value: ValueKind,
    pub args: &'static [ArgDef],
    /// The method, cited: what the Measure computes and how.
    pub method: &'static str,
    pub method_status: MethodStatus,
}

const fn num(name: &'static str) -> ArgDef {
    ArgDef { name, text: false }
}

/// The computed Measures of the core. Input Measures (`input.<name>`) are added by the caller, which
/// knows the inputs and their units.
pub static MEASURES: &[MeasureDef] = &[
    MeasureDef {
        id: "gz.area",
        unit: "m.rad",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg"), num("to_deg")],
        method: "Trapezoid area under the GZ curve on the 1 degree grid, from from_deg to to_deg",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gz.area_to_downflooding",
        unit: "m.rad",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg"), num("to_deg")],
        method: "Trapezoid area under the GZ curve on the 1 degree grid from from_deg to to_deg, or to the angle of down-flooding the designer gave where that is less (the last part with GZ interpolated linearly); none when the angle is not above from_deg",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gz.max_from",
        unit: "m",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg")],
        method: "Largest GZ on the 1 degree grid at a heel of from_deg or more",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gz.angle_of_max",
        unit: "deg",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[],
        method: "Heel of the largest GZ on the 1 degree grid, 0 to 60 degrees; the first of equal values",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "hull.double_side_width",
        unit: "m",
        object: ObjectType::Hull,
        value: ValueKind::Number,
        args: &[],
        method: "Width of the double side of a liquid bulk hull: the layout's, the same along the tank block (the distance from the side shell to the longitudinal bulkhead of the cargo tanks)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "hull.double_bottom_height",
        unit: "m",
        object: ObjectType::Hull,
        value: ValueKind::Number,
        args: &[],
        method: "Height of the double bottom of a liquid bulk hull: the layout's, the same along the tank block",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "hull.lbh",
        unit: "m3",
        object: ObjectType::Hull,
        value: ValueKind::Number,
        args: &[],
        method: "L x B x H as ADN 2025 9.3.2.11.1 (a) defines it for the tank capacity table: L the overall length of the hull, B the extreme breadth, H the moulded depth within the cargo area",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.volume",
        unit: "m3",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "Gross moulded volume of one cargo tank of the liquid bulk layout (a port or starboard tank beside a centreline bulkhead is a tank); slop and ballast tanks are other objects",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.length",
        unit: "m",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "Length of one cargo tank of the liquid bulk layout, along the hull",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.vessel_lbh",
        unit: "m3",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "L x B x H of the hull the cargo tank is in (the value of hull.lbh), so a Rule on one cargo tank can read the table of ADN 2025 9.3.2.11.1 (a) on the same object",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.vessel_length",
        unit: "m",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "The overall length of the hull the cargo tank is in, so a Rule on one cargo tank can read the length limit of ADN 2025 9.3.2.11.1 (d) on the same object",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.adn_vessel_type",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Text,
        args: &[],
        method: "The ADN Tank Vessel Type of the Cargo (N_open, N_closed or C), derived from its Crude Grade by the ADN 2025 3.2.3.3 flowchart; an unknown flag gives C",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.deadweight",
        unit: "t",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The deadweight of the run: the displacement at the reported draught less the lightship, so the weight-bound or volume-bound cargo plus the stores and fuel",
        method_status: MethodStatus::Unverified,
    },
    // Issue #777 (spec 0011 D15): the gross tonnage, its V and its K1. No Rule reads them yet.
    MeasureDef {
        id: "option.enclosed_volume",
        unit: "m3",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "V of ITC 1969 Annex I Regulation 3: the moulded hull volume to the deck at the centre line, plus the enclosed volume above the main deck of the stern block (40 ft units and wheelhouse); no deckhouse yet (issue #779), no excluded spaces taken off",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.gross_tonnage_k1",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "K1 = 0.2 + 0.02 log10 V, ITC 1969 Annex I Regulation 3 (UN Treaty Series vol. 1291, p. 15)",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "option.gross_tonnage",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "GT = K1 V, ITC 1969 Annex I Regulation 3 (UN Treaty Series vol. 1291, p. 15), with V as option.enclosed_volume; a typed gross_tonnage overrides it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "vent.exit_above_deck",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The height of the cargo PRV vent exit above the weather deck (the moulded depth), as the engine placed it for the Service Area (issue #822). Coastal and river: the greater of B/3 or 6 m above the deck (IGC 8.2.10.3) and 6 m above the working area (8.2.10.4). River only: the greater of 2.00 m above the deck (ADN 9.3.1.22.3) and the dome top plus the typed stub",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "vent.exit_above_working_area",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The height of the cargo PRV vent exit above the working area of IGC 8.2.10.4 (6 m asked): the top of the weather shield, or the tank top where the hull has none. The top of the tank domes is not a working area (issue #822)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "vent.distance_to_openings",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The distance from the cargo PRV vent exit to the nearest air intake, outlet or opening to accommodation, service spaces and control stations, IGC 8.2.11.1 (B or 25 m, whichever is less, asked): along the hull, from the aft end of the cargo area less the smallest of the entrance, engine room ventilation inlet and funnel outlet distances. The mast stands as far forward in the cargo zone as the hull allows (issue #822)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "vent.distance_to_accommodation",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The distance from the cargo PRV vent exit to the accommodation and service spaces outside the cargo area, ADN 9.3.1.22.3 (6.00 m asked): along the hull, from the aft end of the cargo area less the entrance distance. The mast stands as far forward in the cargo zone as the hull allows (issue #822)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.marpol_length_ratio",
        unit: "-",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "The length of the cargo tank over the longest tank MARPOL Annex I regulation 23.3.2 allows: 10 m or the formula of 23.3.2.1 to 23.3.2.3 (by the bulkheads of the tanks abreast and bi/B), whichever is the greater; at most 1 meets it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "oil_outflow.ratio",
        unit: "-",
        object: ObjectType::OilOutflow,
        value: ValueKind::Number,
        args: &[],
        method: "The mean oil outflow parameter OM of MARPOL Annex I regulation 23 (damaged tank method: side and bottom damage probabilities of 23.8 and 23.9, outflow by pressure balance of 23.7, combined by 23.5) over the largest OM that 23.3.1 allows for the cargo volume: at most 1 meets it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.cargo_capacity_m3",
        unit: "m3",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Gross moulded volume of the cargo tanks of a liquid bulk layout, slop tanks excluded",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.slop_capacity_share",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The gross volume of the slop tanks over the gross volume of the cargo and slop tanks together; zero with no slop tank",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gm0",
        unit: "m",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[],
        method: "Initial metacentric height of the box section, less the free surface correction",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "wheelhouse.blind_distance",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[ArgDef {
            name: "condition",
            text: true,
        }],
        method: "The distance ahead of the bow up to which the helmsman's view of the water is blocked (light or laden): where the ray from the eye over the cargo or tank top at the forward end of the cargo zone meets the waterline, trimmed where the trim is known",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "wheelhouse.ship_length",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The length overall of the ship whose blind distance is limited (SOLAS V/22.1.1: two ship lengths): the Option's length overall, or for a Formation whose tug has a sightline the length of the unit, tug and barges (issue #989)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "powering.speed_through_water_kmh",
        unit: "km/h",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[ArgDef {
            name: "environment",
            text: true,
        }],
        method: "The speed through the water at the installed power, laden, in the named environment (the powering's service speed, 1 kn = 1.852 km/h)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "powering.speed_over_design",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The lowest service speed of any environment at the installed power, over the Design Speed",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "wheelhouse.lift_enough",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "True when the lift the Blind Distance needs is within the maximum lift of the wheelhouse",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "powering.depth_over_draught",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The depth of the design channel over the loaded draught (h/T)",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "propulsion.diameter_over_max",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The propeller diameter over the largest the Stern Form allows (D_max)",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "propulsion.immersion_over_limit",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The shaft axis submergence over the propeller diameter, at the lightest aft draught of the loading conditions, over the limit of the type (0.76 open, 0.71 ducted; Beveridge 1971, a guide). The axis is half a diameter above the base",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "propulsion.blade_load_over_limit",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The delivered power of one unit at the Installed Power per m2 of disc, over the limit of the type (400 kW/m2 open, 500 ducted; Veth rating basis)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "freeboard.laden",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The Freeboard at the laden Draught: the Moulded Depth less the Draught of the laden result, amidships and level, the deck plating not added (ICLL regulation 3(8) measures the freeboard assigned from the deck line down to the load line)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "propulsion.redundancy_28_04",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "ES-TRIN 28.04(1)(a) from the inputs: a length of 110 m or less, or two or more propulsors with a bow thruster, or one propulsor with a bow thruster. Equal power, independence and a thruster that works unladen are assumed, as the model has no such detail",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "wheelhouse.field_of_view",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the helmsman's field of unobstructed vision meets 7.02(3); the model has no window geometry",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.headroom_m",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The headroom the deckhouse rooms are sized to (the constant the rooms table uses for every room); none on the container block or with no berths",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.cabin_area_m2",
        unit: "m2",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The floor area of one single sleeping cabin of the deckhouse rooms table (one cabin per berth)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.cabin_volume_m3",
        unit: "m3",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The cubic capacity of one single sleeping cabin of the rooms table: its floor area times the headroom the rooms are sized to",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.communal_area_m2",
        unit: "m2",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The floor area of the communal living room and mess of the rooms table (furniture not added)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.communal_area_per_person_m2",
        unit: "m2",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The floor area of the communal room of the rooms table over the berths",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.washbasins_per_cabin",
        unit: "washbasins",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The washbasins of the rooms table over its single sleeping cabins: the washbasins each cabin has, with none in a private bathroom",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.toilet_area_m2",
        unit: "m2",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The floor area of one toilet compartment of the rooms table",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "rooms.persons_per_fixture",
        unit: "persons",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[ArgDef {
            name: "fixture",
            text: true,
        }],
        method: "The berths over the count of the named sanitary fixture (toilet, washbasin or shower) in the rooms table: the persons each fixture serves, with every berth counted as a person who has no personal facilities",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "wheelhouse.conning_arc_deg",
        unit: "deg",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The horizontal field of vision from the conning position, as an arc; the model has no window geometry",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "arrangement.airlock",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether access between the hazardous area on the open weather deck and the non-hazardous spaces is by an airlock; the model places no doors",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "arrangement.airlock_door_spacing_m",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The distance between the two doors of the airlock; the model places no doors",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "arrangement.sleeping_rooms_above_load_line",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the sleeping rooms are above the load line amidships or aft, and not forward of the collision bulkhead; the rooms table does not place rooms by deck",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "wheelhouse.raised_stability",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the raised wheelhouse leaves the stability met; the model has no stability case with it raised",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "formation.coupling_force",
        unit: "kN",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "ES-TRIN ESI-III-6 2.1: F_SB = 270 x P_B x L_S / B_S x 10^-3 kN for a pusher and its pushed craft, with P_B the pusher's installed power and L_S, B_S the pusher's length and beam, from the Formation powering",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "machinery.escape_routes",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The means of escape from a machinery space of Category A; the model places none, and the SOLAS II-2 text is not in the rule sources (a named gap)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "arrangement.cargo_facing_fronts",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "The fire division of a front that faces the cargo area; the model places none, and the SOLAS II-2 text is not in the rule sources (a named gap)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "deckhouse.icll_length",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "L of IGC 1.2.31, the load line length: 96 % of the length on a waterline at 85 % of the moulded depth (ICLL Regulation 3(1)(a)); the alternative of the length from the fore side of the stem to the rudder stock axis is not worked",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "deckhouse.opening_zone",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The zone of IGC 3.2.4.1 from the deckhouse front in which no entrance, air inlet or opening stands on a side: 4 % of L, not less than 3 m and not more than 5 m",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.distance_from_front",
        unit: "m",
        object: ObjectType::Opening,
        value: ValueKind::Number,
        args: &[],
        method: "The distance along the deckhouse from its front (the end facing the cargo area) to the opening: nought on the front itself",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.faces_cargo_area",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the opening is on the deckhouse end that faces the cargo area (IGC 3.2.4.1)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.clear_of_cargo_face",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Bool,
        args: &[],
        method: "True when the opening is not on the deckhouse end that faces the cargo area: the negation of `opening.faces_cargo_area`, so a `check` Rule passes on it (IGC 3.2.4.1)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.in_zone",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the opening is on the front, or on a side nearer the front than the zone length (IGC 3.2.4.1); an opening on the end bulkhead away from the cargo area is never in the zone",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.openable",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the opening opens: a door, an air inlet, or a window designed to open (IGC 3.2.4.2)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.gas_tight",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the opening is designed so that the wheelhouse can be made gas and vapour tight rapidly and efficiently (IGC 3.2.4.2); set on the opening by the layout or the typed list",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.kind",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Text,
        args: &[],
        method: "The kind of the opening: entrance, window, air_inlet or air_intake",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.space",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Text,
        args: &[],
        method: "The space the opening serves: accommodation, service, machinery, control_station or wheelhouse",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "opening.face",
        unit: "-",
        object: ObjectType::Opening,
        value: ValueKind::Text,
        args: &[],
        method: "The face of the deckhouse the opening is in: front, side or aft",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "formation.separate_vessels",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "ABS Steel Barges 5-3-1/1.3: the tug and the barge are classed as two vessels; an approval item, the model does not assess either against class",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "formation.notch_wing_walls",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "ABS Steel Barges 5-3-1/11.3: the analysis of the barge wing walls at a notch (torsional, primary and secondary shear and bending); an approval item, the model has no structure model for it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "openings.front_clear",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "True when the typed deckhouse front has no entrance, air inlet or opening (the input says whether it has one)",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "arrangement.hold_space_segregation",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the hold spaces are segregated from machinery, accommodation, service spaces, control stations, stores and domestic water tanks, by the means IGC 3.1.1 and 3.1.2 allow; the model has no such spaces",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "arrangement.accommodation_outside_cargo_area",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether no accommodation space, service space or control station is within the cargo area; the model does not place them",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.vc",
        unit: "m3",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Gross volume of one tank (cylinder and heads, inside the steel) plus its dome, as IGC 2.4.1.1 defines Vc",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.side_gap",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Clear gap from the moulded side to the outer face of the insulation, as the arrangement places the tank",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.bottom_gap",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Clear height from the moulded base to the outer face of the insulation: double bottom plus support height",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.double_side_width",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Width of the double-hull space of a gas hull, from the moulded side to the longitudinal bulkhead of the wing tank (ADN 9.3.1.11.2 (a): the internal distance between the side plating and the longitudinal bulkheads): the clear side gap less the gap between the wing tank and the tank (issue #850)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.double_bottom_height",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Height of the double bottom of a gas hull, from the moulded base to the inner bottom (ADN 9.3.1.11.2 (a)): the clear bottom gap less the height of the tank supports (issue #850)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.insulation",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Insulation thickness of the tank",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.wall_thickness",
        unit: "mm",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The thinner of the shell and head walls after forming, with the corrosion allowance (pressure-vessel formula, paywalled source)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.weld_factor",
        unit: "-",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The welded joint efficiency factor the wall formula used",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.allowable_stress",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The allowable stress f the wall formula used",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.design_pressure",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The design vapour pressure (gauge) the wall formula used",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "tank.dynamic_pressure_term",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "A C (rho_r)^1.5 of IGC 4.23.1.2: A = 0.00185 (sigma_m / 55)^2 with sigma_m taken as the allowable stress the wall formula used; C the greatest of the tank outer diameter, 0.75 times it and 0.45 times the overall tank length; rho_r the liquid density at the design temperature over 1 t/m3",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.design_temperature",
        unit: "C",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The lowest design temperature of the tank",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "tank.rm",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Rm of the tank material: specified minimum tensile strength at room temperature (IGC 4.18.1.3)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.re",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Re of the tank material: specified minimum yield stress at room temperature (IGC 4.18.1.3)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.condition_ok",
        unit: "",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Bool,
        args: &[],
        method: "The engine's stability check of this loading condition: every IS Code 2008 Part A 2.2 criterion holds",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "stability.damage",
        unit: "",
        object: ObjectType::DamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "The damaged hull meets the damage criteria of the Rule's set. ADN 2025 9.3.2.15.2 and 9.3.3.15.2 (liquid bulk): the heel at the stage of equilibrium is at most 12 degrees and, beyond it, the righting lever reaches 0.05 m with an area of 0.0065 m.rad up to 27 degrees. IGC 2.7.1.2 and 2.7.2.1 (gas, spec 0012): a heel of at most 30 degrees and a range of 20 degrees from the equilibrium with 0.1 m and 0.0175 m.rad, on every IS Code loading condition; a breached Type C tank loses its cargo, an intact one keeps its buoyancy in a flooded hold. Lost buoyancy method on the Sections Table for the final stage of flooding; the intermediate stages are by added mass (see `stability.damage_stage_heel`). The non-watertight openings and the flooding of undamaged compartments are not modelled. Probabilistic damage stability (SOLAS II-1 Part B-1) is not applied: OVD's vessels are outside its scope (ADR 0011; research note docs/research/damage-stability.md, 2.4)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_stage_heel",
        unit: "deg",
        object: ObjectType::DamageCase,
        value: ValueKind::Number,
        args: &[],
        method: "The largest angle of heel of the damaged hull at any stage of flooding: the intermediate stages at 25, 50 and 75 % of the final flood and the final stage (spec 0012 D15). Intermediate stages by added mass: every damaged Space holds the same fraction of its final water, level and upright, as a mass at its centroid, with the free surface of its gross surface (ES-TRIN 2025/1 28.03(4)(e)); a breached tank holds sea water to the stage's fraction of its flood and keeps its cargo for the rest (its share of the cargo leaves in step with the inflow, spec 0012 D18a; IGC gives no stage method), the cargo of an intact tank stays aboard. A stage with no floating position has no value",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_intermediate",
        unit: "",
        object: ObjectType::DamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "Every intermediate stage of flooding (25, 50 and 75 % of the final flood, by added mass as for `stability.damage_stage_heel`) floats and meets the residual stability of the final stage: a range of 20 degrees from its equilibrium with a righting lever of 0.1 m and an area of 0.0175 m.rad (IGC 2.7.1.3 with 2.7.2.1)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_marpol",
        unit: "",
        object: ObjectType::MarpolDamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "The damaged hull floats where the heel at the stage of equilibrium is at most 25 degrees and the righting lever stays positive over 20 degrees beyond it, reaching 0.1 m with an area of 0.0175 m.rad within that range (MARPOL Annex I regulation 28.3.2 and 28.3.3). Lost buoyancy method on the Sections Table, final stage of flooding, for the extents of regulation 28.2; a damaged cargo tank is flooded at 95 % and its oil leaves the loading condition (regulation 28.4.1, any outflow of liquids from damaged compartments; spec 0012 D20)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_adn_gas",
        unit: "",
        object: ObjectType::AdnGasDamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "The damaged gas hull meets ADN 2025 9.3.1.15.2 at the stage of equilibrium: a heel of at most 12 degrees and, beyond it, a righting lever of 0.05 m with an area of 0.0065 m.rad up to 27 degrees. Assumed damage of 9.3.1.15.1: side 0.10 L (at least 5.00 m) long and 0.79 m inboard, bottom 0.10 L (at least 5.00 m) long, 3.00 m wide and 0.59 m high, every compartment the extent touches flooded (athwartships neighbours too for a bottom damage), the end bulkheads of the main engine room intact; permeability 95 %, 85 % in a machinery space, 0 or 95 % in a tank by its filling, a full cargo tank 0 %. Every IS Code loading condition. Lost buoyancy method on the Sections Table, final stage; an intact Type C tank keeps its buoyancy in a flooded hold. The openings (9.3.1.15.1 (c), 9.3.1.15.2) and cross-flooding (9.3.1.15.3, 9.3.1.15.4) are not modelled",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_adn_gas_intermediate",
        unit: "",
        object: ObjectType::AdnGasDamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "Every intermediate stage of flooding (25, 50 and 75 % of the final flood, by added mass as for `stability.damage_stage_heel`) floats and has a righting lever of at least 0.03 m over a positive range of 5 degrees beyond its equilibrium (ADN 2025 9.3.1.15.2). The text names no fractions and no heel limit for the intermediate stage; OVD's three fractions are those of ES-TRIN 28.03(3) (spec 0012 D15)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_es_trin",
        unit: "",
        object: ObjectType::EsTrinDamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "The damaged hull meets ES-TRIN 2025/1 28.03(6) at the final stage of flooding: a heel of at most 12 degrees and, beyond it, a righting lever of 0.05 m with an area of 0.0065 m.rad up to 27 degrees. Assumed damage of 28.03(4): side at least 0.10 L long and 0.59 m inboard, bottom at least 0.10 L long, 3.00 m wide and 0.39 m high, every compartment the extent touches flooded (athwartships neighbours too for a bottom damage), the end bulkheads of the main engine room intact; permeability 95 %, 85 % in a machinery space or pump room, a cargo hold at the greater of 95 % of its volume less the intact tanks in it and 70 % of its gross volume (28.03(4)(d)), 95 % in a void, 0 or 95 % in a tank by its function. A craft over 110 m, on the full load departure (28.03(3): the cargo for the maximum draught, evenly spread, full supplies), for a gas hull and a liquid bulk hull alike. Lost buoyancy method on the Sections Table; an intact Type C tank keeps its buoyancy in a flooded hold. The openings (28.03(6)(a), (6)(d)) and cross-flooding (28.03(7)) are not modelled",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_es_trin_intermediate",
        unit: "",
        object: ObjectType::EsTrinDamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "Every intermediate stage of flooding (25, 50 and 75 % of the final flood, by added mass with the free surface of the gross surface, as for `stability.damage_stage_heel`) floats at a heel of at most 15 degrees and has a righting lever of at least 0.02 m beyond it, before 27 degrees (ES-TRIN 2025/1 28.03(5)(a), (5)(b)). The stage immediately before transverse equilibrium and the openings of 28.03(5)(c) are not modelled",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_normam",
        unit: "",
        object: ObjectType::NormamDamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "The damaged hull meets NORMAM-202/DPC 5.23.9: at the final stage a heel of at most 25 degrees (up to 30 degrees when the deck edge is not immersed), a positive righting lever over at least 20 degrees beyond the equilibrium, and in that range an area of 0.0175 m.rad with a residual righting lever of 0.1 m. Standard damage of 5.23.8 (a), (b), (c), (d): the extents of MARPOL Annex I regulation 28.2 (side 1/3 L^(2/3) or 14.5 m long, B/5 or 11.5 m inboard; bottom B/6 wide and B/15 or 6 m high), placed anywhere in the length, every bulkhead in the damage damaged, the end bulkheads of the main engine room intact; permeability 95 %, 85 % in a machinery space, 0 or 95 % in a tank by its function (5.23.8 (f)), so a full cargo tank is 0 %. Lost buoyancy method on the Sections Table, final stage, on the laden departure. The lesser damage of 5.23.8 (g) and the openings of 5.23.8 (e) and 5.23.9 (b) are not modelled",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "check.ok",
        unit: "",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[ArgDef {
            name: "name",
            text: true,
        }],
        method: "The verdict of the engine's own check of this name, as the result's check list gives it",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "separation.cofferdam",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Hold end clearance at the aft end of the cargo area, as the gas separation used it (not a cofferdam, spec 0011 D18)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "separation.aft",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Aft separation from the tanks to the stern block: the hold end clearance, plus a hold cofferdam when one replaces the A-60 bulkhead. Since spec 0011 (D17) no design has it as a Rule Measure: it no longer holds the opening setback",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "separation.fwd",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Space between the cargo tanks and the forward end bulkhead of the hold",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "separation.tank_end",
        unit: "",
        object: ObjectType::Option,
        value: ValueKind::Text,
        args: &[],
        method: "The shape of the tank ends at the hold end bulkhead: plane or domed",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.material",
        unit: "",
        object: ObjectType::Tank,
        value: ValueKind::Text,
        args: &[],
        method: "The tank material key",
        method_status: MethodStatus::Unverified,
    },
];

/// A Measure of the core registry.
pub fn measure_def(id: &str) -> Option<&'static MeasureDef> {
    MEASURES.iter().find(|m| m.id == id)
}

impl MeasureDef {
    pub fn info(&self) -> MeasureInfo {
        MeasureInfo {
            unit: self.unit.to_string(),
            object: self.object.key().to_string(),
            value: self.value,
            args: self.args.to_vec(),
            method_status: self.method_status,
        }
    }
}

/// The formula shapes OVD registers with the kernel: id, operands, coefficients, and the operands
/// whose unit is not the limit's (the coefficients convert them).
pub static FORMULA_SHAPES: &[FormulaShape] = &[
    // `of - less`: a value less an input (IGC 2.4.1.2 side distance less the insulation).
    shape("less-input", &["of", "less"], &[], &[]),
    // `max(lo, min(hi, a + x * b / c))`: "a + b·x, the lesser of that and hi, not less than lo"
    // (MARPOL Annex I Reg. 19.3.1: w = 0.5 + DW/20,000 or 2.0 m, whichever is the lesser, at least
    // 1.0 m).
    shape("clamp", &["x"], &["a", "b", "c", "lo", "hi"], &["x"]),
    // `max(lo, a + x * b / c)`: the same with no upper limit (MARPOL Annex I Reg. 19.6.2).
    shape("at-least", &["x"], &["a", "b", "c", "lo"], &["x"]),
    // `max(min(a + x * b / c, hi), of) - less`: the greater of a capped value and `of`, less an
    // input (IGC 2.4.1.2 bottom distance: the greater of B/15 or 2 m, whichever is less, and d,
    // less the insulation).
    shape(
        "max-capped-less",
        &["x", "of", "less"],
        &["a", "b", "c", "hi"],
        &["x"],
    ),
    // `min(rm / a, re / b)`: the lesser of two ratios (IGC 4.23.3.1, f = the lesser of Rm/A or
    // Re/B).
    shape("lesser-ratio", &["rm", "re", "a", "b"], &[], &["a", "b"]),
    // Issue #740 (spec 0013): the scantling formulas of ABS Steel Barges (`abs-steel-barges`). The
    // panel model (`hull_panels`) sizes its members with them; each keeps the order of operations of
    // the text and of the code before the move, so the move changes no bit.
    // `a * x + b * s + c`: a plate or web thickness linear in L and the spacing (5-1-2/5.1.1).
    shape("linear-ls", &["x", "s"], &["a", "b", "c"], &["x", "s"]),
    // The same, less a reduction (5-1-2/5.17: less 1 mm with longitudinal framing).
    shape(
        "linear-ls-less",
        &["x", "s"],
        &["a", "b", "c", "less"],
        &["x", "s"],
    ),
    // Two lines split at `x_max`: the first up to it (`x <= x_max`, or `x < x_max` when
    // `strict` is 1), the second above (3-2-2/3.1, 3-2-3/2.1.2(a), 5-1-1/3.1).
    shape(
        "linear-ls-split",
        &["x", "s"],
        &["a1", "b1", "c1", "a2", "b2", "c2", "x_max", "strict"],
        &["x", "s"],
    ),
    // The same with x taken at most `x_cap` (3-2-2/3.5.1 and 5.1: L need not exceed 225 m).
    shape(
        "linear-ls-split-capped",
        &["x", "s"],
        &["a1", "b1", "c1", "a2", "b2", "c2", "x_max", "x_cap"],
        &["x", "s"],
    ),
    // `s (Lb + c1) / (k Lb + c2)`, Lb = min(L, l_max) (3-2-3/2.1.2(b)).
    shape(
        "deck-b",
        &["l", "s"],
        &["l_max", "c1", "k", "c2"],
        &["l", "s"],
    ),
    // SMb of 3-2-1/3.1 with K = 1: C1 C2 L^2 B (Cb + cb_add), C1 by L.
    shape(
        "abs-smr",
        &["l", "b", "cb"],
        &[
            "c1_min", "l_a", "qa", "qb", "qc", "l_div", "l_b", "c1_max", "l_c", "l_d", "l_div_d",
            "p", "c2", "cb_add",
        ],
        &["l", "b", "cb"],
    ),
    // `max(s k sqrt(h) / d + c, t_min, s / e + f)`, k of the panel aspect ratio (3-2-7/3.1, 3-2-6/5.1).
    shape(
        "plate-sqrt-head",
        &["s", "aspect", "h"],
        &[
            "d", "c", "t_min", "e", "f", "a_max", "k_long", "a_min", "k1", "k2", "k3",
        ],
        &["s", "aspect", "h"],
    ),
    // `factor * k c h s l^2`: the section modulus of a stiffener (k = 7.8) or a primary member
    // (k = 4.74) (3-2-6/5.3, 3-2-7/3.3, 5-2-1/7, 5-2-1/11, 5-2-2/3.5, 5-2-2/3.7).
    shape(
        "sm-chsl2",
        &["h", "s", "l"],
        &["factor", "k", "c"],
        &["h", "s", "l"],
    ),
    // The same with c = 1 / (p - q k) for a longitudinal deck beam (5-1-1/3.1), k of the hull girder.
    shape(
        "sm-chsl2-hull-k",
        &["h", "s", "l"],
        &["k", "p", "q", "hull_k"],
        &["h", "s", "l"],
    ),
    // The tank head of 5-2-1/3: the distance to the deck at side plus ha1 by L.
    shape(
        "tank-head",
        &["l", "depth", "z"],
        &["h_short", "l_short", "k", "l_long", "h_long"],
        &["l", "depth", "z"],
    ),
    // The watertight bulkhead head of 3-2-6/5.3: the distance to the deck d, as `k d + c` below `d_max`.
    shape(
        "bulkhead-head",
        &["depth", "z"],
        &["d_max", "k", "c"],
        &["depth", "z"],
    ),
    // The web of a girder, chord or transverse (3-2-4/3.11): `min(a depth + c, t_cap)`, not less than
    // L1 / d1 + c1 (L >= l_min) or L1 / d2 + c2, L1 = max(L, l_floor).
    shape(
        "web-minimum",
        &["l", "depth_mm"],
        &[
            "a", "c", "t_cap", "l_floor", "l_min", "d1", "c1", "d2", "c2",
        ],
        &["l", "depth_mm"],
    ),
];

const fn shape(
    id: &'static str,
    operands: &'static [&'static str],
    coefficients: &'static [&'static str],
    free_unit_operands: &'static [&'static str],
) -> FormulaShape {
    FormulaShape {
        id,
        operands,
        coefficients,
        free_unit_operands,
        value: shape_value,
    }
}

fn shape_value(
    shape: &str,
    coefficients: &[(String, f64)],
    operand: &mut dyn FnMut(&str) -> Result<f64, String>,
) -> Result<f64, String> {
    formula_value(shape, coefficients, operand)
}

/// The value of a formula shape. The order of operations is the text's (`a + x * b / c`, then the
/// upper limit, then the lower), so a Rule written from the text stays bit-identical to a hand
/// calculation of the same text.
pub fn formula_value(
    shape: &str,
    coefficients: &[(String, f64)],
    mut operand: impl FnMut(&str) -> Result<f64, String>,
) -> Result<f64, String> {
    let c = |name: &str| -> Result<f64, String> {
        coefficients
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, x)| *x)
            .ok_or_else(|| format!("formula `{shape}` needs coefficient `{name}`"))
    };
    match shape {
        "less-input" => {
            let of = operand("of")?;
            Ok(of - operand("less")?)
        }
        "clamp" => {
            let v = c("a")? + operand("x")? * c("b")? / c("c")?;
            let (lo, hi) = (c("lo")?, c("hi")?);
            let v = if v < hi { v } else { hi };
            Ok(if v > lo { v } else { lo })
        }
        "at-least" => {
            let v = c("a")? + operand("x")? * c("b")? / c("c")?;
            let lo = c("lo")?;
            Ok(if v > lo { v } else { lo })
        }
        "max-capped-less" => {
            // The order of `py_max(py_min(beam / 15.0, 2.0), d_vc) - ins_m` in the gas arrangement.
            let v = c("a")? + operand("x")? * c("b")? / c("c")?;
            let hi = c("hi")?;
            let capped = if hi < v { hi } else { v };
            let of = operand("of")?;
            let greater = if of > capped { of } else { capped };
            Ok(greater - operand("less")?)
        }
        "lesser-ratio" => {
            // The order of `igc_allowable_stress` in `pressure_vessel`.
            let by_tensile = operand("rm")? / operand("a")?;
            let by_yield = operand("re")? / operand("b")?;
            Ok(if by_yield < by_tensile {
                by_yield
            } else {
                by_tensile
            })
        }
        "linear-ls" => Ok(c("a")? * operand("x")? + c("b")? * operand("s")? + c("c")?),
        "linear-ls-less" => {
            Ok(c("a")? * operand("x")? + c("b")? * operand("s")? + c("c")? - c("less")?)
        }
        "linear-ls-split" | "linear-ls-split-capped" => {
            let mut x = operand("x")?;
            if shape == "linear-ls-split-capped" {
                let cap = c("x_cap")?;
                x = if cap < x { cap } else { x };
            }
            let s = operand("s")?;
            let x_max = c("x_max")?;
            let first = if shape == "linear-ls-split" && c("strict")? != 0.0 {
                x < x_max
            } else {
                x <= x_max
            };
            Ok(if first {
                c("a1")? * x + c("b1")? * s + c("c1")?
            } else {
                c("a2")? * x + c("b2")? * s + c("c2")?
            })
        }
        "deck-b" => {
            let (l, l_max) = (operand("l")?, c("l_max")?);
            let lb = if l_max < l { l_max } else { l };
            Ok(operand("s")? * (lb + c("c1")?) / (c("k")? * lb + c("c2")?))
        }
        "abs-smr" => {
            let l = operand("l")?;
            let c1 = if l < c("l_a")? {
                c("c1_min")?
            } else if l < c("l_b")? {
                let r = l / c("l_div")?;
                c("qa")? * libm::pow(r, 2.0) - c("qb")? * r + c("qc")?
            } else if l <= c("l_c")? {
                c("c1_max")? - libm::pow((c("l_c")? - l) / c("l_div")?, c("p")?)
            } else if l < c("l_d")? {
                c("c1_max")?
            } else {
                c("c1_max")? - libm::pow((l - c("l_d")?) / c("l_div_d")?, c("p")?)
            };
            Ok(c1 * c("c2")? * l * l * operand("b")? * (operand("cb")? + c("cb_add")?))
        }
        "plate-sqrt-head" => {
            let a = operand("aspect")?;
            let k = if a > c("a_max")? {
                c("k_long")?
            } else {
                let a_min = c("a_min")?;
                let a = if a < a_min { a_min } else { a };
                (c("k1")? * libm::sqrt(a) - c("k2")?) / (a + c("k3")?)
            };
            let s = operand("s")?;
            let h = operand("h")?;
            let t = s * k * libm::sqrt(h.max(0.0)) / c("d")? + c("c")?;
            Ok(t.max(c("t_min")?).max(s / c("e")? + c("f")?))
        }
        "sm-chsl2" => {
            let sm =
                c("k")? * c("c")? * operand("h")? * operand("s")? * operand("l")? * operand("l")?;
            Ok(c("factor")? * sm)
        }
        "sm-chsl2-hull-k" => {
            let cc = 1.0 / (c("p")? - c("q")? * c("hull_k")?);
            Ok(c("k")? * cc * operand("h")? * operand("s")? * operand("l")? * operand("l")?)
        }
        "tank-head" => {
            let l = operand("l")?;
            let ha1 = if l <= c("l_short")? {
                c("h_short")?
            } else if l <= c("l_long")? {
                c("k")? * l
            } else {
                c("h_long")?
            };
            Ok((operand("depth")? - operand("z")?).max(0.0) + ha1)
        }
        "bulkhead-head" => {
            let d = (operand("depth")? - operand("z")?).max(0.0);
            Ok(if d < c("d_max")? {
                c("k")? * d + c("c")?
            } else {
                d
            })
        }
        "web-minimum" => {
            let (l, depth) = (operand("l")?, operand("depth_mm")?);
            let by_depth = (c("a")? * depth + c("c")?).min(c("t_cap")?);
            let l1 = l.max(c("l_floor")?);
            let floor_t = if l >= c("l_min")? {
                l1 / c("d1")? + c("c1")?
            } else {
                l1 / c("d2")? + c("c2")?
            };
            Ok(by_depth.max(floor_t))
        }
        other => Err(format!("unknown formula shape `{other}`")),
    }
}

/// Evaluate the selected sets on one Option, with OVD's formula shapes
/// ([`arc_rules::eval::evaluate`]).
pub fn evaluate(sets: &[&RuleSet], source: &dyn MeasureSource) -> Trace {
    arc_rules::eval::evaluate(sets, FORMULA_SHAPES, source)
}

/// The failing rows of [`evaluate`], with OVD's formula shapes ([`arc_rules::eval::failures`]).
pub fn failures(sets: &[&RuleSet], source: &dyn MeasureSource) -> Vec<Failure> {
    arc_rules::eval::failures(sets, FORMULA_SHAPES, source)
}
