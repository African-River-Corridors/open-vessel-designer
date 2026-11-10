//! Seam: the `api` entry's `simulate` (issue #937). A dict of typed inputs gives what `ovd fill --as-new`
//! then `ovd run` give for the same inputs: the same results, hash and Rule Trace.
//!
//! Label: property. Each test compares two paths to one answer; no number is pinned.

use ovd_format::{Profile, RunOutcome, eval, fill_as_new, results_hash, run_option};
use serde_json::{Map, Value, json};

const EXAMPLE: &str =
    include_str!("../../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign");
const COASTAL_CRUDE: &str = include_str!("fixtures/crude_integral_tanks_coastal.vdesign");
const DRY_BULK: &str = include_str!("fixtures/dry_bulk_box_hold.vdesign");

/// The typed inputs of Option `index` of a file, as a dict.
fn typed(file: &str, index: usize) -> Map<String, Value> {
    let mut p = typed_only(file, index);
    // The Service Area has no default (issue #925); a fixture of an older file may not give one.
    p.entry("service_area").or_insert(json!("river-only"));
    // Issue #1034: a river-only Option needs a number for both river limits (high enough not to bind).
    if p["service_area"] == json!("river-only") {
        p.entry("draught_limit_m").or_insert(json!(12.0));
        p.entry("air_draft_limit_m").or_insert(json!(40.0));
    }
    p
}

fn typed_only(file: &str, index: usize) -> Map<String, Value> {
    let root: Value = serde_json::from_str(file).expect("JSON");
    root["options"][index]["inputs"]
        .as_object()
        .expect("inputs")
        .iter()
        .filter(|(_, i)| i["source"] == "typed")
        .map(|(k, i)| (k.clone(), i["value"].clone()))
        .collect()
}

fn with(mut p: Map<String, Value>, k: &str, v: Value) -> Map<String, Value> {
    p.insert(k.to_string(), v);
    p
}

fn file_of(typed: &Map<String, Value>) -> String {
    let inputs: Map<String, Value> = typed
        .iter()
        .map(|(k, v)| (k.clone(), json!({ "value": v, "source": "typed" })))
        .collect();
    json!({ "format": "vdesign", "format_version": 4, "engine_version": env!("CARGO_PKG_VERSION"),
            "units": "SI", "options": [{ "name": "Option 1", "inputs": inputs }] })
    .to_string()
}

/// `ovd fill --as-new` then `ovd run` on the same typed inputs.
fn by_file(typed: &Map<String, Value>) -> (Value, String) {
    let checked = fill_as_new(&file_of(typed), &Profile::built_in());
    assert!(checked.report.is_ok(), "{:?}", checked.report.errors);
    match run_option(&checked.design.expect("design").options[0]) {
        RunOutcome::Results { results, hash } => (results, hash),
        other => panic!("{other:?}"),
    }
}

fn simulate(typed: &Map<String, Value>, damage: bool) -> Result<Value, String> {
    let case = json!({ "fn": "simulate", "args": { "params": typed, "damage": damage } });
    match eval("api", &case) {
        Ok(v) if v.get("ok").is_some() => Ok(v["ok"].clone()),
        Ok(v) => Err(format!("{} {}", v["error"]["type"], v["error"]["message"])),
        Err(e) => Err(e.to_string()),
    }
}

fn same(label: &str, params: &Map<String, Value>) {
    let (want, hash) = by_file(params);
    let got = simulate(params, true).unwrap_or_else(|e| panic!("{label}: {e}"));
    for key in ["cargo_t", "draught_m"] {
        assert_eq!(got["results"][key], want["results"][key], "{label} {key}");
    }
    assert_eq!(got["rule_trace"], want["rule_trace"], "{label} rule trace");
    assert_eq!(got, want, "{label} results");
    assert_eq!(results_hash(&got), hash, "{label} hash");
}

#[test]
fn the_coastal_lng_example_as_river_only_and_coastal_gives_runs_results() {
    let base = typed(EXAMPLE, 0);
    same("coastal", &base);
    same(
        "river-only",
        &with(
            with(
                with(base, "service_area", json!("river-only")),
                "draught_limit_m",
                json!(12.0),
            ),
            "air_draft_limit_m",
            json!(40.0),
        ),
    );
}

#[test]
fn a_coastal_liquid_bulk_file_gives_runs_results() {
    same("crude coastal", &typed(COASTAL_CRUDE, 0));
}

#[test]
fn the_stored_example_through_the_file_path_gives_the_same_results() {
    // The file as stored (its defaults written in) and its every input, given as a dict, are one Design.
    let root: Value = serde_json::from_str(EXAMPLE).expect("JSON");
    let every: Map<String, Value> = root["options"][0]["inputs"]
        .as_object()
        .expect("inputs")
        .iter()
        .map(|(k, i)| (k.clone(), i["value"].clone()))
        .collect();
    let checked = fill_as_new(EXAMPLE, &Profile::built_in());
    let option = &checked.design.expect("design").options[0];
    let RunOutcome::Results { results, hash } = run_option(option) else {
        panic!("a result")
    };
    let got = simulate(&every, true).expect("results");
    // Not the whole results value: a block such as `holding_time` says whether a value was typed or
    // defaulted, and a Rule Trace warning names a typed value, and here every input is typed. The hashed
    // values are equal, and so is the hash.
    for (k, v) in results.as_object().expect("object") {
        if !ovd_format::UNHASHED_KEYS.contains(&k.as_str()) {
            assert_eq!(&got[k], v, "hashed key {k}");
        }
    }
    assert_eq!(results_hash(&got), hash);
}

#[test]
fn damage_is_off_unless_asked_for() {
    let p = typed(EXAMPLE, 0);
    assert_eq!(simulate(&p, true).unwrap()["damage_status"], "run");
    assert_eq!(simulate(&p, false).unwrap()["damage_status"], "not run");
    let case = json!({ "fn": "simulate", "args": { "params": p } });
    assert_eq!(
        eval("api", &case).unwrap()["ok"]["damage_status"],
        "not run"
    );
}

#[test]
fn leaving_out_an_input_an_es_trin_row_reads_gives_the_profile_default() {
    // The ES-TRIN rows read `min_freeboard_m`, `hatch_coaming_height_m` and `length_overall_m`.
    // The first two have a Defaults Profile value: leaving one out gives what typing that value gives.
    for (label, file, names) in [
        (
            "dry bulk",
            DRY_BULK,
            ["min_freeboard_m", "hatch_coaming_height_m"],
        ),
        (
            "liquid",
            COASTAL_CRUDE,
            ["min_freeboard_m", "hatch_coaming_height_m"],
        ),
    ] {
        for name in names {
            let mut left_out = typed(file, 0);
            left_out.remove(name);
            let checked = fill_as_new(&file_of(&left_out), &Profile::built_in());
            let option = &checked.design.expect("design").options[0];
            let Some(stored) = option.input(name) else {
                continue; // the input does not apply to this Cargo Containment
            };
            let given = with(left_out.clone(), name, stored.value.clone());
            let a = simulate(&left_out, true).unwrap_or_else(|e| panic!("{label} {name}: {e}"));
            let b = simulate(&given, true).unwrap_or_else(|e| panic!("{label} {name}: {e}"));
            assert_eq!(a, b, "{label}: {name} left out");
        }
    }
    // The length is required: it is never filled.
    let mut p = typed(EXAMPLE, 0);
    p.remove("length_overall_m");
    assert!(
        simulate(&p, true)
            .unwrap_err()
            .contains("missing required input `length_overall_m`")
    );
}

#[test]
fn a_dict_that_leaves_out_length_beam_and_cargo_raises_naming_each() {
    let mut p = typed(EXAMPLE, 0);
    for k in ["length_overall_m", "beam_m", "cargo"] {
        p.remove(k);
    }
    let e = simulate(&p, false).unwrap_err();
    for name in ["length_overall_m", "beam_m", "cargo"] {
        assert!(
            e.contains(&format!("missing required input `{name}`")),
            "{e}"
        );
    }
}

#[test]
fn a_refused_input_raises_a_bad_input_error() {
    let p = with(typed(EXAMPLE, 0), "cb", json!(0.85));
    let case = json!({ "fn": "simulate", "args": { "params": p } });
    let e = eval("api", &case).unwrap_err();
    assert!(
        e.to_string().contains("`cb` is a result, not an input"),
        "{e}"
    );
    assert!(matches!(e, ovd_format::EvalError::BadInput(_)));
}

#[test]
fn a_convoy_runs_and_gives_its_blocks() {
    let convoy = include_str!("fixtures/pushed_convoy.vdesign");
    same("convoy", &typed(convoy, 0));
}

#[test]
fn propulsive_efficiency_is_taken_where_it_is_used_and_it_moves_the_fuel_and_cargo() {
    // The input is used when no propulsor decides it (`stern_inputs.rs`): a pushed barge, or a
    // self-propelled hull whose `propulsor_type` is `unspecified`. A pushed barge takes it: its cargo
    // stays, and its speed-power curves (kW and fuel) and Rule Trace follow it. The unspecified
    // self-propelled hull sizes its fuel, and so its cargo, from it. A specified propulsor does not use
    // it, and the reader says so.
    let barge = typed(DRY_BULK, 0);
    assert_eq!(barge["formation"]["kind"], "pushed_barge");
    let barge_at = |eta: f64| {
        simulate(
            &with(barge.clone(), "propulsive_efficiency", json!(eta)),
            false,
        )
        .unwrap_or_else(|e| panic!("{e}"))
    };
    let (low_eta, high_eta) = (barge_at(0.3), barge_at(0.6));
    assert_eq!(
        low_eta["results"]["cargo_t"],
        high_eta["results"]["cargo_t"]
    );
    let kw = |r: &Value| {
        let points = r["curves"][0]["points"].as_array().expect("points");
        points.last().expect("a point")["kw"].as_f64().expect("kw")
    };
    assert!(
        kw(&low_eta) > kw(&high_eta),
        "a lower efficiency needs more power: {} vs {}",
        kw(&low_eta),
        kw(&high_eta)
    );
    let mut own = typed(DRY_BULK, 0);
    own.insert("formation".into(), json!({ "kind": "self_propelled" }));
    own.insert("propulsor_type".into(), json!("unspecified"));
    let at = |eta: f64| {
        let r = simulate(
            &with(own.clone(), "propulsive_efficiency", json!(eta)),
            false,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        (
            r["results"]["cargo_t"].as_f64().expect("cargo"),
            r["loading_conditions"]["consumables_t"]
                .as_f64()
                .expect("fuel"),
        )
    };
    let (low, high) = (at(0.3), at(0.6));
    assert!(
        low.1 > high.1,
        "a lower efficiency burns more fuel: {low:?} vs {high:?}"
    );
    assert!(low.0 < high.0, "and carries less: {low:?} vs {high:?}");
    let specified = with(typed(EXAMPLE, 0), "propulsive_efficiency", json!(0.4));
    assert!(
        simulate(&specified, false)
            .unwrap_err()
            .contains("`propulsive_efficiency` is not used when")
    );
}

#[test]
fn a_lock_the_hull_does_not_fit_gives_the_same_answer_by_simulate_and_by_file() {
    // Issue #1043: simulate and `ovd run` share the lock check, and its own Rule row carries the block.
    // The flag stays the search's answer about the tanks.
    let none = simulate(&typed(EXAMPLE, 0), false).unwrap_or_else(|e| panic!("{e}"));
    let params = with(typed(EXAMPLE, 0), "lock_length_m", json!(100.0));
    same("lock", &params);
    let got = simulate(&params, false).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        got["arrangement"]["feasible"], none["arrangement"]["feasible"],
        "the lock leaves the arrangement flag alone"
    );
    let row = got["rule_trace"]["rows"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|r| r["rule"] == "ovd/waterway/within-lock-length")
        })
        .unwrap_or_else(|| panic!("no lock length row in {}", got["rule_trace"]));
    assert_eq!(row["outcome"], "fail", "{row}");
}
