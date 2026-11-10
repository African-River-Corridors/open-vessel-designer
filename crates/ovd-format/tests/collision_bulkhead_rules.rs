//! Seam: the Rule rows of the collision bulkhead (issue #994, the collision bulkhead rework): ES-TRIN
//! 2025/1 Article 3.03(2) and 3.03(1)(a) in `es-trin-2025`, IGC 2.4.4 in `igc-2g`, through a Design.
//!
//! Label: property. Each test states a rule: the house row judges every style and passes with the
//! house at the plane, the position rows warn outside the 3.03(1)(a) band and block below 0.03 L, the
//! IGC row keeps the cargo tanks aft of the plane on a coastal gas design and is not applied on a
//! river one, and the MLC sleeping-room row stays a named gap.

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

const HOUSE_ROW: &str = "es-trin/3.03(2)/no-accommodation-ahead-of-collision-bulkhead";
const BAND_ROW: &str = "es-trin/3.03(1)(a)/collision-bulkhead-position";
const MINIMUM_ROW: &str = "es-trin/3.03(1)(a)/collision-bulkhead-minimum";
const TANKS_ROW: &str = "igc/2.4.4/no-cargo-tank-forward-of-collision-bulkhead";
const SLEEPING_ROW: &str = "mlc/A3.1.6(c)/sleeping-room-location";

fn river_110(style: &str, extra: Value) -> Value {
    design(110.0, 11.4, 6.0, "river-only", style, extra)
}

fn coastal_140(style: &str, extra: Value) -> Value {
    design(140.0, 29.0, 12.0, "coastal-and-river", style, extra)
}

fn design(loa: f64, beam: f64, depth: f64, area: &str, style: &str, extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "service_area": { "value": area, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "moulded_depth_m": { "value": depth, "source": "typed" },
        "arrangement_style": { "value": style, "source": "typed" }
    });
    if area == "river-only" {
        inputs["draught_limit_m"] = json!({ "value": 12.0, "source": "typed" });
        inputs["air_draft_limit_m"] = json!({ "value": 40.0, "source": "typed" });
    } else {
        inputs["max_tank_od_m"] = json!({ "value": 12.0, "source": "typed" });
    }
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

fn outcome(r: &Value, id: &str) -> Value {
    row(r, id).unwrap_or_else(|| panic!("no row {id}"))["outcome"].clone()
}

#[test]
fn the_house_row_judges_every_style_and_passes_with_the_house_at_the_plane() {
    for style in ["deckhouse", "container_block", "wheelhouse_forward"] {
        let r = river_110(style, json!({}));
        let h = row(&r, HOUSE_ROW).unwrap_or_else(|| panic!("{style}: no row {HOUSE_ROW}"));
        assert_eq!(h["outcome"], json!("pass"), "{style}: {h}");
        assert_eq!(h["consequence"], json!("block"), "{style}");
        assert_eq!(h["value"], json!(true), "{style}: {h}");
        assert_eq!(h["origin"], json!("regulation"), "{style}");
        let sets = h["rule_sets"].as_array().unwrap();
        assert!(
            sets.iter()
                .any(|s| s.as_str().is_some_and(|s| s.starts_with("es-trin-2025@"))),
            "{style}: {sets:?}"
        );
    }
}

#[test]
fn the_house_row_is_judged_on_a_coastal_design_too() {
    let r = coastal_140("wheelhouse_forward", json!({}));
    assert_eq!(outcome(&r, HOUSE_ROW), json!("pass"));
}

#[test]
fn the_position_rows_pass_at_the_default_and_at_the_top_of_the_band() {
    let l = 110.0;
    let r = river_110("wheelhouse_forward", json!({}));
    assert_eq!(outcome(&r, BAND_ROW), json!("pass"));
    assert_eq!(outcome(&r, MINIMUM_ROW), json!("pass"));
    let top = river_110(
        "wheelhouse_forward",
        json!({ "collision_bulkhead_m": 0.04 * l + 2.0 }),
    );
    assert_eq!(
        outcome(&top, BAND_ROW),
        json!("pass"),
        "{}",
        row(&top, BAND_ROW).unwrap()
    );
}

#[test]
fn a_bulkhead_outside_the_band_warns_because_a_flooding_calculation_must_prove_it() {
    let l = 110.0;
    for typed in [0.03 * l, 0.04 * l + 2.5] {
        let r = river_110("deckhouse", json!({ "collision_bulkhead_m": typed }));
        let b = row(&r, BAND_ROW).unwrap();
        // A failed warn row reports outcome "warn" (crates/arc-rules/src/eval.rs, `failed`).
        assert_eq!(b["outcome"], json!("warn"), "{typed}: {b}");
        assert_eq!(b["consequence"], json!("warn"), "{typed}");
        // 0.03 L is the floor of the calculation route: the minimum row still passes there.
        assert_eq!(outcome(&r, MINIMUM_ROW), json!("pass"), "{typed}");
    }
}

#[test]
fn a_bulkhead_under_0_03_l_blocks() {
    let l = 110.0;
    let r = river_110(
        "deckhouse",
        json!({ "collision_bulkhead_m": 0.03 * l - 0.5 }),
    );
    let m = row(&r, MINIMUM_ROW).unwrap();
    assert_eq!(m["outcome"], json!("fail"), "{m}");
    assert_eq!(m["consequence"], json!("block"));
}

#[test]
fn on_a_coastal_gas_design_the_igc_row_keeps_the_tanks_aft_of_the_plane() {
    let r = coastal_140("deckhouse", json!({}));
    let t = row(&r, TANKS_ROW).unwrap_or_else(|| panic!("no row {TANKS_ROW}"));
    assert_eq!(t["outcome"], json!("pass"), "{t}");
    assert_eq!(t["consequence"], json!("block"));
    // A bow rake of 1 m ends the cargo zone ahead of the plane: the tanks would stand in the forepeak.
    let short = coastal_140("deckhouse", json!({ "bow_rake_length_m": 1.0 }));
    let cargo_fwd = 140.0 - short["geometry"]["bow_void_length_m"].as_f64().unwrap();
    let plane = short["geometry"]["collision_bulkhead_x_m"]
        .as_f64()
        .unwrap();
    assert!(
        cargo_fwd > plane,
        "the set-up must break the rule: {cargo_fwd} vs {plane}"
    );
    assert_eq!(
        outcome(&short, TANKS_ROW),
        json!("fail"),
        "{}",
        row(&short, TANKS_ROW).unwrap()
    );
    // With the house forward the cargo zone ends aft of the house, so aft of the plane.
    let forward = coastal_140("wheelhouse_forward", json!({ "bow_rake_length_m": 1.0 }));
    assert_eq!(outcome(&forward, TANKS_ROW), json!("pass"));
}

#[test]
fn a_river_design_is_not_judged_by_the_igc_row() {
    let r = river_110("deckhouse", json!({}));
    let o = row(&r, TANKS_ROW).map(|x| x["outcome"].clone());
    assert!(o.is_none() || o == Some(json!("not applicable")), "{o:?}");
}

#[test]
fn the_mlc_sleeping_room_row_stays_a_named_gap() {
    // The rooms table still places no room by deck or position: the 3.03(2) row covers the house,
    // the MLC row keeps its gap.
    let r = coastal_140("wheelhouse_forward", json!({}));
    assert_eq!(outcome(&r, SLEEPING_ROW), json!("not evaluated"));
}
