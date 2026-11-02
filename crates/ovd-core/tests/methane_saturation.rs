//! Seam: saturated methane properties and the gauge / absolute pressure basis (issue #867, spec 0016
//! section 4).
//!
//! The table is the NIST Chemistry WebBook saturation properties of methane, kept in
//! `catalogue/methane_saturation/nist_methane.tsv`. Worked values are read from its rows, as printed:
//! the latent heat constants the
//! boil-off figures have used (517.09, 513.57, 511.77, 509.94, 508.09 kJ/kg at -165, -163, -162, -161,
//! -160 C) are the vapour less the liquid enthalpy of these rows to 0.01.
//!
//! Label: sourced - NIST Chemistry WebBook, saturation properties of methane (catalogue/methane_saturation/); atmosphere 0.101325 MPa.

use ovd_core::gas_fuel::methane_latent_heat_kj_per_kg;
use ovd_core::methane_saturation::*;

fn catalogue() -> Vec<Vec<f64>> {
    let path = format!(
        "{}/../../catalogue/methane_saturation/nist_methane.tsv",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .lines()
        .skip(1)
        .map(|l| l.split('\t').map(|x| x.parse().unwrap()).collect())
        .collect()
}

#[test]
fn the_table_is_the_nist_rows_of_the_catalogue() {
    let rows = catalogue();
    assert_eq!(rows.len(), METHANE.len());
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(r[0], TABLE_LOW_C + i as f64, "one row every 1 C");
        let m = METHANE[i];
        assert_eq!(&r[1..], &[m.0, m.1, m.2, m.3, m.4, m.5, m.6], "{} C", r[0]);
    }
    assert_eq!(rows.last().unwrap()[0], TABLE_HIGH_C);
}

#[test]
fn every_function_equals_the_table_at_its_rows() {
    for (i, r) in catalogue().iter().enumerate() {
        let t = TABLE_LOW_C + i as f64;
        assert_eq!(saturation_pressure_mpa(t), Ok(r[1] / 10.0));
        assert_eq!(liquid_density_kg_m3(t), Ok(r[2]));
        assert_eq!(vapour_density_kg_m3(t), Ok(r[3]));
        assert_eq!(liquid_internal_energy_kj_per_kg(t), Ok(r[4]));
        assert_eq!(vapour_internal_energy_kj_per_kg(t), Ok(r[5]));
        assert_eq!(liquid_enthalpy_kj_per_kg(t), Ok(r[6]));
        assert_eq!(vapour_enthalpy_kj_per_kg(t), Ok(r[7]));
    }
    // Printed NIST rows.
    assert_eq!(liquid_density_kg_m3(-170.0), Ok(434.51));
    assert_eq!(vapour_enthalpy_kj_per_kg(-170.0), Ok(495.99));
}

#[test]
fn between_rows_the_value_is_the_straight_line() {
    let (a, b) = (
        liquid_density_kg_m3(-150.0).unwrap(),
        liquid_density_kg_m3(-149.0).unwrap(),
    );
    assert!((liquid_density_kg_m3(-149.5).unwrap() - (a + b) / 2.0).abs() < 1e-12);
}

#[test]
fn outside_the_table_every_function_is_an_error() {
    for t in [-170.0001, -171.0, -99.9999, 0.0, f64::NAN, f64::INFINITY] {
        assert_eq!(saturation_pressure_mpa(t), Err(OutOfTable), "{t}");
        assert_eq!(liquid_density_kg_m3(t), Err(OutOfTable));
        assert_eq!(vapour_density_kg_m3(t), Err(OutOfTable));
        assert_eq!(liquid_internal_energy_kj_per_kg(t), Err(OutOfTable));
        assert_eq!(vapour_internal_energy_kj_per_kg(t), Err(OutOfTable));
        assert_eq!(liquid_enthalpy_kj_per_kg(t), Err(OutOfTable));
        assert_eq!(vapour_enthalpy_kj_per_kg(t), Err(OutOfTable));
        assert_eq!(latent_heat_kj_per_kg(t), Err(OutOfTable));
        assert_eq!(methane_latent_heat_kj_per_kg(t), Err(OutOfTable));
    }
    for p in [0.047, 0.0, -1.0, 2.7, f64::NAN] {
        assert_eq!(saturation_temperature_c(p), Err(OutOfTable), "{p}");
    }
    // The ends are in.
    assert!(saturation_pressure_mpa(TABLE_LOW_C).is_ok());
    assert!(saturation_pressure_mpa(TABLE_HIGH_C).is_ok());
}

#[test]
fn the_inverse_returns_the_temperature_of_the_pressure() {
    for t in [-170.0, -162.0, -140.5, -100.0] {
        let p = saturation_pressure_mpa(t).unwrap();
        assert!(
            (saturation_temperature_c(p).unwrap() - t).abs() < 1e-9,
            "{t}"
        );
    }
}

#[test]
fn gauge_and_absolute_round_trip_exactly() {
    assert_eq!(ATMOSPHERE_MPA, 0.101325);
    assert_eq!(Pressure::from_gauge_mpa(0.0).absolute_mpa(), 0.101325);
    // Every whole pascal from 0 to 3 MPa g in steps of 7 Pa (over 400,000 values).
    let mut pa = 0_i64;
    while pa <= 3_000_000 {
        let g = pa as f64 / 1.0e6;
        let p = Pressure::from_gauge_mpa(g);
        assert_eq!(p.gauge_mpa(), g, "{pa} Pa g");
        // gauge -> absolute -> gauge, and absolute -> gauge -> absolute.
        let q = Pressure::from_absolute_mpa(p.absolute_mpa());
        assert_eq!(q.gauge_mpa(), g, "{pa} Pa g via absolute");
        assert_eq!(q.absolute_mpa(), p.absolute_mpa());
        let a = Pressure::from_absolute_mpa((pa + 101_325) as f64 / 1.0e6);
        assert_eq!(
            Pressure::from_gauge_mpa(a.gauge_mpa()).absolute_mpa(),
            a.absolute_mpa()
        );
        pa += 7;
    }
}

#[test]
fn the_state_solve_returns_the_state_at_a_table_row() {
    // At -150 C (row values as printed), quality 0.3.
    let (rho_f, rho_g, u_f, u_g) = (405.02, 3.9877, 40.106, 468.95);
    let x = 0.3;
    let v = (1.0 - x) / rho_f + x / rho_g;
    let u = u_f + x * (u_g - u_f);
    let s = state_from_u_v(u, v).unwrap();
    assert!((s.temperature_c + 150.0).abs() < 1e-8, "{s:?}");
    assert!((s.quality - x).abs() < 1e-8, "{s:?}");
    // Saturated liquid at the cold end, saturated vapour at the warm end.
    let s = state_from_u_v(-29.594, 1.0 / 434.51).unwrap();
    assert!(
        (s.temperature_c + 170.0).abs() < 1e-8 && s.quality.abs() < 1e-8,
        "{s:?}"
    );
    let s = state_from_u_v(
        vapour_internal_energy_kj_per_kg(-100.0).unwrap(),
        1.0 / vapour_density_kg_m3(-100.0).unwrap(),
    )
    .unwrap();
    assert!(
        (s.temperature_c + 100.0).abs() < 1e-8 && (s.quality - 1.0).abs() < 1e-8,
        "{s:?}"
    );
}

#[test]
fn the_state_solve_between_rows_by_hand() {
    // -149.5 C, midway between the -150 and -149 rows: each property the mean of the two rows.
    let rho_f = (405.02 + 403.45) / 2.0; // 404.235
    let rho_g = (3.9877 + 4.2416) / 2.0; // 4.11465
    let u_f = (40.106 + 43.669) / 2.0; // 41.8875
    let u_g = (468.95 + 470.07) / 2.0; // 469.51
    let x = 0.25;
    let v = 1.0 / rho_f + x * (1.0 / rho_g - 1.0 / rho_f);
    let u = u_f + x * (u_g - u_f);
    let s = state_from_u_v(u, v).unwrap();
    assert!((s.temperature_c + 149.5).abs() < 1e-8, "{s:?}");
    assert!((s.quality - x).abs() < 1e-8, "{s:?}");
}

#[test]
fn the_state_solve_refuses_states_outside_the_table_and_the_dome() {
    // Superheated vapour (volume above the saturated vapour at every temperature).
    assert_eq!(state_from_u_v(470.0, 10.0), Err(OutOfTable));
    // Compressed liquid (volume below the saturated liquid at every temperature).
    assert_eq!(state_from_u_v(0.0, 0.001), Err(OutOfTable));
    // An energy below the table at this volume, and one above it.
    assert_eq!(state_from_u_v(-100.0, 0.1), Err(OutOfTable));
    assert_eq!(state_from_u_v(1000.0, 0.1), Err(OutOfTable));
    assert_eq!(state_from_u_v(f64::NAN, 0.1), Err(OutOfTable));
    assert_eq!(state_from_u_v(0.0, 0.0), Err(OutOfTable));
}

#[test]
fn the_latent_heat_reproduces_the_constants_the_boil_off_has_used() {
    for (t, v) in [
        (-165.0, 517.09),
        (-163.0, 513.57),
        (-162.0, 511.77),
        (-161.0, 509.94),
        (-160.0, 508.09),
    ] {
        assert_eq!(latent_heat_kj_per_kg(t), Ok(v), "{t} C");
        assert_eq!(methane_latent_heat_kj_per_kg(t), Ok(v), "{t} C");
    }
}
