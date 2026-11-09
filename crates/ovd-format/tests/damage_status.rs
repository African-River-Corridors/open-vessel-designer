//! Seam: a run can skip damage stability, and its results and verdict say whether it ran (issue #923).
//!
//! Label: property - the 140 x 29 m coastal LNG reference design and a dry bulk box barge. A run that
//! skips damage holds no damage case, says `damage_status` `not run`, counts the cases it skipped, reads the
//! damage Rules `not run` (not `not evaluated`) and never says the design "meets every evaluated blocking
//! Rule". Skipping changes nothing else, and a full run after a skipped one is byte-equal to a full run
//! before it. The hash of the results is the same either way, because the new keys are unhashed.

use ovd_format::damage::DamageControl;
use ovd_format::{
    Profile, RunOutcome, check, fill_as_new, results_hash, run_option, run_option_with_damage,
};
use serde_json::{Value, json};

const EXAMPLE: &str =
    include_str!("../../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign");
const DRY_BULK: &str = include_str!("fixtures/dry_bulk_box_hold.vdesign");

fn skip() -> DamageControl {
    DamageControl {
        run: false,
        progress: None,
    }
}

/// The reference gas design as a new file in today's Defaults Profile, run with `control`.
fn gas(service_area: &str, control: &DamageControl) -> (Value, String) {
    let mut file: Value = serde_json::from_str(EXAMPLE).unwrap();
    let mut option = file["options"][0].clone();
    let inputs = option["inputs"].as_object_mut().unwrap();
    inputs.retain(|_, v| v["source"] != "default");
    inputs.insert(
        "service_area".into(),
        json!({ "value": service_area, "source": "typed" }),
    );
    // Issue #1034: a river-only Option needs a number for both river limits (the example's own stay).
    if service_area == "river-only" {
        for (name, v) in [("draught_limit_m", 12.0), ("air_draft_limit_m", 40.0)] {
            inputs
                .entry(name)
                .or_insert(json!({ "value": v, "source": "typed" }));
        }
    }
    option.as_object_mut().unwrap().remove("results_hash");
    file["options"] = json!([option]);
    file.as_object_mut().unwrap().remove("defaults_profile");
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let option = &filled.design.unwrap().options[0];
    match run_option_with_damage(option, ovd_format::rule_set::built_in(), &[], control) {
        RunOutcome::Results { results, hash } => (results, hash),
        other => panic!("{other:?}"),
    }
}

/// The cases a full run worked out: the IGC block and the cases of every rule of `damage_by_rule`.
fn cases_worked_out(r: &Value) -> usize {
    let own = r["damage"]["cases"].as_array().map_or(0, Vec::len);
    let by_rule: usize = r["damage_by_rule"].as_object().map_or(0, |o| {
        o.values()
            .map(|b| b["cases"].as_array().map_or(0, Vec::len))
            .sum()
    });
    own + by_rule
}

/// The results without what damage adds: for the "skipping changes nothing else" check.
fn without_damage(r: &Value) -> Value {
    let mut r = r.clone();
    let o = r.as_object_mut().unwrap();
    for k in [
        "damage",
        "damage_by_rule",
        "marpol_damage",
        "damage_status",
        "damage_cases",
        "rule_trace",
    ] {
        o.remove(k);
    }
    r
}

#[test]
fn skipped_then_run_is_byte_equal_to_one_full_run() {
    for area in ["coastal-and-river", "river-only"] {
        let (before, h0) = gas(area, &DamageControl::default());
        let (skipped, h1) = gas(area, &skip());
        let (after, h2) = gas(area, &DamageControl::default());
        assert_eq!(
            serde_json::to_string(&before).unwrap(),
            serde_json::to_string(&after).unwrap(),
            "{area}: a full run after a skipped one differs from the full run before it"
        );
        assert!(cases_worked_out(&before) > 0, "{area}: no damage cases");
        assert_eq!(cases_worked_out(&skipped), 0, "{area}");
        if area == "coastal-and-river" {
            // Issue #935: the search screens by IGC 2.7 damage when damage runs. A run that skips damage does not
            // screen, keeps the unscreened pick and says so (`damage_screening`); it never picks a smaller tank.
            // The hash is stable for the same stage (a full run after a skipped one equals the full run before it)
            // and moves between the two stages exactly when the pick moves.
            let od = |r: &Value| r["arrangement"]["tank_od_m"].as_f64().unwrap();
            assert!(od(&skipped) >= od(&before), "{area}");
            assert_eq!(skipped["damage_screening"]["screened"], false, "{area}");
            assert!(skipped["damage_screening"]["note"].is_string(), "{area}");
            assert_eq!(before["damage_screening"]["screened"], true, "{area}");
            assert_eq!(h0, h2, "{area}");
            let same_pick = before["arrangement"] == skipped["arrangement"];
            assert_eq!(h0 == h1, same_pick, "{area}");
        } else {
            // Skipping changes no other part of the results, and not the hash.
            assert_eq!(without_damage(&before), without_damage(&skipped), "{area}");
            assert_eq!((&h0, &h1), (&h2, &h2), "{area}");
        }
        assert_eq!(results_hash(&before), h0);
    }
}

#[test]
fn no_damage_on_a_coastal_gas_file_reads_not_run_and_never_meets_every() {
    let (full, _) = gas("coastal-and-river", &DamageControl::default());
    let (r, _) = gas("coastal-and-river", &skip());
    assert_eq!(full["damage_status"], "run");
    assert_eq!(r["damage_status"], "not run");
    // The cases are counted without being worked out: the same number a full run works out.
    assert_eq!(
        r["damage_cases"].as_u64().unwrap() as usize,
        cases_worked_out(&full)
    );
    assert_eq!(full["damage_cases"], r["damage_cases"]);
    let rows: Vec<&Value> = r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["rule"] == "igc/2.7/damage-survival")
        .collect();
    assert!(!rows.is_empty());
    for row in &rows {
        assert_eq!(row["outcome"], "not run", "{row}");
        assert!(row["gap"].as_str().unwrap().contains("not run"));
    }
    // Not `not evaluated`: the full run has real rows for the same Rule.
    let full_rows = full["rule_trace"]["rows"].as_array().unwrap();
    assert!(
        full_rows
            .iter()
            .any(|row| row["rule"] == "igc/2.7/damage-survival"
                && matches!(row["outcome"].as_str(), Some("pass" | "fail" | "warn")))
    );
    let verdict = &r["rule_trace"]["verdict"];
    let (summary, scope) = (
        verdict["summary"].as_str().unwrap(),
        r["rule_trace"]["scope"].as_str().unwrap(),
    );
    for text in [summary, scope] {
        assert!(!text.contains("meets every"), "{text}");
        assert!(text.contains("not run"), "{text}");
    }
    assert!(!verdict["blocking_not_run"].as_array().unwrap().is_empty());
    // The text summary of `ovd run` says it too.
    let lines = ovd_format::summary_lines("Option 1", &r).join("\n");
    assert!(lines.contains("Damage stability: NOT RUN"), "{lines}");
}

#[test]
fn a_full_run_says_run_with_the_case_count() {
    let (full, _) = gas("coastal-and-river", &DamageControl::default());
    assert_eq!(full["damage_status"], "run");
    assert!(
        !full["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["outcome"] == "not run")
    );
    let lines = ovd_format::summary_lines("Option 1", &full).join("\n");
    assert!(lines.contains("Damage stability: run ("), "{lines}");
}

#[test]
fn a_hull_with_no_damage_rule_set_is_not_applicable_whether_or_not_damage_is_skipped() {
    let checked = check(DRY_BULK, &Profile::built_in());
    let option = &checked.design.unwrap().options[0];
    let get = |control: &DamageControl| match run_option_with_damage(
        option,
        ovd_format::rule_set::built_in(),
        &[],
        control,
    ) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    };
    for r in [get(&DamageControl::default()), get(&skip())] {
        assert_eq!(r["damage_status"], "not applicable");
        assert!(r.get("damage_cases").is_none());
    }
    // `run_option`, the old entry point, is the default control.
    match run_option(option) {
        RunOutcome::Results { results, .. } => {
            assert_eq!(results["damage_status"], "not applicable")
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_damage_rule_the_stack_does_not_evaluate_is_not_evaluated_not_not_applicable() {
    // The stored stack of the worked example holds an IGC version whose damage Rule is a gap.
    let checked = check(EXAMPLE, &Profile::built_in());
    let option = &checked.design.unwrap().options[0];
    for control in [DamageControl::default(), skip()] {
        let r =
            match run_option_with_damage(option, ovd_format::rule_set::built_in(), &[], &control) {
                RunOutcome::Results { results, .. } => results,
                other => panic!("{other:?}"),
            };
        assert_eq!(r["damage_status"], "not evaluated");
        assert!(r.get("damage_cases").is_none());
        let rows: Vec<&Value> = r["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["rule"] == "igc/2.7/damage-survival")
            .collect();
        assert!(rows.iter().all(|row| row["outcome"] == "not evaluated"));
        let lines = ovd_format::summary_lines("Option 1", &r).join("\n");
        assert!(lines.contains("Damage stability: not evaluated"), "{lines}");
    }
}
