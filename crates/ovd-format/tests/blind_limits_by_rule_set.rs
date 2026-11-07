//! Seam: the Blind Distance limit comes from the Rule Sets of the stack, one reading for the
//! arrangement search and the Rule Trace (issue #989).
//!
//! Label: sourced - ES-TRIN 2025/1 Article 7.02(2): obstructed view ahead at most 250 m (ES_TRIN_2025_signed_en.pdf,
//! p. 31); SOLAS 1974 V/22.1.1 (MSC.99(73), annex p. 133): at most two ship lengths or 500 m, whichever is less,
//! for a ship of 45 m and over. The expected numbers are the texts' own, worked by hand.

use ovd_core::gas_arrangement::{RuleInputs, blind_limits};
use ovd_core::rule_trace::MeasureValue;
use ovd_core::rules::RuleSet;
use ovd_format::rule_set::Library;
use ovd_format::trace::measure_info;

fn stack(ids: &[(&str, &str)]) -> Vec<RuleSet> {
    let lib = Library::built_in();
    ids.iter()
        .map(|(id, v)| {
            lib.find(id, v)
                .unwrap_or_else(|| panic!("{id}@{v}"))
                .clone()
        })
        .collect()
}

fn inputs(area: &str, loa: f64) -> RuleInputs {
    // `solas-v22` 1.x reads the input length overall, 2.0.0 the ship's length (issue #989).
    let measure = |id: &str| {
        (
            id.to_string(),
            measure_info(id).unwrap(),
            MeasureValue::Number(loa),
        )
    };
    RuleInputs {
        applicability: vec![
            ("service_area".to_string(), area.to_string()),
            (
                "length_band".to_string(),
                ovd_core::measured::length_band(loa).to_string(),
            ),
        ],
        measures: vec![
            measure("input.length_overall_m"),
            measure("wheelhouse.ship_length"),
        ],
    }
}

const ES_TRIN: (&str, &str) = ("es-trin-2025", "2.1.0");
const SOLAS: (&str, &str) = ("solas-v22", "2.0.0");

#[test]
fn a_river_stack_limits_the_light_blind_distance_to_250_m_and_the_laden_one_not_at_all() {
    // ES-TRIN 7.02(2) states the unladen condition only.
    let l = blind_limits(&stack(&[ES_TRIN]), &inputs("river-only", 140.0));
    assert_eq!(l.light_m, Some(250.0));
    assert_eq!(l.laden_m, None);
    assert_eq!(
        l.light_rule.as_deref(),
        Some("es-trin/7.02(2)/blind-zone-unladen")
    );
}

#[test]
fn a_coastal_stack_takes_the_stricter_limit_in_each_condition() {
    // 140 m: SOLAS gives 280 m, so ES-TRIN's 250 m is stricter in the light condition; SOLAS alone
    // limits the laden one.
    let l = blind_limits(
        &stack(&[ES_TRIN, SOLAS]),
        &inputs("coastal-and-river", 140.0),
    );
    assert_eq!((l.light_m, l.laden_m), (Some(250.0), Some(280.0)));
    assert_eq!(
        l.light_rule.as_deref(),
        Some("es-trin/7.02(2)/blind-zone-unladen")
    );
    assert_eq!(
        l.laden_rule.as_deref(),
        Some("solas/V.22.1.1/blind-zone-laden")
    );
    // 100 m: 2 L = 200 m is stricter than 250 m in both conditions.
    let l = blind_limits(
        &stack(&[ES_TRIN, SOLAS]),
        &inputs("coastal-and-river", 100.0),
    );
    assert_eq!((l.light_m, l.laden_m), (Some(200.0), Some(200.0)));
    assert_eq!(
        l.light_rule.as_deref(),
        Some("solas/V.22.1.1/blind-zone-light")
    );
    // 300 m: the 500 m cap, so ES-TRIN is stricter in the light condition.
    let l = blind_limits(
        &stack(&[ES_TRIN, SOLAS]),
        &inputs("coastal-and-river", 300.0),
    );
    assert_eq!((l.light_m, l.laden_m), (Some(250.0), Some(500.0)));
}

#[test]
fn a_ship_under_45_m_is_outside_solas_so_only_es_trin_limits_it() {
    let l = blind_limits(
        &stack(&[ES_TRIN, SOLAS]),
        &inputs("coastal-and-river", 44.9),
    );
    assert_eq!((l.light_m, l.laden_m), (Some(250.0), None));
}

#[test]
fn a_rule_that_only_warns_gives_no_limit() {
    // solas-v22 1.1.0 warns; a warning blocks nothing, so it sizes nothing.
    let l = blind_limits(
        &stack(&[SOLAS_WARNING]),
        &inputs("coastal-and-river", 100.0),
    );
    assert!(l.is_none(), "{l:?}");
}

const SOLAS_WARNING: (&str, &str) = ("solas-v22", "1.1.0");

#[test]
fn a_stack_with_no_blind_distance_rule_gives_no_limit() {
    let l = blind_limits(&stack(&[("ovd", "3.2.0")]), &inputs("river-only", 140.0));
    assert!(l.is_none(), "{l:?}");
    assert!(blind_limits(&[], &inputs("river-only", 140.0)).is_none());
}

#[test]
fn a_warning_only_solas_rule_gives_no_blocking_limit_but_still_sizes_a_fixed_deckhouse() {
    // Issue #779 and ADR 0007, kept by #989: an old coastal stack holds `solas-v22` 1.x, which warns. The
    // blocking limit is ES-TRIN's 250 m alone; the deckhouse sizing limit is the least of the two, 2 L.
    let l = blind_limits(
        &stack(&[ES_TRIN, SOLAS_WARNING]),
        &inputs("coastal-and-river", 100.0),
    );
    assert_eq!((l.light_m, l.laden_m), (Some(250.0), None));
    assert_eq!(l.sizing_light_m, Some(200.0));
    // With a blocking SOLAS the two are the same.
    let l = blind_limits(
        &stack(&[ES_TRIN, SOLAS]),
        &inputs("coastal-and-river", 100.0),
    );
    assert_eq!((l.light_m, l.sizing_light_m), (Some(200.0), Some(200.0)));
}

#[test]
fn an_old_coastal_file_at_100_m_keeps_its_deckhouse_sized_to_two_lengths_and_the_rule_only_warns() {
    // Failing first: the fixture's stored stack holds solas-v22 1.0.0 (a warning). Before the sizing limit,
    // the house was sized to 250 m alone: limit 250 m, light Blind Distance 240.8 m, over SOLAS's 200 m.
    let path = format!(
        "{}/tests/fixtures/gas_deckhouse_self_propelled.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut file: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    file["options"][0]["inputs"]["length_overall_m"]["value"] = serde_json::json!(100.0);
    let design = ovd_format::check(&file.to_string(), &ovd_format::Profile::built_in())
        .design
        .unwrap();
    let r = match ovd_format::run_design(&design).remove(0).outcome {
        ovd_format::RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    };
    let sl = &r["sightline"];
    assert_eq!(sl["limit_m"], serde_json::json!(250.0), "{sl}");
    let blind = sl["blind_distance_light_m"].as_f64().unwrap();
    assert!(blind <= 200.0 + 1e-6, "{blind}");
    let solas = r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rule"] == "solas/V.22.1.1/blind-zone-light")
        .unwrap();
    assert_eq!(solas["outcome"], "pass", "{solas}");
    assert_eq!(solas["consequence"], "warn");
}
