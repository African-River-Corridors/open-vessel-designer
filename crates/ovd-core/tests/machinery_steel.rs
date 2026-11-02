//! Seam: the machinery steel allowances of a stern block (issue #774, spec 0011 D7, D19, sec. 6).
//!
//! Hand sums:
//! - Engine seats, Schneekluth and Bertram, "Ship Design for Efficiency and Economy", sec. 5.1,
//!   p. 158 (edition not shown on the pages read): W = 27 P_B / ((n + 250) (15 + P_B / 1000)) t per
//!   engine. At 1,000 kW and 1,000 rpm: 27,000 / (1,250 x 16) = 1.35 t. Two Cat 3512C at 1,903 kW and
//!   1,800 rpm (the speed an estimate): 2 x 51,381 / (2,050 x 16.903) = 2.96562 t.
//! - Engine-room block steel at 0.057 t/m3 (Table 5.2, p. 164, the layer I deckhouse coefficient at
//!   Fo/Fu = 1.0, taken for the engine room: an unverified estimate). Two Cat 3512C (length 2.590 m)
//!   on shafts: room 1.4 x (1.0 + 2.590 + 1.2 + 0.8) = 7.826 m; x 11.4 m beam x (4.0 - 1.0) m =
//!   267.6492 m3; x 0.057 = 15.2560044 t.
//! - Watson's diesel-electric total 0.72 x 2,400^0.78 = 311.815 t (secondary: lecture notes citing
//!   Watson 1998; docs/research/stern-arrangement.md sec. 3 gives ~ 312 t).
//!
//! Label: sourced - Schneekluth and Bertram, Ship Design for Efficiency and Economy, sec. 5.1 p. 158 and Table 5.2 p. 164 (edition not shown on the pages read); Watson 1998 as cited in docs/research/stern-arrangement.md sec. 3; the sums are hand calculations (above).

use ovd_core::aft_arrangement::{
    AftArrangementInputs, GAS_GENSETS, SternMassInputs, compute as aft, machinery_summary,
};
use ovd_core::barge_capacity::{BargeCapacityInputs, BargeCapacityResult, compute};
use ovd_core::machinery::{
    MachineryInputs, PowerSource, PrimeMovers, arrange, engine_seat_t, steel_items,
    watson_diesel_electric_t,
};
use ovd_core::propulsor::PropulsorType;
use ovd_core::weights::total_mass;

fn machinery(source: PowerSource, propulsor: PropulsorType, kw: f64) -> MachineryInputs {
    MachineryInputs {
        power_source: source,
        propulsor,
        propulsor_count: 2,
        propeller_diameter_m: 2.0,
        installed_power_kw: Some(kw),
        battery_energy_kwh: Some(2000.0),
        bow_thruster_kw: None,
        bow_thruster_lcg_m: 100.0,
        stern_run_m: 4.0,
    }
}

#[test]
fn the_seat_formula_gives_the_hand_value() {
    assert!((engine_seat_t(1000.0, 1000.0) - 1.35).abs() < 1e-12);
    assert_eq!(engine_seat_t(0.0, 1000.0), 0.0);
}

#[test]
fn two_cat_3512c_on_shafts_give_the_hand_sums() {
    let inp = machinery(
        PowerSource::DieselMechanical,
        PropulsorType::ShaftNozzle,
        3806.0,
    );
    let m = arrange(&inp, 11.4, 4.0, 3.5, None);
    let movers = ovd_core::machinery::engine_prime_movers(&inp, &m).unwrap();
    assert_eq!(movers.count, 2);
    assert!(!movers.rpm_sourced);
    let s = steel_items(&m, Some(movers), 11.4, 4.0, 1.0);
    assert!(
        (s.engine_room_volume_m3 - 267.6492).abs() < 1e-9,
        "{}",
        s.engine_room_volume_m3
    );
    assert!(
        (s.engine_room_steel_t - 15.2560044).abs() < 1e-9,
        "{}",
        s.engine_room_steel_t
    );
    assert!(
        (s.seats_t - 2.965_615_859_791_09).abs() < 1e-9,
        "{}",
        s.seats_t
    );
    assert!((total_mass(&s.items) - 15.2560044 - s.seats_t).abs() < 1e-9);
}

#[test]
fn watson_gives_the_research_note_value() {
    assert!((watson_diesel_electric_t(2400.0) - 311.815_187_881_737_9).abs() < 1e-9);
    assert_eq!(watson_diesel_electric_t(0.0), 0.0);
}

#[test]
fn seats_grow_with_power_and_fall_with_speed() {
    // Label: property.
    let mut last = 0.0;
    for kw in [100.0, 500.0, 1000.0, 2000.0, 4000.0] {
        let w = engine_seat_t(kw, 1000.0);
        assert!(w > last);
        assert!(engine_seat_t(kw, 1800.0) < w);
        last = w;
    }
    let mut last = 0.0;
    for kw in [100.0, 1000.0, 5000.0] {
        let w = watson_diesel_electric_t(kw);
        assert!(w > last);
        last = w;
    }
}

#[test]
fn the_block_steel_scales_with_the_engine_room_volume() {
    // Label: property.
    let inp = machinery(
        PowerSource::DieselMechanical,
        PropulsorType::ShaftNozzle,
        3806.0,
    );
    let m = arrange(&inp, 11.4, 4.0, 3.5, None);
    let narrow = steel_items(&m, None, 11.4, 4.0, 1.0);
    let wide = steel_items(&m, None, 22.8, 4.0, 1.0);
    assert!((wide.engine_room_steel_t - 2.0 * narrow.engine_room_steel_t).abs() < 1e-9);
    assert_eq!(narrow.seats_t, 0.0);
    // The items stand in the engine room.
    for i in &wide.items {
        assert!(i.lcg_m > m.engine_room_x0_m && i.lcg_m < m.engine_room_x1_m);
    }
}

#[test]
fn a_battery_electric_block_has_no_engine_room_so_no_machinery_steel() {
    // Label: property. Feasibility: no engine room, no items.
    let inp = machinery(
        PowerSource::BatteryElectric,
        PropulsorType::AzimuthDucted,
        1000.0,
    );
    let m = arrange(&inp, 11.4, 4.0, 3.5, None);
    assert!(ovd_core::machinery::engine_prime_movers(&inp, &m).is_none());
    let s = steel_items(&m, None, 11.4, 4.0, 1.0);
    assert!(s.items.is_empty());
    assert_eq!(s.engine_room_steel_t, 0.0);
}

#[test]
fn a_gas_block_seats_its_gensets_at_the_maker_speed() {
    // Label: property.
    let mut inp = AftArrangementInputs::new(11.4, 6.0);
    inp.propulsion = "gas_fuelled".into();
    inp.n_battery_units = 0;
    inp.electrical_load_kwe = Some(2000.0);
    inp.machinery = Some(machinery(
        PowerSource::GasFuelled,
        PropulsorType::AzimuthDucted,
        2000.0,
    ));
    let r = aft(&inp).unwrap();
    let s = machinery_summary(&inp, &r, 1.0).unwrap();
    let p: PrimeMovers = s.steel.prime_movers.unwrap();
    let row = GAS_GENSETS
        .iter()
        .find(|g| g.genset_model == r.genset_model)
        .unwrap();
    assert!(p.rpm_sourced);
    assert_eq!(p.rpm, row.rpm);
    assert_eq!(i64::from(p.count), r.n_gensets);
    let w = s.watson.unwrap();
    assert_eq!(w.mcr_kw, r.installed_kwe);
    assert!(w.ovd_machinery_t > 0.0);
    // The stern block mass carries the steel items, never Watson's total.
    let items = ovd_core::aft_arrangement::stern_block_mass(
        &inp,
        &r,
        &SternMassInputs {
            battery_unit_energy_kwh: 5000.0,
            installed_power_kw: None,
        },
        1.0,
    );
    for label in ["engine-room steel", "engine seats"] {
        assert_eq!(
            items.iter().filter(|i| i.label == label).count(),
            1,
            "{label}"
        );
    }
}

fn dry_hull(source: PowerSource, with_mass: bool) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 4.0);
    i.cargo_type = "bauxite".into();
    i.self_propelled = true;
    i.lightship_k_t_per_m3 = Some(0.1093);
    i.aft_design.n_battery_units = Some(0);
    i.aft_design.n_accom_units = Some(0);
    i.machinery = Some(machinery(source, PropulsorType::ShaftNozzle, 2400.0));
    if with_mass {
        i.stern_block_mass = Some(SternMassInputs {
            battery_unit_energy_kwh: 5000.0,
            installed_power_kw: None,
        });
    }
    i
}

#[test]
fn watson_is_reported_beside_the_machinery_and_never_added() {
    // Label: property.
    let with = compute(&dry_hull(PowerSource::DieselElectric, true)).unwrap();
    let without = compute(&dry_hull(PowerSource::DieselElectric, false)).unwrap();
    assert!(without.machinery_summary.is_none());
    let s = with.machinery_summary.as_ref().unwrap();
    let w = s.watson.unwrap();
    // N + 1 sets of 1,200 kW.
    assert!((w.mcr_kw - 3600.0).abs() < 1e-9);
    assert!((w.watson_total_t - watson_diesel_electric_t(3600.0)).abs() < 1e-9);
    let block = total_mass(&with.stern_block_items);
    let ls = |r: &BargeCapacityResult| r.lightship_t;
    assert!((ls(&with) - ls(&without) - block).abs() < 1e-6);
    let steel: f64 = with
        .stern_block_items
        .iter()
        .filter(|i| i.label == "engine-room steel" || i.label == "engine seats")
        .map(|i| i.mass_t)
        .sum();
    assert!((steel - s.steel.engine_room_steel_t - s.steel.seats_t).abs() < 1e-9);
    // A diesel-mechanical plant has no gensets: no Watson diesel-electric cross-check.
    let mech = compute(&dry_hull(PowerSource::DieselMechanical, true)).unwrap();
    assert!(mech.machinery_summary.unwrap().watson.is_none());
}
