//! Seam: the input `tank_top_above_deck_m` of issue #992 through a Design. The depth follows the tank
//! (the double bottom, the support and the tank outer diameter, less the standing height); 0 is tanks
//! in hold; a file without the input is unchanged.
//!
//! Label: rule. The depth rule is the maintainer's decision of 2026-11-07 and the design note of
//! issue #992 (D1a); the 0.3 m gap with the tanks in hold is an estimate.

use ovd_format::vdesign::LEGACY_FIXED_OTHER;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

/// The 140 x 29 m coastal LNG barge. `extra` is typed; the depth is left out unless `extra` has it.
fn design(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "min_freeboard_m": { "value": 4.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
}

fn run_with(file: &Value) -> (Value, Vec<String>) {
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let warnings = filled.report.warnings.clone();
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => (results, warnings),
        other => panic!("{other:?}"),
    }
}

fn run(file: &Value) -> Value {
    run_with(file).0
}

/// The same run with damage skipped: the search does not screen by IGC 2.7 damage (issue #935), so the pick is the
/// unscreened one. For the tests that compare two runs of one arrangement (a shield's mass, the probe of the depth),
/// where a lighter or heavier hull would move the damage screen's pick and so change what is compared.
fn run_no_damage(file: &Value) -> Value {
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let skip = ovd_format::damage::DamageControl {
        run: false,
        progress: None,
    };
    match ovd_format::run_option_with_damage(
        &filled.design.unwrap().options[0],
        ovd_format::rule_set::built_in(),
        &[],
        &skip,
    ) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn depth(r: &Value) -> f64 {
    r["dims"]["depth_m"].as_f64().unwrap()
}

#[test]
fn a_file_saved_before_has_no_standing_height_and_an_explicit_none_keeps_the_old_depth() {
    // A file saved before holds none for both, so its depth is the one it had.
    for name in ["tank_top_above_deck_m", "tank_top_above_deck_od_fraction"] {
        let l = LEGACY_FIXED_OTHER.iter().find(|l| l.name == name).unwrap();
        assert_eq!(l.value, "null", "{name}");
    }
    // A new Option that types none gets no block, and today's ratio depth: 0.55 x 29 m.
    let r = run(&design(json!({ "tank_top_above_deck_m": null })));
    let a = r["arrangement"].as_object().unwrap();
    assert!(!a.contains_key("tank_top_above_deck_m"));
    assert!(!a.contains_key("depth_follows_tank"));
    assert!((depth(&r) - 15.95).abs() < 1e-9, "{}", depth(&r));
}

#[test]
fn in_hold_the_depth_is_the_tank_top_plus_the_gap_and_a_standing_height_takes_it_off() {
    // Label: snapshot, printed by the code. Tank top above keel and diameter of the arrangement.
    let hold = run(&design(json!({ "tank_top_above_deck_m": 0.0 })));
    let top = hold["arrangement"]["tank_top_above_keel_m"]
        .as_f64()
        .unwrap();
    assert!(
        (depth(&hold) - (top + 0.3)).abs() < 0.0006,
        "{} vs {top}",
        depth(&hold)
    );
    assert_eq!(hold["arrangement"]["depth_follows_tank"], true);
    // Issue #935: the search screens by IGC 2.7 damage and refines a failing size down to about 0.1 m. The
    // two-across 13.28 m tanks this design chose before do not survive a side breach across the hold bulkhead; the
    // refined pick is two across at 11.72 m, which carries more than one 16 m tank (the first screen gave that).
    // Label: snapshot, printed by the code.
    assert_eq!(hold["arrangement"]["n_wide"], 2);
    assert!(
        (hold["arrangement"]["tank_od_m"].as_f64().unwrap() - 11.718).abs() < 0.001,
        "{}",
        hold["arrangement"]["tank_od_m"]
    );
    assert!((depth(&hold) - 13.621).abs() < 0.0006, "{}", depth(&hold));

    let two = run(&design(json!({ "tank_top_above_deck_m": 2.0 })));
    let top2 = two["arrangement"]["tank_top_above_keel_m"]
        .as_f64()
        .unwrap();
    // The depth is the tank top less the height, to the millimetre, for the tank it carries.
    assert!(
        (depth(&two) - (top2 - 2.0)).abs() < 0.0016,
        "{} vs {top2}",
        depth(&two)
    );
    // Standing 2 m high saves at least 2.3 m of depth: the same tank would, and a smaller one more. Issue #935:
    // both picks are two across at 11.72 m (the refined size that survives damage), so the depths compare.
    // Label: snapshot, printed by the code.
    assert_eq!(two["arrangement"]["n_wide"], 2);
    assert!((two["arrangement"]["tank_od_m"].as_f64().unwrap() - 11.718).abs() < 0.001);
    assert!((depth(&two) - 11.321).abs() < 0.0006, "{}", depth(&two));
    assert!(
        depth(&hold) - depth(&two) > 2.29,
        "{} {}",
        depth(&hold),
        depth(&two)
    );
}

#[test]
fn a_height_above_the_tank_diameter_is_refused_with_the_reason() {
    let r = run(&design(json!({ "tank_top_above_deck_m": 40.0 })));
    assert_eq!(r["arrangement"]["feasible"], false);
    let tried = serde_json::to_string(&r["fit_tried"]).unwrap();
    assert!(tried.contains("more than the tank diameter"), "{tried}");
}

#[test]
fn a_typed_depth_wins_and_a_warning_says_so() {
    let (r, warnings) = run_with(&design(
        json!({ "moulded_depth_m": 12.0, "tank_top_above_deck_m": 2.0 }),
    ));
    assert!((depth(&r) - 12.0).abs() < 1e-9);
    assert!(
        !r["arrangement"]
            .as_object()
            .unwrap()
            .contains_key("tank_top_above_deck_m")
    );
    assert!(
        warnings.iter().any(|w| w.contains("tank_top_above_deck_m")),
        "{warnings:?}"
    );
}

/// Issue #992, the maintainer's decision of 2026-11-07: a new coastal gas Option stands its tanks 2.0 m
/// above the deck under a weather shield; a new river-only one stands them to half their diameter, with
/// no shield. Label: rule (a decision), no calibration case for coastal.
#[test]
fn a_new_option_takes_the_maintainers_default_by_service_area() {
    let p = Profile::built_in();
    assert_eq!(
        p.default_by("tank_top_above_deck_m", "service_area", "coastal-and-river"),
        Some(&json!(2.0))
    );
    assert_eq!(
        p.default_by("tank_top_above_deck_m", "service_area", "river-only"),
        Some(&Value::Null)
    );
    assert_eq!(
        p.default_by(
            "tank_top_above_deck_od_fraction",
            "service_area",
            "river-only"
        ),
        Some(&json!(0.5))
    );
    assert_eq!(
        p.default_by(
            "tank_top_above_deck_od_fraction",
            "service_area",
            "coastal-and-river"
        ),
        Some(&Value::Null)
    );
    assert_eq!(
        p.default_by("weather_shield_thickness_mm", "service_area", "river-only"),
        Some(&json!(0.0))
    );
    // Coastal: 2.0 m, under a shield; the depth follows the tank.
    let c = run(&design(json!({})));
    assert_eq!(c["arrangement"]["tank_top_above_deck_m"], 2.0);
    assert_eq!(above(&c)["weather_shield_fitted"], true);
    assert!(depth(&c) < 15.95);
    // River only: up to half the tank diameter (the search may pick less), no shield.
    let r = run(&river_design(json!({})));
    let od = r["arrangement"]["tank_od_m"].as_f64().unwrap();
    let h = r["arrangement"]["tank_top_above_deck_m"].as_f64().unwrap();
    let share = r["arrangement"]["tank_top_above_deck_od_fraction"]
        .as_f64()
        .unwrap();
    assert_eq!(
        r["arrangement"]["tank_top_above_deck_od_fraction_limit"],
        0.5
    );
    assert!((0.0..=0.5).contains(&share), "share {share}");
    assert!((h - share * od).abs() < 1e-9, "{h} vs {share} of {od}");
    assert_eq!(above(&r)["weather_shield_fitted"], false);
    // The deck sits the chosen share of the diameter below the tank top.
    let top = r["arrangement"]["tank_top_above_keel_m"].as_f64().unwrap();
    assert!(
        (top - depth(&r) - h).abs() < 0.0016 || share == 0.0,
        "{top} {} {h}",
        depth(&r)
    );
}

/// Review of #1027: the maintainer's "up to the widest point" is a limit, not a fixed share. The 100 x 20 m
/// river-only LNG barge, on a fresh fill, is feasible: at half the diameter the one tank across fails
/// the sightline (the tank top 8 m above the deck needs 13.9 m of wheelhouse lift), so the search
/// takes a lower share and more tanks across. Pinning the share at 0.5 (every step of the search at
/// the limit) makes this test fail.
/// Label: rule (design note D1: "the search may pick less if depth, freeboard or sightline need it").
#[test]
fn the_river_barge_is_feasible_because_the_search_takes_a_lower_share_than_half() {
    let r = run(&river_design(json!({})));
    assert_eq!(r["arrangement"]["feasible"], true, "{}", r["arrangement"]);
    let share = r["arrangement"]["tank_top_above_deck_od_fraction"]
        .as_f64()
        .unwrap();
    assert!(
        share < 0.5,
        "the share {share} is the limit, not a lower one"
    );
    // The share chosen is recorded and shown: in the summary lines of `ovd run`.
    let lines = ovd_format::summary_lines("Option 1", &r);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Tank tops stand") && l.contains("share")),
        "{lines:?}"
    );
}

/// A river-only gas barge of the size of the worked inland case, on the default stack.
fn river_design(extra: Value) -> Value {
    let mut base = json!({
        "cargo": "LNG", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
        "length_overall_m": 100.0, "beam_m": 20.0, "service_area": "river-only"
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    let inputs: serde_json::Map<String, Value> = base
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), json!({ "value": v, "source": "typed" })))
        .collect();
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
}

/// Issue #992, review item 1. The depth is never less than the draught limit plus the least freeboard:
/// a shallow depth that follows a small tank must rise to it, not cost draught and cargo. The probe: a
/// river-only 86 x 11.4 m barge, a 3.0 m draught limit, the tank top to stand 2.5 m above the deck (it
/// built two across at 1.9 m, under the draught, before the floor).
/// Label: rule (the design note D1a).
#[test]
fn the_depth_is_never_less_than_the_draught_limit_plus_the_freeboard() {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 86.0, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "service_area": { "value": "river-only", "source": "typed" },
            "min_freeboard_m": { "value": 0.3, "source": "typed" },
            "draught_limit_m": { "value": 3.0, "source": "typed" },
            "tank_top_above_deck_m": { "value": 2.5, "source": "typed" }
        } }]
    });
    let r = run(&file);
    assert!(
        depth(&r) >= 3.0 + 0.3 - 1e-9,
        "depth {} under draught 3.0 + freeboard 0.3",
        depth(&r)
    );
    let by = r["arrangement"]["depth_set_by"].as_str().unwrap();
    assert!(by == "tank" || by == "freeboard", "{by}");
}

/// Issue #992, review item 2. The draught limit of the hull is the settled depth less the freeboard,
/// not the depth of the ratio default the run started from (140 x 29 m coastal at 0 m: depth about 15.2
/// m, but `draught_max_m` came out 11.95 = 15.95 - 4.0).
#[test]
fn the_draught_limit_follows_the_settled_depth_not_the_seed() {
    for h in [0.0, 2.0] {
        let r = run(&design(json!({ "tank_top_above_deck_m": h })));
        let max = r["results"]["draught_max_m"].as_f64().unwrap();
        assert!(
            (max - (depth(&r) - 4.0)).abs() < 0.002,
            "h {h}: draught_max {max} vs depth {} - 4.0",
            depth(&r)
        );
    }
}

fn above(r: &Value) -> &Value {
    &r["arrangement"]["above_deck"]
}

/// Issue #992, phase 2. In hold there is no shield and nothing stands above the deck; the result
/// block is absent without the input, so a result without it is unchanged.
#[test]
fn in_hold_nothing_stands_above_the_deck_and_without_the_input_there_is_no_block() {
    let hold = run(&design(json!({ "tank_top_above_deck_m": 0.0 })));
    let a = above(&hold);
    assert_eq!(a["weather_shield_fitted"], false);
    assert_eq!(a["weather_shield_mass_t"].as_f64().unwrap(), 0.0);
    // Only the domes pass through the deck: they are the highest thing, and not a windage area here.
    assert_eq!(a["weather_shield_top_above_deck_m"].as_f64().unwrap(), 0.0);
    assert_eq!(a["top_above_deck_m"], a["dome_top_above_deck_m"]);
    assert!(a["dome_top_above_deck_m"].as_f64().unwrap() > 0.0);
    assert_eq!(a["windage_side_area_m2"].as_f64().unwrap(), 0.0);
    let plain = run(&design(json!({ "tank_top_above_deck_m": null })));
    assert!(
        !plain["arrangement"]
            .as_object()
            .unwrap()
            .contains_key("above_deck")
    );
}

/// With the tanks standing above the deck the existing weather shield of the weights (issue #410)
/// stands over them: its top is the tank top plus the clearance above the deck, its mass is in the
/// lightship, and its projected areas are the windage. It adds no freeboard (the freeboard is the
/// depth at the side less the draught) and no credit to the hull girder or the buoyancy.
/// Label: rule (design note D2); sums of the shield's own inputs, not a published value.
#[test]
fn a_shield_over_standing_tanks_is_weight_and_windage_and_no_credit() {
    // Issue #935: every run of this test skips damage. The shield's mass would move the damage screen's pick, and
    // the pick is not what is compared here (the shield's weight, windage and credit are).
    let with = run_no_damage(&design(json!({ "tank_top_above_deck_m": 2.0 })));
    let a = above(&with);
    assert_eq!(a["weather_shield_fitted"], true);
    // Tank top 2.0 m above the deck plus the 1.5 m clearance of the shield.
    let top = a["weather_shield_top_above_deck_m"].as_f64().unwrap();
    assert!((top - 3.5).abs() < 0.01, "{top}");
    // The highest thing above the deck is the dome (about 2.3 m above the tank top), through the shield.
    let dome = a["dome_top_above_deck_m"].as_f64().unwrap();
    assert!(dome > 2.0);
    assert!((a["top_above_deck_m"].as_f64().unwrap() - dome.max(top)).abs() < 1e-9);
    let front = a["windage_front_area_m2"].as_f64().unwrap();
    assert!((front - 29.0 * top).abs() < 0.05, "{front}");
    assert!(a["windage_side_area_m2"].as_f64().unwrap() > 0.0);
    assert_eq!(a["weather_shield_watertight"], false);
    assert_eq!(
        a["weather_shield_freeboard_credit_m"].as_f64().unwrap(),
        0.0
    );
    assert_eq!(a["weather_shield_hull_girder_credit"], false);
    // No freeboard credit: the freeboard is the depth less the draught.
    let fb = with["results"]["freeboard_m"].as_f64().unwrap();
    let t = with["results"]["draught_m"].as_f64().unwrap();
    assert!(
        (fb - (depth(&with) - t)).abs() < 1e-6,
        "{fb} {} {t}",
        depth(&with)
    );
    // The shield is in the lightship: with it off, the hull is lighter by its mass, at the same depth.
    let off = run_no_damage(&design(
        json!({ "tank_top_above_deck_m": 2.0, "weather_shield_thickness_mm": 0.0 }),
    ));
    assert_eq!(above(&off)["weather_shield_fitted"], false);
    assert!((depth(&off) - depth(&with)).abs() < 1e-9);
    let mass = a["weather_shield_mass_t"].as_f64().unwrap();
    let dl = with["results"]["lightship_t"].as_f64().unwrap()
        - off["results"]["lightship_t"].as_f64().unwrap();
    // The lightship carries the shield with the steel's own allowance (about 1 %).
    assert!(
        mass > 0.0 && (dl / mass - 1.0).abs() < 0.03,
        "{dl} vs {mass}"
    );
}

/// The floor of the depth applies with `optimise_depth` too (review of #1015). The probe of
/// the review: river-only 86 x 11.4 m, a 3.0 m draught limit, 2.5 m standing.
#[test]
fn the_floor_applies_with_optimise_depth_too() {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 86.0, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "service_area": { "value": "river-only", "source": "typed" },
            "min_freeboard_m": { "value": 0.3, "source": "typed" },
            "draught_limit_m": { "value": 3.0, "source": "typed" },
            "optimise_depth": { "value": true, "source": "typed" },
            "tank_top_above_deck_m": { "value": 2.5, "source": "typed" }
        } }]
    });
    let r = run(&file);
    assert!(
        depth(&r) >= 3.3 - 1e-9,
        "depth {} under 3.0 + 0.3",
        depth(&r)
    );
}

/// Both passes pick the same arrangement: the probe's depth is the run's depth, so the run is done
/// once (`depth_passes` 1), and the draught limit follows the depth.
#[test]
fn the_run_is_done_once_when_the_probe_and_the_run_agree() {
    for h in [0.0, 2.0] {
        // The probe does not run damage, so the two passes agree on the unscreened pick. With the damage screen
        // (issue #935) the run may pick another arrangement than the probe, and the depth is settled again: two
        // passes at most.
        let r = run_no_damage(&design(json!({ "tank_top_above_deck_m": h })));
        assert_eq!(r["arrangement"]["depth_passes"], 1, "h {h}");
        let screened = run(&design(json!({ "tank_top_above_deck_m": h })));
        let passes = screened["arrangement"]["depth_passes"].as_u64().unwrap();
        assert!((1..=3).contains(&passes), "h {h}: {passes} passes");
    }
}

/// The side area of the windage is the cargo zone length times the height of the shield above its
/// foot (the deck, when the wing tank top is lower). Label: rule, a cross-check of two numbers the
/// code works out apart (the area, and the cargo zone length of the arrangement).
#[test]
fn the_side_area_is_the_cargo_zone_length_times_the_shield_height() {
    let r = run(&design(json!({ "tank_top_above_deck_m": 2.0 })));
    let a = above(&r);
    let zone = r["geometry"]["cargo_zone_length_m"]
        .as_f64()
        .expect("the cargo zone length");
    let top = a["weather_shield_top_above_deck_m"].as_f64().unwrap();
    let side = a["windage_side_area_m2"].as_f64().unwrap();
    assert!(
        (side - zone * top).abs() < 0.05 * zone,
        "{side} vs {zone} x {top}"
    );
    // No negative zero in hold.
    let hold = run(&design(json!({ "tank_top_above_deck_m": 0.0 })));
    assert!(
        above(&hold)["weather_shield_mass_t"]
            .as_f64()
            .unwrap()
            .is_sign_positive()
    );
}

/// The 140 x 29 m coastal LNG barge of the worked example, feasible, so damage stability runs on the
/// default Rule Set stack of a new design.
fn example(extra: Value) -> Value {
    let mut base = json!({
        "cargo": "LNG", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
        "length_overall_m": 140.0, "beam_m": 29.0, "cargo_containment": "type_c_tanks",
        "min_freeboard_m": 1.81, "max_tank_od_m": 12.0, "tanks_in_line": 2.0,
        "aft_end_space_m": 0.6, "fwd_end_space_m": 0.5, "max_speed_kn": 12.0,
        "service_area": "coastal-and-river"
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    let inputs: serde_json::Map<String, Value> = base
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), json!({ "value": v, "source": "typed" })))
        .collect();
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
}

fn hold_tops(case: &Value) -> Vec<f64> {
    case["flooded"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["name"].as_str().unwrap().starts_with('H'))
        .map(|s| {
            s["section_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p[1].as_f64().unwrap())
                .fold(f64::MIN, f64::max)
        })
        .collect()
}

/// KN (m) at `deg` of heel of the full load departure condition: GZ plus KG times sin(heel).
fn kn_at(r: &Value, deg: f64) -> f64 {
    let c = r["stability"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Full load, departure")
        .unwrap();
    let gz = c["gz"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p[0].as_f64() == Some(deg))
        .unwrap()[1]
        .as_f64()
        .unwrap();
    gz + c["kg_m"].as_f64().unwrap() * libm::sin(deg.to_radians())
}

/// The volume (m3) of the holds flooded in the governing damage case: each hold section's area
/// times its length.
fn hold_volume(r: &Value) -> f64 {
    r["damage"]["governing"][0]["flooded"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["name"].as_str().unwrap().starts_with('H'))
        .map(|s| {
            let pts: Vec<[f64; 2]> = s["section_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
                .collect();
            let n = pts.len();
            let area = (0..n)
                .map(|i| pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1])
                .sum::<f64>()
                .abs()
                / 2.0;
            let x = s["x_m"].as_array().unwrap();
            area * (x[1].as_f64().unwrap() - x[0].as_f64().unwrap())
        })
        .sum()
}

/// Damage with the tanks standing above the deck or in hold. A side breach across the bulkhead between
/// the tank rows floods both holds; a hold ends at the deck, so what stands above the deck (the tank
/// tops, the shield) gives no buoyancy; the governing case is named; and a shield on or off leaves
/// the hull's buoyancy (its hydrostatic table) as it was.
/// Label: rule (design note D4; IGC 2.7 cases as the engine works them), not a published value.
#[test]
fn damage_floods_both_holds_across_the_bulkhead_and_nothing_above_the_deck_floats() {
    for h in [0.0, 2.0] {
        let r = run(&example(json!({ "tank_top_above_deck_m": h })));
        assert_eq!(r["damage_status"], "run", "h {h}");
        let d = &r["damage"];
        let across = d["cases"].as_array().unwrap().iter().any(|c| {
            let f = c["flooded"].as_array().unwrap();
            let has = |n: &str| f.iter().any(|s| s.as_str() == Some(n));
            has("H1C") && has("H2C")
        });
        assert!(across, "h {h}: no case floods both holds");
        // The governing case is named, and its holds end at the deck.
        let gov = &d["governing"][0];
        assert!(!gov["case"].as_str().unwrap().is_empty());
        for top in hold_tops(gov) {
            assert!(
                (top - depth(&r)).abs() < 1e-6,
                "h {h}: hold top {top} vs depth {}",
                depth(&r)
            );
        }
    }
    // The shield gives no buoyancy: at the same depth the righting lever at large heel, with the
    // weight height taken out (KN = GZ + KG sin(heel)), and the volume of the flooded holds are the
    // same with the shield on or off. A shield that floated would stand the deck edge higher and
    // raise KN at 50 and 60 degrees, and would add to the holds.
    // Issue #935: the shield's mass can move the damage screen's pick, and the pick is not what is compared
    // here, so the tank is capped at a size that survives with the shield on and off: both runs pick it.
    let on = run(&example(
        json!({ "tank_top_above_deck_m": 2.0, "max_tank_od_m": 10.0 }),
    ));
    let off = run(&example(
        json!({ "tank_top_above_deck_m": 2.0, "weather_shield_thickness_mm": 0.0, "max_tank_od_m": 10.0 }),
    ));
    assert_eq!(depth(&on), depth(&off));
    assert_eq!(above(&on)["weather_shield_fitted"], true);
    assert_eq!(above(&off)["weather_shield_fitted"], false);
    for deg in [50.0, 60.0] {
        let (a, b) = (kn_at(&on, deg), kn_at(&off, deg));
        // The shield's mass (about 1.4 % of the displacement) moves KN a little; its buoyancy would
        // move it by a great deal more.
        assert!(
            (a / b - 1.0).abs() < 0.01,
            "KN at {deg}: {a} with the shield, {b} without"
        );
    }
    let (va, vb) = (hold_volume(&on), hold_volume(&off));
    assert!(
        (va / vb - 1.0).abs() < 1e-9,
        "flooded hold volume {va} with the shield, {vb} without"
    );
}

/// The side elevation names the shield as not watertight when the tanks stand above the deck, and keeps
/// its old label without the input (so existing drawings are unchanged).
#[test]
fn the_general_arrangement_says_the_shield_is_not_watertight_only_with_the_input() {
    let with = run(&example(json!({ "tank_top_above_deck_m": 2.0 })));
    let views = ovd_format::ga::ga_views(&with).unwrap();
    let text = views
        .all()
        .iter()
        .map(|(_, v)| v.to_string())
        .collect::<String>();
    assert!(text.contains("WEATHER SHIELD, NOT WATERTIGHT"));
    let without = run(&example(json!({ "moulded_depth_m": 12.0 })));
    let text = ovd_format::ga::ga_views(&without)
        .unwrap()
        .all()
        .iter()
        .map(|(_, v)| v.to_string())
        .collect::<String>();
    assert!(text.contains("WEATHER SHIELD") && !text.contains("NOT WATERTIGHT"));
}

/// The whole run of a river-only design with a share below 0.5 and damage on stays within the damage-run
/// limit: the search's runs (one budget for the share search, at most the limit) and the answer's own
/// damage run (one), counted across the run. `depth_passes` is 1, so the depth was settled in one pass.
/// Label: rule (the limit is `DEFAULT_DAMAGE_RUN_CAP`, 24; issue #935). The bound across several depth
/// passes depends on one runner and budget per run (issue #935, part 2b).
#[test]
fn a_river_design_with_a_lower_share_stays_within_the_damage_run_limit() {
    let before = ovd_format::damage::damage_runs_started();
    let r = run(&river_design(json!({})));
    let runs = ovd_format::damage::damage_runs_started() - before;
    assert_eq!(r["damage_status"], "run");
    let share = r["arrangement"]["tank_top_above_deck_od_fraction"]
        .as_f64()
        .unwrap();
    assert!(
        share < 0.5,
        "share {share}: the test needs a share below the limit"
    );
    assert_eq!(r["arrangement"]["depth_passes"], 1);
    let limit = ovd_core::gas_arrangement::DEFAULT_DAMAGE_RUN_CAP + 1;
    assert!(
        runs <= limit,
        "{runs} damage runs across the whole run, limit {limit}"
    );
}
