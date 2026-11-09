//! Seam: deckhouse tiers are sized from what they hold (issue #943). The 140 x 29 m gas river-only
//! hull is the design-impact sweep case "sweep gas river-only deckhouse 140x29", built here in code;
//! the small coastal hull is "sweep bauxite coastal service 60x10".
//!
//! The "main" values below were printed by the `ovd` built from `main` before this change (every
//! tier the full 488.6 m2: 4 tiers, 1,954.4 m2, wheelhouse front 17.832 m, wheelhouse floor 22.4 m,
//! container-block results hash 67dbe2..., as this test computes it).
//!
//! Label: snapshot

use ovd_format::{Profile, RunOutcome, fill_as_new};
use serde_json::{Value, json};

fn results_of(
    cargo: &str,
    containment: &str,
    loa: f64,
    beam: f64,
    depth: f64,
    area: &str,
    style: &str,
) -> (Value, String) {
    let text = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.0.0",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": cargo, "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
            "length_overall_m": loa, "beam_m": beam, "moulded_depth_m": depth,
            "cargo_containment": containment, "service_area": area, "arrangement_style": style,
            // Issue #1034: a river-only Option has an air-draft limit now, and a typed limit lets the wheelhouse lift.
            // These tests are about the fixed wheelhouse the Option had with no limit, so the lift is zero.
            "wheelhouse_max_lift_m": 0.0
        } }]
    })
    .to_string();
    let d = fill_as_new(&river_limits(&text), &Profile::built_in())
        .design
        .unwrap();
    match ovd_format::run_option(&d.options[0]) {
        RunOutcome::Results { results, hash, .. } => (results, hash),
        other => panic!("{other:?}"),
    }
}

fn river() -> Value {
    results_of(
        "LNG",
        "type_c_tanks",
        140.0,
        29.0,
        12.0,
        "river-only",
        "deckhouse",
    )
    .0
}

fn coastal() -> Value {
    results_of(
        "bauxite",
        "box_hold",
        60.0,
        10.0,
        4.5,
        "coastal-and-river",
        "deckhouse",
    )
    .0
}

fn house(r: &Value) -> &Value {
    &r["drawing"]["aft_arrangement"]["deckhouse"]
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

#[test]
fn the_river_only_house_is_far_smaller_than_four_full_tiers() {
    let r = river();
    let d = house(&r);
    assert!(num(&d["area_m2"]) < 4.0 * 488.6, "{}", d["area_m2"]);
    // The sum of the plan is the area; the volume is that, one tier high.
    let plan = d["tiers_plan"].as_array().unwrap();
    assert_eq!(plan.len() as i64, d["tiers"].as_i64().unwrap());
    let sum: f64 = plan.iter().map(|t| num(&t["area_m2"])).sum();
    assert!((sum - num(&d["area_m2"])).abs() < 1e-6);
    assert!((num(&d["volume_m3"]) - sum * num(&d["tier_height_m"])).abs() < 1e-6);
    // Access tiers are the wheelhouse plan and a stair: small.
    for t in plan.iter().filter(|t| t["kind"] == json!("access")) {
        assert!(num(&t["area_m2"]) < 40.0, "{t}");
    }
}

#[test]
fn the_room_tiers_hold_the_rooms_table_floor() {
    for r in [river(), coastal()] {
        let d = house(&r);
        let rooms: f64 = d["tiers_plan"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t["kind"] == json!("rooms"))
            .map(|t| num(&t["area_m2"]))
            .sum();
        assert!(
            rooms >= num(&d["required_area_m2"]) - 1e-9,
            "{rooms} < {}",
            d["required_area_m2"]
        );
        assert_eq!(d["size_status"], json!("estimate"));
    }
}

#[test]
fn the_wheelhouse_front_and_its_eye_are_where_main_put_them() {
    let r = river();
    let a = &r["drawing"]["aft_arrangement"];
    assert!((num(&a["wheelhouse_x1_m"]) - 17.832).abs() < 1e-9);
    // The eye is where main put it, or higher only by whole tiers if a lighter house needed them.
    assert!(num(&a["wheelhouse_floor_lowered_m"]) >= 22.4 - 1e-9);
    assert!(
        r["checks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"] == json!("wheelhouse_sees_over_cargo"))
            .all(|c| c["ok"] == json!(true))
    );
}

#[test]
fn a_container_block_option_is_unchanged() {
    let (r, hash) = results_of(
        "LNG",
        "type_c_tanks",
        140.0,
        29.0,
        12.0,
        "river-only",
        "container_block",
    );
    assert!(r["drawing"]["aft_arrangement"]["deckhouse"].is_null());
    // The container block has no deckhouse, so this change moves no result. Re-pinned by running the
    // code (issue #919: the insulation of the LNG tanks defaults to 96 kg/m3, was 60, so the lightship
    // rises): the hash `main` gave was 67dbe209694b8b3aa216ba957cb48f23dd7d73fde6c3fc571b2c5ba80c763833.
    // Re-pinned again by running the code (issue #992: a new river-only gas Option has no weather shield,
    // `weather_shield_thickness_mm` 0, so the shield mass leaves the lightship; this Option types its
    // depth, 12.0 m, so the depth and the standing height play no part): was
    // ae3f2ee729ea7cf252e54724b29b0aaccfc0e81ab173164140a8d7c390db2e16.
    // Re-pinned (issue #1034: a river-only Option now carries a number for both river limits, here 12.0 m and 40.0 m,
    // the results now show the air-draft limit check (`limit_m`, one check and one rule row), so the hash moves; no number of the
    // hull moves, checked by a run with the limits stored as null): was
    // a450181cb58452607279476905fb0ce6aa0ef78177c66204828777379e36b8d8.
    assert_eq!(
        hash,
        "5f480ac8787f6ca206694ff7b96263c1b3a87aad67139476281c0fd1fcab3b64"
    );
}

#[test]
fn the_river_only_house_has_the_tiers_this_change_gives_it() {
    // Printed by the engine at this change. Room floor 63.42 m2 + 20 m2 service allowance = 83.42 m2
    // on one room tier: 3.04 m long, so the 3.6 m wheelhouse length governs (3.6 x 27.4 = 98.64 m2).
    // Three access tiers, each the 3.6 x 6.4 m wheelhouse plan plus a 1.6 m2 stair (24.64 m2 each).
    let r = river();
    let d = house(&r);
    assert_eq!(d["room_tiers"], json!(1));
    assert_eq!(d["access_tiers"], json!(3));
    assert!(
        (num(&d["area_m2"]) - 172.56).abs() < 1e-6,
        "{}",
        d["area_m2"]
    );
    assert!((num(&d["volume_m3"]) - 448.656).abs() < 1e-6);
    assert!((num(&d["x_aft_m"]) - 14.232).abs() < 1e-9);
    // Lightship falls from 5588.72 t before this change; cargo is volume-bound and does not move.
    // Re-pinned (issue #919: the insulation density of the LNG tanks defaults to 96 kg/m3, was 60, so the
    // insulation weighs 60 % more); printed 5417.183668599429, was 5313.94495. Re-pinned again (issue #992: a new
    // river-only gas Option has no weather shield, so the shield mass leaves the lightship; the depth is
    // typed, 12.0 m, so the standing height plays no part); printed 5213.6643348078815, was 5417.183668599429.
    assert!(
        (num(&r["results"]["lightship_t"]) - 5_213.664_335).abs() < 1e-4,
        "{}",
        r["results"]["lightship_t"]
    );
    assert!((num(&r["results"]["cargo_t"]) - 9_882.019_135).abs() < 1e-4);
}

/// Issue #1034: a river-only Option needs a number for both river limits. These are high enough not to bind, so
/// no value of these tests moves.
fn river_limits(text: &str) -> String {
    ovd_format::with_river_limits(text, Some(12.0), Some(40.0)).unwrap()
}
