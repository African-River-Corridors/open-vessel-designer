//! Issue #850: the Rule Set `igc-tank-clearance` 1.0.0 holds the two IGC 2.4.1.2 rows (the tank side and
//! bottom distance) with the content `igc-2g` gives them. On a stack that takes it with `adn-gas` and not `igc-2g`, the
//! trace says the rows size the tank, because the double-hull branch of ADN 9.3.1.11.2 (a) gives no distance from the tank to the hull.
//!
//! Label: sourced - IGC Code, resolution MSC.370(93), 2.4.1.1, 2.4.1.2 and 2.4.2 (annex pp. 20-21): the
//! side distance is d(Vc) and the bottom distance the greater of B/15 (at most 2 m) and d(Vc), each
//! less the insulation. The values of d(Vc) are tested with `igc-2g` (`igc_2g_rule_set.rs`); this file
//! tests that the new set carries the same rows and table.

use ovd_format::rule_set::built_in;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

const ROWS: [&str; 2] = ["igc/2.4.1.2/side-clearance", "igc/2.4.1.2/bottom-clearance"];

#[test]
fn the_set_holds_the_two_rows_and_their_table_as_igc_2g_does() {
    let lib = built_in();
    let own = lib.find("igc-tank-clearance", "1.0.0").unwrap();
    let igc = lib.latest("igc-2g").unwrap();
    let ids: Vec<&str> = own.rules.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ROWS);
    for r in &own.rules {
        assert!(r.sizes && r.evaluated);
        let theirs = igc.rule(&r.id).unwrap();
        assert_eq!(
            ovd_format::rule_set::rule_content_json(r),
            ovd_format::rule_set::rule_content_json(theirs),
            "{}",
            r.id
        );
    }
    assert_eq!(own.table("igc-d-vc"), igc.table("igc-d-vc"));
    // Both sets can be selected together: the rows mean the same in each.
    assert!(
        lib.select(&[("igc-2g", &igc.version), ("igc-tank-clearance", "1.0.0")])
            .is_ok()
    );
}

fn results_with(stack: &[(&str, &str)]) -> Value {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 110.0, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 6.0, "source": "typed" },
            "service_area": { "value": "coastal-and-river", "source": "typed" }
        } }]
    });
    let mut design = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let entries: Vec<Value> = stack
        .iter()
        .map(|(id, version)| {
            let set = built_in().find(id, version).unwrap();
            json!({ "id": set.id, "version": set.version, "hash": set.hash, "source": "typed" })
        })
        .collect();
    design.options[0]
        .inputs
        .iter_mut()
        .find(|(n, _)| n == "rule_sets")
        .unwrap()
        .1
        .value = Value::Array(entries);
    match run_option(&design.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn igc_rows(results: &Value) -> Vec<&Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"].as_str().is_some_and(|x| x.starts_with("igc/")))
        .collect()
}

#[test]
fn without_igc_2g_the_two_rows_are_the_only_igc_rows_and_carry_the_note() {
    let results = results_with(&[
        ("adn-gas", "1.0.0"),
        ("igc-tank-clearance", "1.0.0"),
        ("is2008-a", "2.0.0"),
        ("lr-inland-gas", "2.0.0"),
        ("ovd", "3.1.0"),
    ]);
    let rows = igc_rows(&results);
    assert!(!rows.is_empty());
    for r in &rows {
        assert!(ROWS.contains(&r["rule"].as_str().unwrap()));
        assert_eq!(r["outcome"], "pass");
        assert_eq!(r["sizes"], true);
        let note = r["note"].as_str().unwrap();
        assert!(
            note.contains("sizes the tank")
                && note.contains("double-hull branch of ADN 9.3.1.11.2 (a)")
        );
    }
}

#[test]
fn without_adn_gas_the_rows_carry_no_note() {
    let results = results_with(&[
        ("igc-tank-clearance", "1.0.0"),
        ("is2008-a", "2.0.0"),
        ("lr-inland-gas", "2.0.0"),
        ("ovd", "3.1.0"),
    ]);
    assert!(!igc_rows(&results).is_empty());
    assert!(igc_rows(&results).iter().all(|r| r.get("note").is_none()));
}

#[test]
fn beside_igc_2g_the_rows_carry_no_note() {
    let igc = built_in().latest("igc-2g").unwrap().version.clone();
    let results = results_with(&[
        ("igc-2g", &igc),
        ("igc-tank-clearance", "1.0.0"),
        ("is2008-a", "2.0.0"),
        ("lr-inland-gas", "2.0.0"),
        ("ovd", "3.1.0"),
    ]);
    assert!(igc_rows(&results).iter().all(|r| r.get("note").is_none()));
}
