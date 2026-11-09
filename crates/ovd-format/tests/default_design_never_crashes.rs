//! Seam: a default design never crashes on the depth that follows the tank (issue #1033). Since the Defaults Profile
//! stands the tank tops of a coastal design 2 m above the deck, the depth is the tank top less that height; a narrow
//! hull's tank at a trial depth can be smaller than the height, and the depth went to zero or below and the run
//! failed with "beam and depth must be positive". Such a candidate is now refused, with the reason named, as one
//! whose tank is smaller than its standing height always was (design note D1 of #992: the search may pick another
//! arrangement; none is better than a message).
//!
//! Label: rule - a default design runs; the sweep prints its table.

use ovd_format::damage::DamageControl;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option, run_option_with_damage};
use serde_json::{Value, json};

fn coastal(loa: f64, beam: f64, extra: Value) -> ovd_format::vdesign::Design {
    try_coastal(loa, beam, extra).unwrap()
}

/// The design, or the input errors the fill reports (a typed value the inputs themselves refuse).
fn try_coastal(
    loa: f64,
    beam: f64,
    extra: Value,
) -> Result<ovd_format::vdesign::Design, Vec<String>> {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    if !filled.report.errors.is_empty() {
        return Err(filled.report.errors);
    }
    Ok(filled.design.unwrap())
}

fn results(d: &ovd_format::vdesign::Design) -> Value {
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// The 110 x 11.4 m coastal LNG design with nothing typed but its size ran: it failed with "beam and depth must be
/// positive". Now it runs, with a tank that fits (one across, 9.07 m, four along; 8.95 m before issue #1031 let the damage refinement reach the larger size) and a positive depth.
/// Label: snapshot, printed by the code.
#[test]
fn a_default_110_by_11_4_m_coastal_design_runs() {
    let r = results(&coastal(110.0, 11.4, json!({})));
    assert_eq!(r["arrangement"]["feasible"], true, "{}", r["arrangement"]);
    assert!(r["dims"]["depth_m"].as_f64().unwrap() > 0.0);
    assert_eq!(r["arrangement"]["n_wide"], 1);
    assert!((r["arrangement"]["tank_od_m"].as_f64().unwrap() - 9.072).abs() < 0.001);
}

/// The 140 x 29 m hull at a typed depth of 6 m with tanks capped at 8 m runs.
#[test]
fn a_140_by_29_m_design_at_6_m_with_8_m_tanks_runs() {
    let r = results(&coastal(
        140.0,
        29.0,
        json!({ "moulded_depth_m": 6.0, "max_tank_od_m": 8.0 }),
    ));
    assert!((r["dims"]["depth_m"].as_f64().unwrap() - 6.0).abs() < 1e-9);
}

/// Three tanks across on the narrow hull put each tank under the 2 m standing height at the trial depths: the
/// design is infeasible, with a positive depth and the reason in the list of arrangements tried, not a crash.
#[test]
fn three_across_on_the_narrow_hull_is_infeasible_not_a_crash() {
    let r = results(&coastal(110.0, 11.4, json!({ "tanks_across": 3 })));
    assert_eq!(r["arrangement"]["feasible"], false);
    assert!(r["dims"]["depth_m"].as_f64().unwrap() > 0.0);
}

/// A grid of narrow coastal hulls at the defaults: every one runs (no "beam and depth must be positive"), with the
/// damage run skipped to keep the sweep quick (the crash is in the search, before damage). **Mutation proof:**
/// removing the guards on a depth of zero or less brings the crash back, and the 110 x 11.4 m test and this sweep fail.
#[test]
fn narrow_coastal_hulls_at_the_defaults_never_crash() {
    let skip = DamageControl {
        run: false,
        progress: None,
    };
    let mut table = Vec::new();
    for beam in [10.0, 12.0, 14.0, 16.0] {
        for loa in [80.0, 100.0, 120.0, 140.0] {
            let d = coastal(loa, beam, json!({}));
            match run_option_with_damage(
                &d.options[0],
                ovd_format::rule_set::built_in(),
                &[],
                &skip,
            ) {
                RunOutcome::Results { results: r, .. } => table.push(format!(
                    "| {loa:.0} | {beam:.0} | {} | {} x {} | {:.2} | {} |",
                    r["dims"]["depth_m"],
                    r["arrangement"]["n_wide"],
                    r["arrangement"]["n_long"],
                    r["arrangement"]["tank_od_m"].as_f64().unwrap_or(0.0),
                    r["arrangement"]["feasible"]
                )),
                other => panic!("{loa} x {beam}: {other:?}"),
            }
        }
    }
    eprintln!(
        "| LOA m | beam m | depth m | across x along | tank m | feasible |\n|---|---|---|---|---|---|\n{}",
        table.join("\n")
    );
}

/// Issue #1039: the 110 x 11.4 m coastal design with a typed minimum freeboard of 5.0 m failed with "design_depth_m and
/// ukc_m must be non-negative". A candidate's draught cap is its depth less the freeboard, and a depth below the freeboard
/// took the cap below zero. Such a candidate is now refused with the cause named; the design runs.
#[test]
fn a_typed_freeboard_deeper_than_the_depth_runs_not_a_crash() {
    let r = results(&coastal(110.0, 11.4, json!({ "min_freeboard_m": 5.0 })));
    assert!(r["dims"]["depth_m"].as_f64().unwrap() > 0.0);
}

/// Issue #1039: typed minimum freeboards from 0.5 to 6 m on two narrow hulls never crash (damage run skipped, as above).
/// **Mutation proof:** removing the `t_cap < 0.0` guards in `gas_arrangement.rs` brings back "design_depth_m and ukc_m
/// must be non-negative", and this sweep and the 5.0 m test above fail.
#[test]
fn typed_freeboards_on_narrow_hulls_never_crash() {
    let skip = DamageControl {
        run: false,
        progress: None,
    };
    let mut table = Vec::new();
    for (loa, beam) in [(110.0, 11.4), (80.0, 10.0)] {
        for fb in [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
            let d = match try_coastal(loa, beam, json!({ "min_freeboard_m": fb })) {
                Ok(d) => d,
                Err(e) => {
                    // The inputs refuse it before the search, with their own message.
                    table.push(format!(
                        "| {loa:.0} x {beam} | {fb} | - | - | refused at the inputs: {} |",
                        e[0]
                    ));
                    continue;
                }
            };
            match run_option_with_damage(
                &d.options[0],
                ovd_format::rule_set::built_in(),
                &[],
                &skip,
            ) {
                RunOutcome::Results { results: r, .. } => table.push(format!(
                    "| {loa:.0} x {beam} | {fb} | {:.2} | {} x {} | {} |",
                    r["dims"]["depth_m"].as_f64().unwrap(),
                    r["arrangement"]["n_wide"],
                    r["arrangement"]["n_long"],
                    r["arrangement"]["feasible"]
                )),
                other => panic!("{loa} x {beam}, freeboard {fb}: {other:?}"),
            }
        }
    }
    eprintln!(
        "| Hull m | min freeboard m | depth m | across x along | feasible |\n|---|---|---|---|---|\n{}",
        table.join("\n")
    );
}
