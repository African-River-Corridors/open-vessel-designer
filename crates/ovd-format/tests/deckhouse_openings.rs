//! Seam: the deckhouse openings through a Design (issue #912, spec 0011 D17): each opening has a
//! position, a face, a kind and a space in the results; the block is outside the results hash; a
//! typed list stands in for the default layout; the Measures read them.
//!
//! Label: sourced - IGC Code, MSC.370(93), 3.2.4.1 and 3.2.4.2 (the zone: 4 % of L, 3 m at least,
//! 5 m at most, from the deckhouse front). The 110 m hull below has L about 0.96 x the waterline
//! length at 85 % of the depth, so its zone is the 4.0 to 4.2 m the hand working gives.

use ovd_format::{Profile, RunOutcome, fill_as_new, results_hash, run_option};
use serde_json::{Value, json};

fn results_with(service_area: &str, extra: Value) -> (Value, String) {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" },
        "service_area": { "value": service_area, "source": "typed" },
        "arrangement_style": { "value": "deckhouse", "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let text = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
    .to_string();
    let d = fill_as_new(&river_limits(&text), &Profile::built_in())
        .design
        .unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, hash } => (results, hash),
        other => panic!("{other:?}"),
    }
}

fn openings(r: &Value) -> Vec<Value> {
    r["stern_arrangement"]["openings"]
        .as_array()
        .expect("openings")
        .clone()
}

#[test]
fn each_opening_has_a_position_a_face_a_kind_and_a_space() {
    let (r, _) = results_with("coastal-and-river", json!({}));
    let list = openings(&r);
    assert!(!list.is_empty());
    for o in &list {
        for k in ["x_m", "y_m", "z_m", "distance_from_front_m"] {
            assert!(o[k].as_f64().is_some_and(f64::is_finite), "{k}: {o}");
        }
        assert!(["front", "side", "aft"].contains(&o["face"].as_str().unwrap()));
        assert!(
            ["entrance", "window", "air_inlet", "air_intake"]
                .contains(&o["kind"].as_str().unwrap())
        );
        assert!(
            [
                "accommodation",
                "service",
                "machinery",
                "control_station",
                "wheelhouse"
            ]
            .contains(&o["space"].as_str().unwrap())
        );
        assert!(o["faces_cargo_area"].is_boolean() && o["in_zone"].is_boolean());
    }
    assert_eq!(r["stern_arrangement"]["openings_source"], json!("default"));
    // The zone follows L: 4 % of a load line length of about 100 to 110 m is 4.0 to 4.4 m, inside 3 to 5 m.
    let zone = r["stern_arrangement"]["opening_zone_m"].as_f64().unwrap();
    let l = r["stern_arrangement"]["icll_length_m"].as_f64().unwrap();
    assert!((3.0..=5.0).contains(&zone));
    assert!(l > 90.0 && l < 110.0, "{l}");
    assert!((zone - (0.04 * l).clamp(3.0, 5.0)).abs() < 1e-9);
}

#[test]
fn the_openings_are_outside_the_results_hash() {
    // A typed list changes every opening and nothing else: the hash does not move.
    let (default, h1) = results_with("coastal-and-river", json!({}));
    let (typed, h2) = results_with("coastal-and-river", json!({ "deckhouse_openings": [] }));
    assert_eq!(h1, h2);
    assert_eq!(results_hash(&default), results_hash(&typed));
    assert_eq!(
        typed["stern_arrangement"]["openings_source"],
        json!("typed")
    );
    assert!(openings(&typed).is_empty());
    assert_ne!(openings(&default).len(), 0);
}

#[test]
fn a_typed_list_stands_in_for_the_default_layout() {
    let row = json!({
        "kind": "entrance", "space": "accommodation", "face": "front",
        "x_m": 0.0, "y_m": 0.0, "z_m": 7.0, "name": "front door"
    });
    let (r, _) = results_with("coastal-and-river", json!({ "deckhouse_openings": [row] }));
    let list = openings(&r);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], json!("front door"));
    // A door on the front faces the cargo area, and is in the zone, whatever its x.
    assert_eq!(list[0]["faces_cargo_area"], json!(true));
    assert_eq!(list[0]["in_zone"], json!(true));
    assert_eq!(list[0]["distance_from_front_m"], json!(0.0));
}

#[test]
fn a_container_block_option_has_no_openings() {
    let (r, _) = results_with(
        "river-only",
        json!({ "arrangement_style": "container_block" }),
    );
    assert!(r["stern_arrangement"].get("openings").is_none());
}

#[test]
fn a_bad_typed_row_is_refused_with_a_message() {
    let text = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 110.0, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 6.0, "source": "typed" },
            "arrangement_style": { "value": "deckhouse", "source": "typed" },
            "deckhouse_openings": { "value": [{ "kind": "door" }], "source": "typed" }
        } }]
    })
    .to_string();
    let filled = fill_as_new(&river_limits(&text), &Profile::built_in());
    assert!(
        filled.design.is_none() || !filled.report.errors.is_empty(),
        "a row with an unknown kind is an error"
    );
    assert!(
        filled
            .report
            .errors
            .iter()
            .any(|e| e.contains("deckhouse_openings")),
        "{:?}",
        filled.report.errors
    );
}

/// Issue #1034: a river-only Option needs a number for both river limits. These are high enough not to bind, so
/// no value of these tests moves.
fn river_limits(text: &str) -> String {
    ovd_format::with_river_limits(text, Some(12.0), Some(40.0)).unwrap()
}
