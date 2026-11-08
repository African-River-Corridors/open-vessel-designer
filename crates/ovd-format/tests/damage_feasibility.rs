//! Seam: feasible means the answer's own damage run passes (issue #1038). The depth passes of a standing height
//! each search again, and a later pass can reuse the damage verdicts of an earlier one. A later pass moves the
//! installed power, so the lightship and the fuel, a little; a verdict reused on a moved lightship can be stale.
//! Verdicts that failed are reused, ones that passed are worked out again unless the budget is gone, and the
//! arrangement is feasible only if the answer's own damage run has no failing IGC 2.7 row.
//!
//! Label: rule - feasible implies no failing `igc/2.7` row; the sizes are snapshots printed by the code.

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn coastal(loa: f64, beam: f64) -> Value {
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
    let RunOutcome::Results { results, .. } = run_option(&d.options[0]) else {
        panic!("{loa} x {beam}: no results");
    };
    results
}

fn igc_fails(r: &Value) -> usize {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| {
            x["outcome"] == "fail" && x["rule"].as_str().unwrap_or("").starts_with("igc/2.7")
        })
        .count()
}

/// The 60 x 8, 80 x 9 and 80 x 10 m default coastal designs took two or three depth passes and said feasible with
/// 4, 2 and 1 failing `igc/2.7` rows. Now feasible implies no failing row, and these three, which have a size that
/// survives, are feasible with none. **Mutation proof:** reusing every kept verdict as it is and taking feasibility
/// from the search (the code of #1030) brings the failing rows back, and this test fails.
#[test]
fn feasible_implies_no_failing_igc_2_7_row_on_the_narrow_coastal_designs() {
    for (loa, beam) in [(60.0, 8.0), (80.0, 9.0), (80.0, 10.0)] {
        let r = coastal(loa, beam);
        let fails = igc_fails(&r);
        if r["arrangement"]["feasible"] == true {
            assert_eq!(
                fails, 0,
                "{loa} x {beam}: feasible with {fails} failing rows"
            );
        }
        assert_eq!(r["arrangement"]["feasible"], true, "{loa} x {beam}");
        assert_eq!(fails, 0, "{loa} x {beam}");
    }
}
