//! Seam: the gas fuel system allowances of a gas-fuelled stern block (issue #775, spec 0011 D8).
//!
//! Label: estimate. No source was read for any allowance (docs/research/stern-arrangement.md sec. 3
//! lists the mass of a gas valve unit, heater, compressor and gas combustion unit as not found), so
//! each expected value is a hand sum of the stated estimate rates:
//! - gas valve units: 0.4 t per gas engine;
//! - heater: 0.2 kg per kWe; compressor: 0.6 kg per kWe;
//! - gas combustion unit: 2 t, only when asked for.
//! For 2 sets of 1,000 kWe (2,000 kWe): GVU 0.8 t, heater 0.4 t, compressor 1.2 t, GCU 2.0 t.

use ovd_core::aft_arrangement::{AftArrangementInputs, SternMassInputs, compute, stern_block_mass};
use ovd_core::gas_fuel::{FUEL_SYSTEM_STATUS, fuel_system};

fn mass_inputs(gcu: bool) -> SternMassInputs {
    SternMassInputs {
        battery_unit_energy_kwh: 5000.0,
        installed_power_kw: None,
        gas_combustion_unit: gcu,
    }
}

#[test]
fn the_allowances_are_the_hand_sums_of_the_estimate_rates() {
    let f = fuel_system(2, 2000.0, false).unwrap();
    let t = |label: &str| f.items.iter().find(|i| i.label == label).unwrap().mass_t;
    assert!((t("gas valve units") - 0.8).abs() < 1e-12);
    assert!((t("gas heater") - 0.4).abs() < 1e-12);
    assert!((t("gas compressor") - 1.2).abs() < 1e-12);
    assert_eq!(f.items.len(), 3);
    assert!((f.total_t() - 2.4).abs() < 1e-12);
}

#[test]
fn the_gas_combustion_unit_is_an_input_that_adds_one_item() {
    let f = fuel_system(2, 2000.0, true).unwrap();
    assert_eq!(f.items.len(), 4);
    let g = f
        .items
        .iter()
        .find(|i| i.label == "gas combustion unit")
        .unwrap();
    assert_eq!(g.mass_t, 2.0);
    assert!((f.total_t() - 4.4).abs() < 1e-12);
}

#[test]
fn every_allowance_says_it_is_an_estimate() {
    assert_eq!(FUEL_SYSTEM_STATUS, "estimate");
    for i in &fuel_system(2, 2000.0, true).unwrap().items {
        assert!(i.basis.starts_with("estimate:"), "{}", i.label);
    }
}

#[test]
fn no_gas_sets_no_system() {
    assert!(fuel_system(0, 2000.0, true).is_none());
    assert!(fuel_system(2, 0.0, true).is_none());
}

fn gas_block() -> (
    AftArrangementInputs,
    ovd_core::aft_arrangement::AftArrangementResult,
) {
    let mut inp = AftArrangementInputs::new(11.4, 6.0);
    inp.propulsion = "gas_fuelled".into();
    inp.n_battery_units = 0;
    inp.electrical_load_kwe = Some(2000.0);
    let r = compute(&inp).unwrap();
    (inp, r)
}

#[test]
fn a_gas_block_shows_the_gas_system_as_its_own_mass_items() {
    let (inp, r) = gas_block();
    let items = stern_block_mass(&inp, &r, &mass_inputs(false), 1.0);
    for label in ["gas valve units", "gas heater", "gas compressor"] {
        assert_eq!(
            items.iter().filter(|i| i.label == label).count(),
            1,
            "{label}"
        );
    }
    assert!(items.iter().all(|i| i.label != "gas combustion unit"));
    let n = r.n_gensets as f64;
    let kwe = r.installed_kwe;
    let t = |label: &str| items.iter().find(|i| i.label == label).unwrap().mass_t;
    assert!((t("gas valve units") - 0.4 * n).abs() < 1e-12);
    assert!((t("gas heater") - 0.0002 * kwe).abs() < 1e-12);
    assert!((t("gas compressor") - 0.0006 * kwe).abs() < 1e-12);
    // At the GVU room.
    let g = items.iter().find(|i| i.label == "gas valve units").unwrap();
    assert!((g.lcg_m - (r.gvu_room_x0_m + r.gvu_room_x1_m) / 2.0).abs() < 1e-12);
    let with = stern_block_mass(&inp, &r, &mass_inputs(true), 1.0);
    assert_eq!(
        with.iter()
            .filter(|i| i.label == "gas combustion unit")
            .count(),
        1
    );
}

#[test]
fn a_battery_electric_block_has_no_gas_system() {
    let mut inp = AftArrangementInputs::new(11.4, 6.0);
    inp.propulsion = "battery_electric".into();
    let r = compute(&inp).unwrap();
    let items = stern_block_mass(&inp, &r, &mass_inputs(true), 1.0);
    assert!(items.iter().all(|i| !i.label.starts_with("gas ")));
}
