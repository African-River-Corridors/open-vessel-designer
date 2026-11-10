//! Seam: the public `ovd_core::collision_bulkhead` API and the stern block's check against the plane
//! (issue #994, the collision bulkhead rework; ES-TRIN 2025/1 Article 3.03).
//!
//! Label: property. Each test states a rule: the forward perpendicular of a raked bow is where the
//! maximum draught meets the stem, the default distance is 0.04 L, and a house whose front stands ahead
//! of the plane fails the check while one at or aft of it passes. No printed result is asserted.

use ovd_core::aft_arrangement::{
    AftArrangementInputs, ArrangementStyle, ForwardHouseSite, compute, forward_house_sizes as fh,
};
use ovd_core::collision_bulkhead::{default_distance_m, forward_perpendicular_x_m};

const EPS: f64 = 1e-9;

#[test]
fn the_forward_perpendicular_is_where_the_maximum_draught_meets_the_raked_stem() {
    // The keel ends `rake` aft of the bow and rises `rise` to the deck at the bow: at draught t the
    // hull meets the water rake x (1 - t / rise) aft of the bow.
    let (loa, rake, rise) = (110.0, 9.6, 6.0);
    let x = forward_perpendicular_x_m(loa, rake, rise, 5.7);
    assert!((x - (loa - rake * (1.0 - 5.7 / 6.0))).abs() < EPS, "{x}");
    assert!(x < loa && x > loa - rake);
}

#[test]
fn a_bow_without_rake_or_a_draught_to_the_deck_puts_the_perpendicular_at_the_bow() {
    assert_eq!(forward_perpendicular_x_m(110.0, 0.0, 6.0, 3.0), 110.0);
    assert_eq!(forward_perpendicular_x_m(110.0, 9.6, 6.0, 6.0), 110.0);
    assert_eq!(forward_perpendicular_x_m(110.0, 9.6, 6.0, 7.0), 110.0);
}

#[test]
fn a_deeper_maximum_draught_moves_the_perpendicular_forward() {
    let at = |t: f64| forward_perpendicular_x_m(110.0, 9.6, 6.0, t);
    assert!(at(2.0) < at(4.0));
    assert!(at(4.0) < at(5.7));
}

#[test]
fn the_default_distance_is_0_04_l() {
    for l in [70.0, 110.0, 140.0] {
        assert!((default_distance_m(l) - 0.04 * l).abs() < EPS);
    }
}

fn forward_house(x_fwd: f64, plane: f64) -> AftArrangementInputs {
    let mut i = AftArrangementInputs::new(11.4, 6.0);
    i.arrangement_style = ArrangementStyle::WheelhouseForward;
    i.forward_house_site = Some(ForwardHouseSite {
        x_aft_m: x_fwd - fh::LENGTH_M,
        x_fwd_m: x_fwd,
        cargo_length_lost_m: 0.0,
    });
    i.collision_bulkhead_x_m = Some(plane);
    i
}

fn check(r: &ovd_core::aft_arrangement::AftArrangementResult) -> bool {
    r.checks
        .iter()
        .find(|c| c.name == "house aft of collision bulkhead")
        .map(|c| c.ok)
        .expect("the stern block checks the house against the plane")
}

#[test]
fn a_house_ahead_of_the_plane_fails_the_check() {
    let r = compute(&forward_house(108.0, 105.12)).unwrap();
    assert!(!check(&r));
    assert_eq!(r.house_aft_of_collision_bulkhead, Some(false));
}

#[test]
fn a_house_with_its_front_on_or_aft_of_the_plane_passes() {
    for x_fwd in [105.12, 104.0] {
        let r = compute(&forward_house(x_fwd, 105.12)).unwrap();
        assert!(check(&r), "{x_fwd}");
        assert_eq!(r.house_aft_of_collision_bulkhead, Some(true));
    }
}

#[test]
fn a_deckhouse_stern_passes_when_it_knows_the_plane_and_has_no_value_without_one() {
    let mut i = AftArrangementInputs::new(11.4, 6.0);
    i.arrangement_style = ArrangementStyle::Deckhouse;
    i.collision_bulkhead_x_m = Some(105.12);
    let r = compute(&i).unwrap();
    assert!(check(&r));
    assert_eq!(r.house_aft_of_collision_bulkhead, Some(true));
    let bare = compute(&AftArrangementInputs::new(11.4, 6.0)).unwrap();
    assert_eq!(bare.house_aft_of_collision_bulkhead, None);
}
