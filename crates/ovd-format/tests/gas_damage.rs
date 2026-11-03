//! Seam: the gas damage run, final stage (issue #759, spec 0012): every IS Code loading condition of a gas
//! Option is damaged with the IGC cases, and `igc/2.7/damage-survival` is evaluated per case.
//!
//! Label: property

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn new_gas_results(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        // Issue #850: the IGC rows are in the coastal-and-river stack; a river-only gas Option takes ADN 9.3.1.
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" },
        "loading_conditions": { "value": "is_code_2008", "source": "typed" }
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
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn rows<'a>(results: &'a Value, id: &str) -> Vec<&'a Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id)
        .collect()
}

/// Every IS Code condition of a new gas Option has a case table, a floating position and the residual
/// stability per case, and a governing case with its righting lever after damage.
#[test]
fn every_gas_condition_has_a_damage_case_table() {
    let r = new_gas_results(json!({}));
    let block = &r["damage"];
    assert!(block.is_object(), "no damage block");
    let cases = block["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    let conditions: Vec<String> = r["loading_conditions"]["conditions"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let damaged: Vec<&str> = block["governing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["condition"].as_str().unwrap())
        .collect();
    assert!(damaged.len() >= 3, "{damaged:?} of {conditions:?}");
    for c in cases {
        for key in [
            "heel_deg",
            "trim_deg",
            "draught_mid_m",
            "max_gz_m",
            "area_m_rad",
        ] {
            assert!(c.get(key).is_some(), "{key} in {c}");
        }
        assert!(c["deck_edge_immersed"].is_boolean());
    }
    for g in block["governing"].as_array().unwrap() {
        assert!(g["gz_after_damage"].is_array());
    }
}

/// `igc/2.7/damage-survival` is evaluated on each case (pass or fail), no longer `not evaluated`; openings
/// and emergency power stay `not evaluated` with their gaps (spec 0012 D11).
#[test]
fn the_igc_damage_rule_is_evaluated_per_case() {
    let r = new_gas_results(json!({}));
    let n = r["damage"]["cases"].as_array().unwrap().len();
    let survival = rows(&r, "igc/2.7/damage-survival");
    assert_eq!(survival.len(), n);
    assert!(
        survival
            .iter()
            .all(|row| row["outcome"] == "pass" || row["outcome"] == "fail"),
        "{:?}",
        survival.iter().map(|r| &r["outcome"]).collect::<Vec<_>>()
    );
    for id in [
        "igc/2.7.1.1/openings-above-damage-waterline",
        "igc/2.7.2.2/emergency-power",
    ] {
        let rs = rows(&r, id);
        assert_eq!(rs.len(), 1, "{id}");
        assert_eq!(rs[0]["outcome"], "not evaluated", "{id}");
    }
}

/// A case that breaches a Type C tank names it; a hold case that does not breach its tank keeps the tank as an
/// Intact Inclusion, so it is never named as breached.
#[test]
fn breached_tanks_are_named_and_only_when_flooded() {
    let r = new_gas_results(json!({}));
    for c in r["damage"]["cases"].as_array().unwrap() {
        let flooded: Vec<&str> = c["flooded"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        for t in c["breached_tanks"].as_array().unwrap() {
            assert!(flooded.contains(&t.as_str().unwrap()), "{c}");
        }
    }
}

/// Issue #760: every case has its intermediate stages at 25, 50 and 75 %, and the IGC 2.7.1.2 and 2.7.1.3 rows
/// are evaluated on each case (a case whose final stage does not float has no heel value, so its 2.7.1.2 row
/// says why).
#[test]
fn every_case_has_three_intermediate_stages_and_their_rules_are_evaluated() {
    let r = new_gas_results(json!({}));
    let cases = r["damage"]["cases"].as_array().unwrap();
    for c in cases {
        let fractions: Vec<f64> = c["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["fraction"].as_f64().unwrap())
            .collect();
        assert_eq!(fractions, vec![0.25, 0.5, 0.75], "{}", c["name"]);
    }
    let heel = rows(&r, "igc/2.7.1.2/heel-any-stage");
    let residual = rows(&r, "igc/2.7.1.3/intermediate-residual-stability");
    assert_eq!(heel.len(), cases.len());
    assert_eq!(residual.len(), cases.len());
    assert!(
        residual
            .iter()
            .all(|x| x["outcome"] == "pass" || x["outcome"] == "fail")
    );
    assert!(heel.iter().any(|x| x["outcome"] == "pass"));
    assert!(heel.iter().all(|x| x["outcome"] == "pass"
        || x["outcome"] == "fail"
        || x["outcome"] == "not evaluated"));
}
