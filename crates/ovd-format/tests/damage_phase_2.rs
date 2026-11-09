//! Seam: damage stability phase 2 (issue #763, spec 0012 D2, D3): the `adn-gas` set lists all of ADN 9.3.1,
//! ES-TRIN 28.03 is evaluated for a craft over 110 m, and NORMAM 5.23.8 and 5.23.9 are evaluated from the shared
//! generator.
//!
//! Label: sourced - ADN 2025 (ECE/TRANS/351) 9.3.1.15.1 and 9.3.1.15.2; ES-TRIN 2025/1 28.03(4), (5) and (6);
//! NORMAM-202/DPC 5.23.8 and 5.23.9. The physics of the solver is checked in `ovd-core/tests`; these tests are
//! about what each rule's cases are and which rows read them.

use ovd_format::rule_set::Library;
use ovd_format::{Profile, RunOutcome, check, fill_as_new, run_design, run_option};
use serde_json::{Value, json};

fn stack(ids: &[(&str, &str)]) -> Value {
    let lib = Library::built_in();
    Value::Array(
        ids.iter()
            .map(|(id, version)| {
                let set = lib
                    .find(id, version)
                    .unwrap_or_else(|| panic!("{id}@{version}"));
                json!({"id": id, "version": version, "hash": set.hash, "source": "typed"})
            })
            .collect(),
    )
}

fn gas_results(loa: f64, beam: f64, depth: f64, sets: &[(&str, &str)]) -> Value {
    let inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "moulded_depth_m": { "value": depth, "source": "typed" },
        "loading_conditions": { "value": "is_code_2008", "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
        "draught_limit_m": { "value": 12.0, "source": "typed" },
        "air_draft_limit_m": { "value": 40.0, "source": "typed" },
        "rule_sets": { "value": stack(sets), "source": "typed" }
    });
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

fn liquid_results(sets: &[(&str, &str)], loa: Option<f64>) -> Value {
    let path = format!(
        "{}/tests/fixtures/crude_integral_tanks_self_propelled.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let inputs = file["options"][0]["inputs"].as_object_mut().unwrap();
    inputs.insert(
        "rule_sets".to_string(),
        json!({ "value": stack(sets), "source": "typed" }),
    );
    if let Some(l) = loa {
        inputs.insert(
            "length_overall_m".to_string(),
            json!({ "value": l, "source": "typed" }),
        );
    }
    let checked = check(&file.to_string(), &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    match run_design(&checked.design.unwrap()).remove(0).outcome {
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

const GAS: [(&str, &str); 4] = [
    ("igc-2g", "3.1.0"),
    ("is2008-a", "2.0.0"),
    ("adn-gas", "1.0.0"),
    ("ovd", "3.1.0"),
];

/// `adn-gas` lists the paragraphs of ADN 9.3.1: the two it evaluates (the damage stability) and every other one
/// `not evaluated`, each with its reason.
#[test]
fn adn_gas_lists_every_paragraph_and_only_the_damage_rows_are_evaluated() {
    let lib = Library::built_in();
    let set = lib.find("adn-gas", "1.0.0").unwrap();
    assert!(set.rules.len() > 100, "{}", set.rules.len());
    let evaluated: Vec<&str> = set
        .rules
        .iter()
        .filter(|r| r.evaluated)
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(
        evaluated,
        [
            "adn/9.3.1.15.1/damage-stability",
            "adn/9.3.1.15.2/intermediate-stages"
        ]
    );
    for r in set.rules.iter().filter(|r| !r.evaluated) {
        let gap = r.gap.as_deref().unwrap_or("");
        assert!(gap.len() > 20, "{} has no reason", r.id);
    }
    // every paragraph is cited from 9.3.1
    assert!(
        set.rules
            .iter()
            .all(|r| r.source.locator.starts_with("9.3.1."))
    );
    // the paragraphs the damage stability reads, and the ones around it, are listed
    for n in [
        "9.3.1.13.4",
        "9.3.1.14.2",
        "9.3.1.15.3",
        "9.3.1.15.4",
        "9.3.1.11.2",
    ] {
        assert!(
            set.rules.iter().any(|r| r.source.locator == n),
            "{n} is not listed"
        );
    }
}

/// A gas Option with `adn-gas` in its stack: the damage row has a row for each case on each loading condition,
/// the cargo tanks are never breached (full: 0 %), and the unmodelled rows are `not evaluated`.
#[test]
fn the_adn_gas_damage_rows_read_each_case_and_the_rest_are_not_evaluated() {
    let r = gas_results(110.0, 11.4, 6.0, &GAS);
    let block = &r["damage_by_rule"]["adn-gas"];
    assert!(block.is_object(), "no adn-gas damage block");
    let cases = block["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    for c in cases {
        assert!(c["breached_tanks"].as_array().unwrap().is_empty(), "{c}");
        assert_eq!(c["stages"].as_array().unwrap().len(), 3, "{c}");
    }
    let rows = rows(&r, "adn/9.3.1.15.1/damage-stability");
    assert_eq!(rows.len(), cases.len());
    for (row, case) in rows.iter().zip(cases) {
        assert_eq!(row["object_type"], "adn-gas-damage-case");
        let want = if case["ok"] == true { "pass" } else { "warn" };
        assert_eq!(row["outcome"], want, "{case}");
    }
    let stages = rows_of(&r, "adn/9.3.1.15.2/intermediate-stages");
    assert_eq!(stages.len(), cases.len());
    assert!(stages.iter().all(|row| row["outcome"] != "not evaluated"));
    let open = rows_of(&r, "adn/9.3.1.15.3/stability-damaged-condition");
    assert_eq!(open.len(), 1);
    assert_eq!(open[0]["outcome"], "not evaluated");
    assert!(open[0]["gap"].as_str().unwrap().contains("openings"));
    // the damage is not in the results hash
    assert!(ovd_format::UNHASHED_KEYS.contains(&"damage_by_rule"));
}

fn rows_of<'a>(results: &'a Value, id: &str) -> Vec<&'a Value> {
    rows(results, id)
}

/// ES-TRIN 28.03 applies to a craft whose length exceeds 110 m: a 130 m gas hull has the rows with a case each,
/// and a 110 m one has them `not applicable`.
#[test]
fn es_trin_28_03_is_evaluated_for_a_craft_over_110_m_and_not_applicable_below() {
    let es = [
        ("igc-2g", "3.1.0"),
        ("is2008-a", "2.0.0"),
        ("es-trin-2025", "2.0.0"),
        ("ovd", "3.1.0"),
    ];
    let long = gas_results(130.0, 16.0, 7.5, &es);
    let id = "es-trin/28.03(4)/damage-stability";
    let cases = long["damage_by_rule"]["es-trin-2025"]["cases"]
        .as_array()
        .expect("es-trin damage block");
    assert!(!cases.is_empty());
    let r = rows(&long, id);
    assert_eq!(r.len(), cases.len());
    assert!(
        r.iter()
            .all(|row| row["object_type"] == "es-trin-damage-case")
    );
    assert!(r.iter().all(|row| row["outcome"] != "not evaluated"));
    let stages = rows(&long, "es-trin/28.03(5)/intermediate-stages");
    assert_eq!(stages.len(), cases.len());
    // the openings, the heeling test and the cross-flooding are listed, not evaluated
    for id in [
        "es-trin/28.03(2)/heeling-test",
        "es-trin/28.03(6)(a)/openings-above-damage-waterline",
        "es-trin/28.03(7)/cross-flooding",
    ] {
        let row = rows(&long, id);
        assert_eq!(row.len(), 1, "{id}");
        assert_eq!(row[0]["outcome"], "not evaluated", "{id}");
    }
    // 110 m and under: ES-TRIN asks no damage stability, so the rows are not applicable and there is no block
    let short = gas_results(110.0, 11.4, 6.0, &es);
    assert!(short["damage_by_rule"].is_null());
    let r = rows(&short, id);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0]["outcome"], "not applicable");
}

/// ES-TRIN 28.03(4)(b) and the criteria of 28.03(6): the cases are ES-TRIN's own, and the stage criteria are the
/// weaker ones of 28.03(5).
#[test]
fn es_trin_cases_have_the_bottom_damage_of_0_39_m_and_three_stages() {
    let es = [
        ("igc-2g", "3.1.0"),
        ("is2008-a", "2.0.0"),
        ("es-trin-2025", "2.0.0"),
        ("ovd", "3.1.0"),
    ];
    let r = gas_results(130.0, 16.0, 7.5, &es);
    let cases = r["damage_by_rule"]["es-trin-2025"]["cases"]
        .as_array()
        .unwrap();
    for c in cases {
        // ES-TRIN 28.03(3): the cargo for the maximum draught, supplies full: the full load departure only
        assert_eq!(c["condition"], "Full load, departure", "{c}");
        let stages = c["stages"].as_array().unwrap();
        assert_eq!(stages.len(), 3);
        let fractions: Vec<f64> = stages
            .iter()
            .map(|s| s["fraction"].as_f64().unwrap())
            .collect();
        assert_eq!(fractions, [0.25, 0.5, 0.75]);
    }
    // a damaged hull has the cases of the side (0.59 m inboard) and the bottom (0.39 m up)
    let names: Vec<&str> = cases.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(names.iter().any(|n| n.starts_with("side")), "{names:?}");
    assert!(names.iter().any(|n| n.starts_with("bottom")), "{names:?}");
}

/// NORMAM 5.23.8 and 5.23.9: a liquid Option with `normam-202` 2.0.0 in its stack has the cases of 5.23.8
/// (the MARPOL extents, the end bulkheads of the main engine room intact, a full cargo tank at 0 %), a row for
/// each, and the criteria of 5.23.9.
#[test]
fn normam_5_23_8_and_5_23_9_are_evaluated_for_each_case() {
    let sets = [
        ("is2008-a", "2.0.0"),
        ("es-trin-2025", "2.0.0"),
        ("adn-tank", "2.0.0"),
        ("normam-202", "2.0.0"),
        ("ovd", "3.1.0"),
    ];
    let r = liquid_results(&sets, None);
    let block = &r["damage_by_rule"]["normam-202"];
    let cases = block["cases"].as_array().expect("normam damage block");
    assert!(!cases.is_empty());
    for c in cases {
        // a full cargo tank is 0 % (5.23.8 (f)): it floods nothing, so no case holds a cargo tank
        let flooded = c["flooded"].as_array().unwrap();
        assert!(!flooded.is_empty(), "{c}");
        assert!(
            flooded
                .iter()
                .all(|f| !f.as_str().unwrap().starts_with("Cargo")),
            "{c}"
        );
    }
    for id in [
        "normam/5.23.8/standard-damage-cases",
        "normam/5.23.9/damage-criteria-met",
    ] {
        let rows = rows(&r, id);
        assert_eq!(rows.len(), cases.len(), "{id}");
        for (row, case) in rows.iter().zip(cases) {
            assert_eq!(row["object_type"], "normam-damage-case", "{id}");
            assert_eq!(row["object"], case["name"], "{id}");
            let want = if case["ok"] == true { "pass" } else { "warn" };
            assert_eq!(row["outcome"], want, "{id} {}", case["name"]);
        }
    }
    // the rule of 1.0.0 stays as it was: not evaluated
    let old = [
        ("is2008-a", "2.0.0"),
        ("normam-202", "1.0.0"),
        ("ovd", "3.1.0"),
    ];
    let r = liquid_results(&old, None);
    assert!(r["damage_by_rule"].is_null());
    assert!(
        rows(&r, "normam/5.23.8/standard-damage")
            .iter()
            .all(|row| row["outcome"] == "not evaluated")
    );
}

/// No liquid result moves: the ADN and MARPOL damage blocks of a liquid Option are the same with the new sets in
/// the stack as without them.
#[test]
fn the_adn_damage_of_a_liquid_hull_does_not_move_when_normam_is_added() {
    let base = [
        ("is2008-a", "2.0.0"),
        ("adn-tank", "2.0.0"),
        ("ovd", "3.1.0"),
    ];
    let with = [
        ("is2008-a", "2.0.0"),
        ("adn-tank", "2.0.0"),
        ("normam-202", "2.0.0"),
        ("ovd", "3.1.0"),
    ];
    let (a, b) = (liquid_results(&base, None), liquid_results(&with, None));
    assert_eq!(a["damage"], b["damage"]);
    assert_eq!(a["marpol_damage"], b["marpol_damage"]);
    assert_eq!(ovd_format::results_hash(&a), ovd_format::results_hash(&b));
}

/// ES-TRIN 28.03 on a liquid bulk craft over 110 m: the laden departure, with the intermediate stages; a full
/// cargo tank floods nothing. Under 110 m the rows are not applicable.
#[test]
fn es_trin_28_03_is_evaluated_on_a_liquid_craft_over_110_m() {
    let sets = [
        ("is2008-a", "2.0.0"),
        ("es-trin-2025", "2.0.0"),
        ("adn-tank", "2.0.0"),
        ("ovd", "3.1.0"),
    ];
    let long = liquid_results(&sets, Some(120.0));
    let cases = long["damage_by_rule"]["es-trin-2025"]["cases"]
        .as_array()
        .expect("es-trin damage block");
    assert!(!cases.is_empty());
    for c in cases {
        assert_eq!(c["stages"].as_array().unwrap().len(), 3, "{c}");
        assert!(
            c["flooded"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| !f.as_str().unwrap().starts_with("Cargo")),
            "{c}"
        );
    }
    let r = rows(&long, "es-trin/28.03(4)/damage-stability");
    assert_eq!(r.len(), cases.len());
    assert!(
        r.iter()
            .all(|row| row["object_type"] == "es-trin-damage-case")
    );
    let short = liquid_results(&sets, None);
    assert!(short["damage_by_rule"].is_null());
    assert_eq!(
        rows(&short, "es-trin/28.03(4)/damage-stability")[0]["outcome"],
        "not applicable"
    );
    // the ADN cases do not move with the length rule in the stack
    assert!(short["damage"].is_array());
}

/// NORMAM 5.23 is for a gross tonnage over 20: the rows are `not applicable` under it; the rows for 5.23.7 (c),
/// 5.23.8 (e) and (g) are listed `not evaluated`.
#[test]
fn normam_lists_what_it_does_not_check() {
    let sets = [
        ("is2008-a", "2.0.0"),
        ("normam-202", "2.0.0"),
        ("ovd", "3.1.0"),
    ];
    let r = liquid_results(&sets, None);
    for id in [
        "normam/5.23.7(c)/all-loading-conditions",
        "normam/5.23.8(e)/openings-above-damage-waterline",
        "normam/5.23.8(g)/lesser-damage",
    ] {
        let row = rows(&r, id);
        assert_eq!(row.len(), 1, "{id}");
        assert_eq!(row[0]["outcome"], "not evaluated", "{id}");
        assert!(row[0]["gap"].as_str().unwrap().len() > 20, "{id}");
    }
    let lib = Library::built_in();
    let set = lib.find("normam-202", "2.0.0").unwrap();
    assert_eq!(
        set.applies_when
            .conditions
            .iter()
            .find(|(k, _)| k == "gross_tonnage_band")
            .map(|(_, v)| v.clone()),
        Some(vec!["over-20".to_string()])
    );
}
