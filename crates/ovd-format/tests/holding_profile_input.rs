//! Seam: the operation profile of a Type C tank's holding time is an input (spec 0016 D2, D3, D7 to
//! D10; issue #869). Input only: no result reads it, so no stored result moves.
//!
//! Label: property. Each test states a rule: the default is the 20/10 design basis and is labelled
//! default, an old file loads with it and keeps its hash, a bad profile is refused with a message,
//! the inputs apply to Type C tanks only, and a changed profile moves no result.

use ovd_format::vdesign::{
    GAS_POWER_SOURCES, HOLDING_PROFILE_DEFAULT_JSON, INPUTS, LEGACY_FIXED_OTHER, POWER_SOURCES,
};
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check};
use serde_json::{Value, json};

const NEW: [&str; 8] = [
    "holding_profile",
    "holding_preset",
    "holding_adn_min_use_kg_h",
    "holding_igf_domestic_use_kg_h",
    "holding_adn_journey_days",
    "holding_start_pressure_mpa",
    "holding_fuel_service_pressure_mpa",
    "holding_fill_ratio",
];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before these inputs existed.
fn old_file(name: &str) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        let inputs = option["inputs"].as_object_mut().unwrap();
        for key in NEW {
            inputs.remove(key);
        }
    }
    file.to_string()
}

fn gas_session() -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_input("Option 1", "length_overall_m", json!(110.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(11.4)).unwrap();
    s.set_input("Option 1", "moulded_depth_m", json!(6.0))
        .unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn stored(s: &DesignSession, name: &str) -> Option<(Value, Source)> {
    s.design().options[0]
        .input(name)
        .map(|i| (i.value.clone(), i.source))
}

fn results_of(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        // The holding-time block (#870) reads the profile; it is the one result that may move.
        RunOutcome::Results { results, .. } => {
            let mut r = results.clone();
            r.as_object_mut().unwrap().remove("holding_time");
            r
        }
        other => panic!("{other:?}"),
    }
}

/// The errors and the warnings the file check gives for the session's Design.
fn report_of(s: &DesignSession) -> (Vec<String>, Vec<String>) {
    let r = check(&s.design().to_text(), &Profile::built_in()).report;
    (r.errors, r.warnings)
}

fn default_profile() -> Value {
    json!([{ "state": "laden_passage", "days": 20.0 }, { "state": "idle", "days": 10.0 }])
}

#[test]
fn the_inputs_apply_to_type_c_tanks_only() {
    for name in NEW {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(format!("{:?}", spec.applies_to), "Only(\"type_c_tanks\")");
        assert_eq!(spec.group.label(), "Gas tanks");
    }
}

#[test]
fn the_default_is_twenty_days_of_passage_then_ten_idle_in_every_place_it_is_written() {
    let p = Profile::built_in();
    assert_eq!(p.default("holding_profile"), Some(&default_profile()));
    assert_eq!(
        ovd_format::holding_profile::default_value(),
        default_profile()
    );
    let legacy = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "holding_profile")
        .unwrap();
    assert_eq!(legacy.value, HOLDING_PROFILE_DEFAULT_JSON);
    let parsed: Value = serde_json::from_str(legacy.value).unwrap();
    assert_eq!(parsed, default_profile());
    // The other inputs default to none or zero, and a preset is never applied by default.
    assert_eq!(p.default("holding_preset"), Some(&json!("none")));
    for n in ["holding_adn_min_use_kg_h", "holding_igf_domestic_use_kg_h"] {
        assert_eq!(p.default(n), Some(&json!(0.0)), "{n}");
    }
    for n in [
        "holding_adn_journey_days",
        "holding_start_pressure_mpa",
        "holding_fuel_service_pressure_mpa",
        "holding_fill_ratio",
    ] {
        assert_eq!(p.default(n), Some(&Value::Null), "{n}");
    }
    // Every legacy value is the profile's default.
    for n in NEW {
        let l = LEGACY_FIXED_OTHER.iter().find(|l| l.name == n).unwrap();
        let v: Value = serde_json::from_str(l.value).unwrap();
        assert_eq!(Some(&v), p.default(n), "{n}");
    }
}

#[test]
fn a_new_gas_option_has_the_default_profile_labelled_default() {
    let s = gas_session();
    assert_eq!(
        stored(&s, "holding_profile"),
        Some((default_profile(), Source::Default))
    );
    let schema = s.input_schema();
    let spec = schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "holding_profile")
        .unwrap();
    assert_eq!(spec["kind"], "operation_profile");
    assert_eq!(spec["default"], default_profile());
    assert_eq!(spec["default_status"], "estimate");
}

#[test]
fn an_option_without_type_c_tanks_has_none_of_them_and_refuses_them() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("diesel_en590"))
        .unwrap();
    s.set_input("Option 1", "length_overall_m", json!(80.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(11.0)).unwrap();
    s.set_input("Option 1", "moulded_depth_m", json!(4.0))
        .unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    for n in NEW {
        assert!(stored(&s, n).is_none(), "{n}");
    }
    // A value typed anyway is refused: it is not used for this Cargo Containment.
    let err = s
        .set_input("Option 1", "holding_profile", default_profile())
        .unwrap_err();
    assert!(err.contains("`holding_profile` is not used for a"), "{err}");
}

#[test]
fn an_old_file_loads_with_the_default_and_keeps_its_hash() {
    let (_, opened) =
        DesignSession::open(&old_file("gas_igc_2g_exact"), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let (s, _) = DesignSession::open(&old_file("gas_igc_2g_exact"), Profile::built_in()).unwrap();
    assert_eq!(
        stored(&s, "holding_profile"),
        Some((default_profile(), Source::Default))
    );
    assert_eq!(
        stored(&s, "holding_preset"),
        Some((json!("none"), Source::Default))
    );
    // The file as it is now also matches.
    let (_, opened) =
        DesignSession::open(&raw_fixture("gas_igc_2g_exact"), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn a_typed_profile_is_kept_as_typed_and_moves_no_result() {
    let mut s = gas_session();
    let before = results_of(&s);
    let typed = json!([
        { "state": "laden_passage", "days": 5.0, "passage_speed_kn": 8.0 },
        { "state": "idle", "days": 2.0, "gas_use_kg_h": 3.5 },
        { "state": "laden_passage", "days": 4.0 }
    ]);
    s.set_input("Option 1", "holding_profile", typed.clone())
        .unwrap();
    s.set_input("Option 1", "holding_preset", json!("bv_nr467"))
        .unwrap();
    s.set_input("Option 1", "holding_adn_journey_days", json!(3.0))
        .unwrap();
    s.set_input("Option 1", "holding_adn_min_use_kg_h", json!(1.0))
        .unwrap();
    s.set_input("Option 1", "holding_igf_domestic_use_kg_h", json!(2.0))
        .unwrap();
    s.set_input("Option 1", "holding_start_pressure_mpa", json!(0.2))
        .unwrap();
    s.set_input("Option 1", "holding_fuel_service_pressure_mpa", json!(0.5))
        .unwrap();
    s.set_input("Option 1", "holding_fill_ratio", json!(0.9))
        .unwrap();
    assert_eq!(stored(&s, "holding_profile"), Some((typed, Source::Typed)));
    assert_eq!(results_of(&s), before);
}

#[test]
fn a_bad_profile_is_refused_with_a_message() {
    let bad: Vec<(Value, &str)> = vec![
        (json!([]), "at least one leg"),
        (json!(null), "at least one leg"),
        (json!("20 days"), "at least one leg"),
        (json!([5]), "must be an object"),
        (
            json!([{ "state": "loading", "days": 1.0 }]),
            "leg 1: the state `loading` is not known",
        ),
        (json!([{ "days": 1.0 }]), "the state `` is not known"),
        (
            json!([{ "state": "idle", "days": -1.0 }]),
            "`days` must be a number above zero",
        ),
        (
            json!([{ "state": "idle", "days": 0.0 }]),
            "`days` must be a number above zero",
        ),
        (
            json!([{ "state": "idle", "days": "ten" }]),
            "`days` must be a number above zero",
        ),
        (
            json!([{ "state": "idle", "days": 1.0 }, { "state": "idle", "days": 1.0, "gas_use_kg_h": -2.0 }]),
            "leg 2: `gas_use_kg_h` must be a number of zero or more",
        ),
        (
            json!([{ "state": "laden_passage", "days": 1.0, "passage_speed_kn": -8.0 }]),
            "`passage_speed_kn` must be a number above zero",
        ),
        (
            json!([{ "state": "laden_passage", "days": 1.0, "passage_speed_kn": 0.0 }]),
            "`passage_speed_kn` must be a number above zero",
        ),
        (
            json!([{ "state": "laden_passage", "days": 1.0, "passage_speed_kn": "fast" }]),
            "`passage_speed_kn` must be a number above zero",
        ),
        (
            json!([{ "state": "idle", "days": 1.0, "passage_speed_kn": 8.0 }]),
            "an idle leg has none",
        ),
        (
            json!([{ "state": "laden_passage", "days": 1.0, "gas_use_kg_h": 8.0 }]),
            "has a typed gas use",
        ),
        (
            json!([{ "state": "idle", "days": 1.0, "heel": 1 }]),
            "`heel`, which a leg does not take",
        ),
    ];
    for (value, expect) in bad {
        let mut s = gas_session();
        s.set_input("Option 1", "holding_profile", value.clone())
            .unwrap();
        let (errors, _) = report_of(&s);
        assert!(
            errors.iter().any(|e| e.contains(expect)),
            "{value}: {errors:?}"
        );
    }
}

#[test]
fn a_negative_use_or_duration_or_an_unknown_preset_is_refused_in_the_other_inputs_too() {
    for (n, v) in [
        ("holding_adn_min_use_kg_h", json!(-1.0)),
        ("holding_igf_domestic_use_kg_h", json!(-1.0)),
        ("holding_adn_journey_days", json!(-3.0)),
        ("holding_adn_journey_days", json!(0.0)),
        ("holding_adn_journey_days", json!(366.0)),
        ("holding_start_pressure_mpa", json!(-0.1)),
        ("holding_fuel_service_pressure_mpa", json!(-0.1)),
        ("holding_fill_ratio", json!(0.0)),
        ("holding_fill_ratio", json!(1.2)),
        ("holding_preset", json!("bv_nr999")),
    ] {
        let mut s = gas_session();
        s.set_input("Option 1", n, v.clone()).unwrap();
        let (errors, _) = report_of(&s);
        assert!(
            errors.iter().any(|e| e.contains(&format!("`{n}`"))),
            "{n} {v}: {errors:?}"
        );
    }
}

#[test]
fn the_bounds_of_the_other_inputs_are_tight() {
    // The values on the edge that are allowed pass, so a bound that moved would fail a test.
    for (n, v) in [
        ("holding_adn_journey_days", json!(365.0)),
        ("holding_fill_ratio", json!(1.0)),
        ("holding_start_pressure_mpa", json!(0.0)),
        ("holding_fuel_service_pressure_mpa", json!(0.0)),
        ("holding_adn_min_use_kg_h", json!(0.0)),
    ] {
        let mut s = gas_session();
        s.set_input("Option 1", n, v.clone()).unwrap();
        let (errors, _) = report_of(&s);
        assert!(errors.is_empty(), "{n} {v}: {errors:?}");
    }
}

#[test]
fn the_gas_consumer_is_the_gas_power_sources_of_the_format() {
    for p in POWER_SOURCES {
        assert_eq!(
            ovd_core::holding_profile::has_gas_consumer(p),
            GAS_POWER_SOURCES.contains(p),
            "{p}"
        );
    }
    assert!(!ovd_core::holding_profile::has_gas_consumer(""));
}

#[test]
fn the_default_passage_speed_is_the_options_design_speed_marked_default() {
    use ovd_core::holding_profile::passage_speed_kn;
    let mut s = gas_session();
    let (profile, source) = stored(&s, "holding_profile").unwrap();
    assert_eq!(source, Source::Default);
    let design = stored(&s, "design_speed_kn").unwrap().0.as_f64().unwrap();
    let legs = ovd_format::holding_profile::legs_of(&profile).unwrap();
    assert_eq!(passage_speed_kn(&legs[0], design), Some((design, true)));
    // It follows the Option's design speed, not a number copied into the profile.
    s.set_input("Option 1", "design_speed_kn", json!(11.0))
        .unwrap();
    let design = stored(&s, "design_speed_kn").unwrap().0.as_f64().unwrap();
    assert_eq!(design, 11.0);
    assert_eq!(passage_speed_kn(&legs[0], design), Some((11.0, true)));
    // A typed speed overrides it and is not marked default.
    let typed = ovd_format::holding_profile::legs_of(&json!([
        { "state": "laden_passage", "days": 20.0, "passage_speed_kn": 8.0 }
    ]))
    .unwrap();
    assert_eq!(passage_speed_kn(&typed[0], design), Some((8.0, false)));
    // The stored default holds no speed: nothing in the file goes stale.
    assert!(profile[0].get("passage_speed_kn").is_none());
}
