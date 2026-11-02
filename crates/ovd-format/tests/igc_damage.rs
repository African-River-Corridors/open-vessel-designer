//! Seam: `ovd_format::damage::igc_damage_data`, the IGC damage extents and standard of damage read from the
//! `igc-2g` Rule Set's tables (spec 0012, issue #757).
//!
//! Label: sourced - IGC Code (MSC.370(93) as amended) 2.3.1, annex p. 20: side damage (1/3) L^(2/3) or 14.5 m
//! long and B/5 or 11.5 m inboard; bottom damage (1/3) L^(2/3) or 14.5 m long in both zones, B/6 or 10 m wide
//! for 0.3 L from the forward perpendicular and B/6 or 5 m elsewhere, B/15 or 2 m high. 2.6.1.2 and 2.6.1.3,
//! annex p. 30: a type 2G ship of 150 m or less keeps the bulkheads of a machinery space aft intact; above
//! 150 m it does not.

use ovd_format::damage::{IGC_DAMAGE_TABLES, igc_damage_data};
use ovd_format::rule_set::built_in;

/// A hull of 90 x 14 m is below every cap: the extents are the formulas of 2.3.1.
#[test]
fn below_the_caps_the_extents_are_the_formulas_of_igc_2_3_1() {
    let set = built_in().latest("igc-2g").expect("igc-2g");
    let d = igc_damage_data(set, 90.0, 14.0).expect("IGC damage data");
    let third_l23 = libm::pow(90.0, 2.0 / 3.0) / 3.0;
    let e = d.extents;
    for (got, want) in [
        (e.side_long_m, third_l23),
        (e.side_inboard_m, 14.0 / 5.0),
        (e.bottom_fwd_long_m, third_l23),
        (e.bottom_aft_long_m, third_l23),
        (e.bottom_fwd_width_m, 14.0 / 6.0),
        (e.bottom_aft_width_m, 14.0 / 6.0),
        (e.bottom_height_m, 14.0 / 15.0),
        (e.forward_zone, 0.3),
    ] {
        assert!((got - want).abs() < 1e-9, "{got} vs {want}");
    }
    assert!(d.machinery_aft_intact);
}

/// A hull of 400 x 60 m is above every cap: 14.5, 11.5, 14.5 (both bottom zones, unlike MARPOL's 5 m aft),
/// 10, 5 and 2 m (not MARPOL's 6 m).
#[test]
fn above_the_caps_the_extents_are_the_caps_of_igc_2_3_1() {
    let set = built_in().latest("igc-2g").expect("igc-2g");
    let e = igc_damage_data(set, 400.0, 60.0).expect("data").extents;
    assert_eq!(e.side_long_m, 14.5);
    assert_eq!(e.side_inboard_m, 11.5);
    assert_eq!(e.bottom_fwd_long_m, 14.5);
    assert_eq!(e.bottom_aft_long_m, 14.5);
    assert_eq!(e.bottom_fwd_width_m, 10.0);
    assert_eq!(e.bottom_aft_width_m, 5.0);
    assert_eq!(e.bottom_height_m, 2.0);
}

/// 2.6.1.3 applies at 150 m or less; 2.6.1.2 above.
#[test]
fn the_aft_machinery_bulkheads_are_intact_at_150_m_or_less() {
    let set = built_in().latest("igc-2g").expect("igc-2g");
    assert!(
        igc_damage_data(set, 150.0, 20.0)
            .unwrap()
            .machinery_aft_intact
    );
    assert!(
        !igc_damage_data(set, 150.5, 20.0)
            .unwrap()
            .machinery_aft_intact
    );
}

/// The data is in the Rule Set: the latest `igc-2g` holds every table, each citing IGC 2.3 or 2.6; the released
/// 2.0.0 holds none, and reading it names the missing table.
#[test]
fn the_damage_data_is_rule_set_tables() {
    let lib = built_in();
    let latest = lib.latest("igc-2g").expect("igc-2g");
    for id in IGC_DAMAGE_TABLES {
        let t = latest
            .tables
            .iter()
            .find(|t| t.id == *id)
            .unwrap_or_else(|| panic!("{} has no table {id}", latest.key()));
        assert!(
            t.source.locator.starts_with("2.3.1") || t.source.locator.starts_with("2.6.1"),
            "{id}: {}",
            t.source.locator
        );
    }
    let old = lib.find("igc-2g", "2.0.0").expect("igc-2g 2.0.0");
    let err = igc_damage_data(old, 90.0, 14.0).unwrap_err();
    assert!(err.contains("igc-damage-side-long"), "{err}");
}
