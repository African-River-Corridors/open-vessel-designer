//! Seam: the sightline as a Blind Distance in a Design's results (issue #354): a new self-propelled
//! Option lifts its wheelhouse to keep the view ahead within 250 m and reports it; the fit keeps no
//! arrangement that fails the sightline (#264, the Rule of #277); the margin method and a file saved
//! before keep today's check and hash.
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

fn new_option(cargo: &str, l: f64, b: f64, d: f64) -> DesignSession {
    let mut s = river_session();
    for (n, v) in [
        ("cargo", json!(cargo)),
        ("length_overall_m", json!(l)),
        ("beam_m", json!(b)),
        ("moulded_depth_m", json!(d)),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn sightline_check(r: &Value) -> Value {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "wheelhouse sees over cargo")
        .cloned()
        .unwrap_or_else(|| panic!("{}", r["checks"]))
}

#[test]
fn a_new_option_lifts_its_wheelhouse_to_250_m_and_says_so() {
    let r = results(&new_option("LNG", 110.0, 11.4, 6.0));
    let sl = &r["sightline"];
    assert_eq!(sl["method"], "blind_distance");
    assert_eq!(sl["limit_m"], 250.0);
    let blind = sl["blind_distance_light_m"].as_f64().unwrap();
    assert!(blind <= 250.0 + 1e-6, "{sl}");
    assert!(sl["lift_used_m"].as_f64().unwrap() > 0.0);
    let c = sightline_check(&r);
    assert!(
        c["message"]
            .as_str()
            .unwrap()
            .contains("es-trin/7.02(2)/blind-zone-unladen"),
        "{c}"
    );
    // The navigating air draft carries the raised wheelhouse.
    let raised_top = r["drawing"]["aft_arrangement"]["wheelhouse_top_raised_m"]
        .as_f64()
        .unwrap();
    let floor = sl["floor_raised_m"].as_f64().unwrap();
    assert!(raised_top > floor);
}

#[test]
fn the_fit_never_keeps_an_arrangement_that_fails_the_sightline() {
    // Hulls from short and deep to long and shallow; a low column so some must fail.
    for (l, b, d) in [
        (60.0, 10.0, 5.0),
        (90.0, 14.0, 7.7),
        (110.0, 11.4, 6.0),
        (135.0, 11.4, 4.5),
    ] {
        for lift in [6.0, 1.0] {
            let mut s = new_option("LNG", l, b, d);
            s.set_input("Option 1", "wheelhouse_max_lift_m", json!(lift))
                .unwrap();
            let r = results(&s);
            if r["arrangement"]["feasible"] == true {
                assert_eq!(
                    sightline_check(&r)["ok"],
                    true,
                    "{l} x {b} x {d}, lift {lift}"
                );
            }
        }
    }
}

#[test]
fn the_margin_method_and_a_file_saved_before_keep_todays_check() {
    let mut s = new_option("LNG", 110.0, 11.4, 6.0);
    s.set_input("Option 1", "sightline_method", json!("margin"))
        .unwrap();
    let r = results(&s);
    assert!(r.get("sightline").is_none());
    let c = sightline_check(&r);
    assert!(
        c["message"].as_str().unwrap().starts_with("raised eye"),
        "{c}"
    );
    for name in [
        "legacy/v2_gas_lpg_before_313_igc_stack.vdesign",
        "fixtures/gas_igc_2g_self_propelled.vdesign",
    ] {
        let path = format!("{}/tests/{name}", env!("CARGO_MANIFEST_DIR"));
        let (s, opened) =
            DesignSession::open(&read_with_area(path).unwrap(), Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
        assert!(results(&s).get("sightline").is_none(), "{name}");
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
