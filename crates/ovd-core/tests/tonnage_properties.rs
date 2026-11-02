//! Seam: gross tonnage by ITC 1969 (issue #777): the relations the formula and V must keep.
//!
//! Label: property

use ovd_core::aft_arrangement::{AftArrangementInputs, compute};
use ovd_core::sections::{Section, SectionsTable};
use ovd_core::tonnage::{
    EnclosedVolume, GrossTonnage, gross_tonnage, hull_volume_m3, k1, stern_block_above_deck_m3,
};

fn box_table(l: f64, b: f64, d: f64) -> SectionsTable {
    let half = vec![(0.0, 0.0), (b / 2.0, 0.0), (b / 2.0, d), (0.0, d)];
    SectionsTable {
        sections: vec![
            Section {
                x: 0.0,
                points: half.clone(),
            },
            Section { x: l, points: half },
        ],
        depth_at_side: vec![d, d],
    }
}

#[test]
fn gt_rises_with_v_and_k1_stays_in_its_range() {
    let mut last_gt = 0.0;
    let mut last_k1 = 0.0;
    let mut v = 10.0;
    while v <= 1_000_000.0 {
        let (k, gt) = (k1(v), gross_tonnage(v));
        assert!(gt > last_gt, "GT must rise with V: {gt} at V {v}");
        assert!(k > last_k1, "K1 must rise with V");
        // Appendix 2 runs from V = 10 m3 (K1 0.22) to 1,000,000 m3 (K1 0.32).
        assert!(
            (0.22 - 1e-12..=0.32 + 1e-12).contains(&k),
            "K1 {k} at V {v}"
        );
        // So GT is between 0.22 V and 0.32 V.
        assert!(gt >= 0.22 * v - 1e-9 && gt <= 0.32 * v + 1e-9);
        last_gt = gt;
        last_k1 = k;
        v *= 1.5;
    }
}

#[test]
fn no_volume_gives_no_tonnage() {
    assert_eq!(gross_tonnage(0.0), 0.0);
    assert_eq!(gross_tonnage(-5.0), 0.0);
    assert_eq!(k1(0.0), 0.0);
}

#[test]
fn the_hull_volume_of_a_box_is_l_b_d() {
    let (l, b, d) = (60.0, 12.0, 4.0);
    let v = hull_volume_m3(&box_table(l, b, d));
    assert!((v - l * b * d).abs() < 1e-6 * l * b * d, "box volume {v}");
}

#[test]
fn a_bigger_hull_has_the_larger_gt() {
    let small = hull_volume_m3(&box_table(60.0, 12.0, 4.0));
    let deeper = hull_volume_m3(&box_table(60.0, 12.0, 5.0));
    assert!(gross_tonnage(deeper) > gross_tonnage(small));
}

#[test]
fn the_stern_block_adds_its_units_and_wheelhouse() {
    let inp = AftArrangementInputs::new(14.0, 4.5);
    let res = compute(&inp).expect("a default stern block");
    let v = stern_block_above_deck_m3(&res, inp.container_height_m);
    let units = res.container_units.len() as f64
        * inp.container_length_m
        * inp.container_width_m
        * inp.container_height_m;
    let wheelhouse = inp.wheelhouse_length_m * inp.wheelhouse_width_m * inp.wheelhouse_height_m;
    assert!(
        (v - (units + wheelhouse)).abs() < 1e-6,
        "{v} against units {units} + wheelhouse {wheelhouse}"
    );
    assert!(v > 0.0);
    // V counts it on top of the hull, and the deckhouse seam (issue #779) adds to V.
    let hull = 3_000.0;
    let with_block = EnclosedVolume {
        hull_m3: hull,
        stern_block_m3: v,
        deckhouse_m3: 0.0,
    };
    let with_deckhouse = EnclosedVolume {
        deckhouse_m3: 100.0,
        ..with_block
    };
    assert!((with_block.total_m3() - (hull + v)).abs() < 1e-9);
    assert!(
        GrossTonnage::of(with_deckhouse, None).computed
            > GrossTonnage::of(with_block, None).computed
    );
}
