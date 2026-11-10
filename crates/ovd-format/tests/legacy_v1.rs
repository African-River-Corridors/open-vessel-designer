//! Seam 4: a version 1 `.vdesign` file opens in the current format (issue #175). Version 2
//! changed the gas tank inputs: the `ADN_G` ruleset and `adn_tank_cap_m3` are gone and
//! `max_tank_od_m` is new. A version 1 file is read as `IGC_2G`, drops the cap, takes the
//! Defaults Profile diameter cap (version 1 had a fixed 12 m), and says so in a warning. Each test reads the real files in
//! `tests/legacy/`, which are version 1 files as that version wrote them.
//!
//! Label: property

use ovd_format::{DesignSession, HashStatus, Profile, Source, check, fill};
use serde_json::{Value, json};

fn legacy_raw(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The legacy file with a river-only Service Area set, as `ovd fill --service-area river-only` does (issue
/// #925: the legacy files have none, and a Design with none is an error). The migration tests below are about
/// what else an old file gets.
fn legacy(name: &str) -> String {
    let text = ovd_format::with_service_area(&legacy_raw(name), "river-only").unwrap();
    // Issue #1034: a file saved before the rule has no river limit; one with a `null` marked `default` is the
    // only kind that keeps its results. Written in as a file saved after Phase 1 holds them.
    let mut file: Value = serde_json::from_str(&text).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        for limit in ["draught_limit_m", "air_draft_limit_m"] {
            option["inputs"][limit] = json!({ "value": null, "source": "default" });
        }
    }
    file.to_string()
}

#[test]
fn a_legacy_file_has_no_service_area_so_it_is_the_error_until_it_is_set() {
    // Issue #925: no default is read for it, for a version 1 file either.
    for name in ["v1_adn_g_self_propelled", "v1_dry_bulk_box_hold"] {
        let raw = legacy_raw(name);
        let want = ovd_format::service_area_required().to_string();
        for errors in [
            check(&raw, &Profile::built_in()).report.errors,
            fill(&raw, &Profile::built_in()).report.errors,
        ] {
            assert!(
                errors.iter().any(|e| e.contains(&want)),
                "{name}: {errors:?}"
            );
        }
        let set = fill(&legacy(name), &Profile::built_in());
        assert!(set.report.is_ok(), "{name}: {:?}", set.report.errors);
    }
}

fn has(list: &[String], text: &str) -> bool {
    list.iter().any(|m| m.contains(text))
}

const ADN: &str = "`ruleset` ADN_G is no longer offered";
const CAP: &str = "`adn_tank_cap_m3` is no longer an input";
const OD: &str = "this version 1 file has no `max_tank_od_m`; it now uses the default 16 m";

#[test]
fn the_fixture_really_is_a_version_1_file_that_names_adn_g() {
    let raw: Value = serde_json::from_str(&legacy_raw("v1_adn_g_self_propelled")).unwrap();
    assert_eq!(raw["format_version"], 1);
    let inputs = &raw["options"][0]["inputs"];
    assert_eq!(inputs["ruleset"]["value"], "ADN_G");
    assert_eq!(inputs["adn_tank_cap_m3"]["value"], 1000.0);
    assert!(inputs.get("max_tank_od_m").is_none());
}

#[test]
fn validate_opens_it_as_igc_2g_without_the_cap_and_warns() {
    let checked = check(&legacy("v1_adn_g_self_propelled"), &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let w = &checked.report.warnings;
    assert!(has(w, ADN), "{w:?}");
    assert!(has(w, CAP), "{w:?}");
    assert!(has(w, OD), "{w:?}");
    let design = checked.design.unwrap();
    let o = &design.options[0];
    // Issue #274: the ruleset is no longer an input; the name is read, and dropped.
    assert!(o.input("ruleset").is_none());
    assert!(o.input("adn_tank_cap_m3").is_none());
    let od = o.input("max_tank_od_m").unwrap();
    assert_eq!(
        (od.value.clone(), od.source),
        (json!(16.0), Source::Default)
    );
}

#[test]
fn each_warning_names_its_option_and_the_input_it_is_about() {
    let r = check(&legacy("v1_adn_g_self_propelled"), &Profile::built_in()).report;
    assert_eq!(r.warnings.len(), r.warning_inputs.len());
    let about = |text: &str| {
        let i = r.warnings.iter().position(|m| m.contains(text)).unwrap();
        assert!(
            r.warnings[i].starts_with("Option 'Option 1': "),
            "{}",
            r.warnings[i]
        );
        r.warning_inputs[i].clone()
    };
    assert_eq!(about(ADN), ["ruleset"]);
    assert_eq!(about(CAP), ["adn_tank_cap_m3"]);
    assert_eq!(about(OD), ["max_tank_od_m"]);
}

#[test]
fn fill_writes_a_version_2_file_that_validates_without_those_warnings() {
    let p = Profile::built_in();
    let filled = fill(&legacy("v1_adn_g_self_propelled"), &p);
    assert!(filled.report.is_ok(), "{:?}", filled.report);
    assert!(has(&filled.report.warnings, ADN));
    let text = filled.design.unwrap().to_text();
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["format_version"], ovd_format::vdesign::FORMAT_VERSION);
    // Issue #274: the ruleset is no longer an input, so it is not written.
    assert!(saved["options"][0]["inputs"].get("ruleset").is_none());
    assert!(
        saved["options"][0]["inputs"]
            .get("adn_tank_cap_m3")
            .is_none()
    );
    let again = check(&text, &p);
    assert_eq!(again.report.errors, Vec::<String>::new());
    // Issue #1034: the file stores no river limits, so the one note about them stays.
    let rest = again
        .report
        .warnings
        .iter()
        .filter(|w| !w.contains("saved before the rule that needs a number"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(rest, Vec::<String>::new());
    assert_eq!(
        again.report.warnings.len(),
        1,
        "{:?}",
        again.report.warnings
    );
}

#[test]
fn the_design_session_opens_it_with_the_warnings_and_runs_it_as_igc_2g() {
    let (session, opened) =
        DesignSession::open(&legacy("v1_adn_g_self_propelled"), Profile::built_in()).unwrap();
    assert!(opened.report.is_ok(), "{:?}", opened.report);
    let w = &opened.report.warnings;
    assert!(has(w, ADN) && has(w, CAP) && has(w, OD), "{w:?}");
    // The saved hash is of the ADN_G results; these are IGC_2G, so it differs.
    assert!(
        matches!(opened.hash_status[0].1, HashStatus::Differs { .. }),
        "{:?}",
        opened.hash_status
    );
    let e = session.evaluate();
    // Issue #1034: the one note about the river limits the file stores none of.
    let others: Vec<_> = e.options[0]
        .messages
        .iter()
        .filter(|m| !m.text.contains("saved before the rule that needs a number"))
        .collect();
    assert!(others.is_empty(), "{others:?}");
    let outcome = e.options[0].outcome.as_ref().expect("it ran");
    let text = format!("{outcome:?}");
    assert!(text.contains("IGC_2G"), "{text}");
    assert!(!text.contains("ADN_G"), "{text}");
    // Saving writes the current format.
    let saved: Value = serde_json::from_str(&session.save().unwrap()).unwrap();
    assert_eq!(saved["format_version"], ovd_format::vdesign::FORMAT_VERSION);
}

#[test]
fn a_bare_value_form_is_changed_too() {
    let file = json!({
        "format": "vdesign", "format_version": 1, "engine_version": "0.1.0",
        "options": [{ "name": "A", "inputs": {
            "cargo": "LNG", "hull_form": "raked_box", "service_area": "river-only", "draught_limit_m": 12.0, "air_draft_limit_m": 40.0, "length_overall_m": 80.0, "beam_m": 12.0,
            "moulded_depth_m": 5.0, "formation": { "kind": "self_propelled" },
            "ruleset": "ADN_G", "adn_tank_cap_m3": 380.0
        } }]
    });
    let r = fill(&file.to_string(), &Profile::built_in());
    assert!(r.report.is_ok(), "{:?}", r.report);
    assert!(has(&r.report.warnings, ADN) && has(&r.report.warnings, CAP));
    let o = &r.design.unwrap().options[0];
    assert!(o.input("ruleset").is_none());
    // Left out, so the Defaults Profile gives it under the new ruleset: 98 %.
    assert_eq!(o.input("fill_ratio").unwrap().value, json!(0.98));
}

#[test]
fn a_version_1_dry_bulk_file_gets_only_the_note_about_the_new_hold_inputs() {
    // Slice 1 of issue #4: the hold inputs are new, so the one warning is about them. The gas
    // changes of version 2 do not touch a dry-bulk file.
    let checked = check(&legacy("v1_dry_bulk_box_hold"), &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let w = &checked.report.warnings;
    // Issue #448: and the stored block coefficient, now a result of the hull, is dropped. Issue
    // #739: and the structure block of the Hull, which the file lacks. Issue #1034: and the river limits the
    // file stores none of.
    assert_eq!(w.len(), 4, "{w:?}");
    assert!(has(w, "saved before the rule that needs a number"), "{w:?}");
    assert!(has(w, "before the structure block of the Hull"), "{w:?}");
    assert!(
        has(w, "fixed value the engine used before it was an input"),
        "{w:?}"
    );
    assert!(
        has(w, "the block coefficient now comes from the hull shape"),
        "{w:?}"
    );
    assert!(!has(w, ADN) && !has(w, CAP) && !has(w, OD), "{w:?}");
    let (_, opened) =
        DesignSession::open(&legacy("v1_dry_bulk_box_hold"), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn a_version_2_file_gets_no_such_help() {
    // Only version 1 files are changed. A current file naming ADN_G, or the cap, is refused.
    let mut raw: Value = serde_json::from_str(&legacy("v1_adn_g_self_propelled")).unwrap();
    raw["format_version"] = json!(2);
    let r = check(&raw.to_string(), &Profile::built_in()).report;
    assert!(
        has(&r.errors, "`ruleset` = \"ADN_G\" must be one of: IGC_2G"),
        "{r:?}"
    );
    assert!(has(&r.errors, "unknown input `adn_tank_cap_m3`"), "{r:?}");
}
