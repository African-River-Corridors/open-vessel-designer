//! Seam: one damage budget per run (issue #935). With a tank top above the deck the depth is settled over up to
//! three passes, each of which searches again. The whole run keeps one budget of damage runs (the cap of 24 in the
//! searches, plus the answer's run), one planned total that only grows, and one fraction done that never falls.
//!
//! Label: property - the 140 x 29 x 12 m coastal LNG reference design with the tank top 2 m above the deck, the
//! number of tanks along left to the search.

use ovd_format::damage::{ControlFlow, DamageControl, DamageProgress};
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option_with_damage};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

fn design(h: f64) -> ovd_format::vdesign::Design {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 140.0, "source": "typed" },
            "beam_m": { "value": 29.0, "source": "typed" },
            "service_area": { "value": "coastal-and-river", "source": "typed" },
            "min_freeboard_m": { "value": 4.0, "source": "typed" },
            "tank_top_above_deck_m": { "value": h, "source": "typed" },
            "loading_conditions": { "value": "is_code_2008", "source": "typed" }
        } }]
    });
    fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap()
}

#[test]
fn a_standing_height_run_has_one_damage_budget_and_one_monotone_progress() {
    for h in [0.0, 2.0] {
        let d = design(h);
        let log = Rc::new(RefCell::new(Vec::<DamageProgress>::new()));
        let seen = log.clone();
        let control = DamageControl {
            run: true,
            progress: Some(Rc::new(move |p| {
                seen.borrow_mut().push(p);
                ControlFlow::Continue(())
            })),
        };
        // The counter is read after the fill, which works no damage out.
        let before = ovd_format::damage::damage_runs_started();
        let RunOutcome::Results { results, .. } = run_option_with_damage(
            &d.options[0],
            ovd_format::rule_set::built_in(),
            &[],
            &control,
        ) else {
            panic!("no results");
        };
        let runs = ovd_format::damage::damage_runs_started() - before;
        // The cap of 24 in the searches, across every depth pass, plus the answer's run.
        assert!(runs <= 24 + 1, "h {h}: {runs} damage runs");
        assert_eq!(results["damage_status"], "run");
        // The budget was enough for the passes: no pass stopped at the limit.
        assert!(
            results["damage_screening"]["limit_note"].is_null(),
            "h {h}: {}",
            results["damage_screening"]["limit_note"]
        );
        let calls = log.borrow();
        assert!(calls.len() > 10, "h {h}");
        // One count across the passes: it starts at 0 once and never falls, and so does the fraction.
        assert_eq!(
            calls.iter().filter(|p| p.done == 0).count(),
            1,
            "h {h}: the count went back to 0"
        );
        assert!(
            calls.windows(2).all(|w| w[0].done <= w[1].done),
            "h {h}: done fell"
        );
        assert!(
            calls
                .windows(2)
                .all(|w| w[0].done as f64 / w[0].total as f64
                    <= w[1].done as f64 / w[1].total as f64 + 1e-12),
            "h {h}: the fraction fell"
        );
        // The planned total only grows while the searches run: it is never below what is done.
        assert!(calls.iter().all(|p| p.done <= p.total));
        let last = calls.last().unwrap();
        assert_eq!(last.done, last.total, "h {h}");
        eprintln!("h {h}: {runs} damage runs, {} progress calls", calls.len());
    }
}

/// A design of 100 x 20 m: the river-only LNG barge of the worked inland case (the Defaults Profile stands the tank
/// tops to a share of the diameter), or a coastal one with the share limit typed and no standing height, which
/// screens by IGC 2.7 damage and searches the share too.
fn barge(area: &str, extra: serde_json::Value) -> ovd_format::vdesign::Design {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 100.0, "source": "typed" },
        "beam_m": { "value": 20.0, "source": "typed" },
        "service_area": { "value": area, "source": "typed" }
    });
    // Issue #1034: a river-only Option needs a number for both river limits (high enough not to bind).
    if area == "river-only" {
        // A 3.5 m draught limit keeps the share search where it was with no limit (share 0.375).
        inputs["draught_limit_m"] = json!({ "value": 3.5, "source": "typed" });
        inputs["air_draft_limit_m"] = json!({ "value": 40.0, "source": "typed" });
    }
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap()
}

/// The whole run of a design that searches the share of the tank diameter (issue #992) with damage on: at most 24
/// damage runs in the searches of every share and every depth pass together, plus the answer's own, one count from
/// 0 that never falls, and one fraction that never falls. The river barge takes the share 0.375 and screens no
/// IGC damage (one run, the answer's); the coastal one with the share typed at 0.5 and two along searches shares
/// and passes under one budget.
/// Label: snapshot (the counts are what the code printed); **mutation proof:** a runner that counts no runs
/// (each share search and each pass with a budget of its own) takes the coastal run over the limit.
#[test]
fn the_share_search_and_the_depth_passes_of_a_run_share_one_budget_and_one_progress() {
    let cases = [
        ("river-only", json!({}), Some(0.375)),
        (
            "coastal-and-river",
            json!({ "tank_top_above_deck_od_fraction": 0.5, "tank_top_above_deck_m": null, "tanks_in_line": 2 }),
            Some(0.5),
        ),
    ];
    for (area, extra, share) in cases {
        let d = barge(area, extra);
        let log = Rc::new(RefCell::new(Vec::<DamageProgress>::new()));
        let seen = log.clone();
        let control = DamageControl {
            run: true,
            progress: Some(Rc::new(move |p| {
                seen.borrow_mut().push(p);
                ControlFlow::Continue(())
            })),
        };
        let before = ovd_format::damage::damage_runs_started();
        let RunOutcome::Results { results, .. } = run_option_with_damage(
            &d.options[0],
            ovd_format::rule_set::built_in(),
            &[],
            &control,
        ) else {
            panic!("no results");
        };
        let runs = ovd_format::damage::damage_runs_started() - before;
        assert_eq!(results["damage_status"], "run", "{area}");
        if area == "river-only" {
            let got = results["arrangement"]["tank_top_above_deck_od_fraction"].as_f64();
            assert_eq!(got, share, "{area}");
        }
        assert!(runs <= 24 + 1, "{area}: {runs} damage runs");
        let calls = log.borrow();
        assert_eq!(
            calls.iter().filter(|p| p.done == 0).count(),
            1,
            "{area}: the count went back to 0"
        );
        assert!(
            calls.windows(2).all(|w| w[0].done <= w[1].done),
            "{area}: done fell"
        );
        assert!(
            calls
                .windows(2)
                .all(|w| w[0].done as f64 / w[0].total as f64
                    <= w[1].done as f64 / w[1].total as f64 + 1e-12),
            "{area}: the fraction fell"
        );
        let last = calls.last().unwrap();
        assert_eq!(last.done, last.total, "{area}");
        eprintln!("{area}: {runs} damage runs, {} progress calls", calls.len());
    }
}
