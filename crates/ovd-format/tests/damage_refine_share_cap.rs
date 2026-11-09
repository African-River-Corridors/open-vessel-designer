//! Seam: the damage refinement caps the draught of a candidate that stands its tank top by a share of the tank diameter
//! at its own depth less the freeboard, as it does for a typed height (issue #1032, the cap of commit 300a1d06 in #1027).
//! The refinement builds its own candidates between a size that failed damage and the next size down. A share candidate
//! took the uncapped draught limit, which is infinite when the design has no external draught limit.
//!
//! Label: snapshot - the values the code printed on this branch. The cap shows in the pick: with the cap, the refined
//! 2-across size carries 8,614 t at its capped draught and wins; with the draught limit infinite the size loads to its
//! volume, fails the freeboard check at that payload, and the pick falls to the 1-across 16.00 m tank (7,969 t).
//! **Mutation proof:** testing only the typed height in the refinement's cap (`o.tank_top_above_deck_m.is_some()`, the
//! test before 300a1d06) in place of `o.standing_mode()` makes this test fail: the pick is 1 across at 16.00 m, 7,969 t.

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn run(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "min_freeboard_m": { "value": 4.0, "source": "typed" },
        "tanks_in_line": { "value": 2, "source": "typed" }
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

/// The 140 x 29 m coastal LNG design, two tanks along, a share of 0.2 of the tank diameter, a typed minimum freeboard of
/// 7.24 m and no draught limit typed. The answer is the 2-across, 11.843 m size at depth 11.078 m, carrying 8,614.2 t,
/// with a draught limit of 3.838 m (the depth less the freeboard): the values the code printed on this branch.
///
/// The share of 0.2 is below the share's soft minimum of 0.25, so the fill warns; the warning does not stop the run, and
/// a low share is how the cap is made to bind here. A river-only design reaches this branch by default (it takes the
/// share 0.375), but it screens no IGC damage, so it has no refinement to cap; the coastal design is the one that runs it.
/// Label: snapshot of the pick and the cargo, and a rule for the limit.
#[test]
fn a_refined_share_candidate_is_capped_at_its_own_depth_less_the_freeboard() {
    // 140 x 29 m coastal LNG, two tanks along, freeboard 7.24 m, a share of 0.2 of the diameter, no draught limit.
    let r = run(json!({
        "tank_top_above_deck_od_fraction": 0.2,
        "tank_top_above_deck_m": null,
        "min_freeboard_m": 7.24
    }));
    assert_eq!(r["arrangement"]["n_wide"], 2, "{}", r["arrangement"]);
    assert!((r["arrangement"]["tank_od_m"].as_f64().unwrap() - 11.843).abs() < 0.001);
    assert!((r["results"]["cargo_t"].as_f64().unwrap() - 8614.2).abs() < 0.1);
    // The answer's own draught limit is its depth less the freeboard (11.078 - 7.24).
    let depth = r["dims"]["depth_m"].as_f64().unwrap();
    let limit = r["results"]["draught_max_m"].as_f64().unwrap();
    assert!((limit - (depth - 7.24)).abs() < 1e-9, "{limit} {depth}");
}
