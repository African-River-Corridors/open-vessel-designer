//! Seam: feasible means the answer's own damage run passes, even when the damage budget runs out (issue #1038). When
//! the budget is gone a later depth pass reuses a kept verdict that passed, on a lightship that may have moved: the
//! best there is. The answer's own damage run decides, so the arrangement is never feasible with a failing `igc/2.7`
//! row. The damage-run limit is a process-wide knob, so this file holds this one test.
//!
//! Label: rule - feasible implies no failing `igc/2.7` row, at a limit of 3 and 4 damage runs.
//! **Mutation proof:** taking feasibility from the search again makes it fail.

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::json;

#[test]
fn at_a_small_damage_run_limit_feasible_still_implies_no_failing_igc_2_7_row() {
    let mut contradicted = 0;
    for cap in [3, 4] {
        ovd_format::damage::set_damage_run_cap(cap);
        for (loa, beam) in [(60.0, 8.0), (80.0, 9.0), (80.0, 10.0)] {
            let file = json!({
                "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
                "options": [{ "name": "Option 1", "inputs": {
                    "cargo": { "value": "LNG", "source": "typed" },
                    "hull_form": { "value": "raked_box", "source": "typed" },
                    "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
                    "length_overall_m": { "value": loa, "source": "typed" },
                    "beam_m": { "value": beam, "source": "typed" },
                    "service_area": { "value": "coastal-and-river", "source": "typed" }
                } }]
            });
            let d = fill_as_new(&file.to_string(), &Profile::built_in())
                .design
                .unwrap();
            let RunOutcome::Results { results: r, .. } = run_option(&d.options[0]) else {
                panic!("no results");
            };
            let fails = r["rule_trace"]["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|x| {
                    x["outcome"] == "fail"
                        && x["rule"].as_str().unwrap_or("").starts_with("igc/2.7")
                })
                .count();
            if r["arrangement"]["feasible"] == true && fails > 0 {
                contradicted += 1;
            }
            eprintln!(
                "cap {cap} {loa} x {beam}: feasible {} fails {fails}",
                r["arrangement"]["feasible"]
            );
        }
    }
    // The other way (nit 3 of #1028): at a limit of 2 or 4 the reference design's search runs out of damage runs, which
    // is all it reports, and its answer's own damage run passes: it is feasible, with no failing row at all.
    // **Mutation proof:** not raising feasibility when only the limit stopped the search leaves it infeasible with a
    // failing "Feasible arrangement" row, and this fails.
    for cap in [2, 4] {
        ovd_format::damage::set_damage_run_cap(cap);
        let file = json!({
            "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
            "options": [{ "name": "Option 1", "inputs": {
                "cargo": { "value": "LNG", "source": "typed" },
                "hull_form": { "value": "raked_box", "source": "typed" },
                "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
                "length_overall_m": { "value": 140.0, "source": "typed" },
                "beam_m": { "value": 29.0, "source": "typed" },
                "service_area": { "value": "coastal-and-river", "source": "typed" }
            } }]
        });
        let d = fill_as_new(&file.to_string(), &Profile::built_in())
            .design
            .unwrap();
        let RunOutcome::Results { results: r, .. } = run_option(&d.options[0]) else {
            panic!("no results");
        };
        let failing = r["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["outcome"] == "fail")
            .count();
        assert_eq!(
            failing, 0,
            "reference at a limit of {cap}: {failing} failing rows"
        );
        assert_eq!(
            r["arrangement"]["feasible"], true,
            "reference at a limit of {cap}"
        );
    }
    ovd_format::damage::set_damage_run_cap(0);
    assert_eq!(
        contradicted, 0,
        "designs feasible with a failing IGC 2.7 row"
    );
}
