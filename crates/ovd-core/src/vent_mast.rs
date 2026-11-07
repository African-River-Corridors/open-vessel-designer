//! The cargo PRV vent mast of a gas hull (issue #822): where it stands and how high its exit is.
//!
//! The rule texts read for this module (not paraphrased from memory):
//!
//! - IGC Code (resolution MSC.370(93) text), 8.2.10.3: the height of vent exits "shall not be less
//!   than B/3 or 6 m, whichever is the greater, above the weather deck"; 8.2.10.4: "6 m above working
//!   areas and walkways".
//! - IGC Code 8.2.11.1: cargo PRV vent exits "at a distance at least equal to B or 25 m, whichever
//!   is less, from the nearest air intake, outlet or opening to accommodation spaces, service spaces
//!   and control stations, or other non-hazardous areas. For ships less than 90 m in length, smaller
//!   distances may be permitted."
//! - ADN, 9.3.1.22.3: the exhaust outlets of the pressure relief valves "not less than 2.00 m above
//!   the deck at a distance of not less than 6.00 m from the accommodation and from the service
//!   spaces located outside the cargo area. This height may be reduced when within a radius of 1.00 m
//!   round the pressure relief valve outlet there is no equipment, no work is being carried out and
//!   signs indicate the area."
//!
//! This module only places the mast and reads the heights; the Rule rows that judge them are
//! separate (issues #849 and #850). The fuel vent mast of the gas-fuel system is another mast.

/// The least height of an IGC 8.2.10.3 vent exit above the weather deck, m (the "6 m").
pub const IGC_MIN_ABOVE_DECK_M: f64 = 6.0;
/// The height of an IGC 8.2.10.4 vent exit above a working area or walkway, m.
pub const IGC_ABOVE_WORKING_AREA_M: f64 = 6.0;
/// The upper bound of the IGC 8.2.11.1 distance, m (the "25 m").
pub const IGC_DISTANCE_CAP_M: f64 = 25.0;
/// The least height of an ADN 9.3.1.22.3 exhaust outlet above the deck, m.
pub const ADN_ABOVE_DECK_M: f64 = 2.00;
/// The least distance of an ADN 9.3.1.22.3 exhaust outlet from the accommodation and service
/// spaces, m.
pub const ADN_DISTANCE_M: f64 = 6.00;
/// The default height of the exit above the dome top (`vent_stub_m`), m: a working value, not a
/// rule.
pub const DEFAULT_VENT_STUB_M: f64 = 1.0;

/// What the placing needs, all lengths in m. Heights are above the keel; `x` is from the aft end of
/// the hull, forward positive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VentMastInputs {
    /// `river-only` reads ADN 9.3.1.22.3; any other Service Area reads the IGC clauses.
    pub river_only: bool,
    pub beam_m: f64,
    /// The weather deck: the moulded depth.
    pub deck_m: f64,
    /// The top of the cargo tanks, without the domes.
    pub tank_top_m: f64,
    /// The top of the weather shield, when the hull has one.
    pub shield_top_m: Option<f64>,
    /// The top of the tank domes (the tank top where there is no dome).
    pub dome_top_m: f64,
    /// The height of the exit above the dome top on a river-only hull.
    pub stub_m: f64,
    /// Where the mast stands.
    pub mast_x_m: f64,
    /// The aft end of the cargo area.
    pub cargo_area_aft_x_m: f64,
    /// The distance from the cargo area of the nearest entrance or openable window, of the engine
    /// room ventilation inlet and of the funnel outlet.
    pub entrance_m: f64,
    pub engine_room_vent_inlet_m: f64,
    pub funnel_outlet_m: f64,
}

/// Which term of the exit height governs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Governs {
    /// The deck term: IGC 8.2.10.3 (B/3 or 6 m above the weather deck) or ADN (2.00 m above the deck).
    Deck,
    /// IGC 8.2.10.4: 6 m above the working area.
    WorkingArea,
    /// ADN: the dome top and the stub.
    DomeTop,
}

impl Governs {
    pub fn key(self) -> &'static str {
        match self {
            Governs::Deck => "deck",
            Governs::WorkingArea => "working_area",
            Governs::DomeTop => "dome_top",
        }
    }
}

/// The mast: its place, its exit and the distances it keeps.
#[derive(Debug, Clone, PartialEq)]
pub struct VentMast {
    pub river_only: bool,
    pub x_m: f64,
    pub exit_above_keel_m: f64,
    pub exit_above_deck_m: f64,
    /// The top of the working area of IGC 8.2.10.4: the weather shield, or the tank top with none.
    pub working_area_top_m: f64,
    pub working_area_is_shield: bool,
    pub exit_above_working_area_m: f64,
    pub governs: Governs,
    /// To the nearest air intake, outlet or opening (the IGC 8.2.11.1 datum).
    pub distance_to_openings_m: f64,
    /// To the accommodation and service spaces (the ADN 9.3.1.22.3 datum).
    pub distance_to_accommodation_m: f64,
    /// Issue #994: to the nearest opening of the forward house of `wheelhouse_forward`; `None` for any
    /// other style. It is read against the same distance as the others.
    pub distance_to_forward_house_m: Option<f64>,
    /// The distance the Service Area asks for, and the datum it is read on.
    pub target_distance_m: f64,
    pub distance_met: bool,
    pub lowerable: bool,
}

/// The height of the exit above the keel for a Service Area, and the term that sets it.
pub fn exit_above_keel(i: &VentMastInputs) -> (f64, Governs) {
    if i.river_only {
        // ADN 9.3.1.22.3: 2.00 m above the deck; the dome top and the stub are the height the
        // outlet needs to clear the tank dome.
        let deck = i.deck_m + ADN_ABOVE_DECK_M;
        let dome = i.dome_top_m + i.stub_m;
        if dome > deck {
            (dome, Governs::DomeTop)
        } else {
            (deck, Governs::Deck)
        }
    } else {
        // IGC 8.2.10.3 and 8.2.10.4.
        let above_deck = igc_above_deck(i.beam_m);
        let deck = i.deck_m + above_deck;
        let working = working_area_top(i) + IGC_ABOVE_WORKING_AREA_M;
        if working > deck {
            (working, Governs::WorkingArea)
        } else {
            (deck, Governs::Deck)
        }
    }
}

/// The least height of an IGC 8.2.10.3 vent exit above the weather deck: B/3 or 6 m, whichever is the
/// greater.
pub fn igc_above_deck(beam_m: f64) -> f64 {
    let third = beam_m / 3.0;
    if third > IGC_MIN_ABOVE_DECK_M {
        third
    } else {
        IGC_MIN_ABOVE_DECK_M
    }
}

/// The top of the working area: the weather shield, or the tank top where the hull has none.
pub fn working_area_top(i: &VentMastInputs) -> f64 {
    i.shield_top_m.unwrap_or(i.tank_top_m)
}

/// The exit less each datum, before any snapping: above the deck and above the working area.
fn raw_heights(i: &VentMastInputs, exit: f64) -> (f64, f64) {
    (exit - i.deck_m, exit - working_area_top(i))
}

/// The heights above the deck and above the working area. Where an IGC term sets the exit, the sum
/// and the difference can differ from the term by one rounding step, which a Rule with no tolerance
/// (IGC 8.2.10.3 and 8.2.10.4, issue #849) would read as a failure of the mast the engine placed. A
/// height under its term by no more than 1e-9 m is the term; a larger gap is a bug.
fn heights(i: &VentMastInputs, exit: f64) -> (f64, f64) {
    let (deck, working) = raw_heights(i, exit);
    if i.river_only {
        return (deck, working);
    }
    let snap = |raw: f64, term: f64| {
        if raw < term {
            debug_assert!(
                term - raw <= SNAP_M,
                "exit {raw} m is under its term {term} m"
            );
            if term - raw <= SNAP_M {
                return term;
            }
        }
        raw
    };
    (
        snap(deck, igc_above_deck(i.beam_m)),
        snap(working, IGC_ABOVE_WORKING_AREA_M),
    )
}

/// The most a height may sit under the term that set it before it is read as the term, m.
const SNAP_M: f64 = 1e-9;

/// The x of a mast that stands as far as it can from both houses of `wheelhouse_forward` (issue #994),
/// within the cargo area `[cargo_area_aft_x_m, cargo_area_fwd_x_m]`. The distance aft is the mast's
/// distance from the cargo area's aft end plus `nearest_aft_m` (the nearest opening aft, measured from
/// the cargo area, as [`place`] reads it); the distance forward is `forward_opening_x_m` less the mast.
/// The two are equal at the point returned, unless the cargo area is too short to reach it.
pub fn balanced_x_m(
    cargo_area_aft_x_m: f64,
    cargo_area_fwd_x_m: f64,
    nearest_aft_m: f64,
    forward_opening_x_m: f64,
) -> f64 {
    let x = (forward_opening_x_m + cargo_area_aft_x_m - nearest_aft_m) / 2.0;
    if x < cargo_area_aft_x_m {
        cargo_area_aft_x_m
    } else if x > cargo_area_fwd_x_m {
        cargo_area_fwd_x_m
    } else {
        x
    }
}

/// The smallest of the distances from the cargo area to the openings aft (m), as [`place`] reads them.
pub fn nearest_aft_m(i: &VentMastInputs) -> f64 {
    let mut nearest = i.entrance_m;
    if i.engine_room_vent_inlet_m < nearest {
        nearest = i.engine_room_vent_inlet_m;
    }
    if i.funnel_outlet_m < nearest {
        nearest = i.funnel_outlet_m;
    }
    nearest
}

/// Place the mast and read its heights and distances.
pub fn place(i: &VentMastInputs, lowerable: bool) -> VentMast {
    place_with_house(i, lowerable, None)
}

/// [`place`], with the x of the nearest opening of a forward house (issue #994): the distance to it is
/// read too, and the distance is met only if both are.
pub fn place_with_house(
    i: &VentMastInputs,
    lowerable: bool,
    forward_opening_x_m: Option<f64>,
) -> VentMast {
    let (exit, governs) = exit_above_keel(i);
    let working = working_area_top(i);
    // The nearest opening stands aft of the aft end of the cargo area by the least of the three
    // distances (each is measured from the cargo area).
    let mut nearest = i.entrance_m;
    if i.engine_room_vent_inlet_m < nearest {
        nearest = i.engine_room_vent_inlet_m;
    }
    if i.funnel_outlet_m < nearest {
        nearest = i.funnel_outlet_m;
    }
    let from_cargo_area = i.mast_x_m - i.cargo_area_aft_x_m;
    let to_openings = from_cargo_area + nearest;
    let to_accommodation = from_cargo_area + i.entrance_m;
    let (target, achieved) = if i.river_only {
        (ADN_DISTANCE_M, to_accommodation)
    } else {
        let cap = if i.beam_m < IGC_DISTANCE_CAP_M {
            i.beam_m
        } else {
            IGC_DISTANCE_CAP_M
        };
        (cap, to_openings)
    };
    let (above_deck, above_working) = heights(i, exit);
    let to_forward = forward_opening_x_m.map(|x| x - i.mast_x_m);
    let met = achieved >= target && to_forward.is_none_or(|d| d >= target);
    VentMast {
        river_only: i.river_only,
        x_m: i.mast_x_m,
        exit_above_keel_m: exit,
        exit_above_deck_m: above_deck,
        working_area_top_m: working,
        working_area_is_shield: i.shield_top_m.is_some(),
        exit_above_working_area_m: above_working,
        governs,
        distance_to_openings_m: to_openings,
        distance_to_accommodation_m: to_accommodation,
        distance_to_forward_house_m: to_forward,
        target_distance_m: target,
        distance_met: met,
        lowerable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> VentMastInputs {
        VentMastInputs {
            river_only: false,
            beam_m: 29.0,
            deck_m: 9.0,
            tank_top_m: 12.4,
            shield_top_m: None,
            dome_top_m: 14.0,
            stub_m: 1.0,
            mast_x_m: 100.0,
            cargo_area_aft_x_m: 20.0,
            entrance_m: 5.0,
            engine_room_vent_inlet_m: 5.0,
            funnel_outlet_m: 2.0,
        }
    }

    #[test]
    fn an_igc_height_is_the_term_when_the_raw_difference_falls_a_step_under_it() {
        // Issue #849: the raw difference may fall one rounding step under B/3 or 6 m; it may never be
        // more than 1e-9 m under, and the reported height is then exactly the term.
        let mut snapped = 0;
        for deck in [4.5, 6.0, 7.3, 9.0, 9.1, 11.7, 13.37] {
            for beam in [11.4, 14.0, 17.2, 19.9, 24.0, 29.0, 30.1] {
                for tank_top in [3.3, 8.7, 12.4, 16.1] {
                    let i = VentMastInputs {
                        beam_m: beam,
                        deck_m: deck,
                        tank_top_m: tank_top,
                        ..base()
                    };
                    let (exit, _) = exit_above_keel(&i);
                    let (raw_deck, raw_working) = raw_heights(&i, exit);
                    let (term_deck, term_working) =
                        (igc_above_deck(beam), IGC_ABOVE_WORKING_AREA_M);
                    assert!(term_deck - raw_deck <= 1e-9, "{deck} {beam}: {raw_deck}");
                    assert!(
                        term_working - raw_working <= 1e-9,
                        "{deck} {tank_top}: {raw_working}"
                    );
                    let m = place(&i, false);
                    assert_eq!(
                        m.exit_above_deck_m,
                        if raw_deck < term_deck {
                            term_deck
                        } else {
                            raw_deck
                        }
                    );
                    assert_eq!(
                        m.exit_above_working_area_m,
                        if raw_working < term_working {
                            term_working
                        } else {
                            raw_working
                        }
                    );
                    snapped +=
                        usize::from(raw_deck < term_deck) + usize::from(raw_working < term_working);
                }
            }
        }
        assert!(
            snapped > 0,
            "no input reached the rounding step: the test cannot fail"
        );
    }

    #[test]
    fn the_nearest_opening_is_the_smallest_distance() {
        let m = place(&base(), false);
        // 100 - 20 aft of the cargo area end, plus the funnel at 2 m.
        assert!((m.distance_to_openings_m - 82.0).abs() < 1e-12);
        assert!((m.distance_to_accommodation_m - 85.0).abs() < 1e-12);
    }
}
