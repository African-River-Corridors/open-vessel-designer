//! Seam: the arrangement search chooses the number of tanks along from 2, 3 and 4, screened by IGC 2.7 damage
//! survival (issue #935, addendum). A typed `tanks_in_line` pins it; the Defaults Profile's value does not. The
//! first evaluation (damage not run) keeps the profile's number.
//!
//! Label: snapshot - the 140 x 29 x 12 m coastal LNG reference design, the values the code printed on `main` after
//! #987, #988 and the damage screen. The size limits are provisional until #1009 (the free surface of the cargo in
//! a breached tank) lands.

use ovd_format::damage::DamageControl;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option, run_option_with_damage};
use serde_json::{Value, json};

fn design(extra: Value) -> ovd_format::vdesign::Design {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "moulded_depth_m": { "value": 12.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "loading_conditions": { "value": "is_code_2008", "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap()
}

/// The results of a run with damage, and the damage runs it started (the counter is read after the fill).
fn run(extra: Value) -> (Value, usize) {
    let d = design(extra);
    let before = ovd_format::damage::damage_runs_started();
    let RunOutcome::Results { results, .. } = run_option(&d.options[0]) else {
        panic!("no results");
    };
    (results, ovd_format::damage::damage_runs_started() - before)
}

fn igc_fails(r: &Value) -> usize {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| {
            x["outcome"] == "fail" && x["rule"].as_str().unwrap_or("").starts_with("igc/2.7")
        })
        .count()
}

fn volume(r: &Value) -> f64 {
    r["fit_tried"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["outcome"] == "chosen")
        .unwrap()["cargo_volume_m3"]
        .as_f64()
        .unwrap()
}

/// Unpinned, the search picks three tanks along (2 x 3 at 13.38 m, 23,101 m3) in place of two along (11.53 m,
/// 18,324 m3), with no failing damage row, and the results say the number was chosen from 2, 3 and 4. At most the
/// cap of 24 damage runs are worked out in the search, and one more for the answer.
#[test]
fn unpinned_the_search_chooses_three_along_and_no_igc_2_7_row_fails() {
    let (r, runs) = run(json!({}));
    assert_eq!(igc_fails(&r), 0);
    assert_eq!(r["arrangement"]["n_long"], 3);
    assert_eq!(r["arrangement"]["n_wide"], 2);
    assert!((r["arrangement"]["tank_od_m"].as_f64().unwrap() - 13.383).abs() < 0.001);
    assert!((volume(&r) - 23_101.0).abs() < 1.0, "{}", volume(&r));
    assert_eq!(
        r["damage_screening"]["tanks_along"]["chosen_from"],
        json!([2, 3, 4])
    );
    assert_eq!(r["damage_screening"]["tanks_along"]["n"], 3);
    // The worst case is the cap and the answer's run; this design needs fewer.
    assert!(runs <= 24 + 1, "{runs} damage runs");
    assert_eq!(runs, 9, "damage runs, the answer's included");
    // More cargo than two along, the profile's number (a pinned 2 gives 18,324 m3).
    let (two, _) = run(json!({ "tanks_in_line": 2 }));
    assert!(volume(&r) > volume(&two));
}

/// A typed number along pins the search: today's pick for 2 (11.53 m, 18,324 m3), and the sizes that pass for 3 and
/// 4, with the results saying it was not chosen.
#[test]
fn a_typed_number_along_pins_the_search() {
    for (n, od, vol) in [
        (2, 11.5306, 18_324.0),
        (3, 13.3832, 23_101.0),
        (4, 13.4420, 21_576.0),
    ] {
        let (r, _) = run(json!({ "tanks_in_line": n }));
        assert_eq!(igc_fails(&r), 0, "{n} along");
        assert_eq!(r["arrangement"]["n_long"], n);
        assert!(
            (r["arrangement"]["tank_od_m"].as_f64().unwrap() - od).abs() < 0.001,
            "{n} along"
        );
        assert!((volume(&r) - vol).abs() < 1.0, "{n} along: {}", volume(&r));
        assert_eq!(
            r["damage_screening"]["tanks_along"]["chosen_from"],
            Value::Null,
            "{n} along"
        );
    }
}

/// The first evaluation (damage not run) is not screened, so it keeps the profile's number along, 2: the unscreened
/// pick (13.28 m), flagged. The damage run says what it changed, number along included.
#[test]
fn the_first_evaluation_keeps_the_profiles_number_along_and_the_run_says_what_changed() {
    let d = design(json!({}));
    let skip = DamageControl {
        run: false,
        progress: None,
    };
    let RunOutcome::Results { results: one, .. } =
        run_option_with_damage(&d.options[0], ovd_format::rule_set::built_in(), &[], &skip)
    else {
        panic!("no results");
    };
    assert_eq!(one["arrangement"]["n_long"], 2);
    assert_eq!(one["damage_screening"]["screened"], false);
    assert_eq!(
        one["damage_screening"]["tanks_along"]["chosen_from"],
        Value::Null
    );
    let (two, _) = run(json!({}));
    assert_eq!(two["damage_screening"]["stage_one"]["n_tanks_long"], 2);
    let note = two["damage_screening"]["changed_note"].as_str().unwrap();
    assert!(
        note.contains("2 across by 2 along") && note.contains("2 across by 3 along"),
        "{note}"
    );
}

/// The damage cases follow the number along: with n tanks along the cases flood the holds H1C to HnC, and no more.
#[test]
fn the_damage_cases_follow_the_number_of_tanks_along() {
    for n in [2, 3, 4] {
        let (r, _) = run(json!({ "tanks_in_line": n }));
        let mut holds: Vec<String> = r["damage"]["cases"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|c| c["flooded"].as_array().unwrap().iter())
            .filter_map(|v| v.as_str())
            .filter(|v| v.starts_with('H') && v.ends_with('C'))
            .map(String::from)
            .collect();
        holds.sort();
        holds.dedup();
        let want: Vec<String> = (1..=n).map(|k| format!("H{k}C")).collect();
        assert_eq!(holds, want, "{n} along");
    }
}
