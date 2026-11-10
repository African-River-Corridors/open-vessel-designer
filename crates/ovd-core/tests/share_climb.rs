//! Seam: the step rule of the climb share search (issue #1034, Phase 3, design note D2, rule C). With
//! the share of the tank diameter searched, the search starts at the tallest share that passes the
//! intact checks and goes down one share at a time while the cargo rises by at least
//! `SHARE_CLIMB_MIN_GAIN` of the cargo it has; it keeps the last share that rose. A tie, a smaller
//! gain or a fall ends the climb, so a much deeper hull is never bought for a few tonnes.
//!
//! Label: property.
// DONE-TEST #1034 P3 — do not edit in the build

use ovd_core::gas_arrangement::{SHARE_CLIMB_MIN_GAIN, climb_continues};

#[test]
fn the_least_gain_of_a_climb_step_is_one_per_cent() {
    assert_eq!(SHARE_CLIMB_MIN_GAIN, 0.01);
}

/// `climb_continues(from_t, to_t)`: whether the climb takes the step from the cargo it has to the
/// cargo of the next share down.
#[test]
fn the_climb_goes_on_only_when_the_cargo_rises_by_the_least_gain() {
    assert!(climb_continues(1000.0, 1010.5), "a gain over 1 %");
    assert!(climb_continues(1000.0, 2000.0), "a large gain");
    assert!(!climb_continues(1000.0, 1009.5), "a gain under 1 %");
    assert!(
        !climb_continues(1000.0, 1000.0),
        "a tie goes to the taller share"
    );
    assert!(!climb_continues(1000.0, 999.0), "a fall");
    assert!(!climb_continues(1000.0, 0.0), "no cargo");
}
