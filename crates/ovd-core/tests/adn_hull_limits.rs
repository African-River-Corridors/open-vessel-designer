//! Seam: the hull limits of ADN 9.3.1.11.2 (a) size a gas hull beside the IGC 2.4.1.2 tank distances
//! (issue #850).
//!
//! Label: sourced - ADN 2025 (ECE/TRANS/351) 9.3.1.11.2 (a), printed p. 510: in the cargo area the
//! hull is a double hull with a double bottom; "the internal distance between the sideplatings of the
//! vessel and the longitudinal bulkheads shall not be less than 0.80 m, the height of the double
//! bottom shall be not less than 0.60 m". IGC Code (MSC.370(93)) 2.4.1.1 and 2.4.1.2: the tank side
//! distance is d(Vc), the bottom distance is the greater of B/15 (at most 2 m) and d(Vc), each less the
//! insulation (2.4.2: measured to the steel). The engine's wing tank is the side clearance less the gap
//! between the wing tank and the tank (0.6 m by default), so the wing tank is the width ADN limits.
//!
//! Each test works its expected value by hand from those texts. Where each term governs:
//!
//! - Side, ADN governs: 90 x 14.8 m, LNG, default insulation 0.350 m and wing gap 0.6 m. d(Vc) is
//!   below 1.0 m for every tank of that hull (Vc under 5,000 m3), so the IGC clearance is under
//!   0.65 m. ADN: 0.80 + 0.6 = 1.40 m clear side gap, a wing of 0.80 m.
//! - Side, IGC governs: no insulation, no wing gap. d(Vc) is above 0.8 m for Vc above 1,000 m3
//!   (0.75 + Vc x 0.2 / 4,000), so the clearance is d(Vc), above the 0.80 m of ADN.
//! - Bottom, ADN governs: 90 x 14.8 m, support 0.5 m. IGC: B/15 = 0.9867 m, less 0.350 m of
//!   insulation, is 0.6367 m clear, a double bottom of 0.1367 m. ADN: 0.60 m, so the clear height
//!   is 0.60 + 0.5 = 1.10 m.
//! - Bottom, IGC governs: B = 24 m, no insulation, support 0.5 m. B/15 = 1.6 m, above d(Vc) of the
//!   tanks of that hull, so the double bottom is 1.6 - 0.5 = 1.1 m, above the 0.60 m of ADN.

use ovd_core::barge_capacity::{BargeCapacityInputs, BargeCapacityResult, compute};

fn run(loa: f64, beam: f64, f: impl FnOnce(&mut BargeCapacityInputs)) -> BargeCapacityResult {
    let mut i = BargeCapacityInputs::new(loa, beam, 6.5);
    i.cargo_type = "LNG".to_string();
    f(&mut i);
    compute(&i).unwrap()
}

/// d(Vc) of IGC 2.4.1.1, worked from the printed table (not the engine's function).
fn d_of(vc: f64) -> f64 {
    if vc <= 1000.0 {
        0.8
    } else if vc < 5000.0 {
        0.75 + vc * 0.2 / 4000.0
    } else if vc < 30000.0 {
        0.8 + vc / 25000.0
    } else {
        2.0
    }
}

#[test]
fn adn_sizes_the_wing_when_igc_asks_for_less() {
    let plain = run(90.0, 14.8, |_| {});
    let adn = run(90.0, 14.8, |i| i.double_side_min_m = 0.8);
    let igc = plain.side_clearance_m.unwrap();
    assert!(igc < 0.8 + 0.6, "the IGC clearance {igc} m is under the ADN one");
    // 0.80 + 0.6 gap = 1.40 m of clear side gap, a wing of exactly 0.80 m.
    assert!((adn.side_clearance_m.unwrap() - 1.4).abs() < 1e-12);
    let t = adn.tank_rules.as_ref().unwrap();
    assert_eq!(t.double_side_width_m, 0.8);
    assert!((t.side_gap_m - 1.4).abs() < 1e-12);
    // The wider side takes tank diameter: the tank is narrower than without the limit.
    assert!(adn.tank_outer_diameter_m.unwrap() <= plain.tank_outer_diameter_m.unwrap() + 1e-9);
}

#[test]
fn igc_sizes_the_clearance_when_it_asks_for_more_than_adn() {
    // No insulation and no wing gap: the clearance is d(Vc), above 0.80 m for Vc above 1,000 m3.
    let r = run(140.0, 24.0, |i| {
        i.tank_design.insulation_thickness_mm = 0.0;
        i.wing_tank_gap_m = 0.0;
        i.double_side_min_m = 0.8;
    });
    let t = r.tank_rules.as_ref().unwrap();
    let want = d_of(t.vc_m3);
    assert!(want > 0.8, "Vc {} gives d {want}", t.vc_m3);
    assert!((t.side_gap_m - want).abs() < 1e-6, "{} vs {want}", t.side_gap_m);
    // The width is what is left: the clearance with no gap, above the ADN limit.
    assert!((t.double_side_width_m - t.side_gap_m).abs() < 1e-12);
    assert!(t.double_side_width_m > 0.8);
}

#[test]
fn adn_sizes_the_double_bottom_when_igc_asks_for_less() {
    let plain = run(90.0, 14.8, |_| {});
    let adn = run(90.0, 14.8, |i| i.double_bottom_min_m = 0.6);
    // IGC: max(14.8 / 15, d) - 0.350 clear; the double bottom is that less the 0.5 m support.
    let pt = plain.tank_rules.as_ref().unwrap();
    assert!(pt.double_bottom_height_m < 0.6, "{}", pt.double_bottom_height_m);
    let t = adn.tank_rules.as_ref().unwrap();
    assert_eq!(t.double_bottom_height_m, 0.6);
    assert!((t.bottom_gap_m - 1.1).abs() < 1e-12);
    assert!((adn.bottom_clearance_m.unwrap() - 1.1).abs() < 1e-12);
}

#[test]
fn igc_sizes_the_double_bottom_when_it_asks_for_more_than_adn() {
    let r = run(140.0, 24.0, |i| {
        i.tank_design.insulation_thickness_mm = 0.0;
        i.double_bottom_min_m = 0.6;
    });
    let t = r.tank_rules.as_ref().unwrap();
    // The greater of B/15 = 1.6 m (at most 2 m) and d(Vc), with no insulation.
    let want = (24.0_f64 / 15.0).min(2.0).max(d_of(t.vc_m3));
    assert!((t.bottom_gap_m - want).abs() < 1e-6, "{} vs {want}", t.bottom_gap_m);
    assert!((t.double_bottom_height_m - (want - 0.5)).abs() < 1e-6);
    assert!(t.double_bottom_height_m > 0.6);
}

#[test]
fn no_limit_leaves_the_igc_sizing_as_it_was() {
    for (loa, beam) in [(90.0, 14.8), (140.0, 24.0)] {
        let a = run(loa, beam, |_| {});
        let b = run(loa, beam, |i| {
            i.double_side_min_m = 0.0;
            i.double_bottom_min_m = 0.0;
        });
        assert_eq!(a.side_clearance_m, b.side_clearance_m);
        assert_eq!(a.bottom_clearance_m, b.bottom_clearance_m);
        assert_eq!(a.tank_rules, b.tank_rules);
    }
}
