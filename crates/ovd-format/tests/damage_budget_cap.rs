//! Seam: a later depth pass that runs out of the damage budget keeps what an earlier pass found (issue #935). With
//! a tank top 2 m above the deck the depth is settled over several passes, each of which searches again; the
//! verdicts of every candidate a pass screened are kept for the passes after it, so a pass costs no damage run for
//! a candidate that an earlier one screened, and the pick an earlier pass found is still there when the budget is
//! gone. The damage-run limit is a process-wide knob, so this file holds this one test.
//!
//! Label: snapshot - the values the code printed. The limit of 24 is the maintainer's choice.

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::json;

fn run(loa: f64, beam: f64, depth: Option<f64>) -> (serde_json::Value, usize) {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "tank_top_above_deck_m": { "value": 2.0, "source": "typed" }
    });
    if let Some(d) = depth {
        inputs["moulded_depth_m"] = json!({ "value": d, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let d = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let before = ovd_format::damage::damage_runs_started();
    let RunOutcome::Results { results, .. } = run_option(&d.options[0]) else {
        panic!("no results");
    };
    (results, ovd_format::damage::damage_runs_started() - before)
}

/// A coastal 100 x 20 m hull with the share limit typed at 0.5 and no standing height: the search tries the shares.
fn run_share(loa: f64, beam: f64) -> (serde_json::Value, usize) {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": loa, "source": "typed" },
            "beam_m": { "value": beam, "source": "typed" },
            "service_area": { "value": "coastal-and-river", "source": "typed" },
            "tank_top_above_deck_m": { "value": null, "source": "typed" },
            "tank_top_above_deck_od_fraction": { "value": 0.5, "source": "typed" },
            "tanks_in_line": { "value": 2, "source": "typed" }
        } }]
    });
    let d = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let before = ovd_format::damage::damage_runs_started();
    let RunOutcome::Results { results, .. } = run_option(&d.options[0]) else {
        panic!("no results");
    };
    (results, ovd_format::damage::damage_runs_started() - before)
}

fn fails(r: &serde_json::Value) -> usize {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["outcome"] == "fail")
        .count()
}

/// At a limit of 8 or 10 damage runs the 140 x 29 x 12 m reference design, tank top 2 m up, still finds its pick
/// (two across by three along, 13.38 m, which passes every damage row): feasible, no failing row. Without the kept
/// verdicts a later pass runs out of budget before it reaches a passing candidate, and the design comes out
/// infeasible. **Mutation proof:** dropping the kept verdicts (`recall` always `None`) makes this test fail.
/// The 89 x 16 m coastal hull at the default limit: feasible, one tank across by three along, 16 runs in all.
#[test]
fn a_later_depth_pass_keeps_the_pick_an_earlier_pass_found() {
    for cap in [8, 10] {
        ovd_format::damage::set_damage_run_cap(cap);
        let (r, runs) = run(140.0, 29.0, None);
        assert_eq!(
            r["arrangement"]["feasible"], true,
            "cap {cap}: {}",
            r["arrangement"]
        );
        assert_eq!(fails(&r), 0, "cap {cap}");
        assert!(
            (r["arrangement"]["tank_od_m"].as_f64().unwrap() - 13.383).abs() < 0.001,
            "cap {cap}"
        );
        assert_eq!(r["arrangement"]["n_long"], 3, "cap {cap}");
        assert!(runs <= cap + 1, "cap {cap}: {runs} damage runs");
    }
    // A share search (issue #992) and its depth passes draw on the same budget: at a limit of 8 the whole run works
    // at most 8 damage runs in its searches, plus the answer's. **Mutation proof:** a runner that counts no runs
    // (each search with a budget of its own) takes this over the limit.
    ovd_format::damage::set_damage_run_cap(8);
    let (_, runs) = run_share(100.0, 20.0);
    assert!(
        runs <= 8 + 1,
        "share search at a limit of 8: {runs} damage runs"
    );
    ovd_format::damage::set_damage_run_cap(0);
    let (r, runs) = run(89.0, 16.0, None);
    assert_eq!(r["arrangement"]["feasible"], true, "{}", r["arrangement"]);
    assert_eq!(fails(&r), 0);
    assert_eq!(r["arrangement"]["n_wide"], 1);
    assert_eq!(r["arrangement"]["n_long"], 3);
    assert!(runs <= 24 + 1, "{runs} damage runs");
    eprintln!(
        "89 x 16 m: {runs} damage runs, tank {}",
        r["arrangement"]["tank_od_m"]
    );
}
