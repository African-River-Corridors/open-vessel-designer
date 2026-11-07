//! The openings of a stepped deckhouse (issue #943): each tier's entrance is on that tier's own aft
//! face, and the front face, and so the zone measurements, are the same for every tier. No fixed
//! number is a result: each assertion is a relation between the tier plan and the layout.
//!
//! Label: property

use ovd_core::deckhouse_openings::*;

/// A house of 10 m from the stern, 12 m wide, on a 6 m deck, three tiers of 2.6 m.
fn frame() -> Frame {
    Frame {
        x_aft_m: 2.0,
        x_fwd_m: 10.0,
        width_m: 12.0,
        base_m: 6.0,
        tier_height_m: 2.6,
        tiers: 3,
        wheelhouse: Some((6.0, 10.0, 3.0, 13.8)),
        l_icll_m: 100.0,
    }
}

/// A room tier of the house's 12 m width, and two access tiers (the trunk) 6.4 m wide.
fn faces() -> [TierFace; 3] {
    [
        TierFace {
            x_aft_m: 2.0,
            width_m: 12.0,
        },
        TierFace {
            x_aft_m: 5.0,
            width_m: 6.4,
        },
        TierFace {
            x_aft_m: 6.5,
            width_m: 6.4,
        },
    ]
}

fn uniform() -> TierFace {
    TierFace {
        x_aft_m: 2.0,
        width_m: 12.0,
    }
}

fn entrance(layout: &[Opening], tier: i64) -> &Opening {
    layout
        .iter()
        .find(|o| o.id == format!("accommodation entrance, tier {tier}"))
        .unwrap()
}

#[test]
fn a_stepped_house_puts_each_tiers_entrance_on_its_own_aft_face() {
    // Tier 1 is longest; tiers 2 and 3 are shorter, cut from the aft end.
    let l = default_layout_stepped(&frame(), &faces());
    for (t, x) in [(1, 2.0), (2, 5.0), (3, 6.5)] {
        assert_eq!(entrance(&l, t).x_m, x, "tier {t}");
    }
    // The machinery openings stand on tier 1, and the accommodation air inlet on the top tier.
    let get = |id: &str| l.iter().find(|o| o.id == id).unwrap().x_m;
    assert_eq!(get("machinery entrance"), 2.0);
    assert_eq!(get("engine room air inlet"), 2.0);
    assert_eq!(get("accommodation air inlet"), 6.5);
}

#[test]
fn a_uniform_house_is_the_layout_it_was() {
    assert_eq!(
        default_layout(&frame()),
        default_layout_stepped(&frame(), &[])
    );
    assert_eq!(
        default_layout(&frame()),
        default_layout_stepped(&frame(), &[uniform(); 3])
    );
}

#[test]
fn the_zone_measures_from_the_same_front_whatever_the_steps() {
    let stepped = place_stepped(&frame(), &faces(), None);
    let uniform = place(&frame(), None);
    assert_eq!(stepped.zone_m, uniform.zone_m);
    // The wheelhouse doors and the front window are not on a tier: they are where they were.
    for (a, b) in stepped.openings.iter().zip(&uniform.openings) {
        if a.opening.tier == 0 {
            assert_eq!(a.opening.x_m, b.opening.x_m);
            assert_eq!(a.distance_from_front_m, b.distance_from_front_m);
            assert_eq!(a.in_zone, b.in_zone);
        }
    }
    // An aft-face entrance is never in the zone, however far forward its tier starts.
    assert!(
        stepped
            .openings
            .iter()
            .filter(|p| p.opening.face == Face::Aft)
            .all(|p| !p.in_zone)
    );
}

#[test]
fn an_access_tiers_entrance_is_on_the_trunks_own_width() {
    let l = default_layout_stepped(&frame(), &faces());
    // To port of the centreline by a quarter of the tier's own width: 12 m -> 3.0 m, 6.4 m -> 1.6 m.
    assert_eq!(entrance(&l, 1).y_m, -3.0);
    assert_eq!(entrance(&l, 2).y_m, -1.6);
    assert_eq!(entrance(&l, 3).y_m, -1.6);
    // The entrance stands inside its tier.
    for t in 1..=3 {
        assert!(entrance(&l, t).y_m.abs() <= faces()[t as usize - 1].width_m / 2.0);
    }
    // The machinery openings are on tier 1 and use tier 1's width.
    let y = |id: &str| l.iter().find(|o| o.id == id).unwrap().y_m;
    assert_eq!(y("machinery entrance"), 3.0);
}
