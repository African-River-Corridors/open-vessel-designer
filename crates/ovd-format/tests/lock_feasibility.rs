//! Seam: a typed lock length or lock width is a hard route limit (issue #1043). When "within lock length" or
//! "within lock width" fails, the arrangement is not feasible, and the flag and the Rule Trace agree. A lock that
//! fits, or no lock, changes nothing.
//!
//! Label: property. Each test states the rule (a failing lock check blocks, a passing one does not); the lock
//! sizes are chosen against the hull's own length and beam, not copied from a printed result.

use ovd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

fn results(changes: &[(&str, Value)]) -> Value {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut f: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    f["options"] = json!([f["options"][0].clone()]);
    for (name, value) in changes {
        f["options"][0]["inputs"][*name] = json!({ "value": value, "source": "typed" });
    }
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
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

fn feasible_row_fails(r: &Value) -> bool {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["name"] == "Feasible arrangement" && c["ok"] == false)
}

#[test]
fn a_lock_shorter_than_the_hull_makes_the_arrangement_infeasible() {
    let r = results(&[("lock_length_m", json!(100.0))]);
    assert_eq!(check(&r, "within lock length").unwrap()["ok"], false);
    assert_eq!(r["arrangement"]["feasible"], false);
    assert!(feasible_row_fails(&r));
    let row = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Feasible arrangement")
        .unwrap();
    assert!(
        row["message"]
            .as_str()
            .unwrap()
            .contains("within lock length"),
        "{row}"
    );
}

#[test]
fn a_lock_narrower_than_the_beam_makes_the_arrangement_infeasible() {
    let r = results(&[("lock_width_m", json!(20.0))]);
    assert_eq!(check(&r, "within lock width").unwrap()["ok"], false);
    assert_eq!(r["arrangement"]["feasible"], false);
    let row = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Feasible arrangement")
        .unwrap();
    assert!(
        row["message"]
            .as_str()
            .unwrap()
            .contains("within lock width"),
        "{row}"
    );
}

#[test]
fn a_lock_that_fits_changes_nothing() {
    let none = results(&[]);
    let fits = results(&[
        ("lock_length_m", json!(200.0)),
        ("lock_width_m", json!(40.0)),
    ]);
    assert_eq!(check(&fits, "within lock length").unwrap()["ok"], true);
    assert_eq!(check(&fits, "within lock width").unwrap()["ok"], true);
    assert_eq!(
        fits["arrangement"]["feasible"],
        none["arrangement"]["feasible"]
    );
    assert_eq!(feasible_row_fails(&fits), feasible_row_fails(&none));
}

#[test]
fn with_no_lock_typed_there_is_no_lock_check_and_the_flag_is_the_searchs() {
    let r = results(&[]);
    assert!(check(&r, "within lock length").is_none());
    assert!(check(&r, "within lock width").is_none());
    assert_eq!(r["arrangement"]["feasible"], true);
}
