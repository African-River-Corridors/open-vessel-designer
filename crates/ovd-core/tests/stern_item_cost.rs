//! Seam: the item cost lines of a self-propelled hull's own stern (issue #773, spec 0011 D9 and
//! sec. 7): gensets per kWe, machinery steel per t, deckhouse per m2, gas fuel system per kW, beside
//! the propulsion line per kW.
//!
//! Hand sums (quantity x rate), at the example rates 500 per kWe, 2,500 per t, 3,000 per m2 and 200
//! per kW: 3,000 kWe -> 1,500,000; 18 t -> 45,000; 120 m2 -> 360,000; 1,500 kW -> 300,000. A
//! diesel-electric plant of 1,000 kW on two propulsors has three sets of 500 kWe (N + 1) = 1,500 kWe,
//! as its mass takes them (machinery.rs: 1,875 kVA = 1,500 / 0.8). The 140 m LNG block of
//! `gas_fuelled_stern.rs` has 3 x 1,050 kWe = 3,150 kWe.
//!
//! Label: sourced - hand calculations of quantity x rate and of the set counts (above); the
//! Weichai set rating as in `gas_fuelled_stern.rs`. The rest are properties.

use ovd_core::aft_arrangement::{AftArrangementInputs, compute as aft, installed_genset_kwe};
use ovd_core::barge_cost::{
    BargeCostInputs, ItemQuantities, ItemRates, compute_for, compute_with_items, example_rates,
    item_lines,
};
use ovd_core::machinery::{MachineryInputs, PowerSource, arrange};
use ovd_core::propulsor::PropulsorType;

const RATES: ItemRates = ItemRates {
    genset_rate_per_kwe: 500.0,
    machinery_steel_rate: 2500.0,
    deckhouse_rate_per_m2: 3000.0,
    gas_fuel_system_rate_per_kw: 200.0,
};

fn machinery(source: PowerSource) -> MachineryInputs {
    MachineryInputs {
        power_source: source,
        propulsor: PropulsorType::ShaftNozzle,
        propulsor_count: 2,
        propeller_diameter_m: 2.0,
        installed_power_kw: Some(1000.0),
        battery_energy_kwh: Some(2000.0),
        bow_thruster_kw: None,
        bow_thruster_lcg_m: 100.0,
        stern_run_m: 0.0,
    }
}

fn amount(rows: &[(&str, ovd_core::pyint::PyInt)], item: &str) -> f64 {
    rows.iter()
        .find(|(k, _)| *k == item)
        .map(|(_, v)| v.as_str().parse::<f64>().unwrap())
        .unwrap_or_else(|| panic!("{item} not in {rows:?}"))
}

#[test]
fn each_item_line_is_its_quantity_times_its_rate() {
    let q = ItemQuantities {
        genset_kwe: 3000.0,
        machinery_steel_t: 18.0,
        deckhouse_area_m2: 120.0,
        gas_fuel_system_kw: 1500.0,
    };
    assert_eq!(
        item_lines(&q, &RATES),
        [
            ("gensets", 1_500_000.0),
            ("machinery_steel", 45_000.0),
            ("deckhouse", 360_000.0),
            ("gas_fuel_system", 300_000.0),
        ]
    );
}

#[test]
fn the_installed_genset_rating_counts_every_set_and_the_reserve() {
    // Diesel-electric: N + 1 sets of P / N.
    let m = arrange(
        &machinery(PowerSource::DieselElectric),
        11.4,
        4.0,
        3.5,
        None,
    );
    assert_eq!(m.n_engines, 3);
    assert!((m.genset_kwe - 1500.0).abs() < 1e-9);
    // No gensets on a plant with engines on the shafts or none at all.
    for source in [
        PowerSource::DieselMechanical,
        PowerSource::BatteryHybrid,
        PowerSource::BatteryElectric,
    ] {
        let m = arrange(&machinery(source), 11.4, 4.0, 3.5, None);
        assert_eq!(m.genset_kwe, 0.0, "{source:?}");
    }
    // A gas-fuelled block: the 140 m LNG block of gas_fuelled_stern.rs, 3 x 1,050 kWe.
    let mut i = AftArrangementInputs::new(26.0, 7.0);
    i.propulsion = "gas_fuelled".to_string();
    i.n_battery_units = 0;
    i.electrical_load_kwe = Some(1910.0);
    let r = aft(&i).unwrap();
    assert!((installed_genset_kwe(&r) - 3150.0).abs() < 1e-9);
    // A battery-electric block has none.
    let r = aft(&AftArrangementInputs::new(11.4, 4.0)).unwrap();
    assert_eq!(installed_genset_kwe(&r), 0.0);
}

#[test]
fn the_split_moves_nothing_out_of_the_propulsion_line_and_adds_only_the_items() {
    let inp = BargeCostInputs::default();
    let rt = example_rates().unwrap();
    let before = compute_for(&inp, &rt, false).unwrap();
    // All quantities 0: the same budget, with four zero lines after the propulsion line.
    let zero = item_lines(&ItemQuantities::default(), &RATES);
    let same = compute_with_items(&inp, &rt, false, &zero).unwrap();
    assert_eq!(same.unit_total, before.unit_total);
    let names: Vec<&str> = same.barge_breakdown.iter().map(|(k, _)| *k).collect();
    let at = names.iter().position(|k| *k == "propulsion").unwrap();
    assert_eq!(
        &names[at..],
        [
            "propulsion",
            "gensets",
            "machinery_steel",
            "deckhouse",
            "gas_fuel_system",
            "wheelhouse_accommodation"
        ]
    );
    // With quantities: the propulsion line keeps its value, and the total rises by the lines and the
    // yard margin on them (each figure rounds alone, so within a few units).
    let q = ItemQuantities {
        genset_kwe: 3000.0,
        machinery_steel_t: 18.0,
        deckhouse_area_m2: 0.0,
        gas_fuel_system_kw: 0.0,
    };
    let lines = item_lines(&q, &RATES);
    let after = compute_with_items(&inp, &rt, false, &lines).unwrap();
    assert_eq!(
        amount(&after.barge_breakdown, "propulsion"),
        amount(&before.barge_breakdown, "propulsion")
    );
    let added: f64 = lines.iter().map(|(_, v)| v).sum();
    let rise = after.unit_total.as_str().parse::<f64>().unwrap()
        - before.unit_total.as_str().parse::<f64>().unwrap();
    let want = added * (1.0 + rt.yard_margin_fraction);
    assert!((rise - want).abs() <= 2.0, "{rise} vs {want}");
    assert!(rise > 0.0);
}

#[test]
fn a_formation_with_a_tug_has_no_item_lines() {
    // The ATB budget: its tug is a battery-electric pusher; `compute_for` passes no items, so the
    // Python-facing budget is unchanged.
    let inp = BargeCostInputs::default();
    let rt = example_rates().unwrap();
    let a = compute_for(&inp, &rt, true).unwrap();
    let b = compute_with_items(&inp, &rt, true, &[]).unwrap();
    assert_eq!(a, b);
    assert!(
        a.barge_breakdown
            .iter()
            .chain(a.tug_breakdown.iter())
            .all(
                |(k, _)| !["gensets", "machinery_steel", "deckhouse", "gas_fuel_system"]
                    .contains(k)
            )
    );
}
