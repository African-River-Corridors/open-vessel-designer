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
        // Issue #935: these tests are about the damage screen at the profile's two along; the choice of the number
        // along is in `damage_tanks_along.rs`.
        "tanks_in_line": { "value": 2, "source": "typed" },
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

/// With damage in the search, a size cap above what survives gives the largest tank that does: 11.625 m (11.5 m before issue #1031 let the
/// refinement reach it), where
/// the search without damage gave 12.0 m (4 failing rows) and 13.0 m (24). Label: snapshot, printed by the code
/// on `main` after #987 and #988. The limit is provisional until #1009 (the free surface of the cargo in a
/// breached tank) lands.
#[test]
fn the_search_picks_the_largest_tank_that_survives_damage() {
    for cap in [12.0, 13.0] {
        let r = reference(json!({"max_tank_od_m": cap}));
        assert_eq!(igc_fails(&r), Vec::<String>::new(), "cap {cap}");
        assert_eq!(od(&r), 11.625, "cap {cap}");
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
/// run counts its damage runs. Label: snapshot: 5 runs in the search at a 12 m cap and 7 at 13 m (the refinement between sizes included; 3 and 5 before issue #1031), and one more
/// for the answer (the answer's run is the same one until the reuse of PR 4).
#[test]
fn the_search_counts_its_damage_runs() {
    for (cap, runs) in [(12.0, 6), (13.0, 8)] {
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

/// The reference design as a new Option, run at stage 1 (damage skipped: `not run`) and at stage 2 (damage run):
/// (results, hash) of each.
fn two_stages(extra: Value) -> ((Value, String), (Value, String)) {
    use ovd_format::damage::DamageControl;
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "moulded_depth_m": { "value": 12.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        // Issue #935: these tests are about the damage screen at the profile's two along; the choice of the number
        // along is in `damage_tanks_along.rs`.
        "tanks_in_line": { "value": 2, "source": "typed" },
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
    let option = &filled.design.unwrap().options[0];
    let run = |control: &DamageControl| match ovd_format::run_option_with_damage(
        option,
        ovd_format::rule_set::built_in(),
        &[],
        control,
    ) {
        RunOutcome::Results { results, hash } => (results, hash),
        other => panic!("{other:?}"),
    };
    let skip = DamageControl {
        run: false,
        progress: None,
    };
    (run(&skip), run(&DamageControl::default()))
}

/// Stage 1 (damage not run) keeps the unscreened pick and says it was not screened; stage 2 screens, and says
/// what stage 1 had picked when the pick changes. The hash is stable for the same stage, and differs between the
/// stages exactly when the pick differs. Label: snapshot (a 12 m cap gives 12.0 m at stage 1 and 11.625 m at
/// stage 2, an 11 m cap gives 11.0 m at both).
#[test]
fn stage_one_says_it_is_not_screened_and_stage_two_says_what_it_changed() {
    let ((one, h1), (two, h2)) = two_stages(json!({ "max_tank_od_m": 12.0 }));
    assert_eq!(one["damage_status"], "not run");
    assert_eq!(one["damage_screening"]["screened"], false);
    assert!(
        one["damage_screening"]["note"]
            .as_str()
            .unwrap()
            .starts_with("Not screened for damage")
    );
    assert_eq!((od(&one), od(&two)), (12.0, 11.625));
    assert_eq!(two["damage_screening"]["screened"], true);
    assert_eq!(
        two["damage_screening"]["stage_one"]["tank_od_m"],
        json!(12.0)
    );
    assert!(
        two["damage_screening"]["changed_note"]
            .as_str()
            .unwrap()
            .starts_with("Damage changed the arrangement")
    );
    assert_ne!(h1, h2, "the pick moved, so the hash moves");
    // The hash is stable for the same stage.
    let ((_, h1b), (_, h2b)) = two_stages(json!({ "max_tank_od_m": 12.0 }));
    assert_eq!((h1, h2), (h1b, h2b));

    // A pick that survives damage does not move: no change note, and the same hash at both stages.
    let ((one, h1), (two, h2)) = two_stages(json!({ "max_tank_od_m": 11.0 }));
    assert_eq!((od(&one), od(&two)), (11.0, 11.0));
    assert_eq!(one["damage_screening"]["screened"], false);
    assert_eq!(two["damage_screening"]["screened"], true);
    assert!(two["damage_screening"]["stage_one"].is_null());
    assert!(two["damage_screening"]["changed_note"].is_null());
    assert_eq!(h1, h2);
}

/// With no cap the search does not stop at the first size that passes: 13.28 m, 12.78 m, 12.28 m and 11.78 m fail
/// (two across), 11.28 m passes with less cargo than one 16 m tank across, and the search refines the diameter
/// between 11.78 m and 11.28 m, so the pick is two across at about 11.5 m (18,224 m3 at 11.5 m), not one 16 m tank
/// (17,708 m3). Label: snapshot, printed by the code.
#[test]
fn with_no_cap_the_pick_is_two_across_at_about_11_5_m_not_one_16_m_tank() {
    let r = reference(json!({}));
    assert_eq!(igc_fails(&r), Vec::<String>::new());
    assert_eq!(r["arrangement"]["n_wide"], 2);
    let tank = od(&r);
    assert!((11.4..=11.7).contains(&tank), "{tank}");
    let chosen = r["fit_tried"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["outcome"] == "chosen")
        .unwrap();
    let volume = chosen["cargo_volume_m3"].as_f64().unwrap();
    assert!(volume >= 18_200.0, "{volume}");
    assert!(volume > 17_708.0);
}

fn chosen_volume(r: &Value) -> f64 {
    r["fit_tried"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["outcome"] == "chosen")
        .unwrap()["cargo_volume_m3"]
        .as_f64()
        .unwrap()
}

/// Removing the size cap never gives less cargo: a cap of 12 m is a limit on what the search may pick, so the
/// uncapped pick carries at least what the capped one does (the damage screen once gave 17,708 m3 with no cap
/// against 18,224 m3 at 12 m). The refinement bisects the diameter to 0.1 m, so each search ends within one step of
/// the largest size that passes, and a capped and an uncapped search with different size grids can end a step
/// apart (11.593 m uncapped, 11.625 m at a 12 m cap): the uncapped pick carries at least what the capped one does,
/// less one step of diameter (2 x 0.1 / 11.6, under 2 % of the volume). Issue #1045 asks for the strict form.
/// Label: property.
#[test]
fn removing_the_cap_never_gives_less_cargo() {
    let capped = chosen_volume(&reference(json!({ "max_tank_od_m": 12.0 })));
    let uncapped = chosen_volume(&reference(json!({})));
    assert!(uncapped >= capped * 0.98, "{uncapped} against {capped}");
}

/// The progress fraction never falls while the search's runs start: `done` is cumulative and `total` is planned (the
/// runs the search expects, of the first run's cases) and shrinks to the real total when the answer's run starts.
/// Label: property, at a 12 m cap, a 13 m cap and no cap.
#[test]
fn the_progress_fraction_never_falls_over_the_runs_of_the_search() {
    use ovd_format::damage::{ControlFlow, DamageControl, DamageProgress};
    for extra in [
        json!({ "max_tank_od_m": 12.0 }),
        json!({ "max_tank_od_m": 13.0 }),
        json!({}),
    ] {
        let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::<DamageProgress>::new()));
        let seen = log.clone();
        let control = DamageControl {
            run: true,
            progress: Some(std::rc::Rc::new(move |p| {
                seen.borrow_mut().push(p);
                ControlFlow::Continue(())
            })),
        };
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
        let RunOutcome::Results { results, .. } = ovd_format::run_option_with_damage(
            &filled.design.unwrap().options[0],
            ovd_format::rule_set::built_in(),
            &[],
            &control,
        ) else {
            panic!("no results");
        };
        let calls = log.borrow();
        assert!(calls.len() > 10);
        let fractions: Vec<f64> = calls
            .iter()
            .map(|p| p.done as f64 / p.total as f64)
            .collect();
        for (i, w) in fractions.windows(2).enumerate() {
            assert!(
                w[1] >= w[0] - 1e-12,
                "{extra}: call {i}: {} then {}",
                w[0],
                w[1]
            );
        }
        assert!(calls.iter().all(|p| p.done <= p.total));
        assert!(calls.windows(2).all(|w| w[0].done <= w[1].done));
        let last = calls.last().unwrap();
        assert_eq!(last.done, last.total, "{extra}");
        assert!(last.total >= results["damage_cases"].as_u64().unwrap() as usize);
        // The total is planned, so it is not the cases so far: it is at least the real total until the end.
        assert!(
            calls.iter().map(|p| p.total).max().unwrap() >= last.total,
            "{extra}"
        );
        // The results say how many runs the search planned for, for the estimate before a damage run.
        assert!(
            results["damage_screening"]["planned_runs"]
                .as_u64()
                .unwrap()
                >= 1
        );
    }
}
