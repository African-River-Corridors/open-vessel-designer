//! Seam: the stage a stored results hash belongs to (issue #935). The arrangement search screens by IGC 2.7 damage
//! only when damage runs, so the first stage (damage not run) and the run with damage can pick different
//! arrangements and give different hashes. A fill stamps the unscreened stage without running the damage search;
//! a run compares like with like.
//!
//! Label: property

use ovd_format::{
    HashStatus, Profile, ResultsStage, RunOutcome, check, fill_as_new, run_design_with_damage,
};
use serde_json::{Value, json};

fn design(cap: f64) -> String {
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 140.0, "source": "typed" },
            "beam_m": { "value": 29.0, "source": "typed" },
            "moulded_depth_m": { "value": 12.0, "source": "typed" },
            "max_tank_od_m": { "value": cap, "source": "typed" },
            "service_area": { "value": "coastal-and-river", "source": "typed" },
            "loading_conditions": { "value": "is_code_2008", "source": "typed" }
        } }]
    })
    .to_string()
}

fn control(run: bool) -> ovd_format::damage::DamageControl {
    ovd_format::damage::DamageControl {
        run,
        progress: None,
    }
}

/// A fill stamps the unscreened hash and the stage, and runs no damage search to do it (the cost of a second
/// full search on every `--as-new` fill). The run counter is read after the fill.
#[test]
fn a_fill_stamps_the_unscreened_stage_and_runs_no_damage() {
    let before = ovd_format::damage::damage_runs_started();
    let filled = fill_as_new(&design(12.0), &Profile::built_in());
    assert_eq!(
        ovd_format::damage::damage_runs_started(),
        before,
        "the fill ran damage"
    );
    let design = filled.design.unwrap();
    let option = &design.options[0];
    assert_eq!(option.results_stage, Some(ResultsStage::Unscreened));
    // The stamped hash is the hash of the run that skips damage.
    let RunOutcome::Results { hash, .. } = ovd_format::run_option_with_damage(
        option,
        ovd_format::rule_set::built_in(),
        &[],
        &control(false),
    ) else {
        panic!("no results");
    };
    assert_eq!(option.results_hash.as_deref(), Some(hash.as_str()));
    // The stage is in the file and survives a read.
    let text = design.to_text();
    let again = check(&text, &Profile::built_in());
    assert_eq!(
        again.design.unwrap().options[0].results_stage,
        Some(ResultsStage::Unscreened)
    );
    let raw: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(raw["options"][0]["results_stage"], "unscreened");
}

/// A run with damage compares an unscreened stamp with the unscreened stage: it matches, and says the damage screen
/// changes the pick when it does (a 12 m cap: 12.0 m unscreened, 11.5 m with damage), and not when it does not
/// (an 11 m cap).
#[test]
fn a_run_with_damage_matches_an_unscreened_stamp_and_flags_a_changed_pick() {
    for (cap, changes) in [(12.0, true), (11.0, false)] {
        let design = fill_as_new(&design(cap), &Profile::built_in())
            .design
            .unwrap();
        let runs = run_design_with_damage(
            &design,
            ovd_format::rule_set::built_in(),
            &[],
            &control(true),
        );
        assert_eq!(runs[0].hash_status, HashStatus::Matches, "cap {cap}");
        assert_eq!(runs[0].screen_changes_pick, changes, "cap {cap}");
        let skipped = run_design_with_damage(
            &design,
            ovd_format::rule_set::built_in(),
            &[],
            &control(false),
        );
        assert_eq!(skipped[0].hash_status, HashStatus::Matches, "cap {cap}");
        assert!(!skipped[0].screen_changes_pick);
    }
}

/// A file that says its hash is the screened stage's is compared with the run that screens, and not with a run that
/// skips damage (not the same stage): `not_checked`.
#[test]
fn a_screened_stamp_is_compared_with_the_screened_stage_only() {
    let mut design = fill_as_new(&design(12.0), &Profile::built_in())
        .design
        .unwrap();
    let RunOutcome::Results { hash, .. } = ovd_format::run_option_with_damage(
        &design.options[0],
        ovd_format::rule_set::built_in(),
        &[],
        &control(true),
    ) else {
        panic!("no results");
    };
    design.options[0].results_hash = Some(hash);
    design.options[0].results_stage = Some(ResultsStage::Screened);
    let with = run_design_with_damage(
        &design,
        ovd_format::rule_set::built_in(),
        &[],
        &control(true),
    );
    assert_eq!(with[0].hash_status, HashStatus::Matches);
    assert!(!with[0].screen_changes_pick);
    let without = run_design_with_damage(
        &design,
        ovd_format::rule_set::built_in(),
        &[],
        &control(false),
    );
    assert_eq!(without[0].hash_status, HashStatus::NotChecked);
}
