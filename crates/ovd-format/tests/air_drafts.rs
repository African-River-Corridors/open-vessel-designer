//! Issue #888: the river and sea air draft of each loading condition (`air_drafts` of the results).
//!
//! Label: property. The sea air draft of a condition is the sea top less the condition's draught,
//! checked here by hand against the vent exit and the draught; the river air draft is the engine's
//! own `air_draft_m` of the condition; `vent_mast_lowerable` changes the river column only; the block
//! is outside the results hash; the side elevation labels read back the run's figures.

use ovd_draw::{NoFiles, RunResult, run_value};
use ovd_format::{RunOutcome, UNHASHED_KEYS};
use serde_json::{Value, json};

fn run(lowerable: bool, limit: Option<f64>) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gas_igc_2g_self_propelled.vdesign");
    let mut design: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let option = &mut design["options"][0];
    let mut put = |k: &str, v: Value| option["inputs"][k] = json!({"value": v, "source": "typed"});
    put("vent_mast_lowerable", json!(lowerable));
    // One arrangement, so a limit cannot be answered with a lower hull.
    put("tanks_across", json!(1));
    if let Some(l) = limit {
        put("air_draft_limit_m", json!(l));
    }
    option.as_object_mut().unwrap().remove("results_hash");
    let checked = ovd_format::check(&design.to_string(), &ovd_format::Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    match ovd_format::run_design(&checked.design.unwrap())
        .remove(0)
        .outcome
    {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn f(v: &Value) -> f64 {
    v.as_f64().expect("a number")
}

#[test]
fn the_sea_air_draft_of_each_condition_is_the_sea_top_less_its_draught() {
    let r = run(false, None);
    let a = &r["air_drafts"];
    let exit = f(&r["results"]["vent_exit_above_keel_m"]);
    let sea_top = f(&a["sea_top_m"]);
    // the sea top stands at or above the vent exit, which the mast always has
    assert!(sea_top >= exit - 1e-12, "{sea_top} vs {exit}");
    let conditions = a["conditions"].as_array().unwrap();
    assert!(conditions.len() >= 4);
    for c in conditions {
        let draught = f(&c["draught_m"]);
        assert!((f(&c["sea_m"]) - (sea_top - draught)).abs() < 1e-9, "{c}");
    }
    // the laden figures follow the laden draught of the results
    let t = f(&r["results"]["draught_m"]);
    assert!((f(&a["laden"]["draught_m"]) - t).abs() < 1e-12);
    assert!((f(&a["laden"]["sea_m"]) - (sea_top - t)).abs() < 1e-9);
    assert!((f(&a["laden"]["river_m"]) - f(&r["results"]["air_draft_loaded_m"])).abs() < 1e-9);
    // each condition's river air draft is the engine's own
    for (c, s) in conditions
        .iter()
        .zip(r["stability"]["conditions"].as_array().unwrap())
    {
        assert_eq!(c["name"], s["name"]);
        assert!((f(&c["river_m"]) - f(&s["air_draft_m"])).abs() < 1e-9);
    }
}

#[test]
fn a_lowerable_mast_changes_the_river_column_only() {
    let up = run(false, None);
    let down = run(true, None);
    let (au, ad) = (&up["air_drafts"], &down["air_drafts"]);
    assert_eq!(au["sea_top_m"], ad["sea_top_m"]);
    assert!(f(&ad["river_top_m"]) < f(&au["river_top_m"]) - 0.1);
    for (cu, cd) in au["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .zip(ad["conditions"].as_array().unwrap())
    {
        assert_eq!(cu["sea_m"], cd["sea_m"], "{}", cu["name"]);
        assert!(
            f(&cd["river_m"]) < f(&cu["river_m"]) - 0.1,
            "{}",
            cu["name"]
        );
    }
    assert_eq!(au["laden"]["sea_m"], ad["laden"]["sea_m"]);
    assert_eq!(au["light"]["sea_m"], ad["light"]["sea_m"]);
    assert_eq!(au["vent_mast_lowerable"], json!(false));
    assert_eq!(ad["vent_mast_lowerable"], json!(true));
}

#[test]
fn the_limit_outcome_follows_the_river_air_draft_when_a_limit_is_typed() {
    let none = run(true, None);
    assert!(none["air_drafts"]["limit_m"].is_null());
    assert!(
        none["air_drafts"]["conditions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["river_ok"].is_null())
    );
    // A limit no hull meets, and one every hull meets: the outcome follows the river air draft of
    // each condition against the limit, and the sea column has no limit.
    for (limit, want_all_ok) in [(0.5, false), (60.0, true)] {
        let r = run(true, Some(limit));
        assert_eq!(f(&r["air_drafts"]["limit_m"]), limit);
        let rows = r["air_drafts"]["conditions"].as_array().unwrap();
        assert!(!rows.is_empty());
        for c in rows {
            assert_eq!(c["river_ok"], json!(f(&c["river_m"]) <= limit), "{c}");
        }
        let all_ok = rows.iter().all(|c| c["river_ok"] == json!(true));
        assert_eq!(all_ok, want_all_ok);
    }
}

#[test]
fn the_block_is_outside_the_results_hash() {
    assert!(UNHASHED_KEYS.contains(&"air_drafts"));
    let r = run(false, None);
    let mut bare = r.clone();
    bare.as_object_mut().unwrap().remove("air_drafts");
    assert_eq!(
        ovd_format::results_hash(&r),
        ovd_format::results_hash(&bare)
    );
}

fn leaders(document: &Value) -> Vec<String> {
    document["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "leader")
        .map(|e| e["text"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_side_elevation_labels_read_back_the_runs_air_drafts() {
    let r = run(true, None);
    let a = &r["air_drafts"];
    let views = ovd_format::ga_views(&r).expect("views");
    let elevation = &views.side_elevation;
    assert!(matches!(run_value(elevation, &NoFiles), RunResult::Ok(_)));
    let texts = leaders(elevation);
    let m = |x: &Value| format!("{:.2} m", f(x));
    let laden = format!(
        "AIR DRAFT, LADEN: RIVER {}, SEA {}",
        m(&a["laden"]["river_m"]),
        m(&a["laden"]["sea_m"])
    );
    let light = format!(
        "AIR DRAFT, LIGHT: RIVER {}, SEA {}",
        m(&a["light"]["river_m"]),
        m(&a["light"]["sea_m"])
    );
    assert!(texts.contains(&laden), "{laden} in {texts:?}");
    assert!(texts.contains(&light), "{light} in {texts:?}");
    // the sea figure of a lowerable mast is above the river one on both labels
    assert!(f(&a["laden"]["sea_m"]) > f(&a["laden"]["river_m"]));
}
