//! Issue #1083 (spec 0017, D8 and D9): the Required group holds all six required inputs, and each
//! schema entry says whether the input is essential.

use ovd_format::Profile;
use ovd_format::session::input_schema;
use serde_json::Value;

/// The inputs that are essential and not required (spec 0017, D9).
const ESSENTIAL_NOT_REQUIRED: &[&str] = &[
    "moulded_depth_m",
    "min_freeboard_m",
    "max_speed_kn",
    "design_speed_kn",
    "power_source",
    "propulsor_type",
    "max_tank_od_m",
    "tanks_across",
    "tanks_in_line",
    "fill_ratio",
    "hold_covering",
    "draught_limit_m",
    "air_draft_limit_m",
    "lock_length_m",
    "lock_width_m",
];

fn inputs() -> Vec<Value> {
    input_schema(&Profile::built_in())["inputs"]
        .as_array()
        .expect("inputs")
        .clone()
}

#[test]
fn the_six_required_inputs_are_in_the_required_group_in_order() {
    let names: Vec<String> = inputs()
        .iter()
        .filter(|i| i["group"] == "Required")
        .map(|i| i["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        names,
        [
            "cargo",
            "formation",
            "length_overall_m",
            "beam_m",
            "hull_form",
            "service_area"
        ]
    );
}

#[test]
fn only_hull_form_and_service_area_changed_group() {
    // Every other input keeps the group it had: Hull Form and Rules for the two that moved.
    for i in inputs() {
        let name = i["name"].as_str().unwrap();
        if i["group"] == "Required" {
            assert!(i["required"].as_bool().unwrap(), "{name} is not required");
        }
        if name == "hull_form" || name == "service_area" {
            assert_eq!(i["group"], "Required", "{name}");
        }
    }
    let groups = |g: &str| {
        inputs()
            .iter()
            .filter(|i| i["group"] == g)
            .count()
    };
    assert!(groups("Hull Form") > 0, "the other Hull Form inputs stay");
    assert!(groups("Rules") > 0, "the other Rules inputs stay");
}

#[test]
fn every_entry_has_the_essential_flag_and_the_true_set_is_the_list() {
    let mut want: Vec<&str> = ESSENTIAL_NOT_REQUIRED.to_vec();
    let mut got = Vec::new();
    for i in inputs() {
        let name = i["name"].as_str().unwrap();
        let flag = i["essential"]
            .as_bool()
            .unwrap_or_else(|| panic!("{name} has no essential flag"));
        if i["required"].as_bool().unwrap() {
            assert!(flag, "required input {name} is not essential");
        } else if flag {
            got.push(name.to_string());
        }
    }
    want.sort();
    got.sort();
    assert_eq!(got, want);
}
