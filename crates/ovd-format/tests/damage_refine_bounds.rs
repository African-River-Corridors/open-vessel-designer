//! Seam: the damage refinement of the arrangement search finds the largest size that passes damage in each group of
//! (number across, number along), not a size cut short by its own bounds (issue #1031). After a size failed damage,
//! the next sizes of the bisect were built at a side clearance worked out from the moving upper bound instead of the
//! failing candidate's own, so each came out another diameter, was "not offered" and pulled the upper bound down to
//! the lower bound untested. On the 140 x 29 m coastal LNG reference design (tank top 2.0 m above the deck, two
//! tanks along) the 2-across group stopped at 11.28 m and the search kept a 1-across 16.00 m tank that carries less.
//!
//! Label: snapshot - the values the code printed on this branch (the 2-across size that passes is 11.531 m; a verdict
//! kept from an earlier depth pass said 11.53 m fails, and it is worked out again), plus a property over four hulls.
//! The property is slow (a damage search for each hull and number across), so it runs nightly
//! (docs/agents/slow-tests.md); the repro stays in the PR gate.
//! **Mutation proof:** building each size from the moving bound again (the old line) makes the repro fail: the pick
//! goes back to 1 across at 16.00 m (7,969 t).

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn run(loa: f64, beam: f64, extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "tank_top_above_deck_m": { "value": 2.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let d = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let RunOutcome::Results { results, .. } = run_option(&d.options[0]) else {
        panic!("no results");
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

/// The cargo of the chosen arrangement, t (0 when none was chosen).
fn cargo_t(r: &Value) -> f64 {
    r["fit_tried"]
        .as_array()
        .and_then(|a| a.iter().find(|t| t["outcome"] == "chosen"))
        .and_then(|t| t["cargo_t"].as_f64())
        .unwrap_or(0.0)
}

fn limit_hit(r: &Value) -> bool {
    !r["damage_screening"]["limit_note"].is_null()
}

#[test]
fn the_reference_design_keeps_the_largest_two_across_size_that_passes_damage() {
    // Two along, searched across: the pick is 2 across at 11.53 m, not the 1-across 16.00 m (7,969 t) tank.
    let searched = run(140.0, 29.0, json!({ "tanks_in_line": 2 }));
    let fixed = run(
        140.0,
        29.0,
        json!({ "tanks_in_line": 2, "tanks_across": 2 }),
    );
    for r in [&searched, &fixed] {
        assert_eq!(r["arrangement"]["n_wide"], 2);
        assert_eq!(r["arrangement"]["n_long"], 2);
        assert!((r["arrangement"]["tank_od_m"].as_f64().unwrap() - 11.5306).abs() < 0.001);
        assert_eq!(r["arrangement"]["feasible"], true);
        assert_eq!(igc_fails(r), 0);
        assert!(!limit_hit(r));
        assert!(
            cargo_t(r) > 7_968.71 + 1.0,
            "{} t, not more than the 1-across tank",
            cargo_t(r)
        );
    }
    assert!(cargo_t(&searched) >= cargo_t(&fixed) - 1e-6);
}

#[test]
fn the_search_carries_at_least_the_best_fixed_across_pick_that_passes_damage() {
    for (loa, beam) in [(140.0, 29.0), (89.0, 16.0), (110.0, 12.0), (80.0, 12.0)] {
        let searched = run(loa, beam, json!({ "tanks_in_line": 2 }));
        for across in [1, 2] {
            let fixed = run(
                loa,
                beam,
                json!({ "tanks_in_line": 2, "tanks_across": across }),
            );
            if fixed["arrangement"]["feasible"] != true || igc_fails(&fixed) > 0 {
                continue;
            }
            assert!(
                cargo_t(&searched) >= cargo_t(&fixed) - 1e-6 || limit_hit(&searched),
                "{loa} x {beam} m, {across} across fixed: {} t, searched {} t, no limit note",
                cargo_t(&fixed),
                cargo_t(&searched)
            );
        }
        if searched["arrangement"]["feasible"] == true {
            assert_eq!(igc_fails(&searched), 0, "{loa} x {beam} m");
        }
    }
}
