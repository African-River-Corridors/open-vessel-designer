//! Seam: the air draft report and the bridge clearance warning (issue #991). The report gives the air draft
//! light and laden and names the highest point; a bridge clearance that the air draft is over gives a
//! warning, never a block.
//!
//! Label: property. Each test states a relation between the run and its report: the report agrees with the
//! results, the named point is the one whose top sets the air draft, a limit under the air draft warns and
//! one above it does not, and the design stands either way. No number here is a printed result.

use ovd_format::rule_set::Library;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn results(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
        "draught_limit_m": { "value": 12.0, "source": "typed" },
        "air_draft_limit_m": { "value": 40.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option_with(
        &filled.design.unwrap().options[0],
        &Library::built_in(),
        &[],
    ) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn warnings(r: &Value) -> Vec<String> {
    r["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_report_agrees_with_the_results_and_names_the_highest_point() {
    let r = results(json!({}));
    let report = &r["air_draft_report"];
    assert_eq!(report["light_m"], r["results"]["air_draft_light_m"]);
    assert_eq!(report["laden_m"], r["results"]["air_draft_loaded_m"]);
    let key = report["highest_point"]["key"].as_str().unwrap();
    assert!(
        ["tank_dome", "stern_block", "deckhouse", "vent_mast"].contains(&key),
        "{key}"
    );
    assert!(report["highest_point"]["label"].as_str().unwrap().len() > 3);
    // The light air draft is the top less the light draught, so the top less the laden draught is the laden one.
    let top = report["highest_point"]["top_above_keel_m"]
        .as_f64()
        .unwrap();
    let light = report["light_m"].as_f64().unwrap();
    let laden = report["laden_m"].as_f64().unwrap();
    assert!(top >= laden && light >= laden, "{top} {light} {laden}");
    assert!(report["bridge_limit"].is_null());
}

#[test]
fn a_bridge_clearance_under_the_air_draft_warns_and_names_the_highest_point_and_its_source() {
    let r = results(json!({ "bridge_clearance_m": 3.0 }));
    let report = &r["air_draft_report"];
    assert_eq!(report["bridge_limit"]["limit_m"], json!(3.0));
    assert_eq!(report["bridge_limit"]["exceeded"], json!(true));
    assert_eq!(
        report["bridge_limit"]["source"],
        json!("typed input bridge_clearance_m")
    );
    let label = report["highest_point"]["label"].as_str().unwrap();
    let warn = warnings(&r)
        .into_iter()
        .find(|w| w.contains("bridge limit"))
        .expect("a bridge limit warning");
    assert!(warn.contains(label), "{warn}");
    assert!(warn.contains("typed input bridge_clearance_m"), "{warn}");
}

#[test]
fn a_bridge_clearance_over_the_air_draft_gives_no_warning() {
    let r = results(json!({ "bridge_clearance_m": 100.0 }));
    assert_eq!(
        r["air_draft_report"]["bridge_limit"]["exceeded"],
        json!(false)
    );
    assert!(warnings(&r).iter().all(|w| !w.contains("bridge limit")));
}

#[test]
fn the_design_stands_when_the_bridge_clearance_is_exceeded() {
    // A warning, never a block: the same design with and without the limit has the same hash-bearing
    // capacity, and no check fails because of it.
    let with = results(json!({ "bridge_clearance_m": 3.0 }));
    let without = results(json!({}));
    assert_eq!(with["results"]["cargo_t"], without["results"]["cargo_t"]);
    let failed = |r: &Value| -> Vec<String> {
        r["checks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["ok"] == json!(false) && c["severity"] == json!("error"))
            .map(|c| c["name"].to_string())
            .collect()
    };
    assert_eq!(failed(&with), failed(&without));
    assert_eq!(
        with["rule_trace"]["verdict"],
        without["rule_trace"]["verdict"]
    );
}

#[test]
fn the_named_point_is_the_one_whose_top_sets_the_air_draft() {
    // The light air draft is the top of the named point less the light draught; a vent mast that stands
    // highest is the one the results give as the vent exit.
    let r = results(json!({}));
    let report = &r["air_draft_report"];
    let top = report["highest_point"]["top_above_keel_m"]
        .as_f64()
        .unwrap();
    let light = r["results"]["air_draft_light_m"].as_f64().unwrap();
    let draught = r["results"]["light_draught_m"].as_f64().unwrap();
    assert!(
        (top - draught - light).abs() < 1e-9,
        "{top} {draught} {light}"
    );
    if report["highest_point"]["key"] == "vent_mast" {
        assert_eq!(
            top,
            r["results"]["vent_exit_above_keel_m"].as_f64().unwrap()
        );
    }
    // A dry bulk hull names its hatch coaming (a pushed barge carries no stern block).
    let dry = dry_results(json!({}));
    assert_eq!(
        dry["air_draft_report"]["highest_point"]["key"],
        json!("hatch_coaming")
    );
}

fn dry_results(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "bauxite", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "pushed_barge" }, "source": "typed" },
        "length_overall_m": { "value": 76.5, "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
        "draught_limit_m": { "value": 12.0, "source": "typed" },
        "air_draft_limit_m": { "value": 40.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 3.5, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    match run_option_with(
        &filled.design.unwrap().options[0],
        &Library::built_in(),
        &[],
    ) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_light_draught_that_is_not_worked_is_not_read_as_under_the_limit() {
    // `crude_integral_tanks_self_propelled` has no light draught worked (it is not a number: issue #1012), so its light air draft is not a number. A typed limit under the laden air draft still warns,
    // on the laden value, and says so.
    let path = format!(
        "{}/tests/fixtures/crude_integral_tanks_self_propelled.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    file["options"][0]["inputs"]["bridge_clearance_m"] = json!({ "value": 3.0, "source": "typed" });
    let design = ovd_format::check(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let r = match ovd_format::run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    };
    let limit = &r["air_draft_report"]["bridge_limit"];
    assert_eq!(limit["exceeded"], json!(true), "{limit}");
    assert!(
        limit["condition"].as_str().unwrap().starts_with("laden"),
        "{limit}"
    );
    assert!(
        warnings(&r)
            .iter()
            .any(|w| w.contains("bridge limit")
                && w.contains("laden (the light draught is not worked)")),
        "{:?}",
        warnings(&r)
    );
}
