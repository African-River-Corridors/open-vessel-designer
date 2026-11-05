//! The openings of a deckhouse stern (issue #912, spec 0011 D17): the IGC 3.2.4.1 zone from the
//! deckhouse front, the default layout and a typed list.
//!
//! Label: sourced - IGC Code, MSC.370(93), 3.2.4.1 (at least 4 % of the length (L) of the ship, not
//! less than 3 m, not more than 5 m, from the end of the deckhouse that faces the cargo area) and
//! 3.2.4.2 (wheelhouse doors may stand in the zone if the wheelhouse can be made gas-tight rapidly);
//! L by ICLL 1966/88 Regulation 3(1)(a) (MSC.143(77)). Each value is worked by hand beside its test.

use ovd_core::deckhouse_openings::*;

/// A deckhouse of `length` m from the stern, 12 m wide, on a 6 m deck, two tiers of 2.6 m; a
/// wheelhouse over its aft part, from 1.0 m to 5.0 m forward of the stern.
fn frame(length: f64, l_icll: f64) -> Frame {
    Frame {
        x_aft_m: 0.0,
        x_fwd_m: length,
        width_m: 12.0,
        base_m: 6.0,
        tier_height_m: 2.6,
        tiers: 2,
        wheelhouse: Some((1.0, 5.0, 3.0, 11.2)),
        l_icll_m: l_icll,
    }
}

fn typed_side(x: f64) -> Opening {
    Opening {
        id: "side door".into(),
        kind: Kind::Entrance,
        space: Space::Accommodation,
        face: Face::Side,
        x_m: x,
        y_m: 6.0,
        z_m: 7.0,
        openable: true,
        gas_tight: false,
        tier: 1,
    }
}

#[test]
fn the_zone_is_4_percent_of_l_between_3_and_5_m() {
    // L = 140 m: 4 % is 5.6 m, over the 5 m the zone need not exceed: 5.0 m.
    assert_eq!(zone_length_m(140.0), 5.0);
    // L = 70 m: 4 % is 2.8 m, under the 3 m the zone is not less than: 3.0 m.
    assert_eq!(zone_length_m(70.0), 3.0);
    // In between: L = 100 m is 4.0 m. At the two ends: 75 m is exactly 3.0 m, 125 m exactly 5.0 m.
    assert!((zone_length_m(100.0) - 4.0).abs() < 1e-12);
    assert!((zone_length_m(75.0) - 3.0).abs() < 1e-12);
    assert!((zone_length_m(125.0) - 5.0).abs() < 1e-12);
    // A length that is not a number takes the least zone.
    assert_eq!(zone_length_m(f64::NAN), 3.0);
}

fn in_zone_at(l: f64, distance: f64) -> bool {
    let f = frame(20.0, l);
    // A typed side opening `distance` m aft of the deckhouse front (x_fwd = 20).
    let typed = [typed_side(20.0 - distance)];
    let p = place(&f, Some(&typed));
    assert!((p.openings[0].distance_from_front_m - distance).abs() < 1e-9);
    p.openings[0].in_zone
}

#[test]
fn a_side_opening_is_out_of_the_zone_at_exactly_its_length_for_l_140() {
    // Zone 5.0 m: 4.99 m from the front is inside it, 5.00 m is outside.
    assert!(in_zone_at(140.0, 4.99));
    assert!(!in_zone_at(140.0, 5.00));
}

#[test]
fn a_side_opening_is_out_of_the_zone_at_exactly_its_length_for_l_70() {
    // Zone 3.0 m: 2.99 m from the front is inside it, 3.00 m is outside.
    assert!(in_zone_at(70.0, 2.99));
    assert!(!in_zone_at(70.0, 3.00));
}

#[test]
fn an_opening_on_the_front_faces_the_cargo_area_and_is_in_the_zone() {
    let mut door = typed_side(20.0);
    door.face = Face::Front;
    door.y_m = 0.0;
    let p = place(&frame(20.0, 140.0), Some(&[door]));
    let o = &p.openings[0];
    assert!(o.faces_cargo_area && o.in_zone);
    assert_eq!(o.distance_from_front_m, 0.0);
    assert!(p.typed);
}

#[test]
fn an_opening_on_the_aft_end_needs_no_distance() {
    // IGC 3.2.4.1: on the end bulkhead not facing the cargo area, no distance is asked.
    let mut door = typed_side(0.0);
    door.face = Face::Aft;
    let p = place(&frame(20.0, 140.0), Some(&[door]));
    let o = &p.openings[0];
    assert!(!o.faces_cargo_area && !o.in_zone);
    assert_eq!(o.distance_from_front_m, 20.0);
}

#[test]
fn the_default_layout_puts_nothing_on_the_front_but_a_fixed_wheelhouse_window() {
    let p = place(&frame(20.0, 140.0), None);
    assert!(!p.typed);
    for o in &p.openings {
        if o.opening.face == Face::Front {
            // The fixed wheelhouse window: it faces the cargo area, and does not open (3.2.4.2).
            assert_eq!(o.opening.space, Space::Wheelhouse);
            assert_eq!(o.opening.kind, Kind::Window);
            assert!(!o.opening.openable, "{}", o.opening.id);
        }
    }
    // No entrance, air inlet or air intake faces the cargo area.
    assert!(
        p.openings
            .iter()
            .filter(|o| o.opening.kind != Kind::Window)
            .all(|o| !o.faces_cargo_area)
    );
}

#[test]
fn a_deckhouse_shorter_than_the_zone_puts_no_accommodation_door_on_a_side() {
    // A 2 m deckhouse, under the 3 m least zone of a 70 m ship: no side of it is far enough from the
    // front, so every accommodation entrance is on the aft end, one for each tier.
    let p = place(&frame(2.0, 70.0), None);
    assert_eq!(p.zone_m, 3.0);
    let doors: Vec<_> = p
        .openings
        .iter()
        .filter(|o| o.opening.space == Space::Accommodation && o.opening.kind == Kind::Entrance)
        .collect();
    assert_eq!(doors.len(), 2);
    assert!(doors.iter().all(|o| o.opening.face == Face::Aft));
    assert!(doors.iter().all(|o| !o.in_zone && !o.faces_cargo_area));
    assert!(
        p.openings
            .iter()
            .filter(|o| o.opening.space != Space::Wheelhouse)
            .all(|o| o.opening.face == Face::Aft)
    );
}

#[test]
fn the_wheelhouse_side_doors_may_stand_in_the_zone_when_marked_gas_tight() {
    // The wheelhouse middle is 3.0 m from the stern: in a 6 m deckhouse that is 3.0 m from the front,
    // inside the 5.0 m zone of a 140 m ship, and outside the 3.0 m zone of a 70 m ship, when 3.0 m is
    // not under it.
    let p = place(&frame(6.0, 140.0), None);
    let doors: Vec<_> = p
        .openings
        .iter()
        .filter(|o| o.opening.space == Space::Wheelhouse && o.opening.kind == Kind::Entrance)
        .collect();
    assert_eq!(doors.len(), 2);
    for d in &doors {
        assert_eq!(d.opening.face, Face::Side);
        assert!((d.distance_from_front_m - 3.0).abs() < 1e-9);
        assert!(d.in_zone, "3.0 m from the front is inside a 5.0 m zone");
        assert!(d.opening.gas_tight, "IGC 3.2.4.2: designed for rapid gas-tightening");
    }
    let q = place(&frame(6.0, 70.0), None);
    assert!(q
        .openings
        .iter()
        .filter(|o| o.opening.space == Space::Wheelhouse && o.opening.kind == Kind::Entrance)
        .all(|o| !o.in_zone));
}

#[test]
fn a_typed_list_stands_in_for_the_whole_default_layout() {
    let typed = [typed_side(10.0)];
    let p = place(&frame(20.0, 140.0), Some(&typed));
    assert!(p.typed);
    assert_eq!(p.openings.len(), 1);
    // An empty typed list is a deckhouse with no opening.
    let none = place(&frame(20.0, 140.0), Some(&[]));
    assert!(none.typed && none.openings.is_empty());
}

#[test]
fn the_default_layout_has_an_accommodation_entrance_for_each_tier_and_a_position_for_each_opening() {
    let p = place(&frame(20.0, 140.0), None);
    let tiers: Vec<i64> = p
        .openings
        .iter()
        .filter(|o| o.opening.space == Space::Accommodation && o.opening.kind == Kind::Entrance)
        .map(|o| o.opening.tier)
        .collect();
    assert_eq!(tiers, vec![1, 2]);
    // Hand values: the tier 2 entrance is 1.0 m above the deck of tier 2 (6.0 + 2.6), at z = 9.6 m.
    let t2 = p
        .openings
        .iter()
        .find(|o| o.opening.tier == 2 && o.opening.kind == Kind::Entrance)
        .unwrap();
    assert!((t2.opening.z_m - 9.6).abs() < 1e-9);
    assert!(p.openings.iter().all(|o| o.opening.x_m.is_finite()
        && o.opening.y_m.is_finite()
        && o.opening.z_m.is_finite()));
}

#[test]
fn the_measures_give_each_opening_its_distance_its_face_and_its_zone() {
    use ovd_core::measured::Measured;
    use ovd_core::rule_trace::{MeasureValue, ObjectType};
    use ovd_core::rules::MeasureRef;
    // L = 140 m, a 20 m deckhouse: two typed side doors, 4.99 and 5.00 m from the front.
    let typed = [
        Opening {
            id: "door near".into(),
            ..typed_side(20.0 - 4.99)
        },
        Opening {
            id: "door far".into(),
            ..typed_side(20.0 - 5.0)
        },
    ];
    let m = Measured {
        openings: Some(place(&frame(20.0, 140.0), Some(&typed))),
        ..Default::default()
    };
    assert_eq!(
        m.objects(ObjectType::Opening, "Option"),
        vec!["door near".to_string(), "door far".to_string()]
    );
    let r = |id: &str| MeasureRef {
        id: id.to_string(),
        args: Vec::new(),
    };
    let num = |v: Result<MeasureValue, String>| match v.unwrap() {
        MeasureValue::Number(x) => x,
        other => panic!("{other:?}"),
    };
    let flag = |v: Result<MeasureValue, String>| match v.unwrap() {
        MeasureValue::Bool(b) => b,
        other => panic!("{other:?}"),
    };
    assert!((num(m.value(&r("opening.distance_from_front"), 0)) - 4.99).abs() < 1e-9);
    assert!(flag(m.value(&r("opening.in_zone"), 0)));
    assert!(!flag(m.value(&r("opening.in_zone"), 1)));
    assert!(!flag(m.value(&r("opening.faces_cargo_area"), 1)));
    assert_eq!(num(m.value(&r("deckhouse.opening_zone"), 0)), 5.0);
    assert_eq!(num(m.value(&r("deckhouse.icll_length"), 0)), 140.0);
    assert!(m.has(&r("opening.in_zone")));
    // No deckhouse: no openings, so the Measures are not there.
    let none = Measured::default();
    assert!(!none.has(&r("opening.in_zone")));
    assert!(none.objects(ObjectType::Opening, "Option").is_empty());
}
