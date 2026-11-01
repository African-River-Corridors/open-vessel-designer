//! Issue #743: the ends of the plate model (spec 0013, "Model", `sections` / `mesh` row, "Ends", D6).
//! The shell plates follow the Sections Table, one strip per Station interval, the same ruled quads
//! the hull mesh builds; the peak, collision and hold bulkheads, the floors, the inner bottom and the
//! girders take the hull's section where they stand.
//!
//! Label: property - identities and geometry oracles only. The shell plate area equals the surface
//! area of the ruled hull worked out here another way: in closed form on the raked box, and on every
//! other Hull Form by summing each ruled quad as a fine grid of flat triangles. No fixed number.

use ovd_core::hull::hull_form::{
    Bottom, HullFormInputs, HullFormKind, HullShape, RakedBox, generate,
};
use ovd_core::hull_panels::{
    HullPanelModel, HullStructureSpec, function_type as ft, hull_panels, region,
};
use ovd_core::hull_plates::{HullPlateModel, ItemKind, WELD_CLEARANCE_M, hull_plates_on};
use ovd_core::sections::{Section, SectionsTable};
use ovd_core::stern_form::{HullDims, SternForm, SternFormInputs, apply, geometry};
use ovd_core::vessel_model::StructureBlock;

const L: f64 = 140.0;
const B: f64 = 29.0;
const D: f64 = 12.0;

/// The test hull's `HullStructureSpec`: a double hull with a cargo zone from 22 m to 120 m.
fn spec(openings: bool) -> HullStructureSpec {
    HullStructureSpec {
        loa_m: L,
        rule_length_m: 134.4,
        beam_m: B,
        depth_m: D,
        block_coefficient: 0.9,
        cargo_zone_aft_m: 22.0,
        cargo_zone_fwd_m: 120.0,
        double_bottom_m: 1.5,
        wing_width_m: 2.4,
        wing_top_m: 8.0,
        deck_opening_width_m: if openings { 12.0 } else { 0.0 },
        wing_deck_opening_width_m: if openings { 4.0 } else { 0.0 },
        holds_along: 3,
        tanks_abreast: 2,
        steel_density: 7.85,
        structure: StructureBlock::default(),
    }
}

fn table(form: HullFormKind, shape: HullShape) -> SectionsTable {
    generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form,
        shape,
    })
    .unwrap()
}

/// The raked box of the tests: a bow rake 14 m long rising 6 m, a stern rake 6 m long rising 2.5 m.
/// The collision bulkhead (midway along the bow zone, at 130 m) stands on the bow rake, where the keel
/// is above the inner bottom.
const RAKES: (f64, f64, f64, f64) = (14.0, 6.0, 6.0, 2.5);

fn raked() -> HullFormKind {
    HullFormKind::RakedBox {
        bow_rake_length_m: RAKES.0,
        bow_rake_rise_m: RAKES.1,
        stern_rake_length_m: RAKES.2,
        stern_rake_rise_m: RAKES.3,
    }
}

fn plates_on(s: &HullStructureSpec, t: &SectionsTable) -> (HullPanelModel, HullPlateModel) {
    let p = hull_panels(s);
    let m = hull_plates_on(&p, s, t, &[]);
    (p, m)
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * a.abs().max(b.abs()).max(1e-12)
}

fn is_shell(member: &str) -> bool {
    member == ft::BOTTOM_SHELL || member == ft::SIDE_SHELL || member == ft::STRENGTH_DECK
}

/// The shell plate area of a plate model: bottom, side and deck, and the end plating.
fn shell_area(m: &HullPlateModel) -> f64 {
    m.items
        .iter()
        .filter(|i| i.kind == ItemKind::Plate && is_shell(&i.member))
        .map(|i| i.area_m2)
        .sum()
}

type P3 = [f64; 3];

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn tri(a: P3, b: P3, c: P3) -> f64 {
    let (u, v) = (sub(b, a), sub(c, a));
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
}

/// The area of the ruled quad `a b c d` (`a b` on one Section, `d c` on the next) as an `n x n` grid
/// of flat triangles: it converges on the ruled surface as `n` grows.
fn ruled_quad(a: P3, b: P3, c: P3, d: P3, n: usize) -> f64 {
    let at = |u: f64, v: f64| -> P3 {
        let p = |k: usize| {
            (1.0 - u) * ((1.0 - v) * a[k] + v * b[k]) + u * ((1.0 - v) * d[k] + v * c[k])
        };
        [p(0), p(1), p(2)]
    };
    let mut s = 0.0;
    for i in 0..n {
        for j in 0..n {
            let (u0, u1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
            let (v0, v1) = (j as f64 / n as f64, (j + 1) as f64 / n as f64);
            let (p, q, r, t) = (at(u0, v0), at(u0, v1), at(u1, v1), at(u1, v0));
            s += tri(p, q, r) + tri(p, r, t);
        }
    }
    s
}

fn shoelace(pts: &[(f64, f64)]) -> f64 {
    let n = pts.len();
    let mut a = 0.0;
    for i in 0..n {
        let (p, q) = (pts[i], pts[(i + 1) % n]);
        a += p.0 * q.1 - q.0 * p.1;
    }
    0.5 * a.abs()
}

fn slotted(s: &Section) -> bool {
    s.points[0].0 > 0.0
}

/// The surface area of a Sections Table's closed hull, worked out without the plate model: every
/// ruled quad between neighbouring Sections and points (both sides, and the slot wall of a slotted
/// pair) as a grid of flat triangles, and the two end faces by the shoelace formula.
fn surface_area(t: &SectionsTable, n: usize) -> f64 {
    let mut s = 0.0;
    for w in t.sections.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let k = a.points.len();
        let at = |sec: &Section, i: usize| -> P3 { [sec.x, sec.points[i].0, sec.points[i].1] };
        let mut edges: Vec<(usize, usize)> = (0..k - 1).map(|i| (i, i + 1)).collect();
        if slotted(a) || slotted(b) {
            edges.push((k - 1, 0));
        }
        for (i, j) in edges {
            s += 2.0 * ruled_quad(at(a, i), at(a, j), at(b, j), at(b, i), n);
        }
    }
    for sec in [&t.sections[0], &t.sections[t.sections.len() - 1]] {
        s += 2.0 * shoelace(&sec.points);
    }
    s
}

/// Spec 0013, "Testing Decisions": the geometry oracle. On the raked box the shell plates (bottom,
/// rakes, sides, deck and the two end plates) sum to the box's closed-form surface area.
#[test]
fn the_shell_plates_of_the_raked_box_sum_to_its_closed_form_surface_area() {
    let (bl, br, sl, sr) = RAKES;
    let t = RakedBox {
        loa: L,
        beam: B,
        depth: D,
        bow_rake_length: bl,
        bow_rake_rise: br,
        stern_rake_length: sl,
        stern_rake_rise: sr,
    }
    .sections()
    .unwrap();
    let closed = L * B // deck
        + (L - bl - sl) * B // flat bottom
        + B * (bl * bl + br * br).sqrt() // bow rake
        + B * (sl * sl + sr * sr).sqrt() // stern rake
        + 2.0 * (L * D - 0.5 * bl * br - 0.5 * sl * sr) // sides
        + B * (D - br) + B * (D - sr); // the end plates
    let (_, m) = plates_on(&spec(false), &t);
    let area = shell_area(&m);
    assert!(close(area, closed, 1e-10), "{area} vs {closed}");
    // The rakes carry plate: bottom plates whose centre is on each rake, above the base.
    for (x0, x1) in [(0.0, sl), (L - bl, L)] {
        let on_rake = m.items.iter().filter(|i| {
            i.kind == ItemKind::Plate
                && i.member == ft::BOTTOM_SHELL
                && i.centre_m.0 > x0
                && i.centre_m.0 < x1
        });
        let mut n = 0;
        for i in on_rake {
            assert!(i.centre_m.2 > 0.0, "{} is on the rake", i.id);
            n += 1;
        }
        assert!(n > 0, "plates on the rake {x0}..{x1}");
    }
}

/// Every generator Hull Form carries plate exactly: box, raked box, bilge radius, sheer, camber,
/// a chine bottom and the three Stern Forms. The shell plates sum to the hull's surface area.
#[test]
fn every_generator_hull_form_carries_its_shell_plate() {
    let shapes = [
        ("box", HullFormKind::Box, HullShape::default()),
        ("raked box", raked(), HullShape::default()),
        (
            "bilge radius",
            raked(),
            HullShape {
                bilge_radius_m: 1.2,
                ..HullShape::default()
            },
        ),
        (
            "sheer",
            HullFormKind::Box,
            HullShape {
                sheer_aft_m: 0.8,
                sheer_fwd_m: 1.6,
                ..HullShape::default()
            },
        ),
        (
            "camber",
            raked(),
            HullShape {
                camber_m: 0.3,
                ..HullShape::default()
            },
        ),
        (
            "sheer, camber and bilge",
            raked(),
            HullShape {
                bilge_radius_m: 1.0,
                sheer_aft_m: 0.5,
                sheer_fwd_m: 1.0,
                camber_m: 0.3,
                ..HullShape::default()
            },
        ),
        (
            "chine",
            HullFormKind::Box,
            HullShape {
                bottom: Bottom::Chine {
                    deadrise_deg: 8.0,
                    keel_half_width_m: 2.0,
                    chine_half_breadth_m: None,
                    chine_height_m: None,
                },
                ..HullShape::default()
            },
        ),
    ];
    let mut tables: Vec<(String, SectionsTable)> = shapes
        .into_iter()
        .map(|(name, form, shape)| (name.to_string(), table(form, shape)))
        .collect();
    for form in [SternForm::Transom, SternForm::Pram, SternForm::Tunnel] {
        let g = geometry(
            &SternFormInputs {
                form,
                run_length_m: None,
                transom_height_m: None,
                tunnel_crown_height_m: None,
                propulsor_count: 2,
                propeller_diameter_m: None,
                max_unit_diameter_m: None,
                parallel_shafts: true,
            },
            &HullDims {
                loa: L,
                beam: B,
                depth: D,
                design_draught: 4.0,
            },
        );
        let mut t = table(HullFormKind::Box, HullShape::default());
        apply(&g, &mut t).unwrap();
        tables.push((format!("{form:?} stern"), t));
    }
    let s = spec(false);
    for (name, t) in &tables {
        let (_, m) = plates_on(&s, t);
        let area = shell_area(&m);
        let oracle = surface_area(t, 64);
        assert!(close(area, oracle, 1e-6), "{name}: {area} vs {oracle}");
        // Every shell plate is a real piece of the hull: positive area, inside the hull's box.
        for i in m.items.iter().filter(|i| is_shell(&i.member)) {
            let (x, y, z) = i.centre_m;
            assert!((-1e-9..=L + 1e-9).contains(&x), "{name} {} x {x}", i.id);
            assert!(y.abs() <= B / 2.0 + 1e-9, "{name} {} y {y}", i.id);
            assert!((-1e-9..=D + 2.0).contains(&z), "{name} {} z {z}", i.id);
            if i.kind == ItemKind::Plate {
                assert!(i.area_m2 > 0.0, "{name} {}", i.id);
            }
        }
    }
}

/// On a box hull the plate model's shell, double bottom and bulkheads are the panel model's: each
/// member's plates sum to its panel's area. The end plating is new: the two ends, B x D each.
#[test]
fn on_a_box_the_plates_sum_to_their_panels_area_and_the_end_plates_are_added() {
    let s = spec(true);
    let t = table(HullFormKind::Box, HullShape::default());
    let (p, m) = plates_on(&s, &t);
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
        // The centre girder's ends are thinner (85 %), which the panel model weighs as less area.
        if g.member == ft::CENTRE_GIRDER {
            let w: f64 = m
                .items
                .iter()
                .filter(|i| i.member == g.member)
                .map(|i| i.rule_net_t)
                .sum();
            assert!(
                close(w, g.plate_t, 1e-9),
                "centre girder {w} vs {}",
                g.plate_t
            );
            continue;
        }
        assert!(
            close(area, g.plate_area_m2, 1e-9),
            "{} {}: plates {area} vs panel {}",
            g.member,
            g.region,
            g.plate_area_m2
        );
    }
    let ends: f64 = m
        .items
        .iter()
        .filter(|i| i.region == region::END_PLATING && i.kind == ItemKind::Plate)
        .map(|i| i.area_m2)
        .sum();
    assert!(close(ends, 2.0 * B * D, 1e-12), "{ends}");
    // One group per panel, in the panel model's order, then the end plating.
    for (g, pg) in m.groups.iter().zip(&p.members) {
        assert_eq!((&g.member, &g.region), (&pg.member, &pg.region));
    }
    assert_eq!(m.groups.len(), p.members.len() + 1);
    assert_eq!(m.groups.last().unwrap().region, region::END_PLATING);
}

/// The internal end structure takes the hull's section where it stands: on the raked box the
/// collision bulkhead stands on the bow rake, and its plate is the section above the inner bottom
/// there; a floor on a rake is the section below the inner bottom, and none where the keel is above
/// it. The hold bulkheads stand where the Vessel Model puts them.
#[test]
fn the_end_bulkheads_and_floors_follow_the_hull_section() {
    let (bl, br, sl, sr) = RAKES;
    let s = spec(true);
    let t = table(raked(), HullShape::default());
    let p = hull_panels(&s);
    let holds = [22.0, 50.0, 85.0, 120.0];
    let m = hull_plates_on(&p, &s, &t, &holds);
    let db = s.double_bottom_m;
    // The keel height at x on the raked box.
    let keel = |x: f64| {
        if x < sl {
            sr * (1.0 - x / sl)
        } else if x > L - bl {
            br * (x - (L - bl)) / bl
        } else {
            0.0
        }
    };
    let bulkhead = |pred: &dyn Fn(f64) -> bool| -> (f64, f64) {
        let ps: Vec<_> = m
            .items
            .iter()
            .filter(|i| {
                i.member == ft::TRANSVERSE_BULKHEADS
                    && i.kind == ItemKind::Plate
                    && pred(i.centre_m.0)
            })
            .collect();
        assert!(!ps.is_empty());
        let x = ps[0].geometry.points[0].0;
        (x, ps.iter().map(|i| i.area_m2).sum())
    };
    // Collision and aft peak bulkheads: the section above the inner bottom (or the keel, on a rake
    // that rises above it).
    for (pred, what) in [
        (&(|x: f64| x > 120.0) as &dyn Fn(f64) -> bool, "collision"),
        (&(|x: f64| x < 22.0) as &dyn Fn(f64) -> bool, "aft peak"),
    ] {
        let (x, area) = bulkhead(pred);
        let want = B * (D - keel(x).max(db));
        assert!(close(area, want, 1e-9), "{what} at {x}: {area} vs {want}");
    }
    // The hold bulkheads stand at the Vessel Model's positions.
    for h in holds {
        let (x, area) = bulkhead(&|x: f64| (x - h).abs() < 1e-6);
        assert!((x - h).abs() < 1e-9);
        assert!(close(area, B * (D - db), 1e-9), "hold bulkhead at {h}");
    }
    // Floors: the section below the inner bottom at each frame.
    let mut on_rake = 0;
    for f in m
        .items
        .iter()
        .filter(|i| i.member == ft::FLOORS && i.kind == ItemKind::Primary)
    {
        let x = f.geometry.points[0].0;
        let want = B * (db - keel(x)).max(0.0);
        assert!(
            close(f.area_m2, want, 1e-9),
            "floor at {x}: {} vs {want}",
            f.area_m2
        );
        assert!(f.area_m2 > 0.0, "no empty floor item at {x}");
        if keel(x) > 0.0 {
            on_rake += 1;
        }
    }
    assert!(on_rake > 0, "a floor stands on a rake");
    // The inner bottom stops where the keel rises through it.
    let ib: f64 = m
        .items
        .iter()
        .filter(|i| i.member == ft::INNER_BOTTOM)
        .map(|i| i.area_m2)
        .sum();
    let x_aft = sl * (1.0 - db / sr);
    let x_fwd = L - bl + bl * db / br;
    assert!(close(ib, B * (x_fwd - x_aft), 1e-9), "{ib}");
}

/// The table path keeps the plate model's identities: the ordered thickness rounds the rule-net
/// up to the step, every plate fits its stock plate, seams keep clear of the longitudinals, and the
/// scrap is the stock bought less the net plate area.
#[test]
fn the_ends_keep_rounding_stock_and_scrap_identities() {
    for (form, shape) in [
        (raked(), HullShape::default()),
        (
            raked(),
            HullShape {
                bilge_radius_m: 1.2,
                sheer_fwd_m: 1.0,
                camber_m: 0.3,
                ..HullShape::default()
            },
        ),
    ] {
        let mut s = spec(true);
        s.structure.stock_plate_width_m = 2.5;
        s.structure.stock_plate_length_m = 10.0;
        let t = table(form, shape);
        let (_, m) = plates_on(&s, &t);
        let step = s.structure.thickness_step_mm;
        for i in m.items.iter().filter(|i| i.area_m2 > 0.0) {
            assert!(i.ordered_mm >= i.rule_net_mm, "{}", i.id);
            assert!(i.ordered_mm < i.rule_net_mm + step, "{}", i.id);
        }
        for i in m.items.iter().filter(|i| i.kind == ItemKind::Plate) {
            assert!(i.width_m <= 2.5 + 1e-9, "{} width {}", i.id, i.width_m);
            assert!(i.length_m <= 10.0 + 1e-9, "{} length {}", i.id, i.length_m);
            assert!(i.geometry.points.len() >= 3, "{}", i.id);
        }
        let stock = 2.5 * 10.0;
        for p in &m.plates {
            assert_eq!(p.scrap_m2, p.bought_m2 - p.net_m2);
            assert!(close(p.bought_m2, p.count as f64 * stock, 1e-12));
            assert!(p.scrap_m2 >= -1e-9);
        }
        let plated: f64 = m.items.iter().map(|i| i.area_m2).sum();
        assert!(close(m.net_plate_m2, plated, 1e-9));
        assert!(close(m.scrap_m2, m.bought_m2 - m.net_plate_m2, 1e-9));
        assert!(close(m.total_t, m.ordered_t + m.allowance_t, 1e-12));
        let ids: std::collections::HashSet<_> = m.items.iter().map(|i| &i.id).collect();
        assert_eq!(ids.len(), m.items.len());
    }
}

/// The shell longitudinals are one per spacing round the girth: on the box, the bottom carries
/// B / s of them, floored, over the whole length; and no seam of the bottom falls on one.
#[test]
fn the_shell_longitudinals_are_one_per_spacing_and_the_seams_keep_clear() {
    let s = spec(true);
    let t = table(HullFormKind::Box, HullShape::default());
    let (_, m) = plates_on(&s, &t);
    let sp = s.structure.longitudinal_spacing_mm / 1000.0;
    let n_b = (B / sp + 1e-9).floor();
    let bottom: f64 = m
        .items
        .iter()
        .filter(|i| i.kind == ItemKind::Stiffener && i.member == ft::BOTTOM_SHELL)
        .map(|i| i.length_m)
        .sum();
    assert!(close(bottom, n_b * L, 1e-9), "{bottom} vs {}", n_b * L);
    let off = |v: f64| {
        let r = v.rem_euclid(sp);
        r.min(sp - r)
    };
    let mut seams = 0;
    for p in m
        .items
        .iter()
        .filter(|i| i.kind == ItemKind::Plate && i.member == ft::BOTTOM_SHELL)
    {
        for q in &p.geometry.points {
            let y = q.1.abs();
            if y > 1e-9 && y < B / 2.0 - 1e-9 {
                assert!(off(y) >= WELD_CLEARANCE_M - 1e-9, "seam at y {y}");
                seams += 1;
            }
        }
    }
    assert!(seams > 0);
}
