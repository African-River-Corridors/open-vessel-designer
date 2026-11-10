//! Seam: the machinery in a Design's results (issue #352): a new self-propelled Option with a
//! specified Propulsor lays out its machinery below deck and weighs it; with no installed power the
//! engine room waits, with a warning; a file saved before keeps today's stern block and its hash.
//!
//! Label: property.

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome};
use serde_json::{Value, json};

/// Issue #925: a legacy fixture has no Service Area, which is now an error. This test is about something else, so
/// it reads the file with a river-only Service Area set, as `ovd fill --service-area river-only` does.
fn read_with_area(path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
    let text = std::fs::read_to_string(path)?;
    Ok(ovd_format::with_service_area(&text, "river-only").unwrap())
}

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn zones(r: &Value) -> Vec<String> {
    r["drawing"]["aft_arrangement"]["zones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|z| z[0].as_str().unwrap().to_string())
        .collect()
}

fn items(r: &Value) -> Vec<String> {
    r["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap().to_string())
        .collect()
}

fn new_dry() -> DesignSession {
    let mut s = river_session();
    for (n, v) in [
        ("cargo", json!("bauxite")),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(4.0)),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

#[test]
fn a_typed_installed_power_places_the_engines_below_deck_and_in_the_lightship() {
    let mut s = new_dry();
    // A dry hull's default: shafts in nozzles, diesel-mechanical, a channel jet in the bow.
    let r = results(&s);
    assert_eq!(
        zones(&r)[..2],
        [
            "AFT PEAK, STEERING GEAR".to_string(),
            "SHAFT RUN".to_string()
        ]
    );
    // Issue #353 sizes the Installed Power, so the engines are below deck with no warning.
    let warned = |r: &Value| {
        r["warnings"].as_array().unwrap().iter().any(|w| {
            w.as_str()
                .is_some_and(|w| w.contains("installed power is not sized"))
        })
    };
    assert!(
        zones(&r).contains(&"ENGINE ROOM".to_string()),
        "{:?}",
        zones(&r)
    );
    assert!(!warned(&r), "{}", r["warnings"]);
    assert!(items(&r).contains(&"rudders and steering gear".to_string()));
    assert!(items(&r).contains(&"bow thruster".to_string()));
    s.set_input("Option 1", "installed_power_kw", json!(1200.0))
        .unwrap();
    let r = results(&s);
    assert!(
        zones(&r).contains(&"ENGINE ROOM".to_string()),
        "{:?}",
        zones(&r)
    );
    assert!(!warned(&r));
    for label in ["engines", "gearboxes", "shaft lines"] {
        assert!(items(&r).contains(&label.to_string()), "{label}");
    }
}

#[test]
fn an_unspecified_propulsor_keeps_todays_stern_block() {
    let mut s = new_dry();
    s.set_input("Option 1", "propulsor_type", json!("unspecified"))
        .unwrap();
    let r = results(&s);
    assert_eq!(zones(&r)[0], "THRUSTER ROOM");
    assert!(items(&r).contains(&"thrusters".to_string()));
}

#[test]
fn a_file_saved_before_keeps_todays_stern_block_and_its_hash() {
    for name in [
        "legacy/v2_gas_lpg_before_313_igc_stack.vdesign",
        "fixtures/gas_igc_2g_self_propelled.vdesign",
    ] {
        let path = format!("{}/tests/{name}", env!("CARGO_MANIFEST_DIR"));
        let (s, opened) =
            DesignSession::open(&read_with_area(path).unwrap(), Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
        let r = results(&s);
        assert_eq!(zones(&r)[0], "THRUSTER ROOM", "{name}");
    }
}

/// Issue #1034: a new river-only session needs a number for both river limits before it runs. These are high enough
/// not to bind, so no value of these tests moves.
fn river_session() -> ovd_format::DesignSession {
    let mut s =
        ovd_format::DesignSession::new_design_in(ovd_format::Profile::built_in(), "river-only")
            .unwrap();
    s.set_input("Option 1", "draught_limit_m", serde_json::json!(12.0))
        .unwrap();
    s.set_input("Option 1", "air_draft_limit_m", serde_json::json!(40.0))
        .unwrap();
    s
}
