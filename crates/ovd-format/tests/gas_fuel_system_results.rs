//! The gas fuel system of a gas-fuelled Option, as mass and cost items of its own (issue #775).
//!
//! Label: estimate. No source was read for any allowance; the expected values are the hand sums of
//! the stated estimate rates (see `ovd-core/tests/gas_fuel_system.rs`).

use ovd_format::rule_set::Library;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn results(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
        "draught_limit_m": { "value": 12.0, "source": "typed" },
        "air_draft_limit_m": { "value": 40.0, "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let design = filled.design.unwrap();
    match run_option_with(&design.options[0], &Library::built_in(), &[]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn label_t(items: &Value, label: &str) -> Option<f64> {
    items
        .as_array()?
        .iter()
        .find(|i| i["label"] == label)
        .and_then(|i| i["mass_t"].as_f64())
}

#[test]
fn a_gas_fuelled_option_shows_the_gas_system_as_mass_items_marked_estimate() {
    let r = results(json!({ "stern_block_mass_included": true }));
    let block = &r["stern_block_mass"];
    let f = &block["gas_fuel_system"];
    let kwe = f["served_kwe"].as_f64().unwrap();
    assert!(kwe > 0.0);
    assert_eq!(f["reliquefaction"], json!("not modelled"));
    assert_eq!(f["gas_combustion_unit"], json!(false));
    let n = r["drawing"]["aft_arrangement"]["n_gensets"]
        .as_f64()
        .unwrap();
    let t = |l: &str| label_t(&f["items"], l).unwrap();
    assert!((t("gas valve units") - 0.4 * n).abs() < 1e-9);
    assert!((t("gas heater") - 0.0002 * kwe).abs() < 1e-9);
    assert!((t("gas compressor") - 0.0006 * kwe).abs() < 1e-9);
    assert!(label_t(&f["items"], "gas combustion unit").is_none());
    for i in f["items"].as_array().unwrap() {
        assert_eq!(i["status"], json!("estimate"), "{i}");
        assert!(i["basis"].as_str().unwrap().starts_with("estimate:"), "{i}");
    }
    // The same items stand in the stern block's mass items, so the lightship carries them.
    for l in ["gas valve units", "gas heater", "gas compressor"] {
        assert_eq!(label_t(&block["items"], l), label_t(&f["items"], l), "{l}");
    }
}

#[test]
fn the_gas_combustion_unit_input_adds_its_item() {
    let r = results(json!({ "stern_block_mass_included": true, "gas_combustion_unit": true }));
    let f = &r["stern_block_mass"]["gas_fuel_system"];
    assert_eq!(f["gas_combustion_unit"], json!(true));
    assert_eq!(label_t(&f["items"], "gas combustion unit"), Some(2.0));
    assert_eq!(
        label_t(&r["stern_block_mass"]["items"], "gas combustion unit"),
        Some(2.0)
    );
}

#[test]
fn the_cost_has_a_gas_fuel_system_line_of_its_own() {
    let r = results(json!({ "stern_block_mass_included": true }));
    // Hand sum for the 110 x 11.4 m hull: two gas sets of 1,200 kWe each (the engine room the layout
    // gives holds two), so 2 x 1,200 = 2,400 kWe.
    let aft = &r["drawing"]["aft_arrangement"];
    assert_eq!(aft["n_gensets"].as_f64(), Some(2.0));
    assert_eq!(aft["genset_kwe"].as_f64(), Some(1200.0));
    let kwe = json!(2400.0);
    assert_eq!(r["cost_masses"]["stern_items"]["gas_fuel_system_kw"], kwe);
    let line = r["cost"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["item"] == "gas_fuel_system")
        .unwrap();
    assert_eq!(line["quantity"], kwe);
    assert_eq!(line["rate"], json!(200.0));
}

#[test]
fn a_hull_that_is_not_gas_fuelled_has_none_of_it() {
    let r = results(json!({
        "cargo": "bauxite",
        "stern_block_mass_included": true,
        "gas_combustion_unit": true
    }));
    assert!(r["stern_block_mass"]["gas_fuel_system"].is_null());
    // No gas gensets: the cost line has no quantity.
    let kw = &r["cost_masses"]["stern_items"]["gas_fuel_system_kw"];
    assert!(kw.is_null() || *kw == json!(0.0), "{kw}");
}
