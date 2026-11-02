//! Seam: where the opening setback of a gas hull is measured from (issue #381). LR inland rules
//! 3.3.7 and 3.3.11 ask for entrances and openable windows at least 2.00 m from the cargo area, and
//! the cofferdam (the aft end space) stands inside the cargo area (ADN 1.2.1). A new Option measures
//! from the aft face of the end space (`cargo_area`): the aft separation is the end space plus the
//! setback. A file that stores no choice keeps `tank_end`, the earlier reading.
//!
//! Spec 0011 (D17): the hull keeps no length for the setback, so neither choice changes the hull; the
//! setback is checked on the opening distances (LR 3.3.7, 3.3.8, 3.3.11; IGC 3.2.4.1), and a hold
//! cofferdam is chosen only in place of the A-60 bulkhead (IGC 3.1.2, D18).
//!
//! Label: sourced - IGC Code, MSC.370(93), 3.1.2 (gastight A-60 bulkhead or cofferdam) and 3.2.4.1
//! (openings measured on the deckhouse); the rest is property.

use ovd_format::vdesign::{INPUTS, LEGACY_FIXED_OTHER};
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, check, run_design};
use serde_json::{Value, json};

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn results_of(text: &str) -> Value {
    let design = check(text, &Profile::built_in()).design.unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// A fixture with the choice set, and no stored hash.
fn with_basis(name: &str, basis: &str) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    let option = &mut file["options"][0];
    option["inputs"]["opening_setback_basis"] = json!({ "value": basis, "source": "typed" });
    option.as_object_mut().unwrap().remove("results_hash");
    file.to_string()
}

#[test]
fn the_choice_is_a_gas_input_that_a_new_option_takes_as_cargo_area_and_an_old_file_as_tank_end() {
    let spec = INPUTS
        .iter()
        .find(|s| s.name == "opening_setback_basis")
        .unwrap();
    assert_eq!(spec.group.label(), "Gas tanks");
    assert_eq!(spec.applies_to.containments(), ["type_c_tanks"]);
    assert_eq!(
        Profile::built_in()
            .default("opening_setback_basis")
            .and_then(Value::as_str),
        Some("cargo_area")
    );
    let legacy = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "opening_setback_basis")
        .unwrap();
    assert_eq!(legacy.value, "\"tank_end\"");
}

#[test]
fn a_file_that_stores_the_choice_as_tank_end_keeps_its_hash_and_a_file_without_it_does_too() {
    let mut file: Value = serde_json::from_str(&raw_fixture("gas_igc_2g_exact")).unwrap();
    file["options"][0]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("opening_setback_basis");
    let text = file.to_string();
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let notes = opened
        .report
        .warnings
        .iter()
        .filter(|w| w.contains("`opening_setback_basis`"))
        .count();
    assert_eq!(notes, 1, "{:?}", opened.report.warnings);
    let design = check(&text, &Profile::built_in()).design.unwrap();
    assert_eq!(
        design.options[0]
            .input("opening_setback_basis")
            .unwrap()
            .value,
        json!("tank_end")
    );
}

#[test]
fn neither_choice_keeps_a_length_for_the_setback() {
    // The fixture stores 0.60 m aft and a 2.00 m setback.
    let tank_end = results_of(&with_basis("gas_igc_2g_exact", "tank_end"));
    let cargo_area = results_of(&with_basis("gas_igc_2g_exact", "cargo_area"));
    let sep = |r: &Value, key: &str| r["drawing"]["separation"][key].as_f64().unwrap();
    for r in [&tank_end, &cargo_area] {
        assert!((sep(r, "aft_separation_m") - 0.6).abs() < 1e-9);
        assert_eq!(sep(r, "service_space_m"), 0.0);
        assert!(
            r["drawing"]["separation"]["checks"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["name"] != "opening_setback"),
            "the setback is checked on the openings, not on the hull"
        );
    }
    let zone = |r: &Value| r["geometry"]["cargo_zone_length_m"].as_f64().unwrap();
    assert!((zone(&tank_end) - zone(&cargo_area)).abs() < 1e-9);
    assert!((r_cargo(&tank_end) - r_cargo(&cargo_area)).abs() < 1e-9);
}

fn r_cargo(r: &Value) -> f64 {
    r["results"]["cargo_t"].as_f64().unwrap()
}
