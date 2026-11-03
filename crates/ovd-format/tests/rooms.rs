//! Seam: the deckhouse rooms through a Design (issue #780, spec 0011 D5, section 2).
//!
//! Label: property. The rooms table is in the results of a deckhouse Option; its cabins are the
//! berths of the crew block; the tier count follows its floor; a river-only hull in mode B with rest
//! underway gets one cabin each (OVD's own rule in every mode); a hull that reaches sea is sized to the greater of MLC A3.1 and ES-TRIN 15 (issue #892).

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
fn the_cabins_of_a_sea_going_hull_are_the_greater_of_mlc_and_es_trin() {
    let r = results("coastal-and-river", json!({}));
    assert_eq!(r["rooms"]["mlc_applied"], json!(true));
    assert!(
        r["rooms"]["rule"]
            .as_str()
            .unwrap()
            .contains("mlc-2006-a3.1")
    );
    assert_eq!(r["rooms"]["headroom_m"].as_f64().unwrap(), 2.03);
    let b = r["crew"]["berths"].as_i64().unwrap();
    let officers = r["crew"]["officers"].as_i64().unwrap();
    let gt = r["crew"]["inputs"]["gross_tonnage"].as_f64().unwrap();
    // Hand value (A3.1.9(f), (k), (m)): the band floors by gross tonnage.
    let (f, k) = if gt < 3_000.0 {
        (4.5, 7.5)
    } else if gt < 10_000.0 {
        (5.5, 8.5)
    } else {
        (7.0, 10.0)
    };
    let day = officers.min(3);
    let plain = officers - day;
    let ratings = b - officers;
    let count = |id: &str| {
        r["rooms"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == id)
            .map_or(0, |x| x["count"].as_i64().unwrap())
    };
    assert_eq!(count("cabin"), ratings);
    assert_eq!(count("cabin_senior"), day);
    assert_eq!(count("day_room"), day);
    assert_eq!(count("cabin_officer"), plain);
    assert_eq!(count("washbasin"), b);
    let cabins = f * (ratings + day) as f64 + k * plain as f64;
    let sum: f64 = ["cabin", "cabin_senior", "cabin_officer"]
        .iter()
        .filter(|id| count(id) > 0)
        .map(|id| row(&r, id)["area_m2"].as_f64().unwrap())
        .sum();
    assert!((sum - cabins).abs() < 1e-9, "{sum} {cabins}");
    for x in r["rooms"]["rows"].as_array().unwrap() {
        assert!(["es-trin", "mlc", "both"].contains(&x["governing"].as_str().unwrap()));
    }
    // The floor is the rooms alone (the interim floor is no longer a lower bound), and it settles.
    let req = r["rooms"]["required_area_m2"].as_f64().unwrap();
    assert!(
        (req - r["rooms"]["total_m2"].as_f64().unwrap()).abs() < 1e-9,
        "{req}"
    );
    assert_eq!(r["rooms"]["settled"], json!(true));
}
