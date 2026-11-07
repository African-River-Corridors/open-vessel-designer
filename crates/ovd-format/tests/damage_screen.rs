//! Seam: the arrangement search screens candidates by IGC 2.7 damage survival (issue #935).
//!
//! Label: snapshot

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn reference(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "moulded_depth_m": { "value": 12.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "loading_conditions": { "value": "is_code_2008", "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn igc_fails(r: &Value) -> Vec<String> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| {
            x["outcome"] == "fail" && x["rule"].as_str().unwrap_or("").starts_with("igc/2.7")
        })
        .map(|x| format!("{}: {}", x["rule"], x["object"]))
        .collect()
}

fn od(r: &Value) -> f64 {
    r["arrangement"]["tank_od_m"].as_f64().unwrap()
}

/// With damage in the search, a size cap above what survives gives the largest tank that does: 11.5 m, where
/// the search without damage gave 12.0 m (4 failing rows) and 13.0 m (24). Label: snapshot, printed by the code
/// on `main` after #987 and #988. The limit is provisional until #1009 (the free surface of the cargo in a
/// breached tank) lands.
#[test]
fn the_search_picks_the_largest_tank_that_survives_damage() {
    for cap in [12.0, 13.0] {
        let r = reference(json!({"max_tank_od_m": cap}));
        assert_eq!(igc_fails(&r), Vec::<String>::new(), "cap {cap}");
        assert_eq!(od(&r), 11.5, "cap {cap}");
        assert_eq!(r["arrangement"]["n_wide"], 2);
    }
    // Below the limit nothing changes: 11.0 m passes damage, and is the cap.
    let r = reference(json!({"max_tank_od_m": 11.0}));
    assert_eq!(igc_fails(&r), Vec::<String>::new());
    assert_eq!(od(&r), 11.0);
}

/// `igc-2g` 2.0.0 does not evaluate IGC 2.7: no screen, today's tank. Label: snapshot.
#[test]
fn a_stack_without_the_damage_rule_is_not_screened() {
    let lib = ovd_format::rule_set::built_in();
    let stack: Vec<Value> = [("igc-2g", "2.0.0"), ("is2008-a", "2.0.0"), ("ovd", "3.1.0")]
        .iter()
        .map(|(id, v)| {
            let set = lib
                .sets()
                .iter()
                .find(|s| s.id == *id && s.version == *v)
                .unwrap();
            json!({"id": id, "version": v, "hash": set.hash, "source": "typed"})
        })
        .collect();
    for (cap, tank) in [(12.0, 12.0), (13.0, 13.0)] {
        let before = ovd_format::damage::damage_runs_started();
        let r = reference(json!({"max_tank_od_m": cap, "rule_sets": stack}));
        assert_eq!(od(&r), tank);
        assert_eq!(
            ovd_format::damage::damage_runs_started(),
            before,
            "no damage ran"
        );
    }
}

/// A candidate turned down by damage says which Rule and which case: the rule id starts `igc/2.7`, the
/// message holds the condition and the case, and the Rule id tells the final stage from the intermediate ones.
/// A 29 x 9 m hull has no tank that survives, so the closest is reported and the design is infeasible.
#[test]
fn a_candidate_rejected_by_damage_names_the_rule_and_the_case() {
    let r = reference(json!({"moulded_depth_m": 9.0, "max_tank_od_m": 16.0, "tanks_across": 2}));
    assert_eq!(r["arrangement"]["feasible"], false);
    let tried = r["fit_tried"].as_array().unwrap();
    let reasons: Vec<&Value> = tried
        .iter()
        .flat_map(|t| t["reasons"].as_array().unwrap())
        .filter(|k| k["name"].as_str().unwrap().starts_with("igc/2.7"))
        .collect();
    assert!(!reasons.is_empty(), "{tried:?}");
    let names: Vec<&str> = reasons
        .iter()
        .map(|k| k["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"igc/2.7.1.2/heel-any-stage"), "{names:?}");
    assert!(
        names.contains(&"igc/2.7.1.3/intermediate-residual-stability"),
        "{names:?}"
    );
    assert!(
        reasons.iter().all(|k| k["message"]
            .as_str()
            .unwrap()
            .contains("Full load, departure: side damage")),
        "{reasons:?}"
    );
}

/// The search runs damage once per candidate that passes stability and has more cargo than the winner, and a
/// run counts its damage runs. Label: snapshot: 5 runs in the search at a 12 m cap and 9 at 13 m, and one more
/// for the answer (the answer's run is the same one until the reuse of PR 4).
#[test]
fn the_search_counts_its_damage_runs() {
    for (cap, runs) in [(12.0, 6), (13.0, 10)] {
        let before = ovd_format::damage::damage_runs_started();
        let _ = reference(json!({"max_tank_od_m": cap}));
        assert_eq!(
            ovd_format::damage::damage_runs_started() - before,
            runs,
            "cap {cap}"
        );
    }
}

/// A cancel from the progress hook stops the search, as it stops the answer's run (#924, #940).
#[test]
fn a_cancel_stops_the_search() {
    use ovd_format::damage::{ControlFlow, DamageControl};
    let calls = std::rc::Rc::new(std::cell::Cell::new(0usize));
    let seen = calls.clone();
    let control = DamageControl {
        run: true,
        progress: Some(std::rc::Rc::new(move |p| {
            seen.set(seen.get() + 1);
            if p.done >= 2 {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        })),
    };
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 140.0, "source": "typed" },
            "beam_m": { "value": 29.0, "source": "typed" },
            "moulded_depth_m": { "value": 12.0, "source": "typed" },
            "max_tank_od_m": { "value": 13.0, "source": "typed" },
            "service_area": { "value": "coastal-and-river", "source": "typed" },
            "loading_conditions": { "value": "is_code_2008", "source": "typed" }
        } }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    let before = ovd_format::damage::damage_runs_started();
    match ovd_format::run_option_with_damage(
        &filled.design.unwrap().options[0],
        ovd_format::rule_set::built_in(),
        &[],
        &control,
    ) {
        RunOutcome::Failed(m) => assert!(m.contains("cancelled"), "{m}"),
        other => panic!("{other:?}"),
    }
    // The first damage run started and was cancelled: no second candidate ran.
    assert_eq!(ovd_format::damage::damage_runs_started() - before, 1);
}
