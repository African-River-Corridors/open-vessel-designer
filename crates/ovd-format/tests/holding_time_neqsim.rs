//! Seam: the holding-time model against the NeqSim reference cases of issue #868 (spec 0016 section 7,
//! decision D6).
//!
//! The cases are `evaluation/holding_time/neqsim_cases.json` (NeqSim 3.27.0, GERG-2008, pure methane).
//! The floors are set in `evaluation/holding_time/TOLERANCE.md` before any comparison: the larger of the
//! equation-of-state spread and OVD's own step error (the change when the step is halved). The test
//! reads the two machine lines of that file; it carries no tolerance of its own, and a tolerance is
//! never loosened without the maintainer. OVD steps at 600 s; its step error is the change when that
//! is halved to 300 s.
//!
//! The holding time is gated against its floor. The pressure is REPORTED, NOT GATED (maintainer,
//! 2026-11-05): OVD sits above its floor, and the finding is tracked in #TBD-pressure-gap. The floor
//! stays as it is.
//!
//! Label: sourced - NeqSim 3.27.0 with GERG-2008 (evaluation/holding_time/neqsim_cases.json, by tools/neqsim_holding_time.py).

use ovd_core::holding_time::*;
use serde_json::Value;

const STEP_S: f64 = 600.0;

fn root() -> String {
    format!("{}/../..", env!("CARGO_MANIFEST_DIR"))
}

fn cases() -> Value {
    let p = format!("{}/evaluation/holding_time/neqsim_cases.json", root());
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .unwrap()
}

/// The two figures of the machine lines of TOLERANCE.md (percent).
fn tolerances() -> (f64, f64) {
    let p = format!("{}/evaluation/holding_time/TOLERANCE.md", root());
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}"));
    let find = |key: &str| -> f64 {
        text.lines()
            .find_map(|l| l.strip_prefix(key))
            .unwrap_or_else(|| panic!("TOLERANCE.md has no line `{key}`"))
            .trim()
            .trim_end_matches('%')
            .trim()
            .parse()
            .unwrap()
    };
    (
        find("holding-time-tolerance-percent:"),
        find("pressure-tolerance-percent:"),
    )
}

fn f(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap_or_else(|| panic!("{k}"))
}

fn ovd(case: &Value, step_s: f64) -> Outcome {
    let legs = [LegInput {
        days: 100.0,
        withdrawal_kg_s: f(case, "withdrawal_kg_s_vapour"),
    }];
    run(&Inputs {
        volume_m3: f(case, "volume_m3"),
        start_gauge_mpa: 0.0,
        relief_gauge_mpa: f(case, "relief_mpa_gauge"),
        fill: Some(f(case, "fill_fraction")),
        heat_leak: HeatLeak::Constant {
            kw: f(case, "heat_leak_kw"),
        },
        withdrawal: Withdrawal::Vapour,
        legs: &legs,
        step_s,
    })
    .unwrap()
}

/// The pressure (bar absolute) of the series at `t_h`, which is a step time.
fn pressure_bar_abs_at(out: &Outcome, t_h: f64) -> f64 {
    let p = out
        .points
        .iter()
        .find(|p| (p.time_h - t_h).abs() < 1.0e-6)
        .unwrap_or_else(|| panic!("no point at {t_h} h"));
    (p.pressure_gauge_mpa + 0.101325) * 10.0
}

/// The stored trajectory times before the relief setting is reached.
fn stored_times(case: &Value) -> Vec<(f64, f64)> {
    let ref_h = f(case, "holding_time_h");
    case["trajectory_h_bar_abs_c"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r[0].as_f64().unwrap(), r[1].as_f64().unwrap()))
        .filter(|(t, _)| *t < ref_h - 1.0e-6)
        .collect()
}

#[test]
fn the_step_error_is_measured_by_halving_the_step() {
    // Prints the figures TOLERANCE.md records. Run with `-- --nocapture`.
    let doc = cases();
    for c in doc["cases"].as_array().unwrap() {
        let (full, half) = (ovd(c, STEP_S), ovd(c, STEP_S / 2.0));
        let (hf, hh) = (full.holding_time_h.unwrap(), half.holding_time_h.unwrap());
        let step_p = stored_times(c)
            .iter()
            .map(|(t, _)| {
                let (a, b) = (
                    pressure_bar_abs_at(&full, *t),
                    pressure_bar_abs_at(&half, *t),
                );
                100.0 * (a - b).abs() / b
            })
            .fold(0.0, f64::max);
        eprintln!(
            "{}: holding-time step error {:.5} %, pressure step error {step_p:.5} %",
            c["id"],
            100.0 * (hh - hf).abs() / hf,
        );
        assert!(hf.is_finite() && hh.is_finite());
    }
}

#[test]
fn every_case_agrees_with_neqsim_on_holding_time_within_the_floor_and_the_pressure_is_reported() {
    let (t_hold, t_press) = tolerances();
    let doc = cases();
    for c in doc["cases"].as_array().unwrap() {
        let id = c["id"].as_str().unwrap();
        let out = ovd(c, STEP_S);
        let h = out.holding_time_h.unwrap();
        let ref_h = f(c, "holding_time_h");
        let d = 100.0 * (h - ref_h).abs() / ref_h;
        // Gated: the holding time against its floor.
        assert!(
            d <= t_hold,
            "{id}: holding time {h} h against NeqSim {ref_h} h is {d} % (floor {t_hold} %)"
        );
        // Reported, not gated: the pressure at every stored time before the relief setting.
        let worst = stored_times(c)
            .iter()
            .map(|(t, want)| 100.0 * (pressure_bar_abs_at(&out, *t) - want).abs() / want)
            .fold(0.0, f64::max);
        eprintln!(
            "{id}: holding time {d:.4} % (floor {t_hold} %), pressure {worst:.4} % (floor {t_press} %): pressure reported, not gated - see issue #TBD-pressure-gap"
        );
        assert!(worst.is_finite());
        assert_eq!(
            out.legs[0].capped_by_vapour,
            c["withdrawal_capped_by_vapour"].as_bool().unwrap(),
            "{id}"
        );
    }
}
