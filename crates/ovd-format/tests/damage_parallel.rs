//! Seam: the damage cases run on worker threads give the serial results, bit for bit (issue #921).
//!
//! Label: property - serial and parallel runs of the 140 x 29 m coastal LNG reference design, coastal and
//! river-only, give byte-equal results: every damage block, the damage by rule, the rule trace. The stored
//! hash cannot see the damage blocks (they are unhashed, issue #895), so the bytes are compared. Each pair
//! runs three times.

use ovd_format::damage::set_damage_workers;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

const EXAMPLE: &str =
    include_str!("../../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign");

/// The first Option of the example, written as a new file: the inputs stored as defaults are left out, so
/// `fill_as_new` writes in today's Defaults Profile (the rule sets that hold the damage tables), and
/// `service_area` is set.
fn results_of(service_area: &str) -> Value {
    let mut file: Value = serde_json::from_str(EXAMPLE).unwrap();
    let mut option = file["options"][0].clone();
    let inputs = option["inputs"].as_object_mut().unwrap();
    inputs.retain(|_, v| v["source"] != "default");
    inputs.insert(
        "service_area".into(),
        json!({ "value": service_area, "source": "typed" }),
    );
    // Issue #1034: a river-only Option needs a number for both river limits (the example's own stay).
    if service_area == "river-only" {
        for (name, v) in [("draught_limit_m", 12.0), ("air_draft_limit_m", 40.0)] {
            inputs
                .entry(name)
                .or_insert(json!({ "value": v, "source": "typed" }));
        }
    }
    option.as_object_mut().unwrap().remove("results_hash");
    file["options"] = json!([option]);
    file.as_object_mut().unwrap().remove("defaults_profile");
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// Issue #961: a liquid bulk hull of the crude integral-tank fixtures (ADN, and MARPOL under the coastal one),
/// run as it is stored.
fn liquid_results_of(name: &str) -> Value {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let checked = ovd_format::check(
        &std::fs::read_to_string(path).unwrap(),
        &Profile::built_in(),
    );
    assert_eq!(checked.report.errors, Vec::<String>::new());
    match run_option(&checked.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// The damage cases a results file holds: the IGC block, or the cases of every rule in `damage_by_rule`.
fn damage_case_count(results: &Value) -> usize {
    let own = results["damage"]["cases"].as_array().map_or(0, Vec::len);
    let by_rule: usize = results["damage_by_rule"]
        .as_object()
        .map(|o| {
            o.values()
                .map(|b| b["cases"].as_array().map_or(0, Vec::len))
                .sum()
        })
        .unwrap_or(0);
    own + by_rule
}

#[test]
fn serial_and_parallel_damage_results_are_byte_equal() {
    for area in ["coastal-and-river", "river-only"] {
        for round in 1..=3 {
            set_damage_workers(1);
            let serial = results_of(area);
            set_damage_workers(4);
            let parallel = results_of(area);
            assert!(
                damage_case_count(&serial) > 0,
                "{area}: the run has no damage cases, so the test proves nothing"
            );
            assert_eq!(
                serde_json::to_string(&serial).unwrap(),
                serde_json::to_string(&parallel).unwrap(),
                "{area}, round {round}: the parallel results differ from the serial ones"
            );
        }
    }
    // Liquid hulls (issue #961): the ADN and MARPOL cases go through the same runner. The knob is process-wide,
    // so this stays in the one test.
    for name in [
        "crude_integral_tanks_pushed",
        "crude_integral_tanks_coastal",
    ] {
        for round in 1..=3 {
            set_damage_workers(1);
            let serial = liquid_results_of(name);
            set_damage_workers(4);
            let parallel = liquid_results_of(name);
            let cases = serial["damage"].as_array().map_or(0, Vec::len)
                + serial["marpol_damage"].as_array().map_or(0, Vec::len);
            assert!(cases > 4, "{name}: only {cases} liquid damage cases");
            assert_eq!(
                serde_json::to_string(&serial).unwrap(),
                serde_json::to_string(&parallel).unwrap(),
                "{name}, round {round}: the parallel results differ from the serial ones"
            );
        }
    }
    set_damage_workers(0);
}
