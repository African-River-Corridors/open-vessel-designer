//! The crew model on the manning Rule Sets (issue #778, spec 0011 4.1, D5a, D13, D14, D16).
//!
//! Label: sourced for the tables (each value worked from the cited notice, edition and paragraph) and
//! for the two worked cases of 4.1; property for the method. Concept and preliminary design only.

use ovd_format::crew::{CrewFacts, at, crew_json, liberia_columns, rigid_convoy_crew, rpn_cells};
use ovd_format::rule_set::built_in;
use ovd_format::vdesign::{DesignOption, MANNING_RULE_SETS};
use ovd_format::{Profile, RunOutcome, check, fill_as_new, results_hash, run_option};
use serde_json::{Value, json};

fn set(id: &str) -> &'static arc_rules::model::RuleSet {
    built_in()
        .sets()
        .iter()
        .find(|s| s.id == id)
        .unwrap_or_else(|| panic!("no {id}"))
}

fn option(area: &str, extra: Value) -> DesignOption {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" },
        "service_area": { "value": area, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let text = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
    .to_string();
    fill_as_new(&river_limits(&text), &Profile::built_in())
        .design
        .unwrap()
        .options
        .remove(0)
}

fn facts(gt: f64, kw: f64, genset: f64, length: f64) -> CrewFacts {
    CrewFacts {
        gt,
        gt_source: "typed".into(),
        propulsion_kw: kw,
        genset_kw: genset,
        length_m: length,
        gas_cargo: true,
    }
}

fn crew(area: &str, extra: Value, f: &CrewFacts) -> Value {
    crew_json(&option(area, extra), built_in(), f).unwrap()
}

// --- the tables, from the notice ----------------------------------------------------------------

/// Liberia MAN-001 Rev. 11/22, 12.2 (PDF p. 9): the persons of each of the 13 columns, in the order
/// master, chief mate, officer of the watch, able seafarer deck, rating (II/4), chief engineer,
/// second engineer, engineering watch officer, able seafarer engine, rating (III/4). Typed again from
/// the printed table, apart from the Rule Set file.
const MAN001: [(&str, [u32; 10]); 13] = [
    ("c1", [1, 1, 2, 3, 2, 1, 1, 2, 2, 1]),
    ("c2", [1, 1, 2, 3, 1, 1, 1, 2, 2, 1]),
    ("c3", [1, 1, 2, 2, 2, 1, 1, 2, 2, 1]),
    ("c4", [1, 1, 1, 2, 1, 1, 1, 1, 2, 1]),
    ("c5", [1, 0, 1, 1, 1, 1, 0, 1, 1, 1]),
    ("c6", [1, 1, 2, 3, 2, 1, 1, 0, 1, 1]),
    ("c7", [1, 1, 2, 3, 1, 1, 1, 0, 1, 1]),
    ("c8", [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
    ("c9", [1, 1, 2, 2, 2, 1, 1, 0, 1, 1]),
    ("c10", [1, 1, 1, 2, 1, 1, 1, 0, 1, 1]),
    ("c11", [1, 1, 1, 2, 1, 1, 1, 0, 1, 1]),
    ("c12", [1, 0, 1, 1, 1, 1, 0, 1, 1, 1]),
    ("c13", [1, 0, 1, 1, 1, 1, 0, 0, 1, 0]),
];

#[test]
fn liberia_sourced_every_column_is_the_printed_one() {
    // Sourced: Liberia MAN-001 Rev. 11/22, 12.2, PDF p. 9.
    let cols = liberia_columns(set("liberia-man-001")).unwrap();
    assert_eq!(cols.len(), 13);
    for (id, counts) in MAN001 {
        let c = cols.iter().find(|c| c.id == id).unwrap();
        assert_eq!(c.counts, counts, "column {id}");
    }
    // The totals worked by hand from the rows: 16 15 15 12 8 | 13 12 12 12 10 10 8 6.
    let totals: Vec<u32> = cols.iter().map(|c| c.total()).collect();
    assert_eq!(totals, [16, 15, 15, 12, 8, 13, 12, 12, 12, 10, 10, 8, 6]);
}

#[test]
fn liberia_sourced_the_header_rows_of_the_table() {
    // Sourced: 12.2 header rows (GT, KW, Routes/Operations, class notation for UMS), PDF p. 9.
    let cols = liberia_columns(set("liberia-man-001")).unwrap();
    let head = |id: &str| {
        let c = cols.iter().find(|c| c.id == id).unwrap();
        (c.gt_band, c.kw_band, c.ums, c.coastwise)
    };
    assert_eq!(head("c1"), (4, 1, false, false)); // >8000, >3000, No, All
    assert_eq!(head("c5"), (0, 1, false, true)); // <500, >3000, No, Coastwise
    assert_eq!(head("c6"), (4, 1, true, false)); // >8000, >3000, Yes
    assert_eq!(head("c9"), (2, 0, true, false)); // 3000-5000, 750-3000, Yes
    assert_eq!(head("c11"), (1, 0, true, false)); // 500-3000, 750-3000, Yes
    assert_eq!(head("c13"), (0, 0, true, true)); // <500, 750-3000, Yes, Coastwise
}

#[test]
fn liberia_sourced_the_gt_and_kw_bands_at_every_boundary() {
    let s = set("liberia-man-001");
    let gt = |v: f64| at(s, "liberia-gt-band", v).unwrap() as u32;
    // 8,000 is in "5000-8000" (the column ">8000" is strict); 5,000, 3,000 and 500 are in two printed
    // bands, and the higher band is taken.
    assert_eq!(
        [
            gt(8000.1),
            gt(8000.0),
            gt(5000.0),
            gt(4999.9),
            gt(3000.0),
            gt(2999.9)
        ],
        [4, 3, 3, 2, 2, 1]
    );
    // "<500" is strict: 500 is in "500-3000".
    assert_eq!([gt(500.0), gt(499.9)], [1, 0]);
    let kw = |v: f64| at(s, "liberia-kw-band", v).unwrap() as u32;
    // ">3000" is strict: 3,000 kW is in "750-3000". Under 750 kW the grid has no band: the lowest.
    assert_eq!([kw(3000.1), kw(3000.0), kw(750.0), kw(749.9)], [1, 0, 0, 0]);
}

#[test]
fn uk_sourced_annex_a_and_b_at_the_printed_bounds() {
    // Sourced: MSN 1868 (M) Amendment 1, Annex A (PDF p. 10) and Annex B (PDF p. 11).
    let s = set("uk-msn-1868");
    let a = |t: &str, v: f64| at(s, t, v).unwrap() as u32;
    // Unlimited: 3000 or more 1+1+2 = 4; 500 to under 3000 1+1+1 = 3; under 500 1+2(a) = 3.
    assert_eq!(
        [
            a("uk-annex-a-unlimited", 3000.0),
            a("uk-annex-a-unlimited", 2999.9),
            a("uk-annex-a-unlimited", 499.9)
        ],
        [4, 3, 3]
    );
    // Near coastal: 3000 or more 1+1+1 = 3; 500 to under 3000 1+1+1(b) = 3; under 500 1+1 (II/3) = 2.
    assert_eq!(
        [
            a("uk-annex-a-near-coastal", 3000.0),
            a("uk-annex-a-near-coastal", 500.0),
            a("uk-annex-a-near-coastal", 499.9)
        ],
        [3, 3, 2]
    );
    // Unlimited engineers: 3000 or more 3; 750 to under 3000 3; 350 to under 750 2.
    assert_eq!(
        [
            a("uk-annex-b-unlimited", 3000.0),
            a("uk-annex-b-unlimited", 750.0),
            a("uk-annex-b-unlimited", 749.9),
            a("uk-annex-b-unlimited", 350.0)
        ],
        [3, 3, 2, 2]
    );
    // Near-coastal engineers: 9000 or more 2; 3000 to under 9000 2; 750 to under 3000 2; 350 to under 750 1.
    assert_eq!(
        [
            a("uk-annex-b-near-coastal", 9000.0),
            a("uk-annex-b-near-coastal", 3000.0),
            a("uk-annex-b-near-coastal", 750.0),
            a("uk-annex-b-near-coastal", 749.9)
        ],
        [2, 2, 2, 1]
    );
    // Under 350 kW the Annex has no row.
    assert!(at(s, "uk-annex-b-unlimited", 349.9).is_err());
}

#[test]
fn rpn_sourced_19_02_totals_of_every_cell() {
    // Sourced: CCNR RPN 01.01.2026, 19.02(1), PDF p. 57: the printed cells, each summed by hand.
    let cells = rpn_cells(set("ccnr-rpn-19"), "19.02").unwrap();
    let total = |k: &str| {
        cells
            .iter()
            .find(|c| c.key == k)
            .unwrap_or_else(|| panic!("{k}"))
            .total()
    };
    let expected = [
        ("g1-A1-S1", 2),
        ("g1-A1-S2", 2),
        ("g1-A2-S1", 2),
        ("g1-A2-S2", 2),
        ("g1-B-S1", 4),
        ("g1-B-S2", 4),
        ("g2-A1-S1-alt1", 2),
        ("g2-A1-S1-alt2", 3),
        ("g2-A1-S2", 3),
        ("g2-A2-S1", 3),
        ("g2-A2-S2", 3),
        ("g2-B-S1", 4),
        ("g2-B-S2", 4),
        ("g3-A1-S1-alt1", 3),
        ("g3-A1-S1-alt2", 4),
        ("g3-A1-S2", 3),
        ("g3-A2-S1", 4),
        ("g3-A2-S2", 4),
        ("g3-B-S1-alt1", 5),
        ("g3-B-S1-alt2", 4),
        ("g3-B-S2", 5),
    ];
    for (k, t) in expected {
        assert_eq!(total(k), t, "{k}");
    }
    assert_eq!(cells.len(), expected.len());
    // 19.02 has no engineer.
    assert!(cells.iter().all(|c| c.counts[5] == 0));
}

#[test]
fn rpn_sourced_19_03_totals_of_the_pusher_groups() {
    // Sourced: RPN 19.03(1), PDF p. 58, groups 4 and 5 (the engineer rows), each summed by hand.
    let cells = rpn_cells(set("ccnr-rpn-19"), "19.03").unwrap();
    let total = |k: &str| {
        cells
            .iter()
            .find(|c| c.key == k)
            .unwrap_or_else(|| panic!("{k}"))
            .total()
    };
    for (k, t) in [
        ("g4-A1-S1", 4),
        ("g4-A1-S2", 4),
        ("g4-A2-S1", 5),
        ("g4-A2-S2", 5),
        ("g4-B-S1-alt1", 6),
        ("g4-B-S1-alt2", 5),
        ("g4-B-S2-alt1", 6),
        ("g4-B-S2-alt2", 5),
        ("g5-A1-S1-alt1", 5),
        ("g5-A1-S1-alt2", 6),
        ("g5-A1-S2", 5),
        ("g5-A2-S1", 5),
        ("g5-A2-S2", 6),
        ("g5-B-S1-alt1", 7),
        ("g5-B-S1-alt2", 6),
        ("g5-B-S2-alt1", 7),
        ("g5-B-S2-alt2", 6),
    ] {
        assert_eq!(total(k), t, "{k}");
    }
    // Groups 1 to 3 (21 cells as in 19.02) and the printed cells of groups 4 (8) and 5 (9).
    assert_eq!(cells.len(), 21 + 8 + 9);
    // The engineer rows of group 5 mode B S1: 1 and 1 (both alternatives).
    let c = cells.iter().find(|c| c.key == "g5-B-S1-alt1").unwrap();
    assert_eq!(c.counts[5], 1);
}

#[test]
fn a_pusher_takes_the_rpn_19_03_table() {
    // Group 4 (pusher + 2 barges) in mode B: the greatest cell, 6, with an engineer in it.
    let c = rigid_convoy_crew(built_in(), 4, 24.0).unwrap();
    assert_eq!(c.minimum, 6);
    assert!(c.roles.iter().any(|r| r.role == "engineer" && r.count == 1));
    // Group 1 in mode A1 (hours 14): 2.
    assert_eq!(rigid_convoy_crew(built_in(), 1, 14.0).unwrap().minimum, 2);
}

#[test]
fn the_rpn_mode_and_group_boundaries() {
    let s = set("ccnr-rpn-19");
    let mode = |h: f64| at(s, "ccnr-rpn-18.01-operating-mode", h).unwrap() as u32;
    // A1 up to 14 h, A2 up to 18 h, B up to 24 h (18.01(1)).
    assert_eq!(
        [mode(14.0), mode(14.1), mode(18.0), mode(18.1), mode(24.0)],
        [1, 2, 2, 3, 3]
    );
    assert!(at(s, "ccnr-rpn-18.01-operating-mode", 24.1).is_err());
    let group = |l: f64| at(s, "ccnr-rpn-19.02-group", l).unwrap() as u32;
    // Up to 70 m, over 70 to 86 m, over 86 m (19.02(1)).
    assert_eq!(
        [group(70.0), group(70.1), group(86.0), group(86.1)],
        [1, 2, 2, 3]
    );
}

#[test]
fn the_rule_sets_are_drafts_with_unverified_rules_that_cite_a_paragraph() {
    for id in MANNING_RULE_SETS {
        let s = set(id);
        assert_eq!(s.status, arc_rules::model::SetStatus::Draft, "{id}");
        assert!(!s.rules.is_empty());
        for r in &s.rules {
            assert!(
                r.verified.is_none(),
                "{} is not verified by the codifier",
                r.id
            );
            assert!(!r.source.locator.is_empty(), "{}", r.id);
        }
    }
}

// --- the worked cases of 4.1 --------------------------------------------------------------------

#[test]
fn worked_case_140_by_29_coastal_lng_hull_is_13_up_to_8000_gt() {
    // Spec 0011 4.1 (140 x 29 m coastal LNG hull, 2.4 MW, 24 h, UMS) -> minimum 13: 4 deck officers,
    // 2 engineers, 6 ratings, cook. It holds only up to 8,000 GT (research manning.md 6.2): above it
    // the grid has no column for 750 to 3,000 kW and the higher neighbour, column 6, is taken.
    for gt in [6000.0, 7000.0] {
        let b = crew(
            "coastal-and-river",
            json!({}),
            &facts(gt, 1717.0, 2400.0, 140.0),
        );
        assert_eq!(b["minimum"], 13, "GT {gt}");
        assert_eq!(
            (
                &b["deck_officers"],
                &b["engineers"],
                &b["ratings"],
                &b["cook"]
            ),
            (&json!(4), &json!(2), &json!(6), &json!(1)),
            "GT {gt}"
        );
        assert_eq!(b["rule_set"]["id"], "liberia-man-001");
    }
    // Over 8,000 GT: column 6 (13 persons: 7 ratings) and the cook make 14.
    for gt in [12843.0, 20000.0] {
        let b = crew(
            "coastal-and-river",
            json!({}),
            &facts(gt, 1717.0, 2400.0, 140.0),
        );
        assert_eq!(b["manning_scale"]["column"], "c6", "GT {gt}");
        assert_eq!(b["minimum"], 14, "GT {gt}");
        assert_eq!(b["ratings"], 7, "GT {gt}");
        assert_eq!(b["manning_scale"]["other_neighbour_column"], "c9");
    }
}

#[test]
fn berths_are_the_crew_plus_the_margin_plus_two_spare() {
    // Method, not a source: the owner margin (cargo engineer, steward, electro-technical officer) is an
    // estimate, so crew = 13 + 3 and berths = crew + 2 (D16).
    let b = crew(
        "coastal-and-river",
        json!({}),
        &facts(7000.0, 1717.0, 2400.0, 140.0),
    );
    assert_eq!(
        (&b["crew"], &b["berths"], &b["spare_berths"]),
        (&json!(16), &json!(18), &json!(2))
    );
}

#[test]
fn the_power_band_of_an_electric_drive_takes_the_genset_total() {
    // D14: 1,717 kW of propulsion is the band 750-3000, as 2,400 kW of gensets. 3,200 kW of gensets
    // is the band over 3000: the engineering watch officers of the table are not asked for with
    // unattended machinery, but the column changes.
    let low = crew(
        "coastal-and-river",
        json!({}),
        &facts(7000.0, 1717.0, 2400.0, 140.0),
    );
    assert_eq!(low["inputs"]["power_kw"], 2400.0);
    let high = crew(
        "coastal-and-river",
        json!({}),
        &facts(7000.0, 1717.0, 3200.0, 140.0),
    );
    assert_eq!(high["inputs"]["power_kw"], 3200.0);
    // Over 3000 kW and 5,000-8,000 GT, UMS: column c7 (12 persons) exactly.
    assert_eq!(high["manning_scale"]["column"], "c7");
    assert_eq!(high["manning_scale"]["exact"], true);
    assert_eq!(low["manning_scale"]["exact"], false);
    assert!(
        low["inputs"]["power_basis"]
            .as_str()
            .unwrap()
            .contains("genset"),
        "{}",
        low["inputs"]["power_basis"]
    );
}

#[test]
fn worked_case_85_m_inland_type_g_tanker_in_mode_b_is_4() {
    // Sourced: spec 0011 4.1 -> 4 (RPN 19.02, group 2, mode B: 2 boatmasters + 2 boatmen on S1, or 2
    // boatmasters + a boatman and an apprentice on S2).
    let b = crew(
        "river-only",
        json!({ "navigation_hours_per_day": 24.0 }),
        &facts(1500.0, 1000.0, 0.0, 85.0),
    );
    assert_eq!(b["minimum"], 4);
    assert_eq!(b["regime"], "inland");
    assert_eq!(b["rpn"]["group"], 2);
    assert_eq!(b["rpn"]["operating_mode"], "B");
    assert_eq!(b["cook"], 0);
    assert_eq!(b["rule_set"]["id"], "ccnr-rpn-19");
    // 4 + the owner's engineer + 2 spare.
    assert_eq!((&b["crew"], &b["berths"]), (&json!(5), &json!(7)));
    // A typed 14 h is mode A1: group 2 takes the greater alternative, 3.
    let a1 = crew(
        "river-only",
        json!({ "navigation_hours_per_day": 14.0 }),
        &facts(1500.0, 1000.0, 0.0, 85.0),
    );
    assert_eq!(a1["rpn"]["operating_mode"], "A1");
    assert_eq!(a1["minimum"], 3);
    // Issue #929: untyped hours on a river-only Option read 24 h, so RPN mode B (round-the-clock).
    let dflt = crew("river-only", json!({}), &facts(1500.0, 1000.0, 0.0, 85.0));
    assert_eq!(dflt["inputs"]["navigation_hours_per_day"], 24.0);
    assert_eq!(dflt["rpn"]["operating_mode"], "B");
    assert_eq!(dflt["minimum"], 4);
}

#[test]
fn the_rpn_note_does_not_tie_mode_b_to_single_rooms() {
    // Issue #929. RPN 18.02(4): a single room for rest underway is for modes A1 and A2 only; mode B
    // (18.02(3)) sets rest time and no room rule.
    let b = crew("river-only", json!({}), &facts(1500.0, 1000.0, 0.0, 85.0));
    let notes: Vec<&str> = b["notes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n.as_str())
        .collect();
    let note = notes
        .iter()
        .find(|n| n.contains("18.02(4)"))
        .expect("the 18.02(4) note");
    assert!(!note.contains("mode B crew sleeps"), "{note}");
    assert!(note.contains("A1 and A2 only"), "{note}");
    assert!(
        note.contains("mode B (18.02(3)) sets rest time and no room rule"),
        "{note}"
    );
}

// --- UK against Liberia -------------------------------------------------------------------------

#[test]
fn the_uk_and_liberia_are_both_computed_and_a_gap_is_shown() {
    let b = crew(
        "coastal-and-river",
        json!({}),
        &facts(7000.0, 1717.0, 2400.0, 140.0),
    );
    let cc = &b["cross_check"];
    // UK, unlimited, 3,000 GT or more: 1 + 1 + 2 = 4 deck officers; 750 to under 3000 kW: 3 engineers.
    assert_eq!(cc["uk_deck_officers"], 4);
    assert_eq!(cc["uk_engineer_officers"], 3);
    assert_eq!(cc["uk_officers"], 7);
    // Liberia: 4 deck officers and 2 engineers.
    assert_eq!(cc["liberia_officers"], 6);
    assert_eq!(cc["difference"], 1);
    assert_eq!(cc["flagged"], false);
    assert_eq!(cc["rule_set"]["id"], "uk-msn-1868");
}

#[test]
fn the_uk_set_run_alone_takes_the_uk_officers_and_the_liberia_ratings() {
    let b = crew(
        "coastal-and-river",
        json!({ "manning_rule_set": "uk-msn-1868" }),
        &facts(7000.0, 1717.0, 2400.0, 140.0),
    );
    // 4 deck officers + 3 engineers (UK) + 6 ratings (MAN-001 c9) = 13, then the cook: 14.
    assert_eq!(b["rule_set"]["id"], "uk-msn-1868");
    assert_eq!(b["minimum"], 14);
    assert_eq!(b["cook"], 1);
    assert!(b["notes"].to_string().contains("officers only"));
}

// --- typed crew, hours, UMS ---------------------------------------------------------------------

#[test]
fn a_typed_crew_overrides_and_a_crew_under_the_minimum_is_reported() {
    let f = facts(7000.0, 1717.0, 2400.0, 140.0);
    let b = crew("coastal-and-river", json!({ "crew": 18.0 }), &f);
    assert_eq!(
        (&b["source"], &b["crew"], &b["berths"], &b["below_minimum"]),
        (&json!("typed"), &json!(18), &json!(20), &json!(false))
    );
    // The minimum is still worked out.
    assert_eq!(b["minimum"], 13);
    let low = crew("coastal-and-river", json!({ "crew": 9.0 }), &f);
    assert_eq!(
        (&low["crew"], &low["berths"], &low["below_minimum"]),
        (&json!(9), &json!(11), &json!(true))
    );
    assert!(
        low["notes"]
            .to_string()
            .contains("under the manning minimum")
    );
}

#[test]
fn not_unattended_machinery_takes_the_attended_columns() {
    // Sourced: MAN-001 note 4 and the columns 1 to 5 (no class notation for UMS): GT 6,000, over 3,000
    // kW: column 2, 15 persons, plus the cook.
    let b = crew(
        "coastal-and-river",
        json!({ "unmanned_machinery_space": false }),
        &facts(6000.0, 4000.0, 0.0, 140.0),
    );
    assert_eq!(b["manning_scale"]["column"], "c2");
    assert_eq!(b["minimum"], 16);
}

#[test]
fn a_cook_is_added_at_exactly_ten_persons() {
    // Sourced: MAN-001 12.1.9 ("10 or more") and note 5 to 12.2. A 2,000 GT hull of over 3,000 kW with
    // UMS is column 10: 3 deck officers, 2 engineers, 5 ratings = 10 persons, so the cook makes 11.
    let b = crew(
        "coastal-and-river",
        json!({}),
        &facts(2000.0, 4000.0, 0.0, 80.0),
    );
    assert_eq!(b["manning_scale"]["column"], "c10");
    assert_eq!(
        (&b["ratings"], &b["cook"], &b["minimum"]),
        (&json!(5), &json!(1), &json!(11))
    );
    // A river-only hull on the same scale: MLC does not reach it, so no cook.
    let r = crew(
        "river-only",
        json!({ "manning_rule_set": "liberia-man-001" }),
        &facts(2000.0, 4000.0, 0.0, 80.0),
    );
    assert_eq!((&r["cook"], &r["minimum"]), (&json!(0), &json!(10)));
}

#[test]
fn an_unknown_manning_set_is_refused() {
    let o = option("coastal-and-river", json!({}));
    let mut o = o;
    for (n, i) in o.inputs.iter_mut() {
        if n == "manning_rule_set" {
            i.value = json!("nope");
            i.source = ovd_format::Source::Typed;
        }
    }
    assert!(crew_json(&o, built_in(), &facts(7000.0, 1717.0, 2400.0, 140.0)).is_err());
}

#[test]
fn the_manning_rule_set_list_is_the_cores() {
    assert_eq!(MANNING_RULE_SETS, ovd_core::crew::MANNING_RULE_SETS);
}

// --- the new inputs -----------------------------------------------------------------------------

#[test]
fn the_defaults_follow_the_service_area() {
    let get = |area: &str, name: &str| {
        let o = option(area, json!({}));
        o.input(name).map(|i| i.value.clone())
    };
    assert_eq!(
        get("coastal-and-river", "manning_rule_set"),
        Some(json!("liberia-man-001"))
    );
    assert_eq!(
        get("river-only", "manning_rule_set"),
        Some(json!("ccnr-rpn-19"))
    );
    assert_eq!(
        get("coastal-and-river", "navigation_hours_per_day"),
        Some(json!(24.0))
    );
    // Issue #929: round-the-clock by default on a river too (RPN mode B).
    assert_eq!(
        get("river-only", "navigation_hours_per_day"),
        Some(json!(24.0))
    );
    assert_eq!(
        get("coastal-and-river", "unmanned_machinery_space"),
        Some(json!(true))
    );
    assert_eq!(get("coastal-and-river", "crew"), Some(Value::Null));
}

// --- through a run ------------------------------------------------------------------------------

fn example_results() -> Value {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let d = check(&text, &Profile::built_in()).design.unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_worked_example_hull_has_a_crew_of_14_outside_the_results_hash() {
    let results = example_results();
    let crew = &results["crew"];
    assert_eq!(crew["minimum"], 14, "{crew}");
    assert_eq!(crew["regime"], "seagoing");
    assert_eq!(crew["source"], "computed");
    // The hull is over 8,000 GT, and 2,400 kWe of gensets: the grid has no column for that, and the
    // results say so with the column used.
    assert!(results["gross_tonnage"]["gross_tonnage"].as_f64().unwrap() > 8000.0);
    assert_eq!(crew["manning_scale"]["exact"], false);
    assert!(
        crew["manning_scale"]["hole"]
            .as_str()
            .unwrap()
            .contains("c9")
    );
    // The block is outside the results hash: taking it out moves nothing.
    let mut without = results.clone();
    without.as_object_mut().unwrap().remove("crew");
    assert_eq!(results_hash(&results), results_hash(&without));
}

#[test]
fn a_gap_over_one_officer_between_the_uk_and_liberia_is_flagged() {
    // The real tables never differ by more than one officer, so the flag is driven with a UK set
    // (a test copy, version 1.0.1) whose Annex B row for 750 to 3,000 kW gives 5 engineers.
    use ovd_format::rule_set::Library;
    let text = include_str!("../rules/uk-msn-1868/1.0.0.json")
        .replace("\"version\": \"1.0.0\"", "\"version\": \"1.0.1\"");
    let mut v: Value = serde_json::from_str(&text).unwrap();
    let rows = v["tables"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|t| t["id"] == "uk-annex-b-unlimited")
        .unwrap()["rows"]
        .as_array_mut()
        .unwrap();
    rows[1]["a"] = json!(5.0);
    let mut lib = Library::built_in();
    lib.add_text(&v.to_string()).unwrap();
    let b = crew_json(
        &option("coastal-and-river", json!({})),
        &lib,
        &facts(7000.0, 1717.0, 2400.0, 140.0),
    )
    .unwrap();
    // UK 4 + 5 = 9 officers against Liberia 4 + 2 = 6.
    assert_eq!(b["cross_check"]["uk_officers"], 9);
    assert_eq!(b["cross_check"]["difference"], 3);
    assert_eq!(b["cross_check"]["flagged"], true);
    assert!(b["notes"].to_string().contains("over one person"));
}

/// Issue #1034: a river-only Option needs a number for both river limits. These are high enough not to bind, so
/// no value of these tests moves.
fn river_limits(text: &str) -> String {
    ovd_format::with_river_limits(text, Some(12.0), Some(40.0)).unwrap()
}
