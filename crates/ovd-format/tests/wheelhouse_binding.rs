//! Seam: a deckhouse wheelhouse lifts only when a fixed wheelhouse would break the air-draft limit
//! (issue #1034, Phase 2; the maintainer's answer of 2026-11-09). The 140 x 29 x 12 m river-only
//! LNG deckhouse design is the case of the issue.
//!
//! Label: property. Each test states a rule: a limit the fixed wheelhouse meets leaves it fixed, and a
//! limit it breaks lifts it.

use ovd_format::{Profile, RunOutcome, fill_as_new, results_hash, run_option};
use serde_json::{Value, json};

fn run(air_draft_limit_m: f64) -> Value {
    run_with(air_draft_limit_m, None)
}

/// `lift_m` types the greatest lift of the wheelhouse; `None` leaves the default (6.0 m).
fn run_with(air_draft_limit_m: f64, lift_m: Option<f64>) -> Value {
    let text = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.0.0",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": "LNG", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
            "length_overall_m": 140.0, "beam_m": 29.0, "moulded_depth_m": 12.0,
            "cargo_containment": "type_c_tanks", "service_area": "river-only",
            "arrangement_style": "deckhouse"
        } }]
    });
    let mut text = text;
    if let Some(lift) = lift_m {
        text["options"][0]["inputs"]["wheelhouse_max_lift_m"] = json!(lift);
    }
    let text = text.to_string();
    let text = ovd_format::with_river_limits(&text, Some(12.0), Some(air_draft_limit_m)).unwrap();
    let d = fill_as_new(&text, &Profile::built_in()).design.unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// Cargo of the 140 x 29 x 12 m design with a 22 m limit and a lift of 0 (the PR of #1034 Phase 2).
const PINNED_CARGO_T: f64 = 8350.7;

fn air_draft_light(r: &Value) -> f64 {
    r["results"]["air_draft_light_m"].as_f64().unwrap()
}

#[test]
fn a_limit_the_fixed_wheelhouse_meets_leaves_it_fixed() {
    let r = run(40.0);
    let s = &r["stern_arrangement"];
    assert_eq!(s["wheelhouse"], json!("fixed"));
    assert_eq!(s["wheelhouse_basis"], json!("limit_not_binding"));
    assert_eq!(
        r["drawing"]["aft_arrangement"]["deckhouse"]["wheelhouse"],
        json!("fixed")
    );
    // The design does not move: it is the design with a lift of zero, to the hash.
    let no_lift = run_with(40.0, Some(0.0));
    assert_eq!(air_draft_light(&r), air_draft_light(&no_lift));
    assert_eq!(results_hash(&r), results_hash(&no_lift));
}

#[test]
fn a_limit_the_fixed_wheelhouse_breaks_lifts_it() {
    let fixed = air_draft_light(&run(40.0));
    let r = run(fixed - 1.0);
    let s = &r["stern_arrangement"];
    assert_eq!(s["wheelhouse"], json!("elevating"));
    assert_eq!(s["wheelhouse_basis"], json!("air_draft_limit"));
    assert!(air_draft_light(&r) < fixed, "{}", air_draft_light(&r));
    // The lifted design meets the limit it was lifted for.
    assert!(
        air_draft_light(&r) <= fixed - 1.0,
        "lifted air draft {} over the limit {}",
        air_draft_light(&r),
        fixed - 1.0
    );
}

#[test]
fn a_typed_lift_of_zero_under_a_limit_stays_fixed_and_sizes_tiers_in_two_passes() {
    // `wheelhouse_max_lift_m` = 0 with a limit of 22 m: the wheelhouse cannot lift, so the tier count is
    // sized in two passes (barge_capacity.rs, `sized_by_pass`). The cargo is pinned, so a change to that
    // path shows here.
    let r = run_with(22.0, Some(0.0));
    assert_eq!(r["stern_arrangement"]["wheelhouse"], json!("fixed"));
    let cargo_t = r["results"]["cargo_t"].as_f64().unwrap();
    assert!(
        (cargo_t - PINNED_CARGO_T).abs() < 1.0,
        "cargo {cargo_t} t, pinned {PINNED_CARGO_T} t"
    );
}

#[test]
fn a_limit_equal_to_the_fixed_air_draft_keeps_it_fixed() {
    let fixed = air_draft_light(&run(40.0));
    let r = run(fixed);
    assert_eq!(r["stern_arrangement"]["wheelhouse"], json!("fixed"));
}
