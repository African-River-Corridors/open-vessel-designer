//! Seam: the deckhouse carries a plan for each tier (issue #943). No number is fixed here: the
//! plan's length is the tier count, and the areas and the volume are sums over the plan.
//!
//! Label: property

use ovd_core::aft_arrangement::{AftArrangementInputs, ArrangementStyle, TierKind, compute};

fn inputs(beam: f64, accom: i64) -> AftArrangementInputs {
    let mut i = AftArrangementInputs::new(beam, 6.0);
    i.arrangement_style = ArrangementStyle::Deckhouse;
    i.n_battery_units = 0;
    i.n_accom_units = accom;
    i.wheelhouse_max_lift_m = 0.0;
    i
}

#[test]
fn the_plan_has_one_entry_per_tier_and_its_areas_sum_to_the_deckhouse_area() {
    for (beam, accom) in [(20.0, 6), (20.0, 1), (29.0, 12), (12.0, 0)] {
        let r = compute(&inputs(beam, accom)).unwrap();
        let d = r.deckhouse.as_ref().expect("a deckhouse");
        assert_eq!(d.tiers_plan.len() as i64, d.tiers, "{beam} {accom}");
        let sum: f64 = d.tiers_plan.iter().map(|t| t.area_m2()).sum();
        assert!((sum - d.area_m2).abs() < 1e-9, "{beam} {accom}");
        assert!((d.volume_m3 - sum * d.tier_height_m).abs() < 1e-9);
        assert!((d.tier_area_m2 - d.tiers_plan[0].area_m2()).abs() < 1e-9);
        // The floors stack one tier height apart from the main deck.
        for (i, t) in d.tiers_plan.iter().enumerate() {
            assert!((t.base_m - (d.base_m + i as f64 * d.tier_height_m)).abs() < 1e-9);
        }
        assert_eq!(d.tiers_plan[0].kind, TierKind::Rooms);
    }
}
