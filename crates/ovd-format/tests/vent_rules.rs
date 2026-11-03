//! Issue #849: the IGC cargo vent exit Rules of `igc-2g` 3.2.0 on a run: 8.2.10.3, 8.2.10.4 and 8.2.11.1
//! read the mast of issue #822, apply on `coastal-and-river` and not on `river-only`.
//!
//! Label: sourced - IGC Code, resolution MSC.370(93), 8.2.10.3, 8.2.10.4 and 8.2.11.1 (annex p. 112):
//! the exit not less than B/3 or 6 m, whichever is the greater, above the weather deck, and 6 m above
//! working areas; the exit at least B or 25 m, whichever is less, from the nearest opening. The
//! boundary cases of each limit are in the evaluation harness (`evaluation/cases/vent_*.yaml`).

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

const ROWS: [&str; 3] = [
    "igc/8.2.10.3/vent-exit-height",
    "igc/8.2.10.4/vent-exit-above-working-areas",
    "igc/8.2.11.1/vent-exit-distance",
];

fn results_of(service_area: &str) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    inputs["service_area"] = json!({ "value": service_area, "source": "typed" });
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn outcomes(results: &Value, id: &str) -> Vec<String> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id)
        .map(|r| r["outcome"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_new_coastal_gas_option_meets_the_three_vent_rows_with_the_mast_the_engine_placed() {
    let results = results_of("coastal-and-river");
    assert!(
        results["rule_trace"]["rule_sets"]
            .to_string()
            .contains("igc-2g@3.2.0"),
        "{}",
        results["rule_trace"]["rule_sets"]
    );
    for id in ROWS {
        assert_eq!(outcomes(&results, id), ["pass"], "{id}");
    }
}

#[test]
fn the_vent_rows_do_not_apply_on_a_river_only_option() {
    let results = results_of("river-only");
    for id in ROWS {
        // Issue #850: the default river-only gas stack takes ADN 9.3.1 in place of `igc-2g`, so these IGC
        // rows are not in its trace at all (they were `not applicable` while `igc-2g` was in it).
        let o = outcomes(&results, id);
        assert!(o.iter().all(|x| x == "not applicable"), "{id}: {o:?}");
    }
}
