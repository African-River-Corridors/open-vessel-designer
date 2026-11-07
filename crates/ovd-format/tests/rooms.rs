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

/// A coastal or river self-propelled hull of a given size, carrying a bulk cargo.
fn small_hull(cargo: &str, containment: &str, service_area: &str, loa: f64, beam: f64) -> Value {
    let inputs = json!({
        "cargo": { "value": cargo, "source": "typed" },
        "cargo_containment": { "value": containment, "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "moulded_depth_m": { "value": 4.5, "source": "typed" },
        "service_area": { "value": service_area, "source": "typed" },
        "arrangement_style": { "value": "deckhouse", "source": "typed" }
    });
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
    assert_eq!(r["rooms"]["headroom_required_m"].as_f64().unwrap(), 2.03);
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

#[test]
fn a_small_coastal_hull_gains_a_deckhouse_tier_under_mlc() {
    // Issue #892: a 60 x 10 m self-propelled hull. The deckhouse is (10 - 2 x 0.8) = 8.4 m wide, so a
    // tier is 8.4 x its length. The ES-TRIN-only rooms of the same berths fit in 1 tier; the MLC
    // rooms need 2. Hand check: the rooms total exceeds one full-footprint tier area, and fits in two.
    // Issue #943: the room tiers are sized from what they hold, so "a tier" here is the full
    // footprint (the machinery space's length by the width), the most a room tier can be.
    for (cargo, containment) in [("crude_oil", "integral_tanks"), ("bauxite", "box_hold")] {
        let r = small_hull(cargo, containment, "coastal-and-river", 60.0, 10.0);
        let dh = &r["drawing"]["aft_arrangement"]["deckhouse"];
        let tier_area = dh["x_fwd_m"].as_f64().unwrap() * dh["width_m"].as_f64().unwrap();
        let total = r["rooms"]["total_m2"].as_f64().unwrap();
        let n = r["crew"]["berths"].as_u64().unwrap() as u32;
        let officers = r["crew"]["officers"].as_u64().unwrap() as u32;
        let seafarers = r["crew"]["crew"].as_u64().unwrap() as u32;
        let es_only = ovd_core::rooms::size_rooms(&ovd_core::rooms::RoomsBasis {
            berths: n,
            reaches_sea: false,
            gt: 0,
            officers,
            seafarers,
            mode_b: false,
        })
        .total_m2;
        assert!(
            es_only <= tier_area,
            "{cargo}: ES-TRIN rooms {es_only} fit one tier {tier_area}"
        );
        assert!(
            total > tier_area && total <= 2.0 * tier_area,
            "{cargo}: {total} vs {tier_area}"
        );
        assert_eq!(dh["tiers"].as_i64().unwrap(), 2, "{cargo}");
    }
}

#[test]
fn the_owner_margin_officers_are_sized_as_officers() {
    // A3.1.9(k) covers seafarers who perform officers' duties: the electro-technical officer (and a
    // cargo engineer on a gas hull) of the owner margin count with the officers of the manning scale.
    let r = small_hull(
        "crude_oil",
        "integral_tanks",
        "coastal-and-river",
        90.0,
        14.0,
    );
    let margin: i64 = r["crew"]["owner_margin"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "electro-technical-officer" || m["role"] == "cargo-engineer")
        .map(|m| m["count"].as_i64().unwrap())
        .sum();
    assert!(margin >= 1);
    let manning =
        r["crew"]["deck_officers"].as_i64().unwrap() + r["crew"]["engineers"].as_i64().unwrap();
    assert_eq!(r["crew"]["officers"].as_i64().unwrap(), manning + margin);
    assert_eq!(r["rooms"]["officers"].as_i64().unwrap(), manning + margin);
}

#[test]
fn the_rooms_say_the_gross_tonnage_they_used_and_settle_on_the_full_key() {
    let r = small_hull("bauxite", "box_hold", "coastal-and-river", 50.0, 8.2);
    let used = r["rooms"]["gt_used"].as_u64().unwrap();
    assert!(used > 0);
    assert!(
        r["rooms"]["rule_note"]
            .as_str()
            .unwrap()
            .contains(&format!("{used} GT used"))
    );
    // Settled means the crew block's berths, officers, seafarers and GT band are the rooms'.
    assert_eq!(r["rooms"]["settled"], json!(true));
    assert_eq!(r["rooms"]["berths"], r["crew"]["berths"]);
    assert_eq!(r["rooms"]["officers"], r["crew"]["officers"]);
    assert_eq!(r["rooms"]["seafarers"], r["crew"]["crew"]);
}

#[test]
fn a_sea_hull_checks_the_tier_height_against_2_03_m_clear() {
    // Tier height 2.6 m less a 0.5 m allowance for deck structure and overhead = 2.10 m >= 2.03 m.
    let r = small_hull(
        "crude_oil",
        "integral_tanks",
        "coastal-and-river",
        90.0,
        14.0,
    );
    let c = r["drawing"]["aft_arrangement"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "deckhouse_clear_height")
        .expect("check present");
    assert_eq!(c["ok"], json!(true));
    assert!(
        c["message"]
            .as_str()
            .unwrap()
            .contains("2.10 m clear >= 2.03 m")
    );
    let river = small_hull("crude_oil", "integral_tanks", "river-only", 90.0, 14.0);
    assert!(
        river["drawing"]["aft_arrangement"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["name"] != "deckhouse_clear_height")
    );
}

/// The 140 x 29 m coastal LNG example hull, as a river-only deckhouse Option.
fn river_140x29(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "moulded_depth_m": { "value": 12.0, "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
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

#[test]
fn a_river_only_option_with_untyped_hours_sails_round_the_clock_in_mode_b() {
    // Issue #929. Label: snapshot (printed by the code, 140 x 29 m river-only deckhouse). The river-only
    // default is 24 h, RPN operating mode B (before: 14 h, mode A1, crew 4, berths 7). Cabin policy is
    // not part of this test (#894).
    let r = river_140x29(json!({}));
    assert_eq!(r["crew"]["inputs"]["navigation_hours_per_day"], json!(24.0));
    assert_eq!(r["crew"]["rpn"]["operating_mode"], json!("B"));
    // The manning minimum is 5 (was 4); with the owner's engineer that is 6 on board, plus 2 spare = 8 berths.
    assert_eq!(r["crew"]["minimum"], json!(5));
    assert_eq!(r["crew"]["crew"], json!(6));
    assert_eq!(r["crew"]["berths"], json!(8));
    // A typed 14 h still reads mode A1.
    let a1 = river_140x29(json!({ "navigation_hours_per_day": 14.0 }));
    assert_eq!(a1["crew"]["rpn"]["operating_mode"], json!("A1"));
    assert_eq!(a1["crew"]["minimum"], json!(4));
    assert_eq!(a1["crew"]["berths"], json!(7));
}
