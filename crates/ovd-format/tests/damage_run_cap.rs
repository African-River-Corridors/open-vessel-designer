//! Seam: the damage-run limit of the arrangement search (issue #935). With the limit set to 1 the search screens one
//! candidate and stops; the answer says so. The limit is a process-wide knob, so this file holds this one test.
//!
//! Label: property

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::json;

#[test]
fn a_search_that_reaches_the_damage_run_limit_says_it_stopped() {
    ovd_format::damage::set_damage_run_cap(1);
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
    let RunOutcome::Results { results, .. } = run_option(&filled.design.unwrap().options[0]) else {
        panic!("no results");
    };
    // One run in the search, one for the answer.
    assert_eq!(ovd_format::damage::damage_runs_started() - before, 2);
    let s = &results["damage_screening"];
    assert_eq!(s["screened"], true);
    assert!(
        s["limit_note"]
            .as_str()
            .unwrap()
            .starts_with("The search stopped at the damage-run limit")
    );
    // The search found nothing that passed in its one run, so it shows the closest. Whether that is an answer is the
    // answer's own damage run's to say (issue #1038): feasible exactly when no row of the Rule Trace fails.
    let failing = results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["outcome"] == "fail")
        .count();
    assert_eq!(
        results["arrangement"]["feasible"],
        failing == 0,
        "{failing} failing rows"
    );
    ovd_format::damage::set_damage_run_cap(0);
}
