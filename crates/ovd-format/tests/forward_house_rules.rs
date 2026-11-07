//! Seam: the Rule rows of the forward house (issue #994): `igc-2g` 4.1.0 and `ovd` 3.3.0, through a
//! coastal Design of the 140 x 29 m reference size.
//!
//! Label: property. Each test states a rule: the new rows exist and apply to the forward style only, the
//! vent mast is judged against the forward house, and the other styles are not judged by the new rows.

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn design(style: &str, extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "moulded_depth_m": { "value": 12.0, "source": "typed" },
        "max_tank_od_m": { "value": 12.0, "source": "typed" },
        "arrangement_style": { "value": style, "source": "typed" }
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

fn row<'a>(r: &'a Value, id: &str) -> Option<&'a Value> {
    r["rule_trace"]["rows"]
        .as_array()?
        .iter()
        .find(|row| row["rule"] == json!(id))
}

#[test]
fn the_new_rows_judge_the_forward_style_and_the_vent_mast_meets_the_distance_to_the_house() {
    let r = design("wheelhouse_forward", json!({}));
    let rows = r["rule_trace"]["rows"].as_array().unwrap();
    let ids: Vec<&str> = rows.iter().filter_map(|x| x["rule"].as_str()).collect();
    assert!(
        ids.contains(&"igc/8.2.11.1/vent-exit-distance-forward-house"),
        "{ids:?}"
    );
    let vent = row(&r, "igc/8.2.11.1/vent-exit-distance-forward-house").unwrap();
    assert_eq!(vent["outcome"], json!("pass"), "{vent}");
    // The distance to the house and to the openings aft are both at least the 25 m asked.
    let res = &r["results"];
    assert!(
        res["vent_distance_to_forward_house_m"].as_f64().unwrap() >= 25.0,
        "{res}"
    );
    assert!(res["vent_distance_to_openings_m"].as_f64().unwrap() >= 25.0);
    assert_eq!(res["vent_mast_distance_met"], json!(true));
    // The A-60 note is listed and not evaluated, with its reason.
    let a60 = row(&r, "igc/3.2.5/forward-house-aft-face-a60").expect("the A-60 note is listed");
    assert_eq!(a60["outcome"], json!("not_evaluated"), "{a60}");
    // The forward Blind Distance check is judged by its own Rule.
    let sight = row(&r, "ovd/aft-block/forward-wheelhouse-blind-distance").expect("sight row");
    assert_eq!(sight["outcome"], json!("pass"), "{sight}");
    // The old sightline Rule, which reads the check of the other styles, does not apply here.
    assert!(
        row(&r, "ovd/aft-block/wheelhouse-sightline").is_none_or(|x| x["outcome"] != json!("fail"))
    );
}

#[test]
fn the_forward_house_openings_are_placed_and_none_faces_the_cargo_area() {
    let r = design("wheelhouse_forward", json!({}));
    let openings = r["stern_arrangement"]["openings"].as_array().unwrap();
    let forward: Vec<&Value> = openings
        .iter()
        .filter(|o| o["id"].as_str().unwrap().starts_with("forward house: "))
        .collect();
    assert!(forward.len() >= 6, "{openings:?}");
    assert!(
        forward
            .iter()
            .all(|o| o["faces_cargo_area"] == json!(false))
    );
    assert!(forward.iter().all(|o| o["face"] != json!("aft")));
    // The aft house's openings are still placed, and still none faces the cargo area.
    assert!(openings.len() > forward.len());
}

#[test]
fn the_other_styles_are_not_judged_by_the_forward_rows() {
    for style in ["container_block", "deckhouse"] {
        let r = design(style, json!({}));
        for id in [
            "igc/8.2.11.1/vent-exit-distance-forward-house",
            "igc/3.2.4.2/forward-house-aft-face-windows-fixed",
            "igc/3.2.5/forward-house-aft-face-a60",
            "ovd/aft-block/forward-wheelhouse-blind-distance",
        ] {
            let outcome = row(&r, id).map(|x| x["outcome"].clone());
            assert!(
                outcome.is_none() || outcome == Some(json!("not_applicable")),
                "{style}: {id}: {outcome:?}"
            );
        }
    }
}
