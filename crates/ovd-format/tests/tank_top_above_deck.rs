//! Seam: the input `tank_top_above_deck_m` of issue #992 through a Design. The depth follows the tank
//! (the double bottom, the support and the tank outer diameter, less the standing height); 0 is tanks
//! in hold; a file without the input is unchanged.
//!
//! Label: rule. The depth rule is the maintainer's decision of 2026-11-07 and the design note of
//! issue #992 (D1a); the 0.3 m gap with the tanks in hold is an estimate.

use ovd_format::vdesign::LEGACY_FIXED_OTHER;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

/// The 140 x 29 m coastal LNG barge. `extra` is typed; the depth is left out unless `extra` has it.
fn design(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "min_freeboard_m": { "value": 4.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
}

fn run_with(file: &Value) -> (Value, Vec<String>) {
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let warnings = filled.report.warnings.clone();
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => (results, warnings),
        other => panic!("{other:?}"),
    }
}

fn run(file: &Value) -> Value {
    run_with(file).0
}

fn depth(r: &Value) -> f64 {
    r["dims"]["depth_m"].as_f64().unwrap()
}

#[test]
fn a_file_saved_before_has_no_standing_height_and_a_result_without_one_has_no_new_key() {
    let l = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "tank_top_above_deck_m")
        .unwrap();
    assert_eq!(l.value, "null");
    assert_eq!(
        Profile::built_in().default("tank_top_above_deck_m"),
        Some(&Value::Null)
    );
    let r = run(&design(json!({})));
    let a = r["arrangement"].as_object().unwrap();
    assert!(!a.contains_key("tank_top_above_deck_m"));
    assert!(!a.contains_key("depth_follows_tank"));
    // The depth is today's ratio default: 0.55 x 29 m.
    assert!((depth(&r) - 15.95).abs() < 1e-9, "{}", depth(&r));
}

#[test]
fn in_hold_the_depth_is_the_tank_top_plus_the_gap_and_a_standing_height_takes_it_off() {
    // Label: snapshot, printed by the code. Tank top above keel and diameter of the arrangement.
    let hold = run(&design(json!({ "tank_top_above_deck_m": 0.0 })));
    let top = hold["arrangement"]["tank_top_above_keel_m"]
        .as_f64()
        .unwrap();
    assert!(
        (depth(&hold) - (top + 0.3)).abs() < 0.0006,
        "{} vs {top}",
        depth(&hold)
    );
    assert_eq!(hold["arrangement"]["depth_follows_tank"], true);
    // Issue #935: the search screens by IGC 2.7 damage and refines a failing size down to about 0.1 m. The
    // two-across 13.28 m tanks this design chose before do not survive a side breach across the hold bulkhead; the
    // refined pick is two across at 11.72 m, which carries more than one 16 m tank (the first screen gave that).
    // Label: snapshot, printed by the code.
    assert_eq!(hold["arrangement"]["n_wide"], 2);
    assert!(
        (hold["arrangement"]["tank_od_m"].as_f64().unwrap() - 11.718).abs() < 0.001,
        "{}",
        hold["arrangement"]["tank_od_m"]
    );
    assert!((depth(&hold) - 13.621).abs() < 0.0006, "{}", depth(&hold));

    let two = run(&design(json!({ "tank_top_above_deck_m": 2.0 })));
    let top2 = two["arrangement"]["tank_top_above_keel_m"]
        .as_f64()
        .unwrap();
    // The depth is the tank top less the height, to the millimetre, for the tank it carries.
    assert!(
        (depth(&two) - (top2 - 2.0)).abs() < 0.0016,
        "{} vs {top2}",
        depth(&two)
    );
    // Issue #935: the two depths are for different picks, so they are not compared: with the tank top 2 m above
    // the deck the search picks one 16 m tank across at 15.603 m. Label: snapshot, printed by the code. The rule above (the depth follows the tank) holds for each.
    assert_eq!(two["arrangement"]["n_wide"], 1);
    assert_eq!(two["arrangement"]["tank_od_m"], 16.0);
    assert!((depth(&two) - 15.603).abs() < 0.0006, "{}", depth(&two));
}

#[test]
fn a_height_above_the_tank_diameter_is_refused_with_the_reason() {
    let r = run(&design(json!({ "tank_top_above_deck_m": 40.0 })));
    assert_eq!(r["arrangement"]["feasible"], false);
    let tried = serde_json::to_string(&r["fit_tried"]).unwrap();
    assert!(tried.contains("more than the tank diameter"), "{tried}");
}

#[test]
fn a_typed_depth_wins_and_a_warning_says_so() {
    let (r, warnings) = run_with(&design(
        json!({ "moulded_depth_m": 12.0, "tank_top_above_deck_m": 2.0 }),
    ));
    assert!((depth(&r) - 12.0).abs() < 1e-9);
    assert!(
        !r["arrangement"]
            .as_object()
            .unwrap()
            .contains_key("tank_top_above_deck_m")
    );
    assert!(
        warnings.iter().any(|w| w.contains("tank_top_above_deck_m")),
        "{warnings:?}"
    );
}

#[test]
fn the_profile_holds_the_maintainers_defaults_by_service_area_and_does_not_apply_them_yet() {
    let p: Value = serde_json::from_str(include_str!("../src/builtin_profile.json")).unwrap();
    let rule = &p["inputs"]["tank_top_above_deck_default_rule"]["default_by_service_area"];
    assert_eq!(rule["coastal-and-river"]["metres"], 2.0);
    assert_eq!(rule["river-only"]["od_fraction"], 0.5);
    // Not applied: a new Option still has no standing height.
    let r = run(&design(json!({})));
    assert!(
        !r["arrangement"]
            .as_object()
            .unwrap()
            .contains_key("tank_top_above_deck_m")
    );
}

/// Issue #992, review item 1. The depth is never less than the draught limit plus the least freeboard:
/// a shallow depth that follows a small tank must rise to it, not cost draught and cargo. The probe: a
/// river-only 86 x 11.4 m barge, a 3.0 m draught limit, the tank top to stand 2.5 m above the deck (it
/// built two across at 1.9 m, under the draught, before the floor).
/// Label: rule (the design note D1a).
#[test]
fn the_depth_is_never_less_than_the_draught_limit_plus_the_freeboard() {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 86.0, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "service_area": { "value": "river-only", "source": "typed" },
            "min_freeboard_m": { "value": 0.3, "source": "typed" },
            "draught_limit_m": { "value": 3.0, "source": "typed" },
            "tank_top_above_deck_m": { "value": 2.5, "source": "typed" }
        } }]
    });
    let r = run(&file);
    assert!(
        depth(&r) >= 3.0 + 0.3 - 1e-9,
        "depth {} under draught 3.0 + freeboard 0.3",
        depth(&r)
    );
    let by = r["arrangement"]["depth_set_by"].as_str().unwrap();
    assert!(by == "tank" || by == "freeboard", "{by}");
}

/// Issue #992, review item 2. The draught limit of the hull is the settled depth less the freeboard,
/// not the depth of the ratio default the run started from (140 x 29 m coastal at 0 m: depth about 15.2
/// m, but `draught_max_m` came out 11.95 = 15.95 - 4.0).
#[test]
fn the_draught_limit_follows_the_settled_depth_not_the_seed() {
    for h in [0.0, 2.0] {
        let r = run(&design(json!({ "tank_top_above_deck_m": h })));
        let max = r["results"]["draught_max_m"].as_f64().unwrap();
        assert!(
            (max - (depth(&r) - 4.0)).abs() < 0.002,
            "h {h}: draught_max {max} vs depth {} - 4.0",
            depth(&r)
        );
    }
}
