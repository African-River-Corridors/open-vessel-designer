//! Seam: the bilge radius, sheer and camber inputs reach the hull (issue #449). Before, they were
//! inputs only.
//!
//! Label: property. Each test states a rule: a round bilge takes area from the midship section, and
//! a sheer or a camber adds hull above the deck edge, whose shell plating the lightship weighs since
//! issue #745.

use ovd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn results(file: &str, typed: &[(&str, f64)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
    for (name, value) in typed {
        f["options"][0]["inputs"][*name] = json!({ "value": value, "source": "typed" });
    }
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0]
        .outcome
        .as_ref()
        .expect("it ran")
    {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

const FILE: &str = "dry_bulk_box_hold";

#[test]
fn a_bilge_radius_takes_area_from_the_midship_section() {
    let base = results(FILE, &[]);
    let round = results(FILE, &[("bilge_radius_m", 1.0)]);
    let cm = |r: &Value| r["hydrostatics"]["cm"].as_f64().unwrap();
    assert!(
        (cm(&base) - 1.0).abs() < 1e-12,
        "a square bilge: Cm {}",
        cm(&base)
    );
    assert!(cm(&round) < 0.99, "Cm {}", cm(&round));
    assert_ne!(base["hydrostatics"], round["hydrostatics"]);
}

#[test]
fn a_sheer_or_a_camber_adds_shell_steel_above_the_deck_edge() {
    // A gas barge: its IS Code GZ curves heel the deck edge under. Issue #740: the self-propelled
    // gas fixture, whose arrangement passes every check. The tank is held at 7.4 m so the search
    // keeps one arrangement. Issue #745: the lightship takes the plate model, whose shell follows the
    // Sections Table, so the plating a sheer or a camber adds above the deck edge is in the lightship
    // (before, the float was the same to rounding).
    let file = "gas_igc_2g_self_propelled";
    let base = results(file, &[("max_tank_od_m", 7.4)]);
    let steel = |r: &Value| r["cost_masses"]["masses"]["hull_steel_t"].as_f64().unwrap();
    let lightship = |r: &Value| r["results"]["lightship_t"].as_f64().unwrap();
    for typed in [
        &[
            ("max_tank_od_m", 7.4),
            ("sheer_aft_m", 0.5),
            ("sheer_fwd_m", 1.0),
        ][..],
        &[("max_tank_od_m", 7.4), ("camber_m", 0.3)][..],
    ] {
        let r = results(file, typed);
        assert!(
            steel(&r) > steel(&base),
            "{typed:?}: {} vs {}",
            steel(&r),
            steel(&base)
        );
        assert!(lightship(&r) > lightship(&base), "{typed:?}");
        assert_ne!(
            base, r,
            "{typed:?}: the hull above the deck edge is in the results"
        );
    }
}
