//! Seam: the session call that makes a Design from typed choices, and the example list (issue #1079,
//! spec 0017, D4 and D6): the call the desktop command and the WebAssembly session forward.
//!
//! Label: property - the Design the call makes is the Design `ovd fill --as-new` makes from a short
//! file of the same typed inputs: the same inputs, the same sources and the same results hash. No
//! fixed number is pinned.

use ovd_format::{Profile, fill_as_new};
use ovd_shell::AppState;
use serde_json::{Map, Value, json};

/// The 11 typed inputs of the coastal LNG barge example.
fn lng_barge() -> Map<String, Value> {
    json!({
        "cargo": "LNG",
        "hull_form": "raked_box",
        "formation": "self_propelled",
        "length_overall_m": 140.0,
        "beam_m": 29.0,
        "service_area": "coastal-and-river",
        "moulded_depth_m": 12.0,
        "max_speed_kn": 12.0,
        "max_tank_od_m": 12.0,
        "tanks_in_line": 2,
        "min_freeboard_m": 1.81,
    })
    .as_object()
    .unwrap()
    .clone()
}

fn pushed_barge() -> Map<String, Value> {
    json!({
        "cargo": "bauxite",
        "hull_form": "raked_box",
        "formation": { "kind": "pushed_barge" },
        "length_overall_m": 76.5,
        "beam_m": 11.4,
        "service_area": "river-only",
        "draught_limit_m": 3.0,
        "air_draft_limit_m": 9.0,
    })
    .as_object()
    .unwrap()
    .clone()
}

fn by_class() -> Map<String, Value> {
    json!({
        "cargo": "bauxite",
        "hull_form": "raked_box",
        "vessel_class": "CEMT-IV",
        "moulded_depth_m": 3.5,
        "service_area": "river-only",
        "draught_limit_m": 2.8,
        "air_draft_limit_m": 7.0,
    })
    .as_object()
    .unwrap()
    .clone()
}

/// The short file the CLI reads, as `ovd fill --as-new` does: the typed inputs of one Option.
fn short_file(name: &str, inputs: &Map<String, Value>) -> String {
    json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.0.0",
        "options": [{ "name": name, "inputs": inputs }],
    })
    .to_string()
}

fn text_of_cli(name: &str, inputs: &Map<String, Value>) -> String {
    let filled = fill_as_new(&short_file(name, inputs), &Profile::built_in());
    assert!(filled.report.is_ok(), "{:?}", filled.report.errors);
    filled.design.unwrap().to_text()
}

fn text_of_call(name: &str, inputs: &Map<String, Value>) -> String {
    let state = AppState::new_design_from(name, inputs).unwrap();
    state.session().design().to_text()
}

#[test]
fn the_11_typed_inputs_make_the_design_fill_as_new_makes() {
    let inputs = lng_barge();
    let call = text_of_call("Coastal LNG barge", &inputs);
    assert_eq!(call, text_of_cli("Coastal LNG barge", &inputs));
    let design: Value = serde_json::from_str(&call).unwrap();
    let option = &design["options"][0];
    assert_eq!(option["name"], "Coastal LNG barge");
    assert!(option["results_hash"].is_string());
    // Typed as typed; nothing of the starter stays behind.
    assert_eq!(option["inputs"]["cargo"]["value"], "LNG");
    assert_eq!(option["inputs"]["cargo"]["source"], "typed");
    assert_eq!(option["inputs"]["moulded_depth_m"]["value"], 12.0);
    assert_eq!(option["inputs"]["moulded_depth_m"]["source"], "typed");
    let other = &option["inputs"]["fill_ratio"];
    assert_eq!(other["source"], "default");
}

#[test]
fn a_dry_bulk_pushed_barge_and_a_class_option_make_the_same_design_as_fill_as_new() {
    for (name, inputs) in [("Barge", pushed_barge()), ("Class", by_class())] {
        assert_eq!(text_of_call(name, &inputs), text_of_cli(name, &inputs));
    }
    // A Vessel Class works as in the Input panel: length, beam and Formation come from the class.
    let design: Value = serde_json::from_str(&text_of_call("Class", &by_class())).unwrap();
    let inputs = &design["options"][0]["inputs"];
    assert_eq!(inputs["vessel_class"]["source"], "class");
    assert_eq!(inputs["length_overall_m"]["source"], "class");
    assert_eq!(inputs["beam_m"]["source"], "class");
}

#[test]
fn a_choice_the_schema_refuses_gives_the_schemas_error_and_leaves_the_open_design_alone() {
    let open = AppState::new_design().unwrap();
    let before = open.session().design().to_text();
    let mut unknown = lng_barge();
    unknown.insert("no_such_input".into(), json!(1));
    let mut too_deep = lng_barge();
    too_deep.insert("beam_m".into(), json!(-3.0));
    let mut no_limits = pushed_barge();
    no_limits.remove("draught_limit_m");
    no_limits.remove("air_draft_limit_m");
    for inputs in [unknown, too_deep, no_limits] {
        let filled = fill_as_new(&short_file("o", &inputs), &Profile::built_in());
        let want = filled.report.errors.first().cloned().unwrap();
        let got = AppState::new_design_from("o", &inputs).err().unwrap();
        assert_eq!(got, want);
    }
    assert_eq!(open.session().design().to_text(), before);
}

#[test]
fn the_example_list_has_the_barge_and_its_inputs_make_the_design_of_the_first_test() {
    let list = AppState::design_examples();
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1);
    let example = &list[0];
    assert_eq!(example["id"], "coastal-lng-barge");
    assert_eq!(example["title"], "Coastal LNG barge, 140 × 29 m");
    assert!(!example["text"].as_str().unwrap().is_empty());
    let inputs = example["inputs"].as_object().unwrap();
    assert_eq!(inputs.len(), 11);
    assert_eq!(inputs, &lng_barge());
    let title = example["title"].as_str().unwrap();
    assert_eq!(
        text_of_call(title, inputs),
        text_of_call(title, &lng_barge())
    );
    assert_eq!(text_of_call(title, inputs), text_of_cli(title, &lng_barge()));
}
