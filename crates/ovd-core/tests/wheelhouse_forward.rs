//! Seam: the `wheelhouse_forward` Arrangement Style (issue #994): the machinery and the main
//! accommodation stay aft in a deckhouse; the wheelhouse stands in a house over the forepeak.
//!
//! Label: property. Each test states a rule of the layout; no number is pinned except sums worked in
//! the test from the sizes it reads (the house sizes are estimates, no source read).

use ovd_core::aft_arrangement::{
    AftArrangementInputs, ArrangementStyle, ForwardHouseSite, SternMassInputs, compute,
    forward_house_sizes, stern_block_mass,
};

fn forward_inputs() -> AftArrangementInputs {
    let mut i = AftArrangementInputs::new(20.0, 6.0);
    i.arrangement_style = ArrangementStyle::WheelhouseForward;
    i.n_battery_units = 0;
    i.n_accom_units = 6;
    i.wheelhouse_max_lift_m = 0.0;
    i.forward_house_site = Some(ForwardHouseSite {
        x_aft_m: 85.0,
        x_fwd_m: 93.0,
        cargo_length_lost_m: 0.0,
    });
    i
}

#[test]
fn the_key_names_the_style_and_the_list_holds_it() {
    assert_eq!(
        ArrangementStyle::from_key("wheelhouse_forward"),
        Some(ArrangementStyle::WheelhouseForward)
    );
    assert_eq!(ArrangementStyle::WheelhouseForward.key(), "wheelhouse_forward");
    assert!(ovd_core::aft_arrangement::ARRANGEMENT_STYLES.contains(&"wheelhouse_forward"));
    // Like the deckhouse, it holds its accommodation in a house aft; unlike it, the wheelhouse is forward.
    assert!(ArrangementStyle::WheelhouseForward.has_aft_deckhouse());
    assert!(ArrangementStyle::WheelhouseForward.is_wheelhouse_forward());
    assert!(!ArrangementStyle::Deckhouse.is_wheelhouse_forward());
}

#[test]
fn the_machinery_and_the_accommodation_stay_aft_and_the_wheelhouse_goes_forward() {
    let r = compute(&forward_inputs()).unwrap();
    let aft = r.deckhouse.as_ref().expect("the accommodation house stays aft");
    let fwd = r.forward_house.as_ref().expect("a forward house");
    // The block ends where the deckhouse ends: the wheelhouse no longer adds to it.
    assert!(r.length_m <= aft.x_fwd_m + 1e-9, "{} vs {}", r.length_m, aft.x_fwd_m);
    assert!(aft.x_aft_m == 0.0 && aft.tiers >= 1);
    // The wheelhouse stands on the forward house, at its forward end, and clear of the block.
    assert!(r.wheelhouse_x1_m == fwd.x_fwd_m);
    assert!(r.wheelhouse_x0_m >= fwd.x_aft_m && r.wheelhouse_x0_m < r.wheelhouse_x1_m);
    assert!(r.wheelhouse_x0_m > r.length_m);
    // The wheelhouse floor is the top of the forward house, not of the aft deckhouse.
    assert!((r.wheelhouse_floor_lowered_m - fwd.top_m).abs() < 1e-9);
    assert_eq!(fwd.tiers, forward_house_sizes::TIERS);
    assert!(fwd.base_m == 6.0 && fwd.top_m > fwd.base_m);
    // Fixed: no lift, and no check that the eye clears the cargo (the cargo is behind the eye).
    assert_eq!(r.wheelhouse_lift_m, 0.0);
    assert!(r.checks.iter().all(|c| c.name != "wheelhouse_sees_over_cargo"));
    // No accommodation unit stands on deck.
    assert!(r.container_units.is_empty());
}

#[test]
fn the_deckhouse_style_is_what_it_was() {
    // The same inputs as a plain deckhouse: the wheelhouse on the aft house, no forward house.
    let mut i = forward_inputs();
    i.arrangement_style = ArrangementStyle::Deckhouse;
    let r = compute(&i).unwrap();
    assert!(r.forward_house.is_none());
    let d = r.deckhouse.as_ref().unwrap();
    assert!((r.wheelhouse_floor_lowered_m - d.top_m).abs() < 1e-9);
    assert_eq!(r.wheelhouse_x1_m, d.x_fwd_m);
}

#[test]
fn the_forward_house_steel_and_the_wheelhouse_steel_are_forward_of_the_block() {
    let i = forward_inputs();
    let r = compute(&i).unwrap();
    let m = SternMassInputs {
        battery_unit_energy_kwh: 0.0,
        installed_power_kw: None,
        gas_combustion_unit: false,
    };
    let items = stern_block_mass(&i, &r, &m, 1.0);
    let by = |l: &str| items.iter().find(|x| x.label == l).unwrap_or_else(|| panic!("{l}"));
    let house = by("forward house steel");
    let fwd = r.forward_house.as_ref().unwrap();
    assert!((house.lcg_m - (fwd.x_aft_m + fwd.x_fwd_m) / 2.0).abs() < 1e-9);
    assert!(house.mass_t > 0.0);
    assert!(by("wheelhouse steel").lcg_m > r.length_m);
    assert!(by("deckhouse steel").lcg_m < r.length_m);
}

#[test]
fn a_style_with_no_site_places_the_house_after_the_deck_items() {
    let mut i = forward_inputs();
    i.forward_house_site = None;
    let r = compute(&i).unwrap();
    let fwd = r.forward_house.as_ref().unwrap();
    assert!((fwd.x_aft_m - r.deck_length_m).abs() < 1e-9);
    assert!((fwd.x_fwd_m - fwd.x_aft_m - forward_house_sizes::LENGTH_M).abs() < 1e-9);
}

#[test]
fn a_house_that_took_cargo_length_says_so() {
    let mut i = forward_inputs();
    i.forward_house_site = Some(ForwardHouseSite {
        x_aft_m: 85.0,
        x_fwd_m: 93.0,
        cargo_length_lost_m: 2.5,
    });
    let r = compute(&i).unwrap();
    assert_eq!(r.forward_house.as_ref().unwrap().cargo_length_lost_m, 2.5);
    assert!(
        r.warnings.iter().any(|w| w.to_string().contains("2.50 m of cargo length")),
        "{:?}",
        r.warnings
    );
}
