//! Seam: the openings of the forward house, and the vent mast between the two houses (issue #994).
//!
//! Label: sourced - IGC Code (MSC.370(93)) 3.2.4.1: entrances, air inlets and openings must not face the
//! cargo area, and on the sides stand at least 4 % of L (3 to 5 m) from the end facing it; 3.2.4.2,
//! read strictly (no door on a face toward the cargo area); 8.2.11.1: the vent exit at least B or 25 m,
//! whichever is less, from the nearest opening. The sums are worked by hand in each test.

use ovd_core::deckhouse_openings::{
    Face, Frame, Kind, Opening, Space, forward_house_nearest_x_m, place_forward,
    place_forward_typed, zone_length_m,
};
use ovd_core::vent_mast::{VentMastInputs, balanced_x_m, nearest_aft_m, place_with_house};

/// A forward house 8 m long, its aft end at 85 m, with a 3.6 m wheelhouse on it, on a ship of L = 105 m.
fn house() -> Frame {
    Frame {
        x_aft_m: 85.0,
        x_fwd_m: 93.0,
        width_m: 9.8,
        base_m: 6.0,
        tier_height_m: 2.6,
        tiers: 1,
        wheelhouse: Some((89.4, 93.0, 6.4, 8.6)),
        l_icll_m: 105.0,
    }
}

#[test]
fn the_aft_end_of_a_forward_house_is_the_end_that_faces_the_cargo() {
    let p = place_forward(&house());
    // The zone: 4 % of 105 m = 4.2 m, between 3 and 5 m.
    assert!((zone_length_m(105.0) - 4.2).abs() < 1e-12);
    for o in &p.openings {
        assert!(!o.faces_cargo_area, "{} faces the cargo area", o.opening.id);
        assert_ne!(o.opening.face, Face::Aft, "{}", o.opening.id);
    }
    // The wheelhouse side doors stand at the middle of the wheelhouse, x = 91.2 m: 6.2 m from the aft
    // end, past the 4.2 m zone.
    let door = p
        .openings
        .iter()
        .find(|o| o.opening.id == "wheelhouse door, port")
        .unwrap();
    assert!((door.distance_from_front_m - 6.2).abs() < 1e-9);
    assert!(!door.in_zone);
    // The fore face is the one away from the cargo: its openings are never in the zone.
    assert!(
        p.openings
            .iter()
            .filter(|o| o.opening.face == Face::Front)
            .all(|o| !o.in_zone)
    );
}

#[test]
fn a_door_on_the_aft_face_of_a_forward_house_faces_the_cargo_area() {
    let door = Opening {
        id: "wheelhouse door, aft".to_string(),
        kind: Kind::Entrance,
        space: Space::Wheelhouse,
        face: Face::Aft,
        x_m: 85.0,
        y_m: 0.0,
        z_m: 9.6,
        openable: true,
        gas_tight: true,
        tier: 0,
    };
    let p = place_forward_typed(&house(), &[door]);
    let o = &p.openings[0];
    assert!(o.faces_cargo_area && o.in_zone);
    assert_eq!(o.distance_from_front_m, 0.0);
}

#[test]
fn a_side_door_inside_the_zone_of_a_forward_house_is_in_the_zone() {
    // 3 m from the aft end (88 m) is inside the 4.2 m zone.
    let door = Opening {
        id: "side door".to_string(),
        kind: Kind::Entrance,
        space: Space::Service,
        face: Face::Side,
        x_m: 88.0,
        y_m: 4.9,
        z_m: 7.0,
        openable: true,
        gas_tight: false,
        tier: 1,
    };
    let o = &place_forward_typed(&house(), &[door]).openings[0];
    assert!((o.distance_from_front_m - 3.0).abs() < 1e-9);
    assert!(o.in_zone && !o.faces_cargo_area);
}

#[test]
fn the_nearest_opening_of_the_forward_house_is_the_wheelhouse_side() {
    // The side doors and windows at x = 91.2 m are further aft than the fore face at 93 m.
    assert!((forward_house_nearest_x_m(&house()) - 91.2).abs() < 1e-9);
}

fn mast_inputs(entrance: f64) -> VentMastInputs {
    VentMastInputs {
        river_only: false,
        beam_m: 29.0,
        deck_m: 9.0,
        tank_top_m: 12.4,
        shield_top_m: None,
        dome_top_m: 14.0,
        stub_m: 1.0,
        mast_x_m: 0.0,
        cargo_area_aft_x_m: 20.0,
        entrance_m: entrance,
        engine_room_vent_inlet_m: entrance,
        funnel_outlet_m: entrance,
    }
}

#[test]
fn the_mast_stands_between_the_houses_at_equal_distances_when_the_cargo_area_is_long_enough() {
    // Cargo area 20 m to 85 m, the nearest opening aft 5 m from it, the nearest forward at 91.2 m:
    // x = (91.2 + 20 - 5) / 2 = 53.1 m; 53.1 - 20 + 5 = 38.1 m aft, 91.2 - 53.1 = 38.1 m forward.
    let mut i = mast_inputs(5.0);
    let x = balanced_x_m(20.0, 85.0, nearest_aft_m(&i), 91.2);
    assert!((x - 53.1).abs() < 1e-9, "{x}");
    i.mast_x_m = x;
    let v = place_with_house(&i, false, Some(91.2));
    assert!((v.distance_to_openings_m - 38.1).abs() < 1e-9);
    assert!((v.distance_to_forward_house_m.unwrap() - 38.1).abs() < 1e-9);
    // B = 29 m, so the distance asked is 25 m: met on both sides.
    assert_eq!(v.target_distance_m, 25.0);
    assert!(v.distance_met);
}

#[test]
fn the_mast_at_the_house_wall_fails_the_distance_the_balanced_one_meets() {
    let mut i = mast_inputs(5.0);
    i.mast_x_m = 85.0;
    let at_wall = place_with_house(&i, false, Some(91.2));
    assert!((at_wall.distance_to_forward_house_m.unwrap() - 6.2).abs() < 1e-9);
    assert!(!at_wall.distance_met);
    // The same mast with no forward house is judged on the openings aft alone, as before.
    assert!(place_with_house(&i, false, None).distance_met);
}

#[test]
fn a_cargo_area_too_short_for_both_distances_leaves_them_unmet() {
    // Cargo area 20 m to 45 m, the forward opening at 50 m: x = (50 + 20 - 5) / 2 = 32.5 m; 17.5 m each
    // way, under the 25 m asked.
    let mut i = mast_inputs(5.0);
    i.mast_x_m = balanced_x_m(20.0, 45.0, nearest_aft_m(&i), 50.0);
    let v = place_with_house(&i, false, Some(50.0));
    assert!(
        !v.distance_met,
        "{} {:?}",
        v.distance_to_openings_m, v.distance_to_forward_house_m
    );
}
