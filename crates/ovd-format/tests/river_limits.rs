//! Seam: the two river limits of a river-only Option (issue #1034, Phase 2). A river-only Option must carry a
//! number for draught_limit_m and air_draft_limit_m, typed or filled by a Vessel Class. There is no
//! "no limit" for it. A coastal-and-river Option may leave them out. A saved file that stores no value keeps
//! its results and hash, with a validation warning.
//!
//! Label: property. Each test states a rule of the required answer (one message, a class fill counts, a typed
//! number runs, a stored null keeps its hash); no number here is a printed result.

use ovd_format::{DesignSession, Profile, RunOutcome, fill, fill_as_new, run_option};
use serde_json::{Value, json};

fn file(area: &str, extra: Value) -> String {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "service_area": { "value": area, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = v.clone();
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
    .to_string()
}

fn typed(v: f64) -> Value {
    json!({ "value": v, "source": "typed" })
}

fn needs_number(errors: &[String]) -> Vec<&String> {
    errors
        .iter()
        .filter(|e| e.contains("river-only Option needs a number"))
        .collect()
}

#[test]
fn a_new_river_only_option_with_neither_limit_gets_one_message_naming_both() {
    let filled = fill_as_new(&file("river-only", json!({})), &Profile::built_in());
    let one = needs_number(&filled.report.errors);
    assert_eq!(one.len(), 1, "{:?}", filled.report.errors);
    assert!(one[0].contains("draught_limit_m and air_draft_limit_m"));
}

#[test]
fn a_typed_null_is_not_an_answer_for_a_new_river_only_option() {
    let text = file(
        "river-only",
        json!({
            "draught_limit_m": { "value": null, "source": "typed" },
            "air_draft_limit_m": { "value": null, "source": "typed" }
        }),
    );
    let filled = fill_as_new(&text, &Profile::built_in());
    assert_eq!(
        needs_number(&filled.report.errors).len(),
        1,
        "{:?}",
        filled.report.errors
    );
}

#[test]
fn the_message_names_only_the_limit_that_is_missing() {
    let text = file("river-only", json!({ "draught_limit_m": typed(3.0) }));
    let filled = fill_as_new(&text, &Profile::built_in());
    let one = needs_number(&filled.report.errors);
    assert_eq!(one.len(), 1, "{:?}", filled.report.errors);
    assert!(one[0].contains("air_draft_limit_m") && !one[0].contains("draught_limit_m"));
}

#[test]
fn both_numbers_typed_make_the_option_run() {
    let text = file(
        "river-only",
        json!({ "draught_limit_m": typed(3.5), "air_draft_limit_m": typed(9.1) }),
    );
    let filled = fill_as_new(&text, &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let option = &filled.design.unwrap().options[0];
    assert!(matches!(run_option(option), RunOutcome::Results { .. }));
}

#[test]
fn a_vessel_class_pick_answers_both_limits() {
    let mut s = DesignSession::new_design_in(Profile::built_in(), "river-only").unwrap();
    assert!(
        s.design().options[0].input("draught_limit_m").is_none(),
        "no value until answered"
    );
    s.set_input("Option 1", "vessel_class", json!("CEMT-IV"))
        .unwrap();
    let o = &s.design().options[0];
    for name in ["draught_limit_m", "air_draft_limit_m"] {
        let i = o.input(name).unwrap();
        assert!(i.value.is_number(), "{name}: {:?}", i.value);
    }
}

#[test]
fn a_session_option_with_no_limit_fails_a_run_with_the_one_message() {
    let s = DesignSession::new_design_in(Profile::built_in(), "river-only").unwrap();
    match run_option(&s.design().options[0]) {
        RunOutcome::Failed(m) => {
            assert!(m.contains("draught_limit_m and air_draft_limit_m"), "{m}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_coastal_and_river_option_with_no_limits_gets_no_new_message() {
    let filled = fill_as_new(&file("coastal-and-river", json!({})), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    assert!(
        !filled
            .report
            .warnings
            .iter()
            .any(|m| m.contains("river-only")),
        "{:?}",
        filled.report.warnings
    );
}

#[test]
fn a_saved_river_only_file_with_default_nulls_keeps_its_hash_and_warns_in_validation() {
    let stored = file(
        "river-only",
        json!({
            "draught_limit_m": { "value": null, "source": "default" },
            "air_draft_limit_m": { "value": null, "source": "default" }
        }),
    );
    let p = Profile::built_in();
    let filled = fill(&stored, &p);
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let legacy = filled
        .report
        .warnings
        .iter()
        .filter(|w| w.contains("saved before the rule"))
        .count();
    assert_eq!(legacy, 1, "{:?}", filled.report.warnings);
    let RunOutcome::Results { results, hash, .. } = run_option(&filled.design.unwrap().options[0])
    else {
        panic!("a saved file still runs");
    };
    let in_results = results["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w.as_str().unwrap().contains("saved before the rule"));
    assert!(!in_results, "a validation warning, not a results warning");
    // The stored results and hash of such files on `main` are held by `tools/check_fixture_hashes.py`.
    assert!(!hash.is_empty());
}

#[test]
fn an_air_draft_limit_above_a_typed_bridge_clearance_warns() {
    let warns = |limit: f64| {
        let text = file(
            "river-only",
            json!({
                "draught_limit_m": typed(3.5), "air_draft_limit_m": typed(limit),
                "bridge_clearance_m": typed(7.0)
            }),
        );
        fill_as_new(&text, &Profile::built_in())
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("lowest bridge given"))
            .count()
    };
    assert_eq!(warns(9.1), 1);
    assert_eq!(warns(7.0), 0);
    assert_eq!(warns(6.0), 0);
}
