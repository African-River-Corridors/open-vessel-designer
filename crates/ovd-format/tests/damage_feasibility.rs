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

fn cargo_volume(r: &Value) -> f64 {
    r["fit_tried"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["outcome"] == "chosen")
        .map_or(0.0, |t| t["cargo_volume_m3"].as_f64().unwrap())
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

/// The 80 x 12 and 110 x 12 m coastal designs keep the picks they had before #1038 (one tank across by four along,
/// 9.94 m and 9.53 m), which were real: their own fresh damage runs have no failing row. Re-screening every kept
/// pass drained the budget and shrank them to one by two (6.99 m and 6.87 m, -31 % and -40 % cargo). Kept verdicts
/// are reused, and only the winner is worked out again at its own basis. **Mutation proof:** screening every kept
/// pass again (and not reusing it) brings the smaller picks back, and this test fails.
/// Label: snapshot, printed by the code.
#[test]
fn the_picks_a_kept_verdict_helped_find_are_not_lost() {
    for (loa, beam, od, volume) in [(80.0, 12.0, 9.94, 2030.0), (110.0, 12.0, 9.53, 3703.0)] {
        let r = coastal(loa, beam);
        assert_eq!(r["arrangement"]["feasible"], true, "{loa} x {beam}");
        assert_eq!(igc_fails(&r), 0, "{loa} x {beam}");
        assert_eq!(r["arrangement"]["n_wide"], 1, "{loa} x {beam}");
        assert_eq!(r["arrangement"]["n_long"], 4, "{loa} x {beam}");
        let got = r["arrangement"]["tank_od_m"].as_f64().unwrap();
        assert!(
            (got - od).abs() < 0.01,
            "{loa} x {beam}: tank {got} m, expected {od} m"
        );
        assert!(
            (cargo_volume(&r) - volume).abs() < 1.0,
            "{loa} x {beam}: {} m3",
            cargo_volume(&r)
        );
    }
}
