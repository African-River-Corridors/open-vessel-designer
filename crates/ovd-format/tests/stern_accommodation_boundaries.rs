//! Seam: each evaluated stern Rule at both sides of its limit, through a stub `MeasureSource` (issue
//! #781). Each assertion fails if the comparison or the limit of its Rule changes.
//!
//! Label: sourced - MLC 2006 as amended 2022, Standard A3.1 paragraphs 6(a), 9(f), 11(c), 11(d)
//! (mlc-2006-amended-2022.pdf, printed pp. 45-48); ES-TRIN 2025/1 Articles 15.02(6), 15.02(8),
//! 15.02(9), 15.03(1), 15.03(3) (ES_TRIN_2025_signed_en.pdf, pp. 104-105). "Not less than" is `ge`: the
//! limit passes, anything below fails; "one per six" is at most six persons a fixture.

use ovd_core::rule_trace::{
    MeasureInfo, MeasureSource, MeasureValue, ObjectType, Outcome, evaluate,
};
use ovd_core::rules::{ArgValue, MeasureRef};
use ovd_format::rule_set::Library;
use ovd_format::trace::measure_info;

/// One Option where each Measure (by id, with a text argument appended) has the value given.
struct Stub(Vec<(String, f64)>);

fn key(m: &MeasureRef) -> String {
    let mut k = m.id.clone();
    for (_, v) in &m.args {
        if let ArgValue::Text(x) = v {
            k.push(':');
            k.push_str(x);
        }
    }
    k
}

impl MeasureSource for Stub {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }
    fn objects(&self, object: &str) -> Vec<String> {
        match ObjectType::from_key(object).expect("an OVD object type") {
            ObjectType::Option => vec!["Option".into()],
            _ => Vec::new(),
        }
    }
    fn value(&self, m: &MeasureRef, _: usize) -> Result<MeasureValue, String> {
        let k = key(m);
        self.0
            .iter()
            .find(|(id, _)| *id == k)
            .map(|(_, v)| MeasureValue::Number(*v))
            .ok_or(k)
    }
    fn input(&self, name: &str) -> Option<String> {
        // The Service Area the `mlc-2006` set applies to.
        (name == "service_area").then(|| "coastal-and-river".to_string())
    }
}

fn outcome(set: &str, version: &str, rule: &str, values: &[(&str, f64)]) -> Outcome {
    let s = Library::built_in().find(set, version).unwrap().clone();
    let stub = Stub(values.iter().map(|(k, v)| (k.to_string(), *v)).collect());
    evaluate(&[&s], &stub)
        .rows
        .iter()
        .find(|r| r.rule.as_deref() == Some(rule))
        .unwrap_or_else(|| panic!("{rule}"))
        .outcome
}

#[test]
fn each_es_trin_15_rule_at_both_sides_of_its_limit() {
    for (rule, measure, limit, lower_is_worse) in [
        ("es-trin/15.02(6)/headroom", "rooms.headroom_m", 2.0, true),
        (
            "es-trin/15.02(8)/communal-per-person",
            "rooms.communal_area_per_person_m2",
            2.0,
            true,
        ),
        (
            "es-trin/15.02(8)/communal-total",
            "rooms.communal_area_m2",
            8.0,
            true,
        ),
        (
            "es-trin/15.02(9)/cabin-volume",
            "rooms.cabin_volume_m3",
            7.0,
            true,
        ),
        (
            "es-trin/15.03(1)(a)/toilets",
            "rooms.persons_per_fixture:toilet",
            6.0,
            false,
        ),
        (
            "es-trin/15.03(1)(b)/washbasins",
            "rooms.persons_per_fixture:washbasin",
            4.0,
            false,
        ),
        (
            "es-trin/15.03(1)(c)/showers",
            "rooms.persons_per_fixture:shower",
            6.0,
            false,
        ),
        (
            "es-trin/15.03(3)/toilet-floor",
            "rooms.toilet_area_m2",
            1.0,
            true,
        ),
    ] {
        let eps = 0.01;
        let outside = if lower_is_worse {
            limit - eps
        } else {
            limit + eps
        };
        let at = |v: f64| outcome("es-trin-2025", "2.1.0", rule, &[(measure, v)]);
        assert_eq!(at(limit), Outcome::Pass, "{rule}");
        assert_eq!(at(outside), Outcome::Fail, "{rule}");
    }
}

#[test]
fn mlc_headroom_is_203_cm() {
    let at = |v: f64| {
        outcome(
            "mlc-2006",
            "1.0.0",
            "mlc/A3.1.6(a)/headroom",
            &[("rooms.headroom_m", v)],
        )
    };
    assert_eq!(at(2.03), Outcome::Pass);
    assert_eq!(at(2.02), Outcome::Fail);
}

#[test]
fn mlc_single_berth_floor_by_gross_tonnage() {
    // (gross tonnage, floor area): 4.5 under 3,000; 5.5 from 3,000 to under 10,000; 7 from 10,000.
    for (gt, limit) in [(2999.0, 4.5), (3000.0, 5.5), (9999.0, 5.5), (10000.0, 7.0)] {
        let at = |a: f64| {
            outcome(
                "mlc-2006",
                "1.0.0",
                "mlc/A3.1.9(f)/cabin-area",
                &[("rooms.cabin_area_m2", a), ("option.gross_tonnage", gt)],
            )
        };
        assert_eq!(at(limit), Outcome::Pass, "GT {gt}");
        assert_eq!(at(limit - 0.01), Outcome::Fail, "GT {gt}");
    }
}

#[test]
fn mlc_sanitary_ratio_is_six_and_each_sleeping_room_has_a_washbasin() {
    for (rule, fixture) in [
        ("mlc/A3.1.11(c)/toilets", "toilet"),
        ("mlc/A3.1.11(c)/washbasins", "washbasin"),
        ("mlc/A3.1.11(c)/showers", "shower"),
    ] {
        let m = format!("rooms.persons_per_fixture:{fixture}");
        let at = |v: f64| outcome("mlc-2006", "1.0.0", rule, &[(m.as_str(), v)]);
        assert_eq!(at(6.0), Outcome::Pass, "{rule}");
        assert_eq!(at(6.01), Outcome::Fail, "{rule}");
    }
    let at = |v: f64| {
        outcome(
            "mlc-2006",
            "1.0.0",
            "mlc/A3.1.11(d)/sleeping-room-washbasin",
            &[("rooms.washbasins_per_cabin", v)],
        )
    };
    assert_eq!(at(1.0), Outcome::Pass);
    assert_eq!(at(0.99), Outcome::Fail);
}
