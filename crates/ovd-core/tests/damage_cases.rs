//! Seam: `ovd_core::damage_cases`, the one damage case generator for every rule with box extents (spec 0012,
//! issue #757): ADN 9.3.2.15/9.3.3.15, MARPOL I reg. 28 and IGC 2.3/2.6.
//!
//! Label: property

use ovd_core::compartments::{Side, Space, SpaceKind, SpaceShape};
use ovd_core::damage_cases::{
    BoxExtents, DamageCase, Permeability, PermeabilityTable, Placement, StandardOfDamage,
    box_damage_cases, deck_edge_immersed, gas_flooding, igc_damage_cases, marpol_damage_cases,
    without_breached_cargo,
};
use ovd_core::damage_stability::FloodedVolume;
use ovd_core::sections::{Gravity, Waterplane};
use ovd_core::tanks::{HeadKind, TankHead};

fn space(name: &str, kind: SpaceKind, x: (f64, f64), y: (f64, f64), z: (f64, f64)) -> Space {
    Space {
        name: name.to_string(),
        kind,
        side: Side::Starboard,
        x_aft: x.0,
        x_fwd: x.1,
        shape: SpaceShape::Prism {
            section: vec![(y.0, z.0), (y.1, z.0), (y.1, z.1), (y.0, z.1)],
        },
    }
}

fn tank(name: &str, x: (f64, f64), y: f64, z: f64, d: f64) -> Space {
    Space {
        name: name.to_string(),
        kind: SpaceKind::CargoTank,
        side: Side::Starboard,
        x_aft: x.0,
        x_fwd: x.1,
        shape: SpaceShape::Tank {
            axis_x0: x.0,
            axis_x1: x.1,
            y,
            z,
            outer_diameter: d,
            head: TankHead {
                kind: HeadKind::Hemisphere,
                length_m: 0.5 * d,
            },
            inner_volume_m3: 1.0,
        },
    }
}

/// A gas-like layout, 60 x 12 x 6 m: an aft machinery space 0 to 8 m, two holds 8 to 34 and 34 to 60 m,
/// with a double bottom and a starboard wing cut at the same bulkheads, and a Type C tank in each hold.
fn gas_layout() -> Vec<Space> {
    let mut s = vec![space(
        "M1C",
        SpaceKind::Machinery,
        (0.0, 8.0),
        (-6.0, 6.0),
        (0.0, 6.0),
    )];
    for (k, x) in [(8.0, 34.0), (34.0, 60.0)].into_iter().enumerate() {
        let n = k + 1;
        s.push(space(
            &format!("H{n}C"),
            SpaceKind::CargoHold,
            x,
            (-4.5, 4.5),
            (1.0, 6.0),
        ));
        s.push(space(
            &format!("DB{n}C"),
            SpaceKind::DoubleBottomBallast,
            x,
            (-6.0, 6.0),
            (0.0, 1.0),
        ));
        s.push(space(
            &format!("WB{n}S"),
            SpaceKind::WingBallast,
            x,
            (4.5, 6.0),
            (1.0, 6.0),
        ));
        s.push(tank(
            &format!("CT{n}C"),
            (x.0 + 1.0, x.1 - 1.0),
            0.0,
            3.5,
            4.0,
        ));
    }
    s
}

fn names(cases: &[ovd_core::damage_cases::DamageCase]) -> Vec<Vec<String>> {
    cases
        .iter()
        .map(|c| {
            let mut n: Vec<String> = c.flooded.iter().map(|f| f.name.clone()).collect();
            n.sort();
            n
        })
        .collect()
}

/// The MARPOL path is the box generator with the MARPOL extents and standard: the same cases, in the same
/// order, with the same Flooded Volumes, at a length on each side of the 150 m step of regulation 28.1.
#[test]
fn the_marpol_generator_is_the_box_generator_with_the_marpol_extents() {
    for loa in [60.0, 200.0] {
        let spaces = vec![
            space(
                "WB1S",
                SpaceKind::WingBallast,
                (0.2 * loa, 0.8 * loa),
                (4.0, 5.0),
                (0.0, 4.0),
            ),
            space(
                "DB1C",
                SpaceKind::DoubleBottomBallast,
                (0.2 * loa, 0.8 * loa),
                (-5.0, 5.0),
                (0.0, 1.0),
            ),
            space(
                "1",
                SpaceKind::CargoTank,
                (0.2 * loa, 0.5 * loa),
                (-4.0, 4.0),
                (1.0, 4.0),
            ),
            space(
                "2",
                SpaceKind::CargoTank,
                (0.5 * loa, 0.8 * loa),
                (-4.0, 4.0),
                (1.0, 4.0),
            ),
        ];
        let standard = StandardOfDamage {
            placement: if loa <= 150.0 {
                Placement::BetweenBulkheads
            } else {
                Placement::Anywhere
            },
            intact_bulkheads_m: Vec::new(),
        };
        let generic = box_damage_cases(
            &spaces,
            loa,
            10.0,
            &BoxExtents::marpol(loa, 10.0),
            &standard,
            true,
            &|_| 0.95,
        );
        assert_eq!(
            generic,
            marpol_damage_cases(&spaces, loa, 10.0),
            "L = {loa}"
        );
    }
}

fn igc_extents() -> BoxExtents {
    // IGC 2.3.1 at L = 60 m, B = 12 m: (1/3) 60^(2/3) = 5.11 m, B/5 = 2.4 m, B/6 = 2.0 m, B/15 = 0.8 m.
    BoxExtents {
        side_long_m: 5.109,
        side_inboard_m: 2.4,
        bottom_fwd_long_m: 5.109,
        bottom_fwd_width_m: 2.0,
        bottom_aft_long_m: 5.109,
        bottom_aft_width_m: 2.0,
        bottom_height_m: 0.8,
        forward_zone: 0.3,
    }
}

/// IGC 2.6.1.3: a type 2G ship of 150 m or less is damaged anywhere in its length except involving either
/// bulkhead of a machinery space aft. A damage wholly inside the machinery space floods it alone; no case floods
/// it together with another space. Without the exception (2.6.1.2, more than 150 m) a case floods it with the
/// first hold.
#[test]
fn a_2g_ship_of_150_m_or_less_keeps_the_aft_machinery_bulkheads_intact() {
    let spaces = gas_layout();
    let with = igc_damage_cases(
        &spaces,
        60.0,
        12.0,
        &igc_extents(),
        &StandardOfDamage::igc_2g(&spaces, 60.0, true),
        &|_| 0.95,
    );
    let without = igc_damage_cases(
        &spaces,
        60.0,
        12.0,
        &igc_extents(),
        &StandardOfDamage::igc_2g(&spaces, 60.0, false),
        &|_| 0.95,
    );
    assert!(!with.is_empty());
    assert!(
        names(&with)
            .iter()
            .all(|n| !n.iter().any(|s| s == "M1C") || n.len() == 1),
        "{:?}",
        names(&with)
    );
    assert!(
        names(&without)
            .iter()
            .any(|n| n.iter().any(|s| s == "M1C") && n.iter().any(|s| s == "H1C")),
        "{:?}",
        names(&without)
    );
}

/// IGC 2.6.1.2 and .3: a type 2G ship is damaged anywhere in its length, so a damage across the hold bulkhead
/// floods both holds and both wings, at a length where MARPOL 28.1.3 would keep the damage between bulkheads.
#[test]
fn an_igc_damage_lies_anywhere_and_floods_across_a_bulkhead() {
    let spaces = gas_layout();
    let cases = igc_damage_cases(
        &spaces,
        60.0,
        12.0,
        &igc_extents(),
        &StandardOfDamage::igc_2g(&spaces, 60.0, true),
        &|_| 0.95,
    );
    assert!(
        names(&cases).iter().any(|n| {
            ["H1C", "H2C", "WB1S", "WB2S"]
                .iter()
                .all(|w| n.iter().any(|s| s == w))
        }),
        "{:?}",
        names(&cases)
    );
}

/// IGC 2.3.1.1.2: the side damage reaches B/5 inboard. A Type C tank whose outer shell lies within that
/// strip is breached; one clear of it is not. The tank's extent across the hull is its outer diameter.
#[test]
fn a_side_damage_breaches_a_type_c_tank_only_when_it_reaches_the_tank() {
    let spaces = gas_layout();
    let ext = igc_extents();
    let reach = |ext: &BoxExtents| -> bool {
        igc_damage_cases(
            &spaces,
            60.0,
            12.0,
            ext,
            &StandardOfDamage::igc_2g(&spaces, 60.0, true),
            &|_| 0.95,
        )
        .iter()
        .filter(|c| c.name.starts_with("side"))
        .any(|c| c.flooded.iter().any(|f| f.name.starts_with("CT")))
    };
    // The tank's shell is 2.0 m from the centre line, 4.0 m inboard of the side at 6.0 m.
    assert!(!reach(&ext));
    assert!(reach(&BoxExtents {
        side_inboard_m: 4.1,
        ..ext
    }));
}

/// The IGC bottom damage is B/15 high, here 0.8 m: in the cargo zone it floods the double bottom, not the hold
/// above it (aft, the machinery space reaches the bottom), and the IGC generator does not cut the double bottom at the tank ends (the gas hull is already cut at its holds).
#[test]
fn an_igc_bottom_damage_floods_the_double_bottom_compartments_as_given() {
    let spaces = gas_layout();
    let cases = igc_damage_cases(
        &spaces,
        60.0,
        12.0,
        &igc_extents(),
        &StandardOfDamage::igc_2g(&spaces, 60.0, true),
        &|_| 0.95,
    );
    let bottom: Vec<Vec<String>> = names(
        &cases
            .into_iter()
            .filter(|c| c.name.starts_with("bottom"))
            .collect::<Vec<_>>(),
    );
    assert!(!bottom.is_empty());
    for n in &bottom {
        assert!(n.iter().all(|s| s.starts_with("DB") || s == "M1C"), "{n:?}");
        assert!(n.iter().all(|s| !s.contains('/')), "{n:?}");
    }
}

/// A Permeability Table: a fixed row ignores the filling; a by-filling row is its value times the empty
/// fraction, so 0 when full and the full value when empty; a kind the table does not name has none.
#[test]
fn a_by_filling_permeability_scales_with_the_empty_fraction() {
    let t = PermeabilityTable {
        rows: vec![
            (SpaceKind::Machinery, Permeability::Fixed(0.85)),
            (SpaceKind::WingBallast, Permeability::ByFilling(0.95)),
        ],
    };
    assert_eq!(t.of(SpaceKind::Machinery, 0.7), Some(0.85));
    assert_eq!(t.of(SpaceKind::WingBallast, 0.0), Some(0.95));
    assert_eq!(t.of(SpaceKind::WingBallast, 1.0), Some(0.0));
    assert_eq!(t.of(SpaceKind::WingBallast, 0.5), Some(0.95 * 0.5));
    assert_eq!(t.of(SpaceKind::WingBallast, 1.5), Some(0.0));
    assert_eq!(t.of(SpaceKind::CargoHold, 0.0), None);
}

fn igc_table() -> PermeabilityTable {
    PermeabilityTable {
        rows: vec![
            (SpaceKind::CargoHold, Permeability::Fixed(0.95)),
            (SpaceKind::WingBallast, Permeability::ByFilling(0.95)),
            (SpaceKind::CargoTank, Permeability::ByFilling(0.95)),
        ],
    }
}

fn case_of(names: &[&str]) -> DamageCase {
    DamageCase {
        name: "test".to_string(),
        flooded: names
            .iter()
            .map(|n| FloodedVolume::from_box(n, (0.0, 1.0), (0.0, 1.0), (0.0, 1.0), 0.95))
            .collect(),
    }
}

/// Spec 0012 D7 to D9: a flooded hold keeps its tank as an Intact Inclusion; a flooded tank is breached and
/// floods as an empty space; a wing floods by its filling.
#[test]
fn a_gas_case_floods_holds_around_intact_tanks_and_breaches_tanks_it_reaches() {
    let spaces = gas_layout();
    let t = igc_table();
    let half_full = |n: &str| if n == "WB1S" { 0.5 } else { 0.0 };
    let f = gas_flooding(&case_of(&["H1C", "WB1S"]), &spaces, &t, &half_full);
    assert!(f.breached.is_empty());
    let hold = f.flooded.iter().find(|v| v.name == "H1C").unwrap();
    assert_eq!(
        hold.inclusions
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>(),
        vec!["CT1C"]
    );
    let wing = f.flooded.iter().find(|v| v.name == "WB1S").unwrap();
    assert_eq!(wing.permeability, 0.95 * 0.5);
    let g = gas_flooding(&case_of(&["H1C", "CT1C"]), &spaces, &t, &|_| 1.0);
    assert_eq!(g.breached, vec!["CT1C".to_string()]);
    let tank = g.flooded.iter().find(|v| v.name == "CT1C").unwrap();
    assert_eq!(tank.permeability, 0.95);
}

/// IGC 2.5.3, D8: the breached tank's share of the cargo (by interior volume) leaves from the middle of its
/// axis. Two equal tanks, 100 t of cargo at KG 3.5 m: one breached tank at x = 21, y = 0 takes away 50 t.
#[test]
fn a_breached_tank_takes_its_share_of_the_cargo_away() {
    let spaces = gas_layout();
    let g = Gravity {
        lcg_m: 30.0,
        tcg_m: 0.0,
        kg_m: 3.0,
    };
    let (m, gd) = without_breached_cargo(1000.0, g, (100.0, 3.5), &spaces, &["CT1C".to_string()]);
    assert_eq!(m, 950.0);
    assert!((gd.lcg_m - (1000.0 * 30.0 - 50.0 * 21.0) / 950.0).abs() < 1e-9);
    assert!((gd.kg_m - (1000.0 * 3.0 - 50.0 * 3.5) / 950.0).abs() < 1e-9);
    assert_eq!(gd.tcg_m, 0.0);
    let (same, _) = without_breached_cargo(1000.0, g, (100.0, 3.5), &spaces, &[]);
    assert_eq!(same, 1000.0);
}

/// The deck-edge test: upright at a draught below the depth the deck is dry; heeled 30 degrees with the plane
/// at the centre line at 4 m, the deck edge of a 12 m wide, 6 m deep hull (6 tan 30 = 3.46 m lower) is wet.
#[test]
fn the_deck_edge_test_finds_an_immersed_deck_edge() {
    let up = Waterplane::upright(4.0);
    assert!(!deck_edge_immersed(&up, (0.0, 60.0), 6.0, 6.0));
    let heeled = Waterplane {
        heel_deg: 30.0,
        trim_deg: 0.0,
        c: 4.0,
    };
    assert!(deck_edge_immersed(&heeled, (0.0, 60.0), 6.0, 6.0));
}

/// MARPOL Annex I 28.4.1 (spec 0012 D20, issue #764): a breached prism cargo tank loses its share of the oil by
/// volume, from the middle of its length and the centre of its section. Two cargo tanks, 20 x 4 x 3 m to port
/// (y -4 to 0) and starboard (y 0 to 4), 300 t of oil at KG 2.5 m: breaching the starboard tank takes 150 t from
/// x = 30, y = 2.
#[test]
fn a_breached_prism_cargo_tank_loses_its_share_of_the_oil() {
    let spaces = vec![
        space(
            "1P",
            SpaceKind::CargoTank,
            (20.0, 40.0),
            (-4.0, 0.0),
            (1.0, 4.0),
        ),
        space(
            "1S",
            SpaceKind::CargoTank,
            (20.0, 40.0),
            (0.0, 4.0),
            (1.0, 4.0),
        ),
        space(
            "WB1S",
            SpaceKind::WingBallast,
            (20.0, 40.0),
            (4.0, 5.0),
            (0.0, 4.0),
        ),
    ];
    let g = Gravity {
        lcg_m: 30.0,
        tcg_m: 0.0,
        kg_m: 2.0,
    };
    let (m, gd) = without_breached_cargo(1500.0, g, (300.0, 2.5), &spaces, &["1S".to_string()]);
    assert_eq!(m, 1350.0);
    assert!((gd.lcg_m - 30.0).abs() < 1e-9);
    assert!(
        (gd.tcg_m - (-150.0 * 2.0) / 1350.0).abs() < 1e-9,
        "{}",
        gd.tcg_m
    );
    assert!((gd.kg_m - (1500.0 * 2.0 - 150.0 * 2.5) / 1350.0).abs() < 1e-9);
}
