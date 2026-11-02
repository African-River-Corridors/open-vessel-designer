//! Issue #822: the air draft limit reads the river top at both check sites, with the cargo vent
//! mast in it unless the mast is lowerable.
//!
//! Label: property. The two sites are the capacity check `air draft within limit` and the
//! per-condition check `Air draft` of the IS Code loading conditions. A limit just above the light
//! air draft of a hull with a lowerable mast passes both; the same limit fails both when the mast
//! stays up, because the mast's exit then stands above the river top the check reads.

use serde_json::{Value, json};

fn run(lowerable: bool, limit: Option<f64>, tanks_across: Option<i64>) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gas_igc_2g_self_propelled.vdesign");
    let mut design: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let option = &mut design["options"][0];
    let mut put = |k: &str, v: Value| option["inputs"][k] = json!({"value": v, "source": "typed"});
    put("vent_mast_lowerable", json!(lowerable));
    // The lightship condition, whose air draft is the light air draft of the capacity check.
    put("may_sail_without_ballast", json!(true));
    if let Some(l) = limit {
        put("air_draft_limit_m", json!(l));
    }
    // One arrangement only, so the search cannot answer a limit with a lower hull.
    if let Some(n) = tanks_across {
        put("tanks_across", json!(n));
    }
    option.as_object_mut().unwrap().remove("results_hash");
    let checked = ovd_format::check(&design.to_string(), &ovd_format::Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let mut runs = ovd_format::run_design(&checked.design.unwrap());
    match runs.remove(0).outcome {
        ovd_format::RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn check_ok(results: &Value, name: &str) -> Vec<bool> {
    results["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"] == name)
        .map(|c| c["ok"].as_bool().unwrap())
        .collect()
}

#[test]
fn a_limit_just_above_the_light_air_draft_passes_a_lowerable_mast_and_fails_a_mast_that_stays_up() {
    let free = run(true, None, None);
    let light_lowered = free["results"]["air_draft_light_m"].as_f64().unwrap();
    let across = free["arrangement"]["n_wide"].as_i64().unwrap();
    let limit = light_lowered + 0.05;

    let lowered = run(true, Some(limit), Some(across));
    // Site 1, the capacity check: passes.
    assert_eq!(check_ok(&lowered, "air draft within limit"), [true]);
    // Site 2, the per-condition check: no condition fails it, so it has no row, and every
    // condition that carries an air draft says ok.
    assert!(check_ok(&lowered, "Air draft").is_empty());
    let conditions = lowered["stability"]["conditions"].as_array().unwrap();
    assert!(conditions.iter().all(|c| c["air_draft_ok"] == json!(true)));

    let up = run(false, Some(limit), Some(across));
    assert_eq!(check_ok(&up, "air draft within limit"), [false]);
    let per_condition = check_ok(&up, "Air draft");
    assert!(!per_condition.is_empty());
    assert!(per_condition.iter().all(|ok| !ok));
    let conditions = up["stability"]["conditions"].as_array().unwrap();
    assert!(conditions.iter().any(|c| c["air_draft_ok"] == json!(false)));
}

#[test]
fn the_mast_and_its_distances_are_in_the_results() {
    let r = run(false, None, None);
    let res = &r["results"];
    for key in [
        "vent_mast_x_m",
        "vent_exit_above_keel_m",
        "vent_exit_above_deck_m",
        "vent_exit_above_working_area_m",
        "vent_distance_to_openings_m",
        "vent_distance_to_accommodation_m",
        "vent_mast_distance_target_m",
    ] {
        assert!(res[key].is_number(), "{key}");
    }
    assert!(res["vent_mast_distance_met"].is_boolean());
    // The light air draft is the river top less the light draught, and the river top is the exit
    // (the mast stays up): the exit stands above everything else on this hull.
    let exit = res["vent_exit_above_keel_m"].as_f64().unwrap();
    let light_draught = res["light_draught_m"].as_f64().unwrap();
    assert!((res["air_draft_light_m"].as_f64().unwrap() - (exit - light_draught)).abs() < 1e-9);
}

#[test]
fn the_vent_measures_are_in_the_registry_with_a_unit_in_metres() {
    for id in [
        "vent.exit_above_deck",
        "vent.exit_above_working_area",
        "vent.distance_to_openings",
        "vent.distance_to_accommodation",
    ] {
        let info = ovd_format::trace::measure_info(id).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(info.unit, "m");
    }
}
