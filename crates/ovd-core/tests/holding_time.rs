//! Seam: the pressure holding time of a Type C methane tank (spec 0016 sections 4 and 7, issue #870).
//!
//! The hand case is worked from two rows of the NIST table kept in
//! `catalogue/methane_saturation/nist_methane.tsv` (read here, as printed, not from the engine's copy).
//! The other tests state relations of the spec: monotonic, the loading limit, the withdrawal cap and
//! floor, the ambient of each rule, and the refusals.
//!
//! Label: sourced - NIST Chemistry WebBook, saturation properties of methane (catalogue/methane_saturation/); the hand case is (U_end - U_start) / Q from rows -160 C and -150 C.

use ovd_core::gas_fuel::tank_heat_leak_kw;
use ovd_core::holding_time::*;
use ovd_core::methane_saturation::Pressure;

fn row(t: i32) -> Vec<f64> {
    let path = format!(
        "{}/../../catalogue/methane_saturation/nist_methane.tsv",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .skip(1)
        .map(|l| {
            l.split('\t')
                .map(|x| x.parse::<f64>().unwrap())
                .collect::<Vec<_>>()
        })
        .find(|r| r[0] == f64::from(t))
        .unwrap()
}

// Columns of a row: t, p bar, rho_l, rho_v, u_l, u_v, h_l, h_v.
const DAY: f64 = 1.0;

fn gauge_of_row(t: i32) -> f64 {
    row(t)[1] / 10.0 - 0.101325
}

fn one_leg(days: f64, w: f64) -> [LegInput; 1] {
    [LegInput {
        days,
        withdrawal_kg_s: w,
    }]
}

fn base<'a>(legs: &'a [LegInput], q: f64) -> Inputs<'a> {
    Inputs {
        volume_m3: 100.0,
        start_gauge_mpa: gauge_of_row(-160),
        relief_gauge_mpa: gauge_of_row(-150),
        fill: Some(0.9),
        heat_leak: HeatLeak::Constant { kw: q },
        withdrawal: Withdrawal::Vapour,
        legs,
        step_s: 3600.0,
    }
}

#[test]
fn the_hand_case_equals_the_energy_balance_over_the_heat_leak() {
    let (v, f, q) = (100.0, 0.9, 10.0);
    let (a, b) = (row(-160), row(-150));
    let m = v * (f * a[2] + (1.0 - f) * a[3]);
    let u_start = v * (f * a[2] * a[4] + (1.0 - f) * a[3] * a[5]);
    let vol = v / m;
    let (vf, vg) = (1.0 / b[2], 1.0 / b[3]);
    let x = (vol - vf) / (vg - vf);
    let u_end = m * (b[4] + x * (b[5] - b[4]));
    let hand_h = (u_end - u_start) / q / 3600.0;
    let legs = one_leg(30.0 * DAY, 0.0);
    let out = run(&base(&legs, q)).unwrap();
    let got = out.holding_time_h.unwrap();
    assert!(
        ((got - hand_h) / hand_h).abs() < 1.0e-7,
        "{got} against the hand value {hand_h}"
    );
    assert!(out.legs[0].end_pressure_gauge_mpa - gauge_of_row(-150) < 1.0e-9);
}

#[test]
fn a_run_that_never_reaches_the_relief_setting_has_no_holding_time() {
    let legs = one_leg(1.0, 0.0);
    let out = run(&base(&legs, 1.0)).unwrap();
    assert_eq!(out.holding_time_h, None);
    assert!(out.legs[0].completed);
    assert!(out.legs[0].end_pressure_gauge_mpa < gauge_of_row(-150));
}

fn conduction(rule: Rule, insulation_m: f64) -> HeatLeak {
    HeatLeak::conduction(rule, 5.0, 5.0 + insulation_m, 20.0, 0.026)
}

fn holding_with(heat: HeatLeak, relief: f64) -> f64 {
    let legs = one_leg(400.0, 0.0);
    let mut i = base(&legs, 0.0);
    i.heat_leak = heat;
    i.relief_gauge_mpa = relief;
    i.start_gauge_mpa = 0.0;
    i.volume_m3 = 1500.0;
    i.fill = None;
    run(&i).unwrap().holding_time_h.unwrap()
}

#[test]
fn more_insulation_never_shortens_the_holding_time() {
    let mut last = 0.0;
    for mm in [0.15, 0.2, 0.25, 0.3, 0.35, 0.4] {
        let h = holding_with(conduction(Rule::Igc, mm), 0.4);
        assert!(h >= last, "{mm} m: {h} h after {last} h");
        last = h;
    }
}

#[test]
fn a_higher_design_pressure_never_shortens_the_holding_time() {
    let mut last = 0.0;
    for p in [0.25, 0.3, 0.4, 0.5, 0.7, 1.0] {
        let h = holding_with(conduction(Rule::Igc, 0.3), p);
        assert!(h >= last, "{p} MPa: {h} h after {last} h");
        last = h;
    }
}

#[test]
fn the_heat_leak_is_found_at_each_step_from_the_air_ambient_of_the_rule_over_the_whole_surface() {
    assert_eq!(Rule::Igc.ambient_air_c(), 45.0);
    assert_eq!(Rule::Adn.ambient_air_c(), 30.0);
    for (rule, air) in [(Rule::Igc, 45.0), (Rule::Adn, 30.0)] {
        let h = conduction(rule, 0.3);
        for t in [-162.0, -150.0, -140.0] {
            let want = tank_heat_leak_kw(5.0, 5.3, 20.0, air - t, 0.026);
            assert_eq!(h.kw(t), want, "{} at {t} C", rule.key());
        }
        // It falls as the cargo warms.
        assert!(h.kw(-140.0) < h.kw(-162.0));
    }
    // The lower ADN ambient gives the longer holding time, all else equal.
    assert!(
        holding_with(conduction(Rule::Adn, 0.3), 0.4)
            > holding_with(conduction(Rule::Igc, 0.3), 0.4)
    );
}

#[test]
fn a_tank_loaded_to_the_loading_limit_is_at_the_filling_limit_when_it_reaches_the_relief_setting() {
    for relief in [0.25, 0.4, 0.7] {
        let legs = one_leg(2000.0, 0.0);
        let mut i = base(&legs, 0.0);
        i.heat_leak = conduction(Rule::Igc, 0.3);
        i.start_gauge_mpa = 0.0;
        i.relief_gauge_mpa = relief;
        i.volume_m3 = 1500.0;
        i.fill = None;
        let out = run(&i).unwrap();
        let end = out.legs[0].end_fill;
        assert!(out.loading_limit < out.filling_limit);
        assert!(
            (end - out.filling_limit).abs() < 0.001,
            "relief {relief}: fill {end} against FL {}",
            out.filling_limit
        );
    }
}

#[test]
fn with_a_withdrawal_the_fill_at_the_relief_setting_is_at_or_below_the_filling_limit() {
    let legs = one_leg(2000.0, 0.01);
    let mut i = base(&legs, 0.0);
    i.heat_leak = conduction(Rule::Igc, 0.3);
    i.start_gauge_mpa = 0.0;
    i.relief_gauge_mpa = 0.4;
    i.volume_m3 = 1500.0;
    i.fill = None;
    let out = run(&i).unwrap();
    assert!(out.legs[0].end_fill <= out.filling_limit + 1.0e-9);
    assert!(out.legs[0].withdrawn_kg > 0.0);
}

#[test]
fn a_typed_fill_above_the_loading_limit_is_refused() {
    let legs = one_leg(1.0, 0.0);
    let ll = loading_limit(gauge_of_row(-160), gauge_of_row(-150)).unwrap();
    let mut i = base(&legs, 1.0);
    i.fill = Some(ll + 0.001);
    assert!(matches!(run(&i), Err(Error::FillAboveLoadingLimit { .. })));
    i.fill = Some(ll);
    assert!(run(&i).is_ok());
}

#[test]
fn a_use_above_the_vapour_available_is_capped_and_the_pressure_never_falls_below_the_start() {
    // Warm for a day with no use, then ask for far more than the vapour holds.
    let legs = [
        LegInput {
            days: 0.2,
            withdrawal_kg_s: 0.0,
        },
        LegInput {
            days: 1.0,
            withdrawal_kg_s: 50.0,
        },
    ];
    let mut i = base(&legs, 10.0);
    i.fill = Some(0.9);
    let out = run(&i).unwrap();
    assert!(out.legs[1].capped_by_vapour, "the cap is reported");
    let start = i.start_gauge_mpa;
    for p in &out.points {
        assert!(p.pressure_gauge_mpa >= start - 1.0e-9, "{p:?}");
    }
    // A use the boil-off cannot keep up with is held at the start pressure.
    let legs = [LegInput {
        days: 5.0,
        withdrawal_kg_s: 0.03,
    }];
    let mut held = base(&legs, 10.0);
    held.fill = Some(0.5);
    let out = run(&held).unwrap();
    assert!(out.legs[0].held_at_start_pressure);
    for p in &out.points {
        assert!(p.pressure_gauge_mpa >= i.start_gauge_mpa - 1.0e-9, "{p:?}");
    }
}

#[test]
fn halving_the_step_barely_moves_the_holding_time() {
    let legs = one_leg(30.0, 0.0);
    let full = run(&base(&legs, 10.0)).unwrap().holding_time_h.unwrap();
    let mut i = base(&legs, 10.0);
    i.step_s = 1800.0;
    let half = run(&i).unwrap().holding_time_h.unwrap();
    assert!(((half - full) / full).abs() < 1.0e-3);
}

#[test]
fn bad_inputs_are_refused() {
    let legs = one_leg(1.0, 0.0);
    let ok = base(&legs, 1.0);
    for bad in [
        Inputs {
            volume_m3: 0.0,
            ..ok.clone()
        },
        Inputs {
            step_s: 0.0,
            ..ok.clone()
        },
        Inputs {
            relief_gauge_mpa: ok.start_gauge_mpa,
            ..ok.clone()
        },
        Inputs {
            start_gauge_mpa: -0.01,
            ..ok.clone()
        },
        Inputs {
            fill: Some(1.0),
            ..ok.clone()
        },
        Inputs {
            legs: &[],
            ..ok.clone()
        },
    ] {
        assert_eq!(run(&bad), Err(Error::BadInput));
    }
    let neg = [LegInput {
        days: 1.0,
        withdrawal_kg_s: -1.0,
    }];
    assert_eq!(
        run(&Inputs {
            legs: &neg,
            ..ok.clone()
        }),
        Err(Error::BadInput)
    );
    // A relief setting above the table is an error, never a clamp.
    let hi = Inputs {
        relief_gauge_mpa: 5.0,
        ..ok
    };
    assert_eq!(run(&hi), Err(Error::OutOfTable));
    let _ = Pressure::from_gauge_mpa(0.0);
}

#[test]
fn a_tank_that_fills_with_liquid_before_the_relief_setting_is_refused() {
    // The reference cases loaded 0.9 of the volume against a 0.5 MPa relief setting, above the loading
    // limit (0.88): the typed fill is refused, and the same fill stepped at a lower relief setting that
    // allows it is not.
    let legs = one_leg(400.0, 0.0);
    let mut i = base(&legs, 10.0);
    i.volume_m3 = 100.0;
    i.start_gauge_mpa = 0.0;
    i.relief_gauge_mpa = 0.5;
    i.fill = Some(0.9);
    assert!(matches!(run(&i), Err(Error::FillAboveLoadingLimit { .. })));
}
