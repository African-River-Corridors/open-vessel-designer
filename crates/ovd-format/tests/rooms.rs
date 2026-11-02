//! Seam: the deckhouse rooms through a Design (issue #780, spec 0011 D5, section 2).
//!
//! Label: property. The rooms table is in the results of a deckhouse Option; its cabins are the
//! berths of the crew block; the tier count follows its floor; a river-only hull in mode B with rest
//! underway gets one cabin each (OVD's own rule in every mode); a hull that reaches sea says MLC A3.1 is not applied.

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn results(service_area: &str, extra: Value) -> Value {
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
    let d = fill_as_new(&text, &Profile::built_in()).design.unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn row<'a>(r: &'a Value, id: &str) -> &'a Value {
    r["rooms"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == id)
        .unwrap_or_else(|| panic!("{id}"))
}

#[test]
fn a_river_only_mode_b_hull_gets_one_cabin_each() {
    // 24 navigation hours is operating mode B (RPN 18.01); one cabin each, as in every mode.
    let r = results("river-only", json!({ "navigation_hours_per_day": 24.0 }));
    let berths = r["crew"]["berths"].as_i64().unwrap();
    assert!(berths > 0);
    assert_eq!(r["crew"]["rpn"]["operating_mode"], json!("B"));
    assert_eq!(row(&r, "cabin")["count"].as_i64().unwrap(), berths);
    assert_eq!(row(&r, "cabin")["basis"], json!("derived"));
    assert_eq!(r["rooms"]["rule"], json!("es-trin-2025-ch15"));
    assert_eq!(r["rooms"]["settled"], json!(true));
}

#[test]
fn the_tier_count_follows_the_rooms_floor() {
    let r = results("river-only", json!({ "navigation_hours_per_day": 24.0 }));
    let rooms = &r["rooms"];
    let dh = &r["drawing"]["aft_arrangement"]["deckhouse"];
    let tiers = dh["tiers"].as_i64().unwrap();
    let area = dh["tier_area_m2"].as_f64().unwrap();
    let total = rooms["total_m2"].as_f64().unwrap();
    assert_eq!(dh["required_area_m2"].as_f64().unwrap(), total);
    assert!(tiers as f64 * area >= total - 1e-9);
    // Not more tiers than the rooms need (the sightline may add some on a fixed wheelhouse).
    let sum: f64 = rooms["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["area_m2"].as_f64().unwrap())
        .sum();
    assert!((sum - total).abs() < 1e-9);
}

#[test]
fn the_cabins_of_a_sea_going_hull_are_es_trin_and_say_mlc_is_not_applied() {
    let r = results("coastal-and-river", json!({}));
    assert_eq!(r["rooms"]["mlc_applied"], json!(false));
    assert!(r["rooms"]["rule"].as_str().unwrap().contains("not applied"));
    // Hand value: for b berths, cabins 3.5 b, communal max(2 b, 8), toilets ceil(b / 6) x 1.0,
    // corridor 0.6 x ceil(b / 2) x 1.75; the sea floor is the larger of that and the interim floor of
    // 2 gas accommodation units of 12.192 x 2.438 m (= 59.4 m2).
    let b = r["crew"]["berths"].as_f64().unwrap();
    let rooms = 3.5 * b + (2.0 * b).max(8.0) + (b / 6.0).ceil() + 0.6 * (b / 2.0).ceil() * 1.75
        // estimates: basins, showers, galley, work clothes, one stairway
        + 0.5 * (b / 4.0).ceil()
        + 0.81 * (b / 6.0).ceil()
        + 5.0
        + (2.0 + 0.25 * b)
        + 1.6;
    let interim = 2.0 * 12.192 * 2.438;
    assert!((r["rooms"]["total_m2"].as_f64().unwrap() - rooms).abs() < 1e-9);
    let req = r["rooms"]["required_area_m2"].as_f64().unwrap();
    assert!((req - rooms.max(interim)).abs() < 1e-9, "{req}");
    assert_eq!(r["rooms"]["settled"], json!(true));
}

#[test]
fn a_container_block_option_has_no_rooms_block() {
    let r = results(
        "river-only",
        json!({ "arrangement_style": "container_block" }),
    );
    assert!(r.get("rooms").is_none());
}

#[test]
fn mode_a_and_mode_b_differ_in_the_crew_not_in_the_cabin_rule() {
    // In this model every berth gets its own cabin (the cautious reading of ES-TRIN 15.02(10)), so
    // operating mode A1 (14 h) and mode B (24 h) both give one cabin each. What differs is the crew
    // table cell (so the berths) and nothing else. The model has no rule that lets any crew share a cabin.
    let a = results("river-only", json!({ "navigation_hours_per_day": 14.0 }));
    let b = results("river-only", json!({ "navigation_hours_per_day": 24.0 }));
    assert_eq!(a["crew"]["rpn"]["operating_mode"], json!("A1"));
    assert_eq!(b["crew"]["rpn"]["operating_mode"], json!("B"));
    for r in [&a, &b] {
        assert_eq!(row(r, "cabin")["count"], r["crew"]["berths"]);
    }
    assert_eq!(row(&a, "cabin")["clause"], row(&b, "cabin")["clause"]);
    assert!(b["crew"]["berths"].as_i64() >= a["crew"]["berths"].as_i64());
}
