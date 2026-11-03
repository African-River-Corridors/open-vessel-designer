//! The cargo detail sheet of a Type C tank (issue #844): one sheet with an elevation and a
//! cross-section at one scale. Each figure on it is read back and compared with the run.
//!
//! Label: property

use ovd_draw::{NoFiles, RunResult, run_value};
use ovd_format::{GaViews, Profile, RunOutcome, check, ga_views_of, run_design};
use serde_json::Value;
use std::path::Path;

fn results_of(name: &str) -> Value {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .expect("fixture");
    let design = check(&text, &Profile::built_in())
        .design
        .expect("a valid Design");
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("no results: {other:?}"),
    }
}

fn views(results: &Value) -> GaViews {
    ga_views_of(&RunOutcome::Results {
        results: results.clone(),
        hash: String::new(),
    })
    .expect("views")
}

fn gas() -> Value {
    results_of("gas_igc_2g_self_propelled.vdesign")
}

fn sheet(results: &Value) -> Value {
    views(results).cargo_detail.expect("a cargo detail sheet")
}

/// The texts of every leader of a viewport.
fn leaders(document: &Value, viewport: usize) -> Vec<String> {
    document["viewports"][viewport]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "leader")
        .map(|e| e["text"].as_str().unwrap().to_string())
        .collect()
}

fn has(texts: &[String], wanted: &str) -> bool {
    texts.iter().any(|t| t == wanted)
}

fn num(v: &Value) -> f64 {
    v.as_f64().expect("a number")
}

#[test]
fn a_type_c_option_has_a_cargo_detail_sheet_that_the_standard_accepts() {
    let results = gas();
    let all = views(&results);
    let names: Vec<&str> = all.all().iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names,
        ["side-elevation", "plan", "midship-section", "cargo-detail"]
    );
    let document = sheet(&results);
    assert!(matches!(run_value(&document, &NoFiles), RunResult::Ok(_)));
    assert_eq!(document["title_block"]["fields"]["title"], "CARGO DETAIL");
    // the TYPE box (25 mm) holds the last word of the title: the whole of it is too wide for the cell
    assert_eq!(document["title_block"]["fields"]["document_type"], "DETAIL");
}

#[test]
fn a_box_hold_option_has_no_cargo_detail_sheet() {
    let results = results_of("dry_bulk_box_hold.vdesign");
    let all = views(&results);
    assert!(all.cargo_detail.is_none());
    assert_eq!(all.all().len(), 3);
}

#[test]
fn both_views_share_one_scale_and_the_title_block_shows_it() {
    let document = sheet(&gas());
    let views = document["viewports"].as_array().unwrap();
    assert_eq!(views.len(), 2);
    assert_eq!(views[0]["scale"], views[1]["scale"]);
    let RunResult::Ok(norm) = run_value(&document, &NoFiles) else {
        panic!("refused")
    };
    let n = views[0]["scale"].as_u64().unwrap();
    let text = norm.to_value().to_string();
    assert!(text.contains(&format!("1:{n}")), "the scale 1:{n} is shown");
}

#[test]
fn every_annotated_figure_reads_back_as_the_runs() {
    let results = gas();
    let tank = &results["drawing"]["tank"];
    let document = sheet(&results);
    let m = |x: f64| format!("{x:.2} m");
    let mm = |x: f64| format!("{x:.1} mm");
    for viewport in [0, 1] {
        let texts = leaders(&document, viewport);
        assert!(has(
            &texts,
            &format!(
                "CYLINDER WALL {}",
                mm(num(&tank["cylinder_wall_thickness_mm"]))
            )
        ));
        assert!(has(
            &texts,
            &format!(
                "STEEL OUTER DIAMETER {}",
                m(num(&tank["tank_steel_outer_diameter_m"]))
            )
        ));
        assert!(has(
            &texts,
            &format!(
                "OUTER DIAMETER OVER INSULATION {}",
                m(num(&tank["tank_outer_diameter_m"]))
            )
        ));
        assert!(has(
            &texts,
            &format!("INSULATION {:.0} mm", num(&tank["insulation_thickness_mm"]))
        ));
        assert!(has(
            &texts,
            &format!("DOME HEIGHT {}", m(num(&tank["dome_height_m"])))
        ));
        assert!(has(
            &texts,
            &format!(
                "SUPPORT BLOCK {} (ESTIMATE)",
                m(num(&results["drawing"]["tank_support_height_m"]))
            )
        ));
    }
    assert!(has(
        &leaders(&document, 0),
        &format!("HEAD WALL {}", mm(num(&tank["head_wall_thickness_mm"])))
    ));
    // the length overall is the standard's dimension of the head tips
    let dimension = document["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["type"] == "dimension")
        .expect("the length dimension");
    let (a, b) = (
        num(&dimension["from"]["point_m"][0]),
        num(&dimension["to"]["point_m"][0]),
    );
    assert!((b - a - num(&tank["overall_tank_length_m"])).abs() < 1e-9);
}

#[test]
fn the_notes_carry_the_pressures_the_temperature_the_material_and_the_derived_volume() {
    let results = gas();
    let tank = &results["drawing"]["tank"];
    let document = sheet(&results);
    let notes: Vec<&str> = document["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    let all = notes.join("\n");
    assert!(all.contains(&format!(
        "DESIGN VAPOUR PRESSURE {:.3} MPA",
        num(&tank["design_pressure_mpa"])
    )));
    assert!(all.contains(&format!(
        "INTERNAL PRESSURE WITH THE LIQUID HEAD {:.3} MPA",
        num(&tank["internal_pressure_mpa"])
    )));
    if let Some(temperature) = tank["design_temperature_c"].as_f64() {
        assert!(all.contains(&format!("DESIGN TEMPERATURE {temperature:.0} DEG C")));
    }
    assert!(all.contains(&format!(
        "MATERIAL {}",
        tank["material"].as_str().unwrap().to_uppercase()
    )));
    let per_tank = num(&results["arrangement"]["per_tank_m3"]);
    let fill = num(&results["arrangement"]["fill_ratio"]);
    assert!(all.contains(&format!(
        "VOLUME AT FILLING LIMIT, PER TANK: {:.1} M3 (DERIVED: {:.1} X {:.2})",
        per_tank * fill,
        per_tank,
        fill
    )));
    assert!(all.contains("ESTIMATE"));
}

#[test]
fn the_heads_follow_the_head_type() {
    let mut results = gas();
    let arcs = |d: &Value| {
        d["viewports"][0]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["type"] == "arc")
            .count()
    };
    let round = arcs(&sheet(&results));
    results["drawing"]["tank"]["head_type"] = "ellipsoidal_2to1".into();
    let elliptical = arcs(&sheet(&results));
    assert!(round > 0, "hemispherical heads are arcs");
    assert_eq!(elliptical, 0, "2:1 heads are drawn as ellipses, not arcs");
}

#[test]
fn a_failed_check_is_named_in_the_last_note() {
    let mut results = gas();
    results["checks"] = serde_json::json!([{"name": "A TEST CHECK", "ok": false, "message": ""}]);
    let document = sheet(&results);
    let notes = document["notes"].as_array().unwrap();
    let last = notes.last().unwrap().as_str().unwrap();
    assert_eq!(last, "DESIGN FAILS 1 CHECK: A TEST CHECK");
}
