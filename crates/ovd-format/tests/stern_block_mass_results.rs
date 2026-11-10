//! Seam: the stern block mass in a Design's results (issue #348). A new self-propelled Option has
//! `stern_block_mass_included` true, so its lightship carries the block and the results show the
//! breakdown; a file saved before keeps false, gets no new field, and keeps its results hash.
//!
//! Label: property. tests/stern_block_mass.rs in ovd-core holds the sourced rates.

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

#[test]
fn a_new_self_propelled_option_carries_the_block_and_shows_it() {
    let mut s = river_session();
    for (n, v) in [
        ("cargo", json!("LNG")),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(6.0)),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    let with = results(&s);
    let block = with["stern_block_mass"]["total_t"].as_f64().unwrap();
    assert!(block > 50.0, "{block}");
    let sum: f64 = with["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["mass_t"].as_f64().unwrap())
        .sum();
    assert!((sum - block).abs() < 1e-9);
    s.set_input("Option 1", "stern_block_mass_included", json!(false))
        .unwrap();
    let without = results(&s);
    assert!(without.get("stern_block_mass").is_none());
}

#[test]
fn a_file_saved_before_has_no_block_mass_and_keeps_its_hash() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313_igc_stack.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let (s, opened) =
        DesignSession::open(&read_with_area(path).unwrap(), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    assert!(results(&s).get("stern_block_mass").is_none());
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
