//! Seam: the helmsman's eye of a tug over its barges is 1.65 m above the wheelhouse floor (issue #990).
//!
//! Label: sourced - ES-TRIN 2025/1 Article 7.02(3): "a height of eye of 1,65 m above the wheelhouse floor at the
//! steering position" (ES_TRIN_2025_signed_en.pdf, Chapter 7, p. 31). The lowered eye is the tug's deck
//! height above the water plus that 1.65 m, so the expected value is worked by hand from the tug's own
//! depth and draught in the same result.

use ovd_format::{Profile, RunOutcome, check, run_design};
use serde_json::Value;

fn run_fixture(name: &str) -> Value {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let design = check(
        &std::fs::read_to_string(path).unwrap(),
        &Profile::built_in(),
    )
    .design
    .unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_lowered_eye_of_a_tug_is_its_deck_above_the_water_plus_1_65_m() {
    for fixture in ["crude_integral_tanks_pushed", "shipd_dry_bulk"] {
        let r = run_fixture(fixture);
        let tug = &r["formation"]["tug"];
        let deck_above_water =
            tug["depth_m"].as_f64().unwrap() - tug["draught_m"].as_f64().unwrap();
        let eye = r["formation"]["sightline"]["eye_lowered_above_water_m"]
            .as_f64()
            .unwrap();
        assert!(
            (eye - (deck_above_water + 1.65)).abs() < 1e-9,
            "{fixture}: {eye} against {deck_above_water} + 1.65"
        );
    }
    assert_eq!(ovd_core::aft_arrangement::ES_TRIN_EYE_ABOVE_FLOOR_M, 1.65);
}
