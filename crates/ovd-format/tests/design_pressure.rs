//! Seam: the design pressure of a Type C tank follows the Service Area (issue #926). On a coastal
//! Service Area an LNG tank takes the IGC 4.23.1.2 minimum of its own size unless a value is typed;
//! on river-only it keeps the profile value. The results say how the pressure was set.
//!
//! Label: sourced - IGC Code 4.23.1.2 (MSC.370(93)), Po = 0.2 + A C rho_r^1.5, worked by hand in
//! `.agents/artifacts/926/02-research.md` for the 140 x 29 m coastal LNG reference design.

use ovd_core::pressure_vessel::igc_dynamic_pressure_term_mpa;
use ovd_format::{DesignSession, Profile, RunOutcome, Source};
use serde_json::{Value, json};

const OPTION: &str = "A basis";

/// The 140 x 29 m coastal LNG reference design, with its pressure left to the profile default.
fn reference() -> DesignSession {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let (mut s, _) = DesignSession::open(&text, Profile::built_in()).unwrap();
    // The file stores the pressure the profile gave when it was made; a reset takes today's default.
    s.reset_input(OPTION, "tank_design_pressure_mpa").unwrap();
    s
}

fn results(s: &DesignSession) -> Value {
    let ev = s.evaluate();
    let o = ev.options.iter().find(|o| o.name == OPTION).unwrap();
    match o.outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn stored(s: &DesignSession) -> Option<(Value, Source)> {
    s.design()
        .options
        .iter()
        .find(|o| o.name == OPTION)
        .and_then(|o| o.input("tank_design_pressure_mpa"))
        .map(|i| (i.value.clone(), i.source))
}

fn pressure_row(r: &Value) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    for row in r["rule_trace"]["rows"].as_array().unwrap() {
        if row["rule"] == "igc/4.23.1.2/min-design-pressure" && row["outcome"] != "not applicable" {
            out.push((
                row["value"].as_f64().unwrap_or(f64::NAN),
                row["limit"].as_f64().unwrap_or(f64::NAN),
                row["outcome"].as_str().unwrap().to_string(),
            ));
        }
    }
    out
}

/// Label: sourced - IGC 4.23.1.2, hand-worked Po = 0.4149 MPa (research section 5)
#[test]
fn a_coastal_lng_tank_with_no_typed_pressure_takes_the_igc_minimum() {
    let s = reference();
    assert_eq!(stored(&s), Some((Value::Null, Source::Default)));
    let r = results(&s);
    let tank = &r["drawing"]["tank"];
    let p = tank["design_pressure_mpa"].as_f64().unwrap();
    assert!((p - 0.4149).abs() < 1e-4, "{p}");
    assert_eq!(tank["design_pressure_basis"], json!("igc_4_23_1_2_minimum"));
    // The Rule has no margin to spare: the design pressure is its limit.
    for (value, limit, outcome) in pressure_row(&r) {
        assert!((value - limit).abs() < 1e-9, "{value} {limit}");
        assert_eq!(outcome, "pass");
    }
}

/// Label: property. A typed value is kept and says so; a typed value below the minimum fails the Rule.
#[test]
fn a_typed_pressure_is_kept_and_the_igc_row_judges_it() {
    let mut s = reference();
    s.set_input(OPTION, "tank_design_pressure_mpa", json!(0.5))
        .unwrap();
    let r = results(&s);
    assert_eq!(r["drawing"]["tank"]["design_pressure_mpa"], json!(0.5));
    assert_eq!(
        r["drawing"]["tank"]["design_pressure_basis"],
        json!("typed")
    );
    for (_, _, outcome) in pressure_row(&r) {
        assert_eq!(outcome, "pass");
    }
    s.set_input(OPTION, "tank_design_pressure_mpa", json!(0.35))
        .unwrap();
    let rows = pressure_row(&results(&s));
    assert!(!rows.is_empty());
    // `warn` now; it becomes `block` when #927 lands.
    for (_, _, outcome) in rows {
        assert_eq!(outcome, "warn");
    }
}

/// Label: property. The minimum grows with the tank: a larger outer diameter or a longer tank gives
/// a higher Po, all else equal.
#[test]
fn a_larger_tank_has_the_higher_minimum() {
    let po = |od: f64, length: f64| 0.2 + igc_dynamic_pressure_term_mpa(230.0, od, length, 0.45);
    assert!(po(11.0, 48.9) > po(8.0, 30.0));
    assert!(po(11.0, 60.0) > po(11.0, 48.9));
    assert!(po(12.0, 48.9) >= po(11.0, 48.9));
}

/// Label: property. A defaulted pressure follows the Service Area in a session; a typed one does not.
#[test]
fn changing_the_service_area_moves_a_defaulted_pressure_with_it() {
    let mut s = reference();
    assert_eq!(stored(&s), Some((Value::Null, Source::Default)));
    s.set_input(OPTION, "service_area", json!("river-only"))
        .unwrap();
    assert_eq!(stored(&s), Some((json!(0.4), Source::Default)));
    // Issue #1034: a river-only Option needs a number for both river limits before it runs.
    s.set_input(OPTION, "draught_limit_m", json!(12.0)).unwrap();
    s.set_input(OPTION, "air_draft_limit_m", json!(40.0))
        .unwrap();
    let r = results(&s);
    assert_eq!(r["drawing"]["tank"]["design_pressure_mpa"], json!(0.4));
    assert_eq!(
        r["drawing"]["tank"]["design_pressure_basis"],
        json!("default")
    );
    s.set_input(OPTION, "service_area", json!("coastal-and-river"))
        .unwrap();
    assert_eq!(stored(&s), Some((Value::Null, Source::Default)));
    // A typed pressure stays when the Service Area changes.
    s.set_input(OPTION, "tank_design_pressure_mpa", json!(0.6))
        .unwrap();
    s.set_input(OPTION, "service_area", json!("river-only"))
        .unwrap();
    assert_eq!(stored(&s), Some((json!(0.6), Source::Typed)));
}
