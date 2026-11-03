//! Issue #850: a river-only gas Option takes ADN 2025 9.3.1 (`adn-gas`) as its design Rule Set, in place
//! of the IGC Code (`igc-2g`), with the two IGC 2.4.1.2 tank distance rows (`igc-tank-clearance`) and
//! `lr-inland-gas` beside it. A coastal-and-river Option keeps `igc-2g`. A file that stores its Rule Sets
//! keeps them; a gas file that stores none, with no Service Area or a river-only one, is read with the ADN
//! stack.
//!
//! Label: sourced - ADN 2025 (ECE/TRANS/351) 9.3.1.11.2 (a), printed p. 510: wing at least 0.80 m,
//! double bottom at least 0.60 m; 9.3.1.22.3, printed p. 523: the relief valve outlets at least 2.00 m
//! above the deck and at least 6.00 m from the accommodation. The boundary cases of the outlet rows are
//! in the evaluation harness (`evaluation/cases/vent_*.yaml`); the sizing of each term is in
//! `crates/ovd-core/tests/adn_hull_limits.rs`.

use ovd_format::{Profile, RunOutcome, check, fill, fill_as_new, run_option};
use serde_json::{Value, json};

fn file_of(service_area: Option<&str>) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    if let Some(a) = service_area {
        inputs["service_area"] = json!({ "value": a, "source": "typed" });
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
}

fn results_of(file: &Value, as_new: bool) -> Value {
    let text = file.to_string();
    let profile = Profile::built_in();
    let filled = if as_new {
        fill_as_new(&text, &profile)
    } else {
        fill(&text, &profile)
    };
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn sets(results: &Value) -> Vec<String> {
    results["rule_trace"]["rule_sets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn rows<'a>(results: &'a Value, prefix: &str) -> Vec<&'a Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"].as_str().is_some_and(|x| x.starts_with(prefix)))
        .collect()
}

fn has_set(results: &Value, id: &str) -> bool {
    sets(results)
        .iter()
        .any(|s| s.split('@').next() == Some(id))
}

#[test]
fn a_river_only_gas_option_traces_adn_and_lr_rows_and_two_igc_rows() {
    let results = results_of(&file_of(Some("river-only")), true);
    for id in ["adn-gas", "igc-tank-clearance", "lr-inland-gas", "is2008-a"] {
        assert!(has_set(&results, id), "{id} in {:?}", sets(&results));
    }
    assert!(!has_set(&results, "igc-2g"), "{:?}", sets(&results));
    assert!(!rows(&results, "adn/").is_empty());
    assert!(!rows(&results, "lr-inland/").is_empty());
    // The only IGC rows are the two that size the tank, each with the note that says so.
    let igc = rows(&results, "igc/");
    let mut ids: Vec<&str> = igc.iter().map(|r| r["rule"].as_str().unwrap()).collect();
    ids.dedup();
    assert_eq!(
        ids,
        ["igc/2.4.1.2/side-clearance", "igc/2.4.1.2/bottom-clearance"]
    );
    for r in &igc {
        let note = r["note"].as_str().expect("the sizing rows carry a note");
        assert!(note.contains("sizes the tank") && note.contains("9.3.1.11.2 (a)"));
        assert_eq!(r["sizes"], true);
        assert_eq!(r["outcome"], "pass");
    }
    // No other row has a note.
    assert!(
        results["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r.get("note").is_some())
            .all(|r| r["rule"].as_str().unwrap().starts_with("igc/2.4.1.2/"))
    );
}

#[test]
fn a_coastal_and_river_gas_option_keeps_igc_and_has_no_adn_rows() {
    let results = results_of(&file_of(Some("coastal-and-river")), true);
    assert!(has_set(&results, "igc-2g"));
    for id in ["adn-gas", "igc-tank-clearance"] {
        assert!(!has_set(&results, id), "{id}");
    }
    assert!(rows(&results, "adn/").is_empty());
    assert!(rows(&results, "igc/2.4.1.2/").iter().all(|r| r.get("note").is_none()));
}

#[test]
fn the_adn_hull_rows_and_the_relief_valve_rows_are_evaluated_and_pass_on_the_hull_the_engine_sized() {
    let results = results_of(&file_of(Some("river-only")), true);
    let want = [
        ("adn/9.3.1.11.2(a)/double-hull-width", 0.8),
        ("adn/9.3.1.11.2(a)/double-bottom-height", 0.6),
        ("adn/9.3.1.22.3/relief-valve-outlet-height", 2.0),
        ("adn/9.3.1.22.3/relief-valve-outlet-distance", 6.0),
    ];
    for (id, limit) in want {
        let r = rows(&results, id);
        assert!(!r.is_empty(), "{id}");
        for row in r {
            assert_eq!(row["outcome"], "pass", "{id}: {row}");
            assert_eq!(row["limit"].as_f64(), Some(limit), "{id}");
            assert_eq!(row["consequence"], "block", "{id}");
            assert_eq!(row["rule_status"], "unverified", "{id}");
            assert!(row["value"].as_f64().unwrap() >= limit, "{id}: {row}");
        }
    }
    // The wing is the clear side gap less the 0.6 m gap the engine keeps beside the wing tank.
    let wing = rows(&results, "adn/9.3.1.11.2(a)/double-hull-width")[0]["value"]
        .as_f64()
        .unwrap();
    let side = rows(&results, "igc/2.4.1.2/side-clearance")[0]["value"]
        .as_f64()
        .unwrap();
    assert!((wing - (side - 0.6)).abs() < 1e-9 || wing == 0.8, "{wing} {side}");
}

#[test]
fn a_file_that_stores_its_rule_sets_keeps_them() {
    // A river-only Option that stores the IGC stack keeps it: no ADN row, no note.
    let filled = fill_as_new(&file_of(Some("river-only")).to_string(), &Profile::built_in());
    let mut design = filled.design.unwrap();
    let option = &mut design.options[0];
    let igc_stack = json!([
        { "id": "igc-2g", "version": "3.1.0", "source": "typed" },
        { "id": "is2008-a", "version": "2.0.0", "source": "typed" },
        { "id": "lr-inland-gas", "version": "2.0.0", "source": "typed" },
        { "id": "ovd", "version": "3.1.0", "source": "typed" }
    ]);
    let hashes: Vec<Value> = igc_stack
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let lib = ovd_format::rule_set::built_in();
            let set = lib
                .find(e["id"].as_str().unwrap(), e["version"].as_str().unwrap())
                .unwrap();
            json!({ "id": set.id, "version": set.version, "hash": set.hash, "source": "typed" })
        })
        .collect();
    option
        .inputs
        .iter_mut()
        .find(|(n, _)| n == "rule_sets")
        .unwrap()
        .1
        .value = Value::Array(hashes);
    match run_option(option) {
        RunOutcome::Results { results, .. } => {
            assert!(has_set(&results, "igc-2g"));
            assert!(!has_set(&results, "adn-gas"));
            assert!(rows(&results, "adn/").is_empty());
            assert!(rows(&results, "igc/2.4.1.2/").iter().all(|r| r.get("note").is_none()));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_old_gas_file_with_no_service_area_and_no_rule_sets_is_read_with_the_adn_stack() {
    // Neither input is stored: the Service Area reads river-only and the stack is the ADN one.
    let results = results_of(&file_of(None), false);
    assert!(has_set(&results, "adn-gas") && has_set(&results, "igc-tank-clearance"));
    assert!(!has_set(&results, "igc-2g"));
    // The legacy IS Code version it was read with is kept.
    assert!(sets(&results).contains(&"is2008-a@1.0.0".to_string()), "{:?}", sets(&results));
}

#[test]
fn an_old_gas_file_that_names_a_coastal_service_area_and_no_rule_sets_keeps_the_igc_stack() {
    let results = results_of(&file_of(Some("coastal-and-river")), false);
    assert!(has_set(&results, "igc-2g"));
    assert!(!has_set(&results, "adn-gas"));
}

#[test]
fn a_hand_written_file_with_no_service_area_is_valid_as_it_was() {
    // `check` refuses a file that leaves out an input, as it did; `fill` writes them in.
    let text = file_of(None).to_string();
    let checked = check(&text, &Profile::built_in());
    assert!(!checked.report.errors.is_empty());
    let filled = fill(&text, &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
}
