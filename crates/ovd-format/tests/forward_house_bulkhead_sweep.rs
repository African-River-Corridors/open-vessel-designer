//! Seam: the arrangement search keeps walking on the damage verdicts it already has when it reaches the damage-run
//! limit (issue #994 rework). A standing height settles the depth over up to three passes, and the passes share one
//! budget of damage runs. On the 80 x 9 m forward-house design at the default collision bulkhead (3.2 m) the second
//! pass proved a 4-along 6.56 m tank passes, and the third pass had one run left. It spent it on a 2-along 5.06 m
//! tank, reached the limit on the next size, and stopped the walk there: the 4-along sizes, whose verdicts were kept
//! and cost no run, were never looked at, and the refinement that rebuilds 6.56 m was skipped. The design carried
//! 294 t where the bulkhead 0.05 m either side gave 436 t and 477 t.
//!
//! Label: snapshot - the cargo the code printed on this branch after the fix.
//! **Mutation proof:** stopping the walk at the limit again (`break 'counts` on `CapReached`) makes the test fail:
//! the pick goes back to 1 across by 2 along, 5.06 m (294 t).

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn run_forward_house(loa: f64, beam: f64) -> Value {
    let inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "arrangement_style": { "value": "wheelhouse_forward", "source": "typed" }
    });
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

/// The cargo of the chosen arrangement, t (0 when none was chosen).
fn cargo_t(r: &Value) -> f64 {
    r["fit_tried"]
        .as_array()
        .and_then(|a| a.iter().find(|t| t["outcome"] == "chosen"))
        .and_then(|t| t["cargo_t"].as_f64())
        .unwrap_or(0.0)
}

#[test]
fn at_the_damage_run_limit_the_search_still_takes_the_size_an_earlier_pass_proved() {
    let r = run_forward_house(80.0, 9.0);
    let a = &r["arrangement"];
    assert_eq!(a["feasible"], true);
    assert!(
        cargo_t(&r) >= 400.0,
        "{} t: {} across by {} along, {} m tanks",
        cargo_t(&r),
        a["n_wide"],
        a["n_long"],
        a["tank_od_m"]
    );
}
