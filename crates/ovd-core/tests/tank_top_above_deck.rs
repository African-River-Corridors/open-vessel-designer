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
