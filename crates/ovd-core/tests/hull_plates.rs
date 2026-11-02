//! Issue #742: the plate model (`ovd_core::hull_plates`), a re-cut of the panel model into plates,
//! stiffeners and primary members (spec 0013, "Testing Decisions").
//!
//! Label: property - identities only: the plates sum to their panel's area, the stiffeners are one
//! per spacing as the panel model counts them, the rule-net weight is the panel weight, the ordered
//! thickness is the rule-net rounded up to the step, and the scrap is the stock bought less the net
//! plate area. No fixed number.

use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::hull_panels::{
    HullPanelModel, HullStructureSpec, function_type as ft, hull_panels, region,
};
use ovd_core::hull_plates::{HullPlateModel, ItemKind, OPENINGS_NOTE, hull_plates, ordered_mm};
use ovd_core::vessel_model::StructureBlock;

fn spec(depth: f64) -> HullStructureSpec {
    HullStructureSpec {
        loa_m: 140.0,
        rule_length_m: 134.4,
        beam_m: 29.0,
        depth_m: depth,
        block_coefficient: 0.9,
        cargo_zone_aft_m: 22.0,
        cargo_zone_fwd_m: 120.0,
        double_bottom_m: 1.5,
        wing_width_m: 2.4,
        wing_top_m: 8.0,
        deck_opening_width_m: 12.0,
        wing_deck_opening_width_m: 4.0,
        holds_along: 3,
        tanks_abreast: 2,
        steel_density: 7.85,
        structure: StructureBlock::default(),
    }
}

/// The panel and plate models of a spec, with a structure block changed by `f`.
fn both(
    depth: f64,
    f: impl Fn(&mut StructureBlock),
) -> (HullStructureSpec, HullPanelModel, HullPlateModel) {
    let mut s = spec(depth);
    f(&mut s.structure);
    let p = hull_panels(&s);
    let m = hull_plates(&p, &s);
    (s, p, m)
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * a.abs().max(b.abs()).max(1e-12)
}

#[test]
fn the_plates_sum_to_their_panels_area() {
    for depth in [9.0, 12.0] {
        let (_, p, m) = both(depth, |_| ());
        for g in &p.members {
            if g.plate_area_m2 <= 0.0 {
                continue;
            }
            let area: f64 = m
                .items
                .iter()
                .filter(|i| i.member == g.member && i.region == g.region && i.area_m2 > 0.0)
                .map(|i| i.area_m2)
                .sum();
            assert!(
                close(area, g.plate_area_m2, 0.001),
                "{} {}: plates {area} vs panel {}",
                g.member,
                g.region,
                g.plate_area_m2
            );
        }
    }
}

#[test]
fn the_rule_net_weight_is_the_panel_weight() {
    for step in [0.5, 1.0, 0.3] {
        let (_, p, m) = both(12.0, |s| s.thickness_step_mm = step);
        assert!(
            close(m.rule_net_t, p.net_t, 1e-9),
            "{} vs {}",
            m.rule_net_t,
            p.net_t
        );
        for (g, pg) in m.groups.iter().zip(&p.members) {
            assert_eq!((&g.member, &g.region), (&pg.member, &pg.region));
            assert!(
                close(g.rule_net_t, pg.total_t, 1e-9),
                "{} {}",
                g.member,
                g.region
            );
        }
        assert!(m.ordered_t >= m.rule_net_t);
        assert!(close(
            m.allowance_t,
            m.ordered_t * m.allowance_pct / 100.0,
            1e-12
        ));
        assert!(close(m.total_t, m.ordered_t + m.allowance_t, 1e-12));
    }
}

#[test]
fn the_ordered_thickness_is_the_rule_net_rounded_up_to_the_step() {
    for step in [0.5, 1.0, 0.3] {
        let (_, _, m) = both(12.0, |s| s.thickness_step_mm = step);
        let mut rounded = 0;
        for i in m.items.iter().filter(|i| i.area_m2 > 0.0) {
            assert!(i.ordered_mm >= i.rule_net_mm, "{}", i.id);
            assert!(i.ordered_mm < i.rule_net_mm + step, "{}", i.id);
            if i.ordered_mm > i.rule_net_mm {
                rounded += 1;
            }
        }
        if step > 0.5 {
            assert!(rounded > 0, "a coarser step rounds some plate up");
        }
    }
    assert_eq!(ordered_mm(10.2, 0.5), 10.5);
    assert_eq!(ordered_mm(10.5, 0.5), 10.5);
}

#[test]
fn the_scrap_is_the_stock_bought_less_the_net_area() {
    let (s, _, m) = both(12.0, |_| ());
    let stock = s.structure.stock_plate_width_m * s.structure.stock_plate_length_m;
    let mut bought = 0.0;
    let mut net = 0.0;
    for p in &m.plates {
        assert_eq!(p.scrap_m2, p.bought_m2 - p.net_m2);
        assert!(close(p.bought_m2, p.count as f64 * stock, 1e-12));
        assert!(p.scrap_m2 >= -1e-9, "{}", p.ordered_mm);
        bought += p.bought_m2;
        net += p.net_m2;
    }
    assert!(close(m.scrap_m2, bought - net, 1e-9));
    let plated: f64 = m.items.iter().map(|i| i.area_m2).sum();
    assert!(close(net, plated, 1e-9));
    assert!(m.scrap_t > 0.0);
}

#[test]
fn every_plate_fits_its_stock_plate() {
    let (s, _, m) = both(12.0, |s| {
        s.stock_plate_width_m = 2.5;
        s.stock_plate_length_m = 10.0;
    });
    for i in m.items.iter().filter(|i| i.kind == ItemKind::Plate) {
        assert!(
            i.width_m <= s.structure.stock_plate_width_m + 1e-9,
            "{}",
            i.id
        );
        assert!(
            i.length_m <= s.structure.stock_plate_length_m + 1e-9,
            "{}",
            i.id
        );
        assert_eq!(i.pieces, 1);
    }
}

/// Seams of a stiffened panel and butts of a lengthwise panel keep clear of the longitudinals and
/// the frames.
#[test]
fn seams_and_butts_keep_clear_of_stiffeners_and_frames() {
    use ovd_core::hull_plates::WELD_CLEARANCE_M;
    let (s, _, m) = both(12.0, |_| ());
    let sp = s.structure.longitudinal_spacing_mm / 1000.0;
    let web = s.structure.frame_spacing_m;
    let off = |v: f64, step: f64| {
        let r = v.rem_euclid(step);
        r.min(step - r)
    };
    let y0 = -s.beam_m / 2.0;
    let mut seams = 0;
    let mut butts = 0;
    for p in m.items.iter().filter(|i| {
        i.kind == ItemKind::Plate && i.member == ft::BOTTOM_SHELL && i.region == region::MIDSHIP
    }) {
        let low = p.geometry.points[0].1 - y0;
        if low > 1e-9 {
            assert!(off(low, sp) >= WELD_CLEARANCE_M - 1e-9, "seam at {low} m");
            seams += 1;
        }
        let x = p.geometry.points[0].0;
        if x > 0.3 * s.loa_m + 1e-9 {
            assert!(off(x, web) >= WELD_CLEARANCE_M - 1e-9, "butt at x {x} m");
            butts += 1;
        }
    }
    assert!(seams > 0 && butts > 0);
}

/// One stiffener per spacing, as the panel model counts them: the bottom carries B / s of them,
/// floored; each floor one at every bottom longitudinal.
#[test]
fn the_stiffeners_are_one_per_spacing() {
    let (s, p, m) = both(12.0, |_| ());
    let sp = s.structure.longitudinal_spacing_mm;
    let n_b = (s.beam_m * 1000.0 / sp + 1e-9).floor() as usize;
    let count = |member: &str, reg: &str| {
        m.items
            .iter()
            .filter(|i| i.kind == ItemKind::Stiffener && i.member == member && i.region == reg)
            .count()
    };
    assert_eq!(count(ft::BOTTOM_SHELL, region::MIDSHIP), n_b);
    // The ends are two pieces, one line each per spacing.
    assert_eq!(count(ft::BOTTOM_SHELL, region::ENDS), 2 * n_b);
    let floors = p.members.iter().find(|g| g.member == ft::FLOORS).unwrap();
    assert_eq!(count(ft::FLOORS, ""), n_b * floors.count);
    // Every stiffener and profile weighs its section over its length.
    for g in &p.members {
        let lines: f64 = m
            .items
            .iter()
            .filter(|i| i.member == g.member && i.region == g.region && i.section.is_some())
            .map(|i| i.length_m)
            .sum();
        assert!(
            close(lines, g.stiffener_length_m, 1e-9),
            "{} {}",
            g.member,
            g.region
        );
    }
}

#[test]
fn primary_members_are_one_item_per_frame() {
    let (_, p, m) = both(12.0, |_| ());
    for name in [ft::FLOORS, ft::DECK_TRANSVERSES, ft::WING_WEB_FRAMES] {
        let g = p.members.iter().find(|g| g.member == name).unwrap();
        let n = m
            .items
            .iter()
            .filter(|i| i.kind == ItemKind::Primary && i.member == name)
            .count();
        assert_eq!(n, g.count, "{name}");
    }
}

#[test]
fn every_item_has_an_id_a_citation_and_a_centre_inside_the_hull() {
    let (s, _, m) = both(12.0, |_| ());
    let mut ids = std::collections::HashSet::new();
    for i in &m.items {
        assert!(ids.insert(i.id.clone()), "{}", i.id);
        assert_eq!(i.rule_set, "abs-steel-barges");
        assert!(
            !i.paragraph.is_empty() && i.rule.contains(&i.paragraph),
            "{}",
            i.id
        );
        assert!(i.basis == "rule" || i.basis == "estimate");
        let (x, y, z) = i.centre_m;
        assert!((-1e-9..=s.loa_m + 1e-9).contains(&x), "{} x {x}", i.id);
        assert!(y.abs() <= s.beam_m / 2.0 + 1e-9, "{} y {y}", i.id);
        assert!((-1e-9..=s.depth_m + 1e-9).contains(&z), "{} z {z}", i.id);
        assert!(i.geometry.points.len() == 4 || i.geometry.points.len() == 2);
    }
}

#[test]
fn the_steel_list_says_lightening_holes_are_not_deducted() {
    assert!(OPENINGS_NOTE.contains("Lightening holes") && OPENINGS_NOTE.contains("not deducted"));
}

#[test]
fn a_gas_hull_gets_a_plate_model() {
    let mut i = BargeCapacityInputs::new(140.0, 29.0, 9.0);
    i.cargo_type = "LNG_ethane".into();
    let r = compute(&i).unwrap();
    let b = r.gas_basis.as_ref().unwrap();
    let s = HullStructureSpec::of_gas_arrangement(
        &b.inputs,
        &b.tanks,
        b.inputs.block_coefficient,
        StructureBlock::default(),
        None,
    );
    let p = hull_panels(&s);
    let m = hull_plates(&p, &s);
    assert!(close(m.rule_net_t, p.net_t, 1e-9));
    assert!(m.items.iter().any(|i| i.kind == ItemKind::Plate));
    assert!(m.scrap_t > 0.0 && m.total_t > p.total_t * 0.99);
}
