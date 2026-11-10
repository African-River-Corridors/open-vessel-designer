//! Seam: a typed lock the hull does not fit blocks the Design on every hull, through the lock's own Rule rows
//! (issue #1043, follow-up). The Waterway Limit checks "within lock length" and "within lock width" each report a
//! Rule of the `ovd` set, `ovd/waterway/within-lock-length` and `ovd/waterway/within-lock-width`, that blocks. A gas,
//! a dry and a liquid hull fail the same way, and the Rule Trace names the lock as the cause. The lock does not
//! touch the gas arrangement search's own answer (`arrangement.feasible`), and it adds no "Feasible arrangement"
//! check: the search cannot see the lock, so its answer about the tanks stays its own.
//!
//! Each hull runs with the latest built-in `ovd` set, because the fixtures store an older version and a saved file
//! keeps the version it stores (spec 0004). The lock sizes are chosen against the hull's own length and beam, not
//! copied from a printed result.
//!
//! Label: property. Done-test for #1043 (`.done-test/1043`); build lanes must not edit this file.

use ovd_format::rule_set::Library;
use ovd_format::trace::rule_of_check;
use ovd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

const LENGTH_RULE: &str = "ovd/waterway/within-lock-length";
const WIDTH_RULE: &str = "ovd/waterway/within-lock-width";
const FEASIBLE_RULE: &str = "ovd/gas-arrangement/feasible-arrangement";

/// The three hull kinds the rule must reach: a gas hull (the 140 x 29 m example), a dry hull (a pushed box-hold
/// bauxite barge) and a liquid hull (a pushed crude-oil barge with integral tanks).
fn hulls() -> Vec<(&'static str, String)> {
    let root = env!("CARGO_MANIFEST_DIR");
    vec![
        (
            "gas",
            format!("{root}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign"),
        ),
        (
            "dry",
            format!("{root}/tests/fixtures/dry_bulk_box_hold.vdesign"),
        ),
        (
            "liquid",
            format!("{root}/tests/fixtures/crude_integral_tanks_pushed.vdesign"),
        ),
    ]
}

/// The results of the first Option of `path`, with `changes` typed, checked against the latest built-in `ovd` set
/// (the other stored sets stay as the file has them).
fn results(path: &str, changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    f["options"] = json!([f["options"][0].clone()]);
    let library = Library::built_in();
    let latest = library.latest("ovd").unwrap();
    let sets = f["options"][0]["inputs"]["rule_sets"]["value"]
        .as_array_mut()
        .unwrap_or_else(|| panic!("{path} stores no rule_sets"));
    for set in sets.iter_mut() {
        if set["id"] == "ovd" {
            set["version"] = json!(latest.version);
            set["hash"] = json!(latest.hash);
        }
    }
    for (name, value) in changes {
        f["options"][0]["inputs"][*name] = json!({ "value": value, "source": "typed" });
    }
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{path}: {other:?}"),
    }
}

fn check(r: &Value, name: &str) -> Option<Value> {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .cloned()
}

fn row(r: &Value, rule: &str) -> Value {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rule"] == rule)
        .cloned()
        .unwrap_or_else(|| {
            let ids: Vec<&str> = r["rule_trace"]["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|row| row["rule"].as_str())
                .collect();
            panic!("no Rule Trace row {rule}; rows: {ids:?}")
        })
}

fn passes(r: &Value) -> bool {
    r["rule_trace"]["verdict"]["passes"] == json!(true)
}

fn loa(r: &Value) -> f64 {
    r["dims"]["loa_m"].as_f64().unwrap()
}

fn beam(r: &Value) -> f64 {
    r["dims"]["beam_m"].as_f64().unwrap()
}

/// The failing lock row names the lock: the row's title says so, and the row blocks.
fn assert_lock_row_fails(r: &Value, rule: &str, hull: &str) {
    let row = row(r, rule);
    assert_eq!(row["outcome"], "fail", "{hull}: {row}");
    assert_eq!(row["consequence"], "block", "{hull}: {row}");
    assert_eq!(row["value"], false, "{hull}: {row}");
    let title = row["title"].as_str().unwrap().to_lowercase();
    assert!(
        title.contains("lock"),
        "{hull}: the title must name the lock: {row}"
    );
}

#[test]
fn a_lock_shorter_than_the_hull_fails_the_lock_length_rule_on_every_hull() {
    for (hull, path) in hulls() {
        let base = results(&path, &[]);
        let r = results(&path, &[("lock_length_m", json!(loa(&base) - 1.0))]);
        assert_eq!(
            check(&r, "within lock length").unwrap()["ok"],
            false,
            "{hull}"
        );
        assert_lock_row_fails(&r, LENGTH_RULE, hull);
        assert!(!passes(&r), "{hull}: {}", r["rule_trace"]["verdict"]);
        // The width was not typed: its row is listed, not left out, and not applicable.
        assert_eq!(row(&r, WIDTH_RULE)["outcome"], "not applicable", "{hull}");
        // The lock is the cause. The tank-arrangement Rule does not fail for it: it is as it was with no lock.
        assert_eq!(
            row(&r, FEASIBLE_RULE)["outcome"],
            row(&base, FEASIBLE_RULE)["outcome"],
            "{hull}"
        );
    }
}

#[test]
fn a_lock_narrower_than_the_beam_fails_the_lock_width_rule_on_every_hull() {
    for (hull, path) in hulls() {
        let base = results(&path, &[]);
        let r = results(&path, &[("lock_width_m", json!(beam(&base) - 1.0))]);
        assert_eq!(
            check(&r, "within lock width").unwrap()["ok"],
            false,
            "{hull}"
        );
        assert_lock_row_fails(&r, WIDTH_RULE, hull);
        assert!(!passes(&r), "{hull}: {}", r["rule_trace"]["verdict"]);
        assert_eq!(row(&r, LENGTH_RULE)["outcome"], "not applicable", "{hull}");
        assert_eq!(
            row(&r, FEASIBLE_RULE)["outcome"],
            row(&base, FEASIBLE_RULE)["outcome"],
            "{hull}"
        );
    }
}

#[test]
fn both_lock_limits_failed_give_two_failing_rows_and_the_verdict_counts_both() {
    for (hull, path) in hulls() {
        let base = results(&path, &[]);
        let r = results(
            &path,
            &[
                ("lock_length_m", json!(loa(&base) - 1.0)),
                ("lock_width_m", json!(beam(&base) - 1.0)),
            ],
        );
        assert_lock_row_fails(&r, LENGTH_RULE, hull);
        assert_lock_row_fails(&r, WIDTH_RULE, hull);
        let failing = r["rule_trace"]["verdict"]["failing_rows"].as_u64().unwrap();
        let base_failing = base["rule_trace"]["verdict"]["failing_rows"]
            .as_u64()
            .unwrap();
        assert_eq!(
            failing,
            base_failing + 2,
            "{hull}: {}",
            r["rule_trace"]["verdict"]
        );
    }
}

#[test]
fn with_no_lock_typed_both_lock_rules_are_listed_as_not_applicable_on_every_hull() {
    // A Rule the engine does not evaluate is listed, never left out (GLOSSARY, "Rule Trace").
    for (hull, path) in hulls() {
        let r = results(&path, &[]);
        assert!(check(&r, "within lock length").is_none(), "{hull}");
        assert!(check(&r, "within lock width").is_none(), "{hull}");
        assert_eq!(row(&r, LENGTH_RULE)["outcome"], "not applicable", "{hull}");
        assert_eq!(row(&r, WIDTH_RULE)["outcome"], "not applicable", "{hull}");
    }
}

#[test]
fn the_lock_leaves_the_gas_arrangement_flag_alone_and_adds_no_feasible_arrangement_check() {
    // The search cannot see the lock (the hull's length and beam are inputs), so its answer about the tanks is
    // its own. The lock's own Rule rows carry the block; no second row restates it under the tank Rule's title.
    let (_, path) = hulls().into_iter().find(|(h, _)| *h == "gas").unwrap();
    let base = results(&path, &[]);
    let r = results(
        &path,
        &[
            ("lock_length_m", json!(loa(&base) - 1.0)),
            ("lock_width_m", json!(beam(&base) - 1.0)),
        ],
    );
    assert_eq!(base["arrangement"]["feasible"], true);
    assert_eq!(
        r["arrangement"]["feasible"],
        base["arrangement"]["feasible"]
    );
    assert_eq!(
        check(&r, "Feasible arrangement"),
        check(&base, "Feasible arrangement"),
        "the lock must not add or change a Feasible arrangement check"
    );
    assert_eq!(
        row(&r, FEASIBLE_RULE)["outcome"],
        row(&base, FEASIBLE_RULE)["outcome"]
    );
}

#[test]
fn each_lock_check_reports_a_blocking_rule_of_the_latest_ovd_set_whose_title_names_the_lock() {
    let library = Library::built_in();
    let latest = library.latest("ovd").unwrap();
    for (name, id) in [
        ("within lock length", LENGTH_RULE),
        ("within lock width", WIDTH_RULE),
    ] {
        assert_eq!(rule_of_check(name), Some(id), "{name}");
        let rule = latest
            .rule(id)
            .unwrap_or_else(|| panic!("no Rule {id} in ovd@{}", latest.version));
        assert!(
            rule.title.to_lowercase().contains("lock"),
            "{id}: {}",
            rule.title
        );
        assert_eq!(rule.measure.id, "check.ok", "{id}");
    }
}
