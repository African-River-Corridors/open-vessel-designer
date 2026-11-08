//! Seam: the search with a tank top standing above the deck (issue #992): the depth of each candidate
//! follows its tank, and an arrangement that cannot be judged is turned down with a reason.
//!
//! Label: rule. The depth rule is the maintainer's decision of 2026-11-07 (design note D1a).

use ovd_core::gas_arrangement::{
    FitOptions, depth_for_standing_height, fit, tank_top_above_keel_m,
};

fn options(h: f64) -> FitOptions {
    FitOptions {
        depths: vec![15.95],
        sightline_limit_m: Some(250.0),
        required_freeboard_m: 4.0,
        tank_top_above_deck_m: Some(h),
        ..FitOptions::default()
    }
}

#[test]
fn the_depth_rule_is_the_tank_top_less_the_standing_height_and_a_gap_only_in_hold() {
    assert_eq!(depth_for_standing_height(14.884, 2.0), 12.884);
    assert_eq!(depth_for_standing_height(14.884, 0.0), 15.184);
    assert_eq!(depth_for_standing_height(14.884, 0.3), 14.584);
}

/// The table of the hydrostatics ends at the depth. For 14.684 m, `depth * 10 / 10` is an ulp above
/// the depth, a draught with no waterplane: the table failed for a hull of that depth (a depth that
/// follows the tank is not a round number). Label: rule, the last row is the depth itself.
#[test]
fn the_depth_of_the_arrangement_found_follows_its_own_tank() {
    for h in [0.0, 0.2, 2.0] {
        let f = fit(140.0, 29.0, "LNG", 11.95, &options(h)).unwrap();
        // Standing 2 m high with the 250 m Blind Distance limit and a 6 m lift fails the
        // sightline check (the tank top is then the obstruction): the depth still follows the tank.
        if h < 1.0 {
            assert!(f.feasible, "{h}: {:?}", f.failed_checks);
        }
        let want = depth_for_standing_height(tank_top_above_keel_m(&f.engine_result), h);
        assert!(
            (f.depth_m - want).abs() < 0.0016,
            "{h}: {} vs {want}",
            f.depth_m
        );
        // Every arrangement tried reports its own depth, not the seed depth.
        assert!(f.tried.iter().all(|t| (t.depth_m - 15.95).abs() > 1e-9));
    }
    // A tank cannot stand higher than its own diameter.
    let f = fit(140.0, 29.0, "LNG", 11.95, &options(40.0)).unwrap();
    assert!(!f.feasible);
}

#[test]
fn the_hydrostatic_table_of_a_hull_of_any_depth_ends_at_its_depth() {
    use ovd_core::hull::hull_form::HullShape;
    use ovd_core::hydrostatics::{Hull, HullSpec};
    for depth in [14.684, 14.584, 12.928, 15.228, 17.383, 9.946] {
        let h = Hull::build(&HullSpec {
            loa: 140.0,
            beam: 29.0,
            depth,
            bow_rake_length_m: 0.0,
            bow_rake_rise_m: 0.0,
            stern: None,
            shape: HullShape::default(),
            downflooding_deg: None,
            openings: Vec::new(),
        })
        .unwrap();
        let table = h
            .hydrostatic_table(1.025)
            .unwrap_or_else(|e| panic!("{depth}: {e}"));
        assert_eq!(table.len(), 10);
        assert_eq!(table[9].draught_m, depth, "{depth}");
    }
}

/// Review item 1 at the core: with a draught limit the depth is not less than it plus the freeboard.
#[test]
fn the_search_keeps_the_depth_at_or_above_the_draught_limit_plus_the_freeboard() {
    let o = FitOptions {
        draught_limit_m: Some(3.0),
        required_freeboard_m: 0.3,
        river_only: true,
        ..options(2.5)
    };
    let f = fit(86.0, 11.4, "LNG", 3.0, &o).unwrap();
    assert!(f.depth_m >= 3.3 - 1e-9, "{}", f.depth_m);
    assert!(
        f.tried
            .iter()
            .all(|t| t.depth_m == 0.0 || t.depth_m >= 3.3 - 1e-9)
    );
}

/// The probe of the depth stops before the costly stability stage; the search it stands for must pick
/// the same depth. Label: rule (review of #1015: both passes pick the same arrangement).
#[test]
fn the_probe_gives_the_depth_the_full_search_settles_on() {
    use ovd_core::gas_arrangement::probe_depth;
    for h in [0.0, 0.2] {
        let o = options(h);
        let f = fit(140.0, 29.0, "LNG", 11.95, &o).unwrap();
        let probed = probe_depth(140.0, 29.0, "LNG", 11.95, &o).unwrap().unwrap();
        assert!(
            (probed - f.depth_m).abs() < 0.0015,
            "{h}: {probed} vs {}",
            f.depth_m
        );
    }
    // At 2 m standing the 250 m Blind Distance limit leaves no candidate: no probe, and the full
    // search is the one that reports the closest.
    assert!(
        probe_depth(140.0, 29.0, "LNG", 11.95, &options(2.0))
            .unwrap()
            .is_none()
    );
}

/// A shield higher than the dome is the highest point of the cargo area, so the air draft and the
/// sightline take it; with the input left out the dome top stands as before.
/// Label: rule (design note D4). The clearance 4.0 m is a test value above the dome rule's about 2.3 m.
#[test]
fn a_shield_above_the_dome_is_the_highest_point_only_with_a_standing_height() {
    let high = |h: Option<f64>| {
        let o = FitOptions {
            tank_top_above_deck_m: h,
            weather_shield_clearance_m: 4.0,
            self_propelled: false,
            vent_mast_lowerable: true,
            depths: vec![15.95],
            ..FitOptions::default()
        };
        let f = fit(140.0, 29.0, "LNG", 11.95, &o).unwrap();
        let r = f.engine_result;
        (r.highest_point.unwrap(), tank_top_above_keel_m(&r))
    };
    let (with, top) = high(Some(2.0));
    assert_eq!(with.key, "weather_shield");
    assert!(
        (with.top_above_keel_m - (top + 4.0)).abs() < 1e-9,
        "{} {top}",
        with.top_above_keel_m
    );
    let (without, _) = high(None);
    assert_ne!(without.key, "weather_shield");
}

/// Review of #1023: with the shield the highest point (clearance above the dome), the river-only
/// vent exit is governed by the shield top, not named the dome top; and the sightline sees it: the
/// lift the Blind Distance limit needs is more than with the shield below the dome.
/// Label: rule (design note D4). Clearances 4.0 and 1.5 m are test values; the dome rule gives about 2.3 m.
#[test]
fn a_shield_that_is_the_high_point_names_the_vent_exit_and_raises_the_lift_needed() {
    use ovd_core::vent_mast::Governs;
    let run = |clearance: f64, river_only: bool| {
        let o = FitOptions {
            tank_top_above_deck_m: Some(2.0),
            weather_shield_clearance_m: clearance,
            sightline_limit_m: Some(250.0),
            river_only,
            // A mast that lowers does not set the air draft, so the shield stays the high point.
            vent_mast_lowerable: true,
            depths: vec![15.95],
            ..FitOptions::default()
        };
        fit(140.0, 29.0, "LNG", 11.95, &o).unwrap().engine_result
    };
    // The vent mast: river only, shield above the dome.
    let high = run(4.0, true);
    assert_eq!(high.highest_point.map(|p| p.key), Some("weather_shield"));
    assert_eq!(high.vent_mast.as_ref().unwrap().governs, Governs::ShieldTop);
    let low = run(1.5, true);
    assert_ne!(low.vent_mast.as_ref().unwrap().governs, Governs::ShieldTop);
    // The sightline: the higher obstruction needs a higher eye.
    let (a, b) = (
        run(4.0, false).sightline.unwrap().lift_needed_m,
        run(1.5, false).sightline.unwrap().lift_needed_m,
    );
    assert!(
        a > b + 0.5,
        "lift needed {a} with the shield high, {b} with it low"
    );
}
