//! Seam: at the damage-run limit the arrangement search never picks a candidate it could not screen (issue #1101,
//! the follow-up of #994). The walk skips a candidate that needs a damage run once the limit is reached, and goes on
//! with the verdicts an earlier depth pass kept. A skipped candidate has no verdict, so it is never the search's pick:
//! when nothing passed before the limit, the result says the limit was reached, shows the closest candidate, and
//! that candidate is feasible only if the answer's own damage run passes every IGC 2.7 row. The damage-run limit is a
//! process-wide knob, so this file holds this one test.
//!
//! Label: snapshot - at a limit of one damage run the 80 x 9 m forward-house design has no passing candidate (no
//! "chosen" row, infeasible) and the 140 x 29 m reference is feasible on its own damage run; the rest is a property.
//! **Mutation proof:** judging the candidate without a verdict at the limit (dropping the `continue` on
//! `Screened::CapReached` in the walk, so `passes_all` runs on it) gives the 80 x 9 m design a "chosen" row, and this
//! test fails (exit 101).

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn run(loa: f64, beam: f64, style: &str) -> Value {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": loa, "source": "typed" },
            "beam_m": { "value": beam, "source": "typed" },
            "service_area": { "value": "coastal-and-river", "source": "typed" },
            "arrangement_style": { "value": style, "source": "typed" }
        } }]
    });
    let d = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let RunOutcome::Results { results, .. } = run_option(&d.options[0]) else {
        panic!("no results");
    };
    results
}

fn chosen_rows(r: &Value) -> usize {
    r["fit_tried"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t["outcome"] == "chosen")
        .count()
}

fn failing_damage_rows(r: &Value) -> usize {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| {
            x["outcome"] == "fail" && x["rule"].as_str().unwrap_or("").starts_with("igc/2.7")
        })
        .count()
}

#[test]
fn at_the_damage_run_limit_an_unscreened_candidate_is_never_the_pick() {
    ovd_format::damage::set_damage_run_cap(1);
    let forward = run(80.0, 9.0, "wheelhouse_forward");
    let reference = run(140.0, 29.0, "deckhouse");
    ovd_format::damage::set_damage_run_cap(0);
    for (what, r) in [("80 x 9 m", &forward), ("140 x 29 m", &reference)] {
        let a = &r["arrangement"];
        // The result says the search stopped at the limit, and the answer's own damage ran.
        assert!(
            !r["damage_screening"]["limit_note"].is_null(),
            "{what}: no limit note: {}",
            r["damage_screening"]
        );
        assert_eq!(r["damage_status"], "run", "{what}");
        // Feasible only on the answer's own damage run: never with a failing IGC 2.7 row.
        if a["feasible"] == true {
            assert_eq!(
                failing_damage_rows(r),
                0,
                "{what}: feasible with failing damage"
            );
        }
    }
    // Nothing passed before the limit: no candidate is the search's pick, and the closest one fails.
    assert_eq!(
        chosen_rows(&forward),
        0,
        "80 x 9 m: {}",
        forward["fit_tried"]
    );
    assert_eq!(forward["arrangement"]["feasible"], false);
    assert!(failing_damage_rows(&forward) > 0);
    // The reference: its closest candidate passes its own damage run, so it is feasible.
    assert_eq!(reference["arrangement"]["feasible"], true);
}
