//! Seam: a Formation's tug sightline reaches the Rule Trace, light and laden, and SOLAS V/22.1.1 reads the
//! length of the unit, tug and barges (issue #989).
//!
//! Label: property. Each test states a relation between the formation block and the Rule Trace of one run.

use ovd_format::rule_set::Library;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn run(service_area: &str, formation: Value, loa: f64) -> Value {
    run_with(service_area, formation, loa, "blind_distance")
}

fn run_with(service_area: &str, formation: Value, loa: f64, method: &str) -> Value {
    let inputs = json!({
        "cargo": { "value": "bauxite", "source": "typed" },
        "service_area": { "value": service_area, "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": formation, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 3.5, "source": "typed" },
        "tug_included": { "value": true, "source": "typed" },
        "tug_sightline_method": { "value": method, "source": "typed" }
    });
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let design = fill_as_new(&river_limits(&file.to_string()), &Profile::built_in())
        .design
        .unwrap();
    match run_option_with(&design.options[0], &Library::built_in(), &[]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn row<'a>(r: &'a Value, rule: &str) -> &'a Value {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["rule"] == rule && x["outcome"] != "not applicable")
        .unwrap_or_else(|| panic!("{rule}"))
}

#[test]
fn the_tug_sightline_reaches_the_rule_trace_light_and_laden() {
    let r = run(
        "coastal-and-river",
        json!({ "kind": "pushed_convoy", "abreast": 1, "in_line": 3 }),
        76.5,
    );
    let s = &r["formation"]["sightline"];
    let light = row(&r, "es-trin/7.02(2)/blind-zone-unladen");
    assert_eq!(light["value"], s["blind_distance_light_m"], "{light}");
    let solas_laden = row(&r, "solas/V.22.1.1/blind-zone-laden");
    assert_eq!(
        solas_laden["value"], s["blind_distance_laden_m"],
        "{solas_laden}"
    );
    assert_eq!(s["limit_laden_m"], solas_laden["limit"]);
}

#[test]
fn solas_two_lengths_is_worked_on_the_length_of_the_unit_not_the_barge() {
    // Three barges in line and a pusher: the unit is longer than one barge, so 2 L is the unit's.
    let r = run(
        "coastal-and-river",
        json!({ "kind": "pushed_convoy", "abreast": 1, "in_line": 3 }),
        76.5,
    );
    let unit = r["formation"]["equivalent_hull"]["length_m"]
        .as_f64()
        .unwrap();
    assert!(unit > 76.5 * 3.0, "{unit}");
    let solas = row(&r, "solas/V.22.1.1/blind-zone-light");
    let want = (2.0 * unit).min(500.0);
    assert_eq!(solas["limit"].as_f64().unwrap(), want, "{solas}");
    // The sightline block is sized to the least limit that blocks: the same reading.
    let least = want.min(250.0);
    assert_eq!(
        r["formation"]["sightline"]["limit_m"].as_f64().unwrap(),
        least
    );
}

#[test]
fn a_river_only_formation_is_judged_on_the_es_trin_rule_alone() {
    let r = run(
        "river-only",
        json!({ "kind": "pushed_convoy", "abreast": 1, "in_line": 3 }),
        76.5,
    );
    let light = row(&r, "es-trin/7.02(2)/blind-zone-unladen");
    assert_eq!(light["limit"], json!(250.0));
    assert_eq!(
        light["value"],
        r["formation"]["sightline"]["blind_distance_light_m"]
    );
    assert!(r["formation"]["sightline"].get("limit_laden_m").is_none());
}

#[test]
fn the_unit_length_reaches_the_blind_distance_rules_only() {
    // The ICLL freeboard belongs to each barge: its row is the same with the tug's sightline switched on and off.
    let convoy = json!({ "kind": "pushed_convoy", "abreast": 1, "in_line": 3 });
    let on = run_with("coastal-and-river", convoy.clone(), 76.5, "blind_distance");
    let off = run_with("coastal-and-river", convoy, 76.5, "margin");
    let rule = "icll/28(1)/type-a-tabular-freeboard-laden-barge";
    let (a, b) = (row(&on, rule), row(&off, rule));
    assert_eq!(a["limit"], b["limit"], "{a} {b}");
    assert_eq!(a["outcome"], b["outcome"]);
}

#[test]
fn solas_two_lengths_of_the_unit_is_capped_at_500_m() {
    let r = run(
        "coastal-and-river",
        json!({ "kind": "pushed_convoy", "abreast": 1, "in_line": 4 }),
        76.5,
    );
    let unit = r["formation"]["equivalent_hull"]["length_m"]
        .as_f64()
        .unwrap();
    assert!(2.0 * unit > 500.0, "{unit}");
    assert_eq!(
        row(&r, "solas/V.22.1.1/blind-zone-light")["limit"],
        json!(500.0)
    );
    assert_eq!(
        row(&r, "solas/V.22.1.1/blind-zone-laden")["limit"],
        json!(500.0)
    );
}

/// Issue #1034: a river-only Option needs a number for both river limits. These are high enough not to bind, so
/// no value of these tests moves.
fn river_limits(text: &str) -> String {
    ovd_format::with_river_limits(text, Some(12.0), Some(40.0)).unwrap()
}
