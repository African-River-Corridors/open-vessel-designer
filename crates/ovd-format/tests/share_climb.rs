//! Seam: the climb share rule through a Design (issue #1034, Phase 3, design note D2, rule C). With the
//! share of the tank diameter searched, the search starts at the tallest share that passes the intact
//! checks and goes down one share at a time while the cargo rises by at least 1 %
//! (`SHARE_CLIMB_MIN_GAIN`); it keeps the last share that rose. River-only designs so carry the most
//! cargo within their river limits, and a climb that would buy a much deeper hull for no cargo stops.
//! The summary line of `ovd run` names the rule, and the share the climb left with its cargo.
//!
//! Label: snapshot. The tonnes and depths are what the code printed on the base of Phase 3
//! (`ovd run --json`, each share typed as a limit, 2026-11-10; design note D2 and research section 5
//! of issue #1034 measured the same). Which share is picked, and where the climb stops, is the rule.
// DONE-TEST #1034 P3 — do not edit in the build

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option, summary_lines};
use serde_json::{Value, json};

/// A river-only LNG design on the default stack, self-propelled, with the two river limits typed.
fn river_design(loa: f64, beam: f64, draught_limit_m: f64, air_draft_limit_m: f64) -> Value {
    let base = json!({
        "cargo": "LNG", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
        "length_overall_m": loa, "beam_m": beam, "service_area": "river-only",
        "draught_limit_m": draught_limit_m, "air_draft_limit_m": air_draft_limit_m
    });
    let inputs: serde_json::Map<String, Value> = base
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), json!({ "value": v, "source": "typed" })))
        .collect();
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
}

fn run(file: &Value) -> Value {
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn share(r: &Value) -> f64 {
    r["arrangement"]["tank_top_above_deck_od_fraction"]
        .as_f64()
        .unwrap()
}

fn cargo_t(r: &Value) -> f64 {
    r["results"]["cargo_t"].as_f64().unwrap()
}

fn depth_m(r: &Value) -> f64 {
    r["dims"]["depth_m"].as_f64().unwrap()
}

/// The summary line that names the share.
fn share_line(r: &Value) -> String {
    summary_lines("Option 1", r)
        .into_iter()
        .find(|l| l.contains("Tank tops stand"))
        .unwrap_or_else(|| panic!("no share line in {:?}", summary_lines("Option 1", r)))
}

/// The 100 x 20 m river-only LNG barge under the CEMT VIa limits (draught 4.5 m, air draft 9.1 m).
/// The tallest share that passes is 0.5: 2 x 2 tanks of 6.1 m, 1,328 t on a 4.80 m hull. One share
/// down, 0.375 carries 1,559 t (+17 %) on a 5.22 m hull; 0.25 falls to 1,527 t. The climb keeps 0.375.
/// On the base ("tallest that passes") this picks 0.5 and 1,328 t.
#[test]
fn with_binding_limits_the_climb_picks_the_share_with_the_most_cargo() {
    let r = run(&river_design(100.0, 20.0, 4.5, 9.1));
    assert_eq!(r["arrangement"]["feasible"], true, "{}", r["arrangement"]);
    assert_eq!(share(&r), 0.375, "{}", r["arrangement"]);
    assert_eq!(r["arrangement"]["n_wide"], 2);
    assert_eq!(r["arrangement"]["n_long"], 2);
    let od = r["arrangement"]["tank_od_m"].as_f64().unwrap();
    assert!((od - 6.6).abs() < 0.01, "tank OD {od} m");
    let t = cargo_t(&r);
    assert!((t - 1558.95).abs() < 0.5, "cargo {t} t");
    let d = depth_m(&r);
    assert!((d - 5.225).abs() < 0.01, "depth {d} m");
    // The summary names the rule and the share the climb left, with its cargo.
    let line = share_line(&r);
    assert!(line.contains("share 0.375"), "{line}");
    assert!(line.contains("the most cargo within the limits"), "{line}");
    assert!(line.contains("0.500 carries 1328.1 t"), "{line}");
}

/// The same barge with limits that do not bind (draught 3.5 m, air draft 40 m). 0.5 fails the intact
/// checks; 0.375 is the tallest that passes, 2 x 2 tanks of 8.1 m, 2,351 t on a 6.16 m hull. One
/// share down, 0.25 carries 2,345 t on a 7.18 m hull: the cargo falls, so the climb stops at 0.375 and
/// never reaches 0.125 (one tank across, 13.0 m, 3,055 t on a 12.5 m hull, air draft 15.6 m).
/// "Most cargo over every share" would take that hull; the base already gives 0.375, with no rule named.
#[test]
fn the_climb_stops_when_the_cargo_falls() {
    let r = run(&river_design(100.0, 20.0, 3.5, 40.0));
    assert_eq!(r["arrangement"]["feasible"], true, "{}", r["arrangement"]);
    assert_eq!(share(&r), 0.375, "{}", r["arrangement"]);
    assert_eq!(r["arrangement"]["n_wide"], 2);
    let t = cargo_t(&r);
    assert!((t - 2350.73).abs() < 0.5, "cargo {t} t");
    let d = depth_m(&r);
    assert!((d - 6.163).abs() < 0.01, "depth {d} m");
    // The climb did not move the share: the line names the rule and no other share.
    let line = share_line(&r);
    assert!(line.contains("share 0.375"), "{line}");
    assert!(line.contains("the most cargo within the limits"), "{line}");
    assert!(!line.contains("carries"), "{line}");
}

/// The 110 x 11.4 m river-only LNG tanker (CEMT Va size) with limits that do not bind (3.5 m, 40 m).
/// 0.5 passes: one tank across, 2 along, 7.6 m, 1,163 t on a 4.90 m hull. 0.375 carries 1,461 t
/// (+26 %) on a 6.48 m hull: the climb takes it. 0.25 and 0.125 carry the same 1,461 t on 7.55 and
/// 8.63 m hulls: a gain under 1 % (here 0 %) does not move the share, so the climb keeps 0.375.
/// On the base this picks 0.5 and 1,163 t.
#[test]
fn a_gain_under_one_per_cent_does_not_move_the_share() {
    let r = run(&river_design(110.0, 11.4, 3.5, 40.0));
    assert_eq!(r["arrangement"]["feasible"], true, "{}", r["arrangement"]);
    assert_eq!(share(&r), 0.375, "{}", r["arrangement"]);
    assert_eq!(r["arrangement"]["n_wide"], 1);
    assert_eq!(r["arrangement"]["n_long"], 2);
    let t = cargo_t(&r);
    assert!((t - 1461.09).abs() < 0.5, "cargo {t} t");
    let d = depth_m(&r);
    assert!((d - 6.475).abs() < 0.01, "depth {d} m");
    let line = share_line(&r);
    assert!(line.contains("share 0.375"), "{line}");
    assert!(line.contains("0.500 carries 1163.1 t"), "{line}");
}
