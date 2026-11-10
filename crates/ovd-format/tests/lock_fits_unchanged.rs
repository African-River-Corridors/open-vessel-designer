//! Seam: a typed lock that the hull fits changes nothing but the two lock checks and their two Rule rows (issue
//! #1043, follow-up; QA finding C of PR #1074). On a gas, a dry and a liquid hull, the results with a lock that fits,
//! less the two lock checks, their text ids and the two lock rows, are the results with no lock; and the results
//! hash is the same, because the lock checks are outside it on both sides (the rows are in `rule_trace`, which is
//! not hashed).
//!
//! Each hull runs with the latest built-in `ovd` set, because the fixtures store an older version and a saved file
//! keeps the version it stores (spec 0004). The lock sizes are chosen against the hull's own length and beam.
//!
//! Label: property. Done-test for #1043 (`.done-test/1043`); build lanes must not edit this file.

use ovd_format::rule_set::Library;
use ovd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

const LOCK_RULES: [&str; 2] = [
    "ovd/waterway/within-lock-length",
    "ovd/waterway/within-lock-width",
];
const LOCK_CHECKS: [&str; 2] = ["within lock length", "within lock width"];
const LOCK_TEXT_IDS: [&str; 4] = [
    "check-lock-length",
    "check-lock-width",
    "name-check-lock-length",
    "name-check-lock-width",
];

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

/// The results without the lock's own marks: its two checks, their text ids and its two Rule rows.
fn without_the_lock(r: &Value) -> Value {
    let mut out = r.clone();
    if let Some(checks) = out["checks"].as_array_mut() {
        checks.retain(|c| !LOCK_CHECKS.iter().any(|n| c["name"] == *n));
    }
    if let Some(ids) = out["text_ids"].as_object_mut() {
        ids.retain(|_, v| !LOCK_TEXT_IDS.iter().any(|id| v["id"] == *id));
    }
    if let Some(rows) = out["rule_trace"]["rows"].as_array_mut() {
        rows.retain(|row| !LOCK_RULES.iter().any(|id| row["rule"] == *id));
    }
    out
}

#[test]
fn a_lock_the_hull_fits_adds_two_passing_checks_and_two_passing_rows_and_changes_nothing_else() {
    for (hull, path) in hulls() {
        let none = results(&path, &[]);
        let loa = none["dims"]["loa_m"].as_f64().unwrap();
        let beam = none["dims"]["beam_m"].as_f64().unwrap();
        let fits = results(
            &path,
            &[
                ("lock_length_m", json!(loa + 10.0)),
                ("lock_width_m", json!(beam + 10.0)),
            ],
        );
        // What the lock adds: two passing checks and two passing rows (not applicable with no lock).
        for name in LOCK_CHECKS {
            let c = fits["checks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["name"] == name)
                .unwrap_or_else(|| panic!("{hull}: no check {name}"));
            assert_eq!(c["ok"], true, "{hull}: {c}");
        }
        for id in LOCK_RULES {
            assert_eq!(row(&fits, id)["outcome"], "pass", "{hull}: {id}");
            assert_eq!(row(&none, id)["outcome"], "not applicable", "{hull}: {id}");
        }
        // And nothing else: the same results, row for row. (Not the same hash: the two lock checks are
        // part of the hashed results, so a typed lock always changes the hash, on main as well.)
        assert_eq!(without_the_lock(&fits), without_the_lock(&none), "{hull}");
    }
}

#[test]
fn a_lock_the_hull_fits_keeps_the_verdict_of_no_lock() {
    for (hull, path) in hulls() {
        let none = results(&path, &[]);
        let loa = none["dims"]["loa_m"].as_f64().unwrap();
        let beam = none["dims"]["beam_m"].as_f64().unwrap();
        let fits = results(
            &path,
            &[
                ("lock_length_m", json!(loa + 10.0)),
                ("lock_width_m", json!(beam + 10.0)),
            ],
        );
        assert_eq!(
            fits["rule_trace"]["verdict"], none["rule_trace"]["verdict"],
            "{hull}"
        );
    }
}
