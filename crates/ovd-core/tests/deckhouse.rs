//! Seam: the Arrangement Style and the deckhouse of a stern block (issue #779, spec 0011 D3, D4,
//! D6, sec. 2 and 4.2).
//!
//! Hand sums, on a 20.0 m beam, 6.0 m depth, no machinery (the thruster room, 3.50 m, is the
//! machinery run), six 40 ft accommodation units in the container block:
//! - Deckhouse width: 20.0 - 2 x 0.8 side deck = 18.4 m; one tier 3.50 x 18.4 = 64.4 m2.
//! - Accommodation floor held: 6 x 12.192 x 2.438 = 178.344576 m2; 178.344576 / 64.4 = 2.77 -> 3 tiers.
//! - Tier height 2.6 m (an estimate), so the top is 6.0 + 3 x 2.6 = 13.8 m; the wheelhouse floor
//!   stands there and its top at 13.8 + 3.4 = 17.2 m.
//! - Volume 3 x 64.4 x 2.6 = 502.32 m3; steel at 0.057 t/m3 (Schneekluth and Bertram, Table 5.2,
//!   p. 164, layer I, an unverified estimate) = 28.63224 t. The wheelhouse (3.6 m long) stands at
//!   the deckhouse front and cannot start aft of the stern, so here it is 3.5 m long: steel
//!   3.5 x 6.4 x 3.4 = 76.16 m3 x 0.057 = 4.34112 t.
//!
//! Label: sourced - the tier height, side deck and tier limit are estimates (no source read); the
//! steel rate is Schneekluth and Bertram, Ship Design for Efficiency and Economy, sec. 5, Table 5.2,
//! p. 164 (edition not shown on the pages read); the sums above are hand calculations.

use ovd_core::aft_arrangement::{
    AftArrangementInputs, AftArrangementResult, ArrangementStyle, SternMassInputs, compute,
    stern_block_mass,
};

fn deckhouse_inputs() -> AftArrangementInputs {
    let mut i = AftArrangementInputs::new(20.0, 6.0);
    i.arrangement_style = ArrangementStyle::Deckhouse;
    i.n_battery_units = 0;
    i.n_accom_units = 6;
    i.wheelhouse_max_lift_m = 0.0;
    i
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn the_style_keys_round_trip_and_the_default_is_the_container_block() {
    assert_eq!(
        ArrangementStyle::default(),
        ArrangementStyle::ContainerBlock
    );
    for style in [
        ArrangementStyle::ContainerBlock,
        ArrangementStyle::Deckhouse,
    ] {
        assert_eq!(ArrangementStyle::from_key(style.key()), Some(style));
    }
    assert_eq!(ArrangementStyle::from_key("tiers"), None);
    assert_eq!(
        AftArrangementInputs::new(14.8, 5.4).arrangement_style,
        ArrangementStyle::ContainerBlock
    );
}

#[test]
fn the_container_block_has_no_deckhouse_and_is_what_it_was() {
    // The style left at its default gives today's block: the golden values of the block tests.
    let r = compute(&AftArrangementInputs::new(14.8, 5.4)).unwrap();
    assert!(r.deckhouse.is_none());
    assert_eq!(r.length_m, 19.892000000000003);
    assert_eq!(r.block_top_lowered_m, 8.8);
    assert_eq!(r.container_labels.len(), 3);
}

#[test]
fn a_deckhouse_has_the_tiers_the_accommodation_needs() {
    let r = compute(&deckhouse_inputs()).unwrap();
    let d = r.deckhouse.as_ref().expect("a deckhouse");
    assert!(close(d.width_m, 18.4), "{}", d.width_m);
    assert!(close(d.tier_area_m2, 64.4), "{}", d.tier_area_m2);
    assert!(
        close(d.required_area_m2, 178.344_576),
        "{}",
        d.required_area_m2
    );
    assert_eq!(d.tiers, 3);
    assert!(close(d.top_m, 13.8), "{}", d.top_m);
    assert!(close(d.area_m2, 193.2), "{}", d.area_m2);
    assert!(close(d.volume_m3, 502.32), "{}", d.volume_m3);
    assert!(close(d.x_fwd_m, 3.5) && d.x_aft_m == 0.0);
    assert!(!d.wheelhouse_elevating);
}

#[test]
fn the_deckhouse_holds_the_accommodation_so_no_accommodation_unit_stands_on_deck() {
    // D6: aft accommodation only; the units are in the house.
    let r = compute(&deckhouse_inputs()).unwrap();
    assert!(r.container_units.is_empty());
    assert!(r.container_labels.is_empty());
    // The container block of the same hull keeps its six units.
    let mut block = deckhouse_inputs();
    block.arrangement_style = ArrangementStyle::ContainerBlock;
    assert_eq!(compute(&block).unwrap().container_labels.len(), 6);
}

#[test]
fn battery_units_still_stand_on_deck_forward_of_the_deckhouse() {
    let mut i = deckhouse_inputs();
    i.n_battery_units = 2;
    let r = compute(&i).unwrap();
    assert_eq!(r.container_labels, vec!["BATTERY 40 FT"; 2]);
    let d = r.deckhouse.as_ref().unwrap();
    for (_, x_aft, _, _) in &r.container_units {
        assert!(*x_aft >= d.x_fwd_m, "{x_aft} is aft of the deckhouse front");
    }
    assert!(r.length_m >= r.container_x1_m);
}

#[test]
fn the_wheelhouse_stands_on_the_top_tier_at_the_deckhouse_front() {
    let r = compute(&deckhouse_inputs()).unwrap();
    let d = r.deckhouse.as_ref().unwrap();
    assert!(close(r.wheelhouse_floor_lowered_m, 13.8));
    assert!(close(r.wheelhouse_top_lowered_m, 17.2));
    assert!(close(r.wheelhouse_x1_m, d.x_fwd_m));
    // The block's tallest point is the wheelhouse top: no battery unit stands on this hull.
    assert!(
        close(r.block_top_lowered_m, 17.2),
        "{}",
        r.block_top_lowered_m
    );
    // A fixed wheelhouse does not lift, even with a sightline to meet (D4): the house grows.
    let mut i = deckhouse_inputs();
    i.sightline_obstruction_m = Some(30.0);
    let r = compute(&i).unwrap();
    assert_eq!(r.wheelhouse_lift_m, 0.0);
    assert!(r.deckhouse.as_ref().unwrap().tiers > 3);
}

#[test]
fn an_elevating_wheelhouse_lifts_from_the_top_tier() {
    // D4: with an air-draft limit the wheelhouse elevates. Eye 13.8 + 1.5 = 15.3 m; the sightline
    // needs 15.0 + 0.5 = 15.5 m: a lift of 0.2 m.
    let mut i = deckhouse_inputs();
    i.wheelhouse_max_lift_m = 6.0;
    i.sightline_obstruction_m = Some(15.0);
    let r = compute(&i).unwrap();
    assert!(r.deckhouse.as_ref().unwrap().wheelhouse_elevating);
    assert!(close(r.wheelhouse_lift_m, 0.2), "{}", r.wheelhouse_lift_m);
    assert!(close(r.wheelhouse_floor_raised_m, 14.0));
}

#[test]
fn a_taller_house_is_a_warning_not_an_error() {
    // 30 units over a short machinery space: more than four tiers.
    let mut i = deckhouse_inputs();
    i.n_accom_units = 30;
    let r = compute(&i).unwrap();
    assert!(r.deckhouse.as_ref().unwrap().tiers > 4);
    let c = r
        .checks
        .iter()
        .find(|c| c.name == "deckhouse_tier_count")
        .unwrap();
    assert!(!c.ok);
    assert_eq!(c.severity, "warning");
}

fn mass_items(r: &AftArrangementResult, i: &AftArrangementInputs) -> Vec<(String, f64)> {
    let m = SternMassInputs {
        battery_unit_energy_kwh: 5000.0,
        installed_power_kw: None,
        gas_combustion_unit: false,
    };
    stern_block_mass(i, r, &m, 1.0)
        .iter()
        .map(|it| (it.label.to_string(), it.mass_t))
        .collect()
}

#[test]
fn the_deckhouse_steel_is_volume_times_the_textbook_rate() {
    let i = deckhouse_inputs();
    let r = compute(&i).unwrap();
    let items = mass_items(&r, &i);
    let get = |n: &str| items.iter().find(|(l, _)| l == n).map(|(_, m)| *m);
    assert!(
        close(get("deckhouse steel").unwrap(), 28.632_24),
        "{items:?}"
    );
    assert!(
        close(get("wheelhouse steel").unwrap(), 4.341_12),
        "{items:?}"
    );
    // The one-ship 8 t wheelhouse, the accommodation rate and a fixed wheelhouse's column stay out.
    assert!(get("wheelhouse").is_none());
    assert!(get("accommodation unit").is_none());
    assert!(get("lift column").is_none());
}

#[test]
fn an_elevating_wheelhouse_keeps_its_lift_column() {
    let mut i = deckhouse_inputs();
    i.wheelhouse_max_lift_m = 6.0;
    let r = compute(&i).unwrap();
    let items = mass_items(&r, &i);
    assert!(
        items
            .iter()
            .any(|(l, m)| l == "lift column" && close(*m, 5.0))
    );
}

#[test]
fn the_container_block_keeps_its_wheelhouse_and_accommodation_mass_items() {
    let mut i = deckhouse_inputs();
    i.arrangement_style = ArrangementStyle::ContainerBlock;
    let r = compute(&i).unwrap();
    let items = mass_items(&r, &i);
    assert!(
        items
            .iter()
            .any(|(l, m)| l == "wheelhouse" && close(*m, 8.0))
    );
    assert!(items.iter().any(|(l, _)| l == "accommodation unit"));
    assert!(!items.iter().any(|(l, _)| l == "deckhouse steel"));
}

#[test]
fn more_accommodation_never_means_fewer_tiers_or_less_steel() {
    // Label: property.
    let (mut last_tiers, mut last_volume) = (0, 0.0);
    for n in [0, 1, 2, 4, 6, 8, 12, 20] {
        let mut i = deckhouse_inputs();
        i.n_accom_units = n;
        let d = compute(&i).unwrap().deckhouse.unwrap();
        assert!(d.tiers >= last_tiers && d.volume_m3 >= last_volume, "{n}");
        assert!(d.tiers >= 1, "a wheelhouse needs a house to stand on");
        (last_tiers, last_volume) = (d.tiers, d.volume_m3);
    }
}

#[test]
fn a_fixed_wheelhouse_gets_tiers_until_its_eye_clears_the_cargo() {
    // Eye 1.5 m above the floor, margin 0.5 m, cargo top 20 m: the floor must reach 19.0 m, which is
    // 13.0 m over the 6.0 m deck, so ceil(13.0 / 2.6) = 5 tiers (top 19.0 m).
    let mut i = deckhouse_inputs();
    i.sightline_obstruction_m = Some(20.0);
    let r = compute(&i).unwrap();
    assert_eq!(r.deckhouse.as_ref().unwrap().tiers, 5);
    let eye = r.wheelhouse_floor_raised_m + 1.5;
    assert!(eye >= 20.5 - 1e-9, "{eye}");
    assert!(
        r.checks
            .iter()
            .find(|c| c.name == "wheelhouse_sees_over_cargo")
            .unwrap()
            .ok
    );
    // An elevating wheelhouse lifts instead of adding tiers.
    i.wheelhouse_max_lift_m = 12.0;
    assert_eq!(compute(&i).unwrap().deckhouse.unwrap().tiers, 3);
}

#[test]
fn the_second_pass_on_the_light_waterline_never_adds_a_tier_and_keeps_the_sightline() {
    // Issue #779: the first pass sizes a fixed wheelhouse for a waterline at the keel. `compute`
    // then sizes it again on the trimmed light waterline of the first pass, and keeps the result
    // only with fewer tiers and a passing sightline.
    use ovd_core::barge_capacity::{BargeCapacityInputs, compute as capacity};
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
    i.cargo_type = "LPG_propane".to_string();
    i.self_propelled = true;
    i.sightline_limit_m = Some(250.0);
    i.aft_design.arrangement_style = ArrangementStyle::Deckhouse;
    i.aft_design.wheelhouse_max_lift_m = 0.0;
    let tiers = |r: &ovd_core::barge_capacity::BargeCapacityResult| {
        r.aft_arrangement
            .as_ref()
            .unwrap()
            .deckhouse
            .as_ref()
            .unwrap()
            .tiers
    };
    let two_pass = capacity(&i).unwrap();
    // The keel pass alone: a hint of a waterline at the keel is the first pass's own assumption.
    let mut keel = i.clone();
    keel.light_waterline_hint = Some((0.0, 0.0));
    let keel = capacity(&keel).unwrap();
    assert!(
        tiers(&two_pass) <= tiers(&keel),
        "{} > {}",
        tiers(&two_pass),
        tiers(&keel)
    );
    assert!(two_pass.sightline.as_ref().unwrap().enough);
}

#[test]
fn a_tighter_blind_distance_limit_adds_a_tier_to_the_fixed_wheelhouse() {
    // Issue #989: the limit is an input, read from the Rule Sets of the stack (ES-TRIN 250 m, and under
    // 125 m SOLAS V/22.1.1: 2 L). The wheelhouse is sized to the one it is given, the least that binds.
    // On a 60 m hull the 250 m limit leaves a light Blind Distance of 234.7 m; 2 L = 120 m adds a tier.
    use ovd_core::barge_capacity::{BargeCapacityInputs, compute as capacity};
    let run = |limit: f64| {
        let mut i = BargeCapacityInputs::new(60.0, 11.4, 6.0);
        i.cargo_type = "LPG_propane".to_string();
        i.self_propelled = true;
        i.sightline_limit_m = Some(limit);
        i.aft_design.arrangement_style = ArrangementStyle::Deckhouse;
        i.aft_design.wheelhouse_max_lift_m = 0.0;
        let r = capacity(&i).unwrap();
        let tiers = r
            .aft_arrangement
            .as_ref()
            .unwrap()
            .deckhouse
            .as_ref()
            .unwrap()
            .tiers;
        (tiers, r.sightline.as_ref().unwrap().blind_distance_m)
    };
    let (loose_tiers, loose_blind) = run(250.0);
    let (tight_tiers, tight_blind) = run(120.0);
    assert_eq!((loose_tiers, tight_tiers), (3, 4));
    assert!(
        loose_blind > 120.0 && tight_blind <= 120.0 + 1e-9,
        "{loose_blind}, {tight_blind}"
    );
}
