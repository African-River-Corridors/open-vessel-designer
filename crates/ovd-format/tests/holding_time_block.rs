//! Seam: the `holding_time` block of the results (spec 0016, issue #870): a new output outside the
//! results hash, written for a Type C LNG Option, that reads the operation profile and moves no other
//! result.
//!
//! Label: property. Each test states a rule: the block is unhashed, it round-trips through JSON, the
//! profile moves only it, and a longer profile or more gas use never changes another result.

use ovd_format::{DesignSession, RunOutcome, UNHASHED_KEYS, results_hash};
use serde_json::{Value, json};

fn session() -> DesignSession {
    let mut s = river_session();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_input("Option 1", "length_overall_m", json!(110.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(11.4)).unwrap();
    s.set_input("Option 1", "moulded_depth_m", json!(6.0))
        .unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn results(s: &DesignSession) -> (Value, String) {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, hash } => (results.clone(), hash.clone()),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_block_is_outside_the_hash() {
    assert!(UNHASHED_KEYS.contains(&"holding_time"));
    let (r, hash) = results(&session());
    assert!(
        r["holding_time"]["evaluated"].as_bool().is_some(),
        "{:?}",
        r["holding_time"]
    );
    let mut without = r.clone();
    without.as_object_mut().unwrap().remove("holding_time");
    assert_eq!(results_hash(&without), hash);
    assert_eq!(results_hash(&r), hash);
}

#[test]
fn the_block_round_trips_through_json() {
    let (r, _) = results(&session());
    let block = r["holding_time"].clone();
    let text = serde_json::to_string(&block).unwrap();
    let back: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(back, block);
    if block["evaluated"] == json!(true) {
        let row = &block["rows"][0];
        assert_eq!(row["id"], "design_basis");
        assert_eq!(row["ambient_air_c"], 45.0);
        assert_eq!(row["legs"].as_array().unwrap().len(), 2);
        let n = block["series"].as_array().unwrap().len();
        assert!(n > 100 && n < 400, "a decimated series, {n} points");
        assert!(
            block["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .all(|i| i["source"].is_string())
        );
    }
}

#[test]
fn a_changed_profile_moves_only_the_holding_time_block() {
    let (a, ha) = results(&session());
    let mut s = session();
    s.set_input(
        "Option 1",
        "holding_profile",
        json!([{ "state": "idle", "days": 3.0 }]),
    )
    .unwrap();
    let (b, hb) = results(&s);
    assert_eq!(ha, hb);
    let strip = |v: &Value| {
        let mut v = v.clone();
        v.as_object_mut().unwrap().remove("holding_time");
        v
    };
    assert_eq!(strip(&a), strip(&b));
    if a["holding_time"]["evaluated"] == json!(true) {
        assert_ne!(a["holding_time"], b["holding_time"]);
    }
}

/// Issue #1034: a new river-only session needs a number for both river limits before it runs. These are high enough
/// not to bind, so no value of these tests moves.
fn river_session() -> ovd_format::DesignSession {
    let mut s =
        ovd_format::DesignSession::new_design_in(ovd_format::Profile::built_in(), "river-only")
            .unwrap();
    s.set_input("Option 1", "draught_limit_m", serde_json::json!(12.0))
        .unwrap();
    s.set_input("Option 1", "air_draft_limit_m", serde_json::json!(40.0))
        .unwrap();
    s
}
