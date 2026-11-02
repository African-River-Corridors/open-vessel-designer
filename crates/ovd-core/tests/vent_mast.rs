//! Seam: the cargo PRV vent mast of a gas hull (issue #822): its exit height by Service Area, its
//! place, the air drafts it adds to, and the four `vent.*` Measures.
//!
//! Label: sourced - IGC Code 8.2.10.3, 8.2.10.4 and 8.2.11.1 (resolution MSC.370(93) text) and ADN
//! 9.3.1.22.3, as `vent_mast.rs` quotes them; the expected heights are hand calculations of those
//! clauses on the inputs of each test.

use ovd_core::barge_capacity::{BargeCapacityInputs, BargeCapacityResult, compute};
use ovd_core::measured::Measured;
use ovd_core::rule_trace::{MeasureValue, measure_def};
use ovd_core::rules::MeasureRef;
use ovd_core::vent_mast::{Governs, VentMastInputs, place};

fn hull(loa: f64, beam: f64, depth: f64) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(loa, beam, depth);
    i.cargo_type = "LNG_ethane".into();
    i
}

/// The 140 x 29 m gas hull of the worked example, coastal and river.
fn example() -> BargeCapacityInputs {
    let mut i = hull(140.0, 29.0, 12.0);
    // Tanks of 6 m, as the example's: the tank top and the shield stay under the deck term.
    i.max_tank_od_m = 6.0;
    i
}

fn tank_top(r: &BargeCapacityResult) -> f64 {
    let b = r.gas_basis.as_ref().unwrap();
    b.inputs.double_bottom_height.unwrap_or(0.0)
        + b.inputs.tank_support_height
        + b.tanks.outer_diameter
}

fn pure() -> VentMastInputs {
    VentMastInputs {
        river_only: false,
        beam_m: 29.0,
        deck_m: 10.0,
        tank_top_m: 11.0,
        shield_top_m: None,
        dome_top_m: 12.0,
        stub_m: 1.0,
        mast_x_m: 100.0,
        cargo_area_aft_x_m: 20.0,
        entrance_m: 5.0,
        engine_room_vent_inlet_m: 4.0,
        funnel_outlet_m: 2.0,
    }
}

// --- Exit height ----------------------------------------------------------------------------

#[test]
fn coastal_and_river_the_deck_term_is_the_weather_deck_plus_b_over_3_on_the_example() {
    let r = compute(&example()).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    // B/3 = 9.667 m is more than 6 m.
    assert!(
        (v.exit_above_deck_m - 29.0 / 3.0).abs() < 1e-9,
        "{}",
        v.exit_above_deck_m
    );
    assert!((v.exit_above_keel_m - (12.0 + 29.0 / 3.0)).abs() < 1e-9);
    assert_eq!(v.governs, Governs::Deck);
    assert!(!v.river_only);
}

#[test]
fn a_narrow_hull_takes_6_m_above_the_deck() {
    // B/3 = 3.8 m, so the 6 m of IGC 8.2.10.3 is the greater; the working area is lower.
    let mut i = hull(110.0, 11.4, 6.0);
    i.max_tank_od_m = 3.0;
    let r = compute(&i).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    assert_eq!(v.governs, Governs::Deck);
    assert!((v.exit_above_deck_m - 6.0).abs() < 1e-9);
    assert!(v.working_area_top_m + 6.0 < v.exit_above_keel_m);
}

#[test]
fn the_weather_shield_term_governs_when_the_shield_stands_high() {
    // A shield 12 m over the tanks: its top plus 6 m is above the deck term (deck + B/3).
    let mut i = example();
    i.weather_shield_clearance_m = 12.0;
    let r = compute(&i).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    let shield_top = tank_top(&r) + 12.0;
    assert_eq!(v.governs, Governs::WorkingArea);
    assert!((v.working_area_top_m - shield_top).abs() < 1e-9);
    assert!(v.working_area_is_shield);
    assert!((v.exit_above_keel_m - (shield_top + 6.0)).abs() < 1e-9);
    assert!((v.exit_above_working_area_m - 6.0).abs() < 1e-9);
    assert!(v.exit_above_deck_m > 29.0 / 3.0);
}

#[test]
fn with_no_weather_shield_the_working_area_is_the_tank_top() {
    let mut i = example();
    i.weather_shield_thickness_mm = 0.0;
    let r = compute(&i).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    assert!(!v.working_area_is_shield);
    assert!((v.working_area_top_m - tank_top(&r)).abs() < 1e-9);
    // Not the dome top, which stands above the tank top.
    assert!(r.gas_basis.as_ref().unwrap().inputs.dome_height > 0.0);
    assert!((v.exit_above_working_area_m - (v.exit_above_keel_m - tank_top(&r))).abs() < 1e-9);
}

#[test]
fn river_only_the_deck_term_is_the_deck_plus_2_m() {
    // ADN 9.3.1.22.3: 2.00 m above the deck. Dome 12 m + stub 1 m is lower than 10 + 2 m? No: it is
    // higher, so shorten the dome here.
    let mut i = pure();
    i.river_only = true;
    i.dome_top_m = 10.5;
    let m = place(&i, false);
    assert_eq!(m.governs, Governs::Deck);
    assert!((m.exit_above_keel_m - 12.0).abs() < 1e-12);
    assert!((m.exit_above_deck_m - 2.0).abs() < 1e-12);
}

#[test]
fn river_only_the_dome_term_is_the_dome_top_plus_the_stub() {
    let mut i = pure();
    i.river_only = true;
    i.dome_top_m = 12.0;
    i.stub_m = 1.5;
    let m = place(&i, false);
    assert_eq!(m.governs, Governs::DomeTop);
    assert!((m.exit_above_keel_m - 13.5).abs() < 1e-12);
}

#[test]
fn river_only_on_a_hull_each_term_governs_once() {
    let exit = |stub: f64| {
        let mut i = example();
        i.river_only = true;
        i.vent_stub_m = stub;
        compute(&i).unwrap().vent_mast.unwrap()
    };
    // A stub of 100 m makes the dome term govern, and the exit less the stub is the dome top.
    let dome_top = exit(100.0).exit_above_keel_m - 100.0;
    assert!(dome_top < 14.0, "{dome_top}");
    // No stub: the dome top is under the 2.00 m term on this hull.
    let low = exit(0.0);
    assert_eq!(low.governs, Governs::Deck);
    assert!((low.exit_above_keel_m - 14.0).abs() < 1e-9);
    // A stub that lifts the dome term above it.
    let stub = 14.0 - dome_top + 0.75;
    let high = exit(stub);
    assert_eq!(high.governs, Governs::DomeTop);
    assert!((high.exit_above_keel_m - (dome_top + stub)).abs() < 1e-9);
}

#[test]
fn river_only_a_tall_hull_takes_the_deck_term() {
    let mut i = hull(140.0, 29.0, 30.0);
    i.river_only = true;
    let r = compute(&i).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    assert_eq!(v.governs, Governs::Deck);
    assert!((v.exit_above_deck_m - 2.0).abs() < 1e-9);
}

// --- Position ---------------------------------------------------------------------------------

#[test]
fn the_position_meets_the_target_on_the_example() {
    let r = compute(&example()).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    // B or 25 m, whichever is less: 25 m on a 29 m beam.
    assert_eq!(v.target_distance_m, 25.0);
    assert!(v.distance_to_openings_m >= 25.0);
    assert!(v.distance_met);
    // At the forward end of the cargo area: where the bow rake begins.
    assert!((v.x_m - (140.0 - r.bow_rake_length_m.unwrap())).abs() < 1e-9);
}

/// The example hull with a hold cofferdam of 0.6 m, an end space of 0.3 m and the three opening
/// distances 6, 3 and 8 m.
fn with_cofferdam(from_cargo_area: bool) -> BargeCapacityResult {
    let mut i = example();
    i.entrance_distance_m = 6.0;
    i.engine_room_vent_inlet_distance_m = 3.0;
    i.funnel_outlet_distance_m = 8.0;
    i.separation = Some(ovd_core::gas_separation::GasSeparationInputs {
        cofferdam_m: 0.3,
        hold_cofferdam_m: 0.6,
        setback_from_cargo_area: from_cargo_area,
        ..Default::default()
    });
    compute(&i).unwrap()
}

#[test]
fn the_distances_by_hand_from_the_cargo_area() {
    // By hand: the stern block's length L, then the hold cofferdam 0.6 m: the hold's aft bulkhead
    // (the aft face of the cargo area) stands at L + 0.6 from the stern. The end space of 0.3 m
    // follows it. The mast stands where the bow rake R begins: 140 - R. The nearest opening is the
    // engine room inlet, 3 m aft of the cargo area; the entrance is 6 m aft.
    let r = with_cofferdam(true);
    let v = r.vent_mast.as_ref().unwrap();
    let block = r.aft_arrangement.as_ref().unwrap().length_m;
    let rake = r.bow_rake_length_m.unwrap();
    let cargo_area_aft = block + 0.6;
    let mast = 140.0 - rake;
    assert!((v.x_m - mast).abs() < 1e-9);
    assert!((v.distance_to_openings_m - (mast - cargo_area_aft + 3.0)).abs() < 1e-9);
    assert!((v.distance_to_accommodation_m - (mast - cargo_area_aft + 6.0)).abs() < 1e-9);
}

#[test]
fn on_the_tank_end_basis_the_distance_runs_from_the_tank_end() {
    // The same hull; the setback is measured from the tank end, 0.3 m forward of the hold bulkhead,
    // so each distance is 0.3 m shorter than from the cargo area.
    let area = with_cofferdam(true);
    let tank = with_cofferdam(false);
    let (a, t) = (area.vent_mast.unwrap(), tank.vent_mast.unwrap());
    assert!((a.distance_to_openings_m - t.distance_to_openings_m - 0.3).abs() < 1e-9);
    assert!((a.distance_to_accommodation_m - t.distance_to_accommodation_m - 0.3).abs() < 1e-9);
    // By hand: block + cofferdam 0.6 + end space 0.3 is the tank end.
    let block = tank.aft_arrangement.as_ref().unwrap().length_m;
    let mast = 140.0 - tank.bow_rake_length_m.unwrap();
    assert!((t.distance_to_openings_m - (mast - (block + 0.9) + 3.0)).abs() < 1e-9);
}

#[test]
fn a_hull_too_short_gives_the_largest_distance_and_says_it_is_not_met() {
    // B = 14 m asked; a 40 m hull has a 10 m cargo zone and cannot give it.
    let mut i = hull(40.0, 14.0, 6.0);
    i.max_tank_od_m = 4.0;
    let r = compute(&i).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    assert_eq!(v.target_distance_m, 14.0);
    assert!(!v.distance_met, "{}", v.distance_to_openings_m);
    assert!(v.distance_to_openings_m < 14.0);
    // The largest distance: the forward end of the cargo area.
    assert!((v.x_m - (40.0 - r.bow_rake_length_m.unwrap())).abs() < 1e-9);
    assert!(
        r.warnings
            .iter()
            .any(|w| w.msg.as_ref().map(|m| m.id) == Some("warn-vent-mast-distance"))
    );
    // A longer hull of the same beam meets it.
    let mut j = hull(70.0, 14.0, 6.0);
    j.max_tank_od_m = 4.0;
    assert!(compute(&j).unwrap().vent_mast.unwrap().distance_met);
}

#[test]
fn river_only_the_target_is_6_m_from_the_accommodation() {
    let mut i = example();
    i.river_only = true;
    let r = compute(&i).unwrap();
    let v = r.vent_mast.as_ref().unwrap();
    assert_eq!(v.target_distance_m, 6.0);
    assert!(v.distance_met);
}

// --- Air drafts -------------------------------------------------------------------------------

#[test]
fn a_mast_that_stays_up_raises_all_four_air_drafts_to_its_exit() {
    let up = compute(&example()).unwrap();
    let mut j = example();
    j.vent_mast_lowerable = true;
    let down = compute(&j).unwrap();
    let exit = up.vent_mast.as_ref().unwrap().exit_above_keel_m;
    // With the mast lowered the river top is what it was before the mast.
    let top = down.air_draft_top_above_keel_m;
    assert!(exit > top, "the exit {exit} is above the old top {top}");
    let rise = exit - top;
    for (a, b, name) in [
        (up.air_draft_m, down.air_draft_m, "loaded"),
        (
            up.air_draft_light_m.unwrap(),
            down.air_draft_light_m.unwrap(),
            "light",
        ),
        (
            up.air_draft_ballast_m.unwrap(),
            down.air_draft_ballast_m.unwrap(),
            "ballast",
        ),
    ] {
        assert!((a - b - rise).abs() < 1e-9, "{name}: {a} {b} {rise}");
    }
    assert!((up.air_draft_top_above_keel_m - exit).abs() < 1e-9);
}

#[test]
fn a_lowerable_mast_leaves_the_river_air_drafts_but_the_sea_air_draft_keeps_it() {
    let mut j = example();
    j.vent_mast_lowerable = true;
    let down = compute(&j).unwrap();
    let v = down.vent_mast.as_ref().unwrap();
    assert!(v.lowerable);
    // The sea top is the exit when the exit is above the raised wheelhouse.
    let nav_top = down.navigating_top_above_keel_m.unwrap();
    assert!(nav_top >= v.exit_above_keel_m - 1e-9);
    let light = down.light_draft_m.unwrap();
    assert!((down.air_draft_navigating_light_m.unwrap() - (nav_top - light)).abs() < 1e-9);
    // The river top does not: it stands under the exit.
    assert!(down.air_draft_top_above_keel_m < v.exit_above_keel_m);
    // The warning is there; a mast that stays up has none.
    assert!(
        down.warnings
            .iter()
            .any(|w| w.msg.as_ref().map(|m| m.id) == Some("warn-vent-mast-lowerable"))
    );
    let up = compute(&example()).unwrap();
    assert!(
        !up.warnings
            .iter()
            .any(|w| w.msg.as_ref().map(|m| m.id) == Some("warn-vent-mast-lowerable"))
    );
    // The sea air draft is the same either way: the mast is never lowered at sea.
    assert!(
        (up.air_draft_navigating_light_m.unwrap() - down.air_draft_navigating_light_m.unwrap())
            .abs()
            < 1e-9
    );
}

#[test]
fn the_navigating_air_draft_rises_by_the_exit_above_its_top_today() {
    let mut j = example();
    j.vent_mast_lowerable = true;
    let r = compute(&j).unwrap();
    let exit = r.vent_mast.as_ref().unwrap().exit_above_keel_m;
    let light = r.light_draft_m.unwrap();
    // Today's sea top is the raised wheelhouse or the dome top, whichever is higher; the result's
    // sea top is the greater of that and the exit.
    assert!((r.navigating_top_above_keel_m.unwrap() - exit).abs() < 1e-9);
    assert!((r.air_draft_navigating_light_m.unwrap() - (exit - light)).abs() < 1e-9);
}

#[test]
fn a_typed_limit_just_above_the_light_air_draft_passes_lowerable_and_fails_without() {
    let mut j = example();
    j.vent_mast_lowerable = true;
    let down = compute(&j).unwrap();
    let limit = down.air_draft_light_m.unwrap() + 0.05;
    let mut lowered = example();
    lowered.vent_mast_lowerable = true;
    lowered.ballast_capacity_t = 0.0;
    lowered.air_draft_limit_m = Some(limit);
    let ok = compute(&lowered).unwrap();
    let check = |r: &BargeCapacityResult| {
        r.checks
            .iter()
            .find(|c| c.name == "air_draft_within_limit")
            .map(|c| c.ok)
            .unwrap()
    };
    assert!(check(&ok));
    let mut up = example();
    up.air_draft_limit_m = Some(limit);
    assert!(!check(&compute(&up).unwrap()));
}

// --- Measures ---------------------------------------------------------------------------------

#[test]
fn the_four_measures_are_registered_and_return_what_the_result_holds() {
    let r = compute(&example()).unwrap();
    let v = r.vent_mast.clone().unwrap();
    let m = Measured {
        vent_mast: Some(v.clone()),
        ..Measured::default()
    };
    for (id, want) in [
        ("vent.exit_above_deck", v.exit_above_deck_m),
        ("vent.exit_above_working_area", v.exit_above_working_area_m),
        ("vent.distance_to_openings", v.distance_to_openings_m),
        (
            "vent.distance_to_accommodation",
            v.distance_to_accommodation_m,
        ),
    ] {
        let def = measure_def(id).unwrap_or_else(|| panic!("{id} is not registered"));
        assert_eq!(def.unit, "m");
        let mr = MeasureRef {
            id: id.to_string(),
            args: Vec::new(),
        };
        assert!(m.has(&mr), "{id}");
        match m.value(&mr, 0).unwrap() {
            MeasureValue::Number(x) => assert_eq!(x, want, "{id}"),
            other => panic!("{other:?}"),
        }
        // A hull with no vent mast has none of them.
        assert!(!Measured::default().has(&mr), "{id}");
    }
}

#[test]
fn a_hull_that_is_not_gas_has_no_vent_mast_and_its_tops_do_not_move() {
    let mut i = BargeCapacityInputs::new(76.5, 11.4, 3.5);
    i.cargo_type = "bauxite".into();
    let r = compute(&i).unwrap();
    assert!(r.vent_mast.is_none());
    assert!(r.air_draft_top_above_keel_m <= 3.5 + i.coaming_height + 5.0);
}
