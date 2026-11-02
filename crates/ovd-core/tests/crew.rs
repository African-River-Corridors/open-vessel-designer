//! The crew model's method (issue #778, spec 0011 4.1) on small hand tables.
//!
//! Label: property for the method (a rule per test). The sourced numbers are in
//! `ovd-format/tests/crew.rs`, where the Rule Sets are.

use ovd_core::crew::*;

fn col(id: &str, gt: u32, kw: u32, ums: bool, coastwise: bool, counts: [u32; 10]) -> LiberiaColumn {
    LiberiaColumn {
        id: id.into(),
        gt_band: gt,
        kw_band: kw,
        ums,
        coastwise,
        counts,
    }
}

fn grid() -> Vec<LiberiaColumn> {
    // Only the shape of the grid matters here: UMS columns at (2, 1), (2, 0), (1, 0); a coastwise
    // one at (0, 0); a non-UMS one at (2, 1).
    vec![
        col("a", 2, 1, true, false, [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
        col("b", 2, 0, true, false, [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
        col("c", 1, 0, true, false, [1, 1, 1, 2, 1, 1, 1, 0, 1, 1]),
        col("d", 0, 0, true, true, [1, 0, 1, 1, 1, 1, 0, 0, 1, 0]),
        col("e", 2, 1, false, false, [1, 1, 2, 2, 2, 1, 1, 2, 2, 1]),
    ]
}

#[test]
fn an_exact_column_is_taken_as_it_is() {
    let g = grid();
    let c = select_liberia_column(&g, 2, 0, true).unwrap();
    assert_eq!(
        (g[c.index].id.as_str(), c.exact, c.other_neighbour),
        ("b", true, None)
    );
}

#[test]
fn a_hole_with_one_neighbour_takes_it() {
    let g = grid();
    // GT band 4, kW band 0, UMS: the same kW band has b (2) and c (1): b is nearest.
    let c = select_liberia_column(&g, 4, 0, true).unwrap();
    assert_eq!(g[c.index].id, "b");
    assert!(!c.exact);
    // The column that keeps the GT band does not exist at band 4: none to show.
    assert_eq!(c.other_neighbour, None);
    // GT band 3: b (distance 1) beats c (distance 2).
    assert_eq!(
        g[select_liberia_column(&g, 3, 0, true).unwrap().index].id,
        "b"
    );
}

#[test]
fn a_hole_on_a_tie_takes_the_higher_gt_band() {
    let g = vec![
        col("lo", 1, 0, true, false, [1, 1, 1, 1, 1, 1, 1, 0, 1, 1]),
        col("hi", 3, 0, true, false, [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
    ];
    // Band 2 is one from each: the higher.
    assert_eq!(
        g[select_liberia_column(&g, 2, 0, true).unwrap().index].id,
        "hi"
    );
}

#[test]
fn a_hole_takes_the_higher_neighbour() {
    let g = grid();
    // GT band 2, kW band 0 exists; ask for a kW band no column has: only the GT band is kept.
    let c = select_liberia_column(&g, 2, 5, true).unwrap();
    // No column has kW band 5, so the same GT band is taken (a and b: the higher total, the last tie).
    assert_eq!(g[c.index].gt_band, 2);
    assert!(!c.exact);
}

#[test]
fn without_unattended_machinery_the_attended_columns_are_used() {
    let g = grid();
    // Non-UMS at kW band 0: no such column, so the column of the same GT band at the higher power.
    let c = select_liberia_column(&g, 2, 0, false).unwrap();
    assert_eq!(g[c.index].id, "e");
    assert!(!c.exact);
}

#[test]
fn a_coastwise_column_is_never_taken() {
    let g = grid();
    // GT band 0, kW band 0, UMS: column d is the exact fit but is coastwise only.
    let c = select_liberia_column(&g, 0, 0, true).unwrap();
    assert_ne!(g[c.index].id, "d");
}

#[test]
fn the_watches_follow_the_hours_and_the_master_is_free_above_3000_gt() {
    let w = watches(5000.0, 24.0);
    assert_eq!(
        (w.watchkeepers, w.master_watch_free, w.deck_officers),
        (3, true, 4)
    );
    let w = watches(5000.0, 12.0);
    assert_eq!((w.watchkeepers, w.deck_officers), (2, 3));
    // Exactly 12 hours is two watchkeepers; just over is three.
    assert_eq!(watches(5000.0, 12.01).watchkeepers, 3);
    // At 3,000 GT the master still stands a watch.
    let w = watches(3000.0, 24.0);
    assert_eq!((w.master_watch_free, w.deck_officers), (false, 3));
}

#[test]
fn the_cook_comes_at_ten_persons_on_a_seagoing_hull_only() {
    let g = grid();
    // Column b: 1+1+2 deck, 1+1+0 engineers, 2+2+1+1 ratings = 12 persons.
    let with = seagoing_crew(&g[1], 5000.0, 24.0, true);
    assert_eq!((with.cook, with.minimum), (1, 13));
    let without = seagoing_crew(&g[1], 5000.0, 24.0, false);
    assert_eq!((without.cook, without.minimum), (0, 12));
    // Column d: 8 persons: no cook.
    assert_eq!(seagoing_crew(&g[3], 400.0, 24.0, true).cook, 0);
}

#[test]
fn a_watch_floor_above_the_flag_table_adds_officers_of_the_watch() {
    // A column with 1 master, no mate, 1 officer: 2 deck officers; 24 h on a hull over 3,000 GT
    // asks for master + 3 watchkeepers = 4.
    let c = col("x", 2, 0, true, false, [1, 0, 1, 1, 1, 1, 1, 0, 1, 1]);
    let s = seagoing_crew(&c, 4000.0, 24.0, true);
    assert_eq!((s.deck_officers, s.added_by_watches), (4, 2));
    assert_eq!(s.roles[2].count, 3);
}

#[test]
fn the_inland_crew_is_the_greatest_cell_of_the_group_and_mode() {
    let cells = vec![
        RpnCell {
            key: "g2-B-S1".into(),
            counts: [2, 0, 0, 2, 0, 0],
        },
        RpnCell {
            key: "g2-B-S2".into(),
            counts: [2, 0, 0, 1, 1, 0],
        },
        RpnCell {
            key: "g2-A1-S1-alt1".into(),
            counts: [1, 0, 1, 0, 0, 0],
        },
        RpnCell {
            key: "g2-A1-S1-alt2".into(),
            counts: [1, 0, 0, 1, 1, 0],
        },
        RpnCell {
            key: "g3-B-S1".into(),
            counts: [2, 1, 0, 2, 0, 0],
        },
    ];
    let b = inland_crew(&cells, 2, "B").unwrap();
    assert_eq!((b.minimum, b.least), (4, 4));
    // Alternatives of one cell: the greater, 3, against 2.
    let a = inland_crew(&cells, 2, "A1").unwrap();
    assert_eq!((a.minimum, a.least, a.key.as_str()), (3, 2, "g2-A1-S1"));
    assert!(inland_crew(&cells, 1, "B").is_none());
}

#[test]
fn berths_are_the_crew_plus_two_and_a_typed_crew_overrides() {
    let margin = owner_margin(true, true);
    assert_eq!(margin.iter().map(|r| r.count).sum::<u32>(), 3);
    let b = berths(13, &margin, None);
    assert_eq!((b.crew, b.berths, b.planning), (16, 18, (14, 16)));
    let t = berths(13, &margin, Some(11));
    assert_eq!((t.crew, t.berths), (11, 13));
    // A hull with no margin: the berths are the minimum + 2.
    assert_eq!(berths(4, &owner_margin(false, false), None).berths, 6);
}

#[test]
fn the_cross_check_flags_a_difference_over_one_person() {
    assert!(!cross_check(7, 6).flagged);
    assert!(!cross_check(5, 6).flagged);
    assert!(!cross_check(6, 6).flagged);
    let c = cross_check(8, 6);
    assert!(c.flagged);
    assert_eq!(c.difference, 2);
    assert!(cross_check(4, 6).flagged);
}

#[test]
fn the_rule_set_ids_are_the_three_the_spec_names() {
    assert_eq!(
        MANNING_RULE_SETS,
        ["liberia-man-001", "uk-msn-1868", "ccnr-rpn-19"]
    );
}

#[test]
fn a_hole_takes_the_neighbour_with_more_persons_and_shows_the_other() {
    // GT band 3, kW band 0: the column that keeps the kW band has 12 persons, the one that keeps the
    // GT band 13. The higher is taken; the other is shown.
    let g = vec![
        col("kw", 2, 0, true, false, [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
        col("gt", 3, 1, true, false, [1, 1, 2, 3, 2, 1, 1, 0, 1, 1]),
    ];
    let c = select_liberia_column(&g, 3, 0, true).unwrap();
    assert_eq!((g[c.index].id.as_str(), c.exact), ("gt", false));
    assert_eq!(c.other_neighbour.map(|i| g[i].id.as_str()), Some("kw"));
    // On equal totals the column that keeps the kW band is taken.
    let g = vec![
        col("kw", 2, 0, true, false, [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
        col("gt", 3, 1, true, false, [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
    ];
    assert_eq!(
        g[select_liberia_column(&g, 3, 0, true).unwrap().index].id,
        "kw"
    );
}
