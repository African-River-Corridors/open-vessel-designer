//! Seam: the damage criteria at their limits (issue #763, spec 0012; docs/rules/codification.md step 6).
//!
//! Label: sourced - the verdicts are worked out by `evaluation/oracle/damage_oracle.py`, an independent
//! closed-form calculation for a wall-sided box (it never calls the engine), at a hair either side of each
//! limit of ADN 2025 9.3.1.15.2, ES-TRIN 2025/1 28.03(5) and (6): the heel of 12 and 15 degrees, the area of
//! 0.0065 m.rad, the intermediate lever of 0.03 m. The constants are also held to the figures of the texts
//! one by one, so a changed constant fails here. The oracle's cases are in `evaluation/damage/cases.json`.

use ovd_core::damage_cases::normam_final_heel_ok;
use ovd_core::damage_stability::{DamageCriteria, Floats, FloodedVolume, assess};
use ovd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovd_core::sections::{Gravity, equilibrium};
use serde_json::Value;

fn criteria(name: &str) -> DamageCriteria {
    match name {
        "adn_9_3_1_final" => DamageCriteria::ADN_9_3_1_FINAL,
        "es_trin_28_03_final" => DamageCriteria::ES_TRIN_28_03_FINAL,
        "es_trin_28_03_intermediate" => DamageCriteria::ES_TRIN_28_03_INTERMEDIATE,
        "adn_9_3_1_intermediate" => DamageCriteria::ADN_9_3_1_INTERMEDIATE,
        other => panic!("{other}"),
    }
}

/// The engine agrees with the oracle on every case: the verdict exactly, the numbers to a small tolerance
/// (a case whose lever is lost inside the range is compared on its verdict only).
#[test]
fn the_engine_agrees_with_the_oracle_either_side_of_each_limit() {
    let path = format!(
        "{}/../../evaluation/damage/cases.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let file: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let h = &file["hull"];
    let f = |v: &Value| v.as_f64().unwrap();
    let (l, b, d, m) = (
        f(&h["length_m"]),
        f(&h["beam_m"]),
        f(&h["depth_m"]),
        f(&h["mass_t"]),
    );
    let table = generate(&HullFormInputs {
        length_overall_m: l,
        beam_m: b,
        moulded_depth_m: d,
        form: HullFormKind::Box,
        shape: HullShape::default(),
    })
    .unwrap();
    let fl = &h["flooded"];
    let (len, height, perm) = (
        f(&fl["length_m"]),
        f(&fl["height_m"]),
        f(&fl["permeability"]),
    );
    let flooded = [FloodedVolume::from_box(
        "F",
        (0.5 * (l - len), 0.5 * (l + len)),
        (-0.5 * b, 0.5 * b),
        (0.0, height),
        perm,
    )];
    let cases = file["cases"].as_array().unwrap();
    assert!(cases.len() >= 12);
    for c in cases {
        let id = c["id"].as_str().unwrap();
        let g = |tcg: f64| Gravity {
            lcg_m: 0.5 * l,
            tcg_m: tcg,
            kg_m: f(&c["kg_m"]),
        };
        let eq = equilibrium(&table, m, g(0.0), 0.0, 1.0).unwrap();
        let out = assess(
            &table,
            &flooded,
            m,
            g(f(&c["tcg_m"])),
            1.0,
            (eq.waterplane.c, eq.waterplane.trim_deg),
            &criteria(c["criteria"].as_str().unwrap()),
        );
        let want = &c["expect"];
        assert_eq!(out.ok, want["ok"].as_bool().unwrap(), "{id}: {out:?}");
        if out.floats == Floats::Yes && !want["heel_deg"].is_null() {
            assert!(
                (out.heel_deg.abs() - f(&want["heel_deg"]).abs()).abs() < 2e-3,
                "{id}: heel {} against {}",
                out.heel_deg,
                want["heel_deg"]
            );
        }
        if want["failing"]
            .as_array()
            .is_some_and(|a| !a.iter().any(|x| x == "range"))
        {
            assert!(
                (out.max_gz_m - f(&want["max_gz_m"])).abs() < 2e-4,
                "{id}: {out:?}"
            );
            assert!(
                (out.area_m_rad - f(&want["area_m_rad"])).abs() < 2e-4,
                "{id}: {out:?}"
            );
        }
    }
    // the oracle put cases on both sides of each limit it covers
    for key in ["heel-12", "heel-15", "lever"] {
        let ok: Vec<bool> = cases
            .iter()
            .filter(|c| c["id"].as_str().unwrap().contains(key))
            .map(|c| c["expect"]["ok"].as_bool().unwrap())
            .collect();
        assert!(ok.contains(&true) && ok.contains(&false), "{key}");
    }
}

/// Each criterion is the figure its text gives, typed here from the text and not read from the constants'
/// source: ADN 9.3.1.15.2 (12 degrees, 0.05 m, 0.0065 m.rad, 27 degrees; intermediate 0.03 m over 5 degrees),
/// ES-TRIN 28.03(6)(b), (c) (12, 0.05, 0.0065, 27) and 28.03(5)(a), (b) (15 degrees, 0.02 m, 27 degrees), IGC
/// 2.7.1.2 and 2.7.2.1 and NORMAM 5.23.9 (30 degrees, a range of 20 degrees, 0.1 m, 0.0175 m.rad).
#[test]
fn each_criterion_is_the_figure_its_text_gives() {
    let want =
        |c: DamageCriteria, heel: f64, gz: f64, area: f64, limit: f64, range: Option<f64>| {
            assert_eq!(
                (
                    c.max_heel_deg,
                    c.min_gz_m,
                    c.min_area_m_rad,
                    c.limit_deg,
                    c.range_beyond_deg
                ),
                (heel, gz, area, limit, range)
            );
        };
    want(
        DamageCriteria::ADN_9_3_1_FINAL,
        12.0,
        0.05,
        0.0065,
        27.0,
        None,
    );
    want(DamageCriteria::ADN_2025, 12.0, 0.05, 0.0065, 27.0, None);
    want(
        DamageCriteria::ES_TRIN_28_03_FINAL,
        12.0,
        0.05,
        0.0065,
        27.0,
        None,
    );
    want(
        DamageCriteria::ES_TRIN_28_03_INTERMEDIATE,
        15.0,
        0.02,
        0.0,
        27.0,
        None,
    );
    // ADN gives the intermediate stage no heel limit (90 degrees is past capsize) and no area
    want(
        DamageCriteria::ADN_9_3_1_INTERMEDIATE,
        90.0,
        0.03,
        0.0,
        0.0,
        Some(5.0),
    );
    want(DamageCriteria::IGC_2_7, 30.0, 0.1, 0.0175, 0.0, Some(20.0));
    want(
        DamageCriteria::NORMAM_5_23_9,
        30.0,
        0.1,
        0.0175,
        0.0,
        Some(20.0),
    );
    assert_eq!(
        ovd_core::damage_stability::INTERMEDIATE_FRACTIONS,
        [0.25, 0.5, 0.75]
    );
}

/// NORMAM 5.23.9 (c): 25 degrees, or up to 30 when the deck is not immersed; either side of 25 and of the deck.
#[test]
fn normam_accepts_up_to_30_degrees_only_with_the_deck_dry() {
    assert!(normam_final_heel_ok(true, 24.9, true));
    assert!(normam_final_heel_ok(true, -24.9, true));
    assert!(!normam_final_heel_ok(true, 25.1, true));
    assert!(!normam_final_heel_ok(true, -25.1, true));
    assert!(normam_final_heel_ok(true, 25.1, false));
    assert!(normam_final_heel_ok(true, 29.9, false));
    assert!(!normam_final_heel_ok(false, 20.0, false));
    // the criteria of the verdict with the 30 degree limit: 30.1 degrees fails whatever the deck does
    assert_eq!(DamageCriteria::NORMAM_5_23_9.max_heel_deg, 30.0);
}
