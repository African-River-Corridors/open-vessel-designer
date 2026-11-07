//! Seam: the sightline from the eye of a forward wheelhouse (issue #994, design D3).
//!
//! Label: property. With the eye forward of the cargo, what stands behind the eye gives no blind
//! distance; what stands ahead of it (the bow, a mooring winch) still does.

use ovd_core::wheelhouse::{Obstruction, Waterline, assess, blind_distance};

fn bow_only(loa: f64, depth: f64) -> Vec<Obstruction> {
    vec![Obstruction {
        label: "bow",
        x_m: loa,
        top_above_keel_m: depth,
    }]
}

fn with_cargo(loa: f64, depth: f64, cargo_x: f64, cargo_top: f64) -> Vec<Obstruction> {
    let mut v = vec![Obstruction {
        label: "cargo top",
        x_m: cargo_x,
        top_above_keel_m: cargo_top,
    }];
    v.extend(bow_only(loa, depth));
    v
}

#[test]
fn cargo_behind_the_forward_eye_gives_no_blind_distance() {
    let (loa, depth) = (140.0, 12.0);
    let w = Waterline::even(5.0, loa);
    let eye = (loa - 3.0, depth + 2.6 + 1.5);
    let only_bow = blind_distance(eye.0, eye.1, &bow_only(loa, depth), loa, &w);
    // Tall cargo, far behind the eye: the same blind distance, however tall.
    for top in [14.0, 30.0, 200.0] {
        let b = blind_distance(eye.0, eye.1, &with_cargo(loa, depth, 100.0, top), loa, &w);
        assert_eq!(b, only_bow, "cargo top {top}");
    }
    assert!(only_bow < 20.0, "{only_bow}");
}

#[test]
fn no_lift_is_needed_when_only_the_bow_is_ahead() {
    let (loa, depth) = (140.0, 12.0);
    let w = Waterline::even(5.0, loa);
    let eye_floor = depth + 2.6;
    let s = assess(
        loa - 3.0,
        eye_floor + 1.5,
        1.5,
        0.0,
        &with_cargo(loa, depth, 100.0, 40.0),
        loa,
        &w,
        250.0,
    );
    assert_eq!(s.lift_needed_m, 0.0);
    assert!(s.enough);
}

#[test]
fn an_obstruction_ahead_of_the_eye_still_counts() {
    let (loa, depth) = (140.0, 12.0);
    let w = Waterline::even(5.0, loa);
    let eye = (loa - 8.0, depth + 2.6 + 1.5);
    let bow = blind_distance(eye.0, eye.1, &bow_only(loa, depth), loa, &w);
    let mut with_winch = bow_only(loa, depth);
    with_winch.push(Obstruction {
        label: "mooring winch",
        x_m: loa - 2.0,
        top_above_keel_m: depth + 3.0,
    });
    let winch = blind_distance(eye.0, eye.1, &with_winch, loa, &w);
    assert!(winch > bow, "{winch} vs {bow}");
}
