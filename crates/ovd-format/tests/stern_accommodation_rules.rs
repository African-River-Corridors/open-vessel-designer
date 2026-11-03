//! Seam: the stern Rules of the deckhouse (issue #781, spec 0011 section 5): the IGC airlock (3.6.1),
//! SOLAS V/22.1.3 (a gap row), MLC 2006 Standard A3.1 and ES-TRIN 2025/1 Chapter 15.
//!
//! Label: sourced - IGC Code MSC.370(93) 3.6.1 (annex p. 37); SOLAS V/22.1.3 (MSC.99(73), annex p. 133);
//! MLC 2006 as amended 2022, Standard A3.1 paragraphs 6(a), 6(c), 9(f), 11(c) (mlc-2006-amended-2022.pdf,
//! printed pp. 45-48); ES-TRIN 2025/1 Articles 15.02(6), 15.02(8), 15.02(9), 15.03(1), 15.03(3)
//! (ES_TRIN_2025_signed_en.pdf, pp. 104-105)

use ovd_core::rule_trace::{MeasureValue, table_value};
use ovd_core::rules::{Compare, Consequence, Limit};
use ovd_format::rule_set::Library;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn results(service_area: &str, style: &str) -> Value {
    let inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" },
        "service_area": { "value": service_area, "source": "typed" },
        "arrangement_style": { "value": style, "source": "typed" }
    });
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let d = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    match run_option_with(&d.options[0], &Library::built_in(), &[]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn rows(r: &Value, id: &str) -> Vec<Value> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["rule"] == id && x["outcome"] != "not applicable")
        .cloned()
        .collect()
}

const NEW_ROWS: &[(&str, &str, &str)] = &[
    ("igc-2g", "3.3.0", "igc/3.6.1/airlock"),
    ("igc-2g", "3.3.0", "igc/3.6.1/airlock-door-spacing-min"),
    ("igc-2g", "3.3.0", "igc/3.6.1/airlock-door-spacing-max"),
    ("solas-v22", "1.1.0", "solas/V.22.1.3/conning-arc"),
    ("mlc-2006", "1.0.0", "mlc/A3.1.6(a)/headroom"),
    ("mlc-2006", "1.0.0", "mlc/A3.1.6(c)/sleeping-room-location"),
    ("mlc-2006", "1.0.0", "mlc/A3.1.9(f)/cabin-area"),
    ("mlc-2006", "1.0.0", "mlc/A3.1.11(c)/toilets"),
    ("mlc-2006", "1.0.0", "mlc/A3.1.11(c)/washbasins"),
    ("mlc-2006", "1.0.0", "mlc/A3.1.11(c)/showers"),
    (
        "mlc-2006",
        "1.0.0",
        "mlc/A3.1.11(d)/sleeping-room-washbasin",
    ),
    ("es-trin-2025", "2.1.0", "es-trin/15.02(6)/headroom"),
    (
        "es-trin-2025",
        "2.1.0",
        "es-trin/15.02(8)/communal-per-person",
    ),
    ("es-trin-2025", "2.1.0", "es-trin/15.02(8)/communal-total"),
    ("es-trin-2025", "2.1.0", "es-trin/15.02(9)/cabin-volume"),
    ("es-trin-2025", "2.1.0", "es-trin/15.03(1)(a)/toilets"),
    ("es-trin-2025", "2.1.0", "es-trin/15.03(1)(b)/washbasins"),
    ("es-trin-2025", "2.1.0", "es-trin/15.03(1)(c)/showers"),
    ("es-trin-2025", "2.1.0", "es-trin/15.03(3)/toilet-floor"),
];

#[test]
fn every_new_rule_is_unverified_and_cites_a_locator_that_matches_its_id() {
    let lib = Library::built_in();
    for (set, version, id) in NEW_ROWS {
        let rule = lib.find(set, version).unwrap().rule(id).unwrap();
        assert!(rule.verified.is_none(), "{id}");
        assert_eq!(rule.finding, None, "{id}");
        assert_eq!(rule.tolerance, 0.0, "{id}");
        let paragraph = id.split('/').nth(1).unwrap().replace(' ', "");
        assert_eq!(
            rule.source.locator.replace(' ', ""),
            paragraph,
            "{id}: the paragraph segment is the locator"
        );
    }
}

#[test]
fn the_rows_the_model_cannot_check_are_not_evaluated_and_never_pass() {
    let lib = Library::built_in();
    for id in [
        "igc/3.6.1/airlock",
        "igc/3.6.1/airlock-door-spacing-min",
        "igc/3.6.1/airlock-door-spacing-max",
    ] {
        let rule = lib.find("igc-2g", "3.3.0").unwrap().rule(id).unwrap();
        assert!(!rule.evaluated, "{id}");
        assert!(rule.gap.is_some(), "{id}");
    }
    for (set, version, id) in [
        ("solas-v22", "1.1.0", "solas/V.22.1.3/conning-arc"),
        ("mlc-2006", "1.0.0", "mlc/A3.1.6(c)/sleeping-room-location"),
    ] {
        let rule = lib.find(set, version).unwrap().rule(id).unwrap();
        assert!(!rule.evaluated && rule.gap.is_some(), "{id}");
    }
    // Through a coastal gas deckhouse Option: each reports `not evaluated`, with its gap.
    let r = results("coastal-and-river", "deckhouse");
    for id in [
        "igc/3.6.1/airlock",
        "igc/3.6.1/airlock-door-spacing-min",
        "igc/3.6.1/airlock-door-spacing-max",
        "solas/V.22.1.3/conning-arc",
        "mlc/A3.1.6(c)/sleeping-room-location",
    ] {
        let found = rows(&r, id);
        assert_eq!(found.len(), 1, "{id}");
        assert_eq!(found[0]["outcome"], "not evaluated", "{id}");
        assert!(found[0]["gap"].is_string(), "{id}");
    }
}

#[test]
fn the_solas_arc_is_225_degrees() {
    // SOLAS V/22.1.3: "not less than 225 degrees".
    let lib = Library::built_in();
    let rule = lib
        .find("solas-v22", "1.1.0")
        .unwrap()
        .rule("solas/V.22.1.3/conning-arc")
        .unwrap();
    assert_eq!(rule.compare, Some(Compare::Ge));
    let Some(Limit::Constant { value, .. }) = &rule.limit else {
        panic!("a constant limit")
    };
    assert_eq!(*value, 225.0);
}

#[test]
fn the_airlock_doors_are_1_5_to_2_5_m_apart() {
    // IGC 3.6.1: "at least 1.5 m but no more than 2.5 m apart".
    let lib = Library::built_in();
    let set = lib.find("igc-2g", "3.3.0").unwrap();
    for (id, compare, limit) in [
        ("igc/3.6.1/airlock-door-spacing-min", Compare::Ge, 1.5),
        ("igc/3.6.1/airlock-door-spacing-max", Compare::Le, 2.5),
    ] {
        let rule = set.rule(id).unwrap();
        assert_eq!(rule.compare, Some(compare), "{id}");
        let Some(Limit::Constant { value, .. }) = &rule.limit else {
            panic!("{id}: a constant limit")
        };
        assert_eq!(*value, limit, "{id}");
    }
}

#[test]
fn the_mlc_single_berth_floor_is_4_5_5_5_and_7_m2_by_gross_tonnage() {
    // MLC A3.1.9(f), each branch and each boundary: (i) under 3,000 GT, (ii) 3,000 or more but under
    // 10,000, (iii) 10,000 or more.
    let lib = Library::built_in();
    let set = lib.find("mlc-2006", "1.0.0").unwrap();
    let table = set.table("mlc-a3-1-9f-single-berth").unwrap();
    let at = |gt: f64| table_value(table, &MeasureValue::Number(gt)).unwrap();
    assert_eq!(at(1.0), 4.5);
    assert_eq!(at(2999.9), 4.5);
    assert_eq!(at(3000.0), 5.5);
    assert_eq!(at(9999.9), 5.5);
    assert_eq!(at(10000.0), 7.0);
    assert_eq!(at(50000.0), 7.0);
    let rule = set.rule("mlc/A3.1.9(f)/cabin-area").unwrap();
    assert_eq!(rule.compare, Some(Compare::Ge));
    assert_eq!(rule.consequence, Consequence::Block);
}

#[test]
fn the_mlc_headroom_is_203_cm_and_the_sanitary_ratio_is_six() {
    let lib = Library::built_in();
    let set = lib.find("mlc-2006", "1.0.0").unwrap();
    let rule = set.rule("mlc/A3.1.6(a)/headroom").unwrap();
    assert_eq!(rule.compare, Some(Compare::Ge));
    let Some(Limit::Constant { value, .. }) = &rule.limit else {
        panic!("a constant limit")
    };
    assert_eq!(*value, 2.03);
    for id in [
        "mlc/A3.1.11(c)/toilets",
        "mlc/A3.1.11(c)/washbasins",
        "mlc/A3.1.11(c)/showers",
    ] {
        let rule = set.rule(id).unwrap();
        assert_eq!(rule.compare, Some(Compare::Le), "{id}");
        let Some(Limit::Constant { value, .. }) = &rule.limit else {
            panic!("{id}: a constant limit")
        };
        assert_eq!(*value, 6.0, "{id}");
    }
}

#[test]
fn es_trin_15_limits_are_the_printed_numbers() {
    let lib = Library::built_in();
    let set = lib.find("es-trin-2025", "2.1.0").unwrap();
    for (id, compare, limit) in [
        ("es-trin/15.02(6)/headroom", Compare::Ge, 2.0),
        ("es-trin/15.02(8)/communal-per-person", Compare::Ge, 2.0),
        ("es-trin/15.02(8)/communal-total", Compare::Ge, 8.0),
        ("es-trin/15.02(9)/cabin-volume", Compare::Ge, 7.0),
        ("es-trin/15.03(1)(a)/toilets", Compare::Le, 6.0),
        ("es-trin/15.03(1)(b)/washbasins", Compare::Le, 4.0),
        ("es-trin/15.03(1)(c)/showers", Compare::Le, 6.0),
        ("es-trin/15.03(3)/toilet-floor", Compare::Ge, 1.0),
    ] {
        let rule = set.rule(id).unwrap();
        assert_eq!(rule.compare, Some(compare), "{id}");
        let Some(Limit::Constant { value, .. }) = &rule.limit else {
            panic!("{id}: a constant limit")
        };
        assert_eq!(*value, limit, "{id}");
        assert_eq!(rule.consequence, Consequence::Block, "{id}");
    }
}

#[test]
fn a_deckhouse_option_meets_the_es_trin_rooms_and_a_coastal_one_misses_three_mlc_rows() {
    // The rooms table is sized to ES-TRIN 15, so the ES-TRIN rows pass. MLC A3.1.6(a) asks 2.03 m of
    // headroom (the rooms use 2.00 m) and A3.1.9(f) at least 4.5 m2 a single cabin under 3,000 GT
    // (the rooms give 3.5 m2): a hull that reaches sea fails both until the rooms apply MLC.
    let r = results("coastal-and-river", "deckhouse");
    for id in [
        "es-trin/15.02(6)/headroom",
        "es-trin/15.02(8)/communal-per-person",
        "es-trin/15.02(8)/communal-total",
        "es-trin/15.02(9)/cabin-volume",
        "es-trin/15.03(1)(a)/toilets",
        "es-trin/15.03(1)(b)/washbasins",
        "es-trin/15.03(1)(c)/showers",
        "es-trin/15.03(3)/toilet-floor",
        "mlc/A3.1.11(c)/toilets",
        "mlc/A3.1.11(c)/washbasins",
        "mlc/A3.1.11(c)/showers",
    ] {
        let found = rows(&r, id);
        assert_eq!(found.len(), 1, "{id}");
        assert_eq!(found[0]["outcome"], "pass", "{id}: {:?}", found[0]);
    }
    // 11(d): the rooms hold one washbasin per four berths, not one per sleeping room.
    let basin = rows(&r, "mlc/A3.1.11(d)/sleeping-room-washbasin");
    assert_eq!(basin[0]["outcome"], "fail");
    assert!(basin[0]["value"].as_f64().unwrap() < 1.0);
    let head = rows(&r, "mlc/A3.1.6(a)/headroom");
    assert_eq!(head[0]["outcome"], "fail");
    assert_eq!(head[0]["value"], json!(2.0));
    assert_eq!(head[0]["limit"], json!(2.03));
    let area = rows(&r, "mlc/A3.1.9(f)/cabin-area");
    assert_eq!(area[0]["outcome"], "fail");
    assert_eq!(area[0]["value"], json!(3.5));
    assert_eq!(area[0]["limit"], json!(4.5));
}

#[test]
fn a_river_only_option_takes_no_mlc_rows_and_a_container_block_has_no_rooms_to_check() {
    let river = results("river-only", "deckhouse");
    assert!(
        !river["rule_trace"]["rule_sets"]
            .to_string()
            .contains("mlc-2006")
    );
    assert_eq!(rows(&river, "es-trin/15.02(9)/cabin-volume").len(), 1);
    // The container block holds no rooms table: the rooms Rules are not applicable (not shown).
    let block = results("coastal-and-river", "container_block");
    assert!(rows(&block, "mlc/A3.1.9(f)/cabin-area").is_empty());
    assert!(rows(&block, "es-trin/15.02(9)/cabin-volume").is_empty());
}
