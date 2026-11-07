//! Seam: `ovd_core::damage_stability` on a gas hold: the oracles of spec 0012 D13 (issue #762).
//!
//! Label: sourced - closed forms worked out here for a wall-sided box hull (L x B x D = 60 x 10 x 4 m, density
//! 1.0): a horizontal cylinder with hemispherical heads inside a flooded box hold, cut by the waterline (the
//! circular segment of the cylinder and the spherical segment of the two heads); an independent grid integral of
//! a flooded hold less its tank below a heeled and trimmed waterplane; and an intermediate stage by added mass of
//! a one-sided flood, from the wall-sided righting lever GZ = sin(phi) (GM + BM tan^2(phi) / 2) - TCG cos(phi).
//!
//! The external mesh cross-check of D13 is not here: running a third-party tool means downloading and running
//! code outside this repository, which this build does not do. The published-case line stays open (#341).

use core::f64::consts::PI;
use libm::{acos, cos, sin, sqrt, tan};
use ovd_core::compartments::SpaceShape;
use ovd_core::damage_stability::{
    DamageCriteria, Floats, FloodedVolume, IntactInclusion, assess, intermediate_stages,
};
use ovd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovd_core::sections::{Gravity, SectionsTable, Waterplane, equilibrium};
use ovd_core::tanks::{HeadKind, TankHead};

const L: f64 = 60.0;
const B: f64 = 10.0;
const D: f64 = 4.0;
const KG: f64 = 1.8;

fn hull() -> SectionsTable {
    generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form: HullFormKind::Box,
        shape: HullShape::default(),
    })
    .unwrap()
}

fn gravity() -> Gravity {
    Gravity {
        lcg_m: L / 2.0,
        tcg_m: 0.0,
        kg_m: KG,
    }
}

fn start(table: &SectionsTable, mass: f64) -> (f64, f64) {
    let eq = equilibrium(table, mass, gravity(), 0.0, 1.0).unwrap();
    (eq.waterplane.c, eq.waterplane.trim_deg)
}

/// A Type C tank on the hold's centre line: axis 21 to 39 m, at z `zc`, radius 1 m, hemispherical heads of
/// 1 m inside the axis length (so a cylinder of 16 m and the two heads, one sphere).
fn tank(zc: f64) -> IntactInclusion {
    IntactInclusion {
        name: "CT1C".to_string(),
        x_aft_m: 21.0,
        x_fwd_m: 39.0,
        shape: SpaceShape::Tank {
            axis_x0: 21.0,
            axis_x1: 39.0,
            y: 0.0,
            z: zc,
            outer_diameter: 2.0,
            insulation_m: 0.0,
            head: TankHead {
                kind: HeadKind::Hemisphere,
                length_m: 1.0,
            },
            inner_volume_m3: 0.0,
        },
    }
}

/// The hold: 20 to 40 m, 8 m wide on the centre line, from 1 m to the deck at 4 m, permeability 1.
fn hold(inclusions: Vec<IntactInclusion>) -> FloodedVolume {
    FloodedVolume {
        inclusions,
        ..FloodedVolume::from_box("H1C", (20.0, 40.0), (-4.0, 4.0), (1.0, 4.0), 1.0)
    }
}

/// The volume of the tank (radius 1, cylinder 16 m, a sphere of heads) below a level plane `d` above its
/// bottom: the circular segment times 16, plus the spherical segment pi d^2 (3 r - d) / 3.
fn tank_below(d: f64) -> f64 {
    let r = 1.0_f64;
    let d = d.clamp(0.0, 2.0 * r);
    let seg = r * r * acos((r - d) / r) - (r - d) * sqrt(2.0 * r * d - d * d);
    16.0 * seg + PI * d * d * (3.0 * r - d) / 3.0
}

/// Upright and level, the waterline through the tank: 600 T - 160 (T - 1) + V_tank(T - z_bottom) = mass, solved
/// here by bisection. With the tank centre at 2.8 m (bottom 1.8, top 3.8) and 1,500 t the waterline lies at
/// about 3.0 m, through the upper half of the tank.
#[test]
fn a_tank_cut_by_the_waterline_in_a_flooded_hold_follows_the_segment_closed_form() {
    let t = hull();
    let mass = 1500.0;
    let zc = 2.8;
    let f = |tt: f64| 600.0 * tt - 160.0 * (tt - 1.0) + tank_below(tt - (zc - 1.0)) - mass;
    let (mut lo, mut hi) = (1.0, 4.0);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let want = 0.5 * (lo + hi);
    assert!(
        want > zc - 1.0 && want < zc + 1.0,
        "the waterline cuts the tank: {want}"
    );
    let out = assess(
        &t,
        &[hold(vec![tank(zc)])],
        mass,
        gravity(),
        1.0,
        start(&t, mass),
        &DamageCriteria::IGC_2_7,
    );
    assert_eq!(out.floats, Floats::Yes);
    assert!(out.heel_deg.abs() < 1e-6 && out.trim_deg.abs() < 1e-6);
    assert!(
        (out.draught_mid_m - want).abs() < 1e-6,
        "{} against {want}",
        out.draught_mid_m
    );
}

fn plane_through(heel_deg: f64, trim_deg: f64, p: (f64, f64, f64)) -> Waterplane {
    let wp = Waterplane {
        heel_deg,
        trim_deg,
        c: 0.0,
    };
    let (kx, ky, kz) = wp.normal();
    Waterplane {
        c: kx * p.0 + ky * p.1 + kz * p.2,
        ..wp
    }
}

/// Whether a point is inside the tank: the cylinder 22 to 38 m, or one of the two hemispheres.
fn in_tank(x: f64, y: f64, z: f64, zc: f64) -> bool {
    let r2 = y * y + (z - zc) * (z - zc);
    if (22.0..=38.0).contains(&x) {
        return r2 <= 1.0;
    }
    let cx = if x < 22.0 { 22.0 } else { 38.0 };
    r2 + (x - cx) * (x - cx) <= 1.0
}

/// An independent grid integral: the water in the flooded hold less its tank below a heeled and trimmed
/// waterplane, counted on a midpoint grid of 0.04 m cells (an independent method: no polygon clipping, no
/// segment formula, no Gauss-Legendre), against the engine's `water_m3`, within 0.3 %.
#[test]
fn the_water_in_a_gas_hold_matches_an_independent_grid_integral() {
    let zc = 2.6;
    let v = hold(vec![tank(zc)]);
    for (heel, trim, through) in [
        (12.0, 0.0, (30.0, 0.0, 2.8)),
        (25.0, 1.5, (30.0, 0.5, 3.0)),
        (-8.0, -2.0, (28.0, -0.5, 2.2)),
    ] {
        let wp = plane_through(heel, trim, through);
        let (kx, ky, kz) = wp.normal();
        let h = 0.04;
        let (nx, ny, nz) = ((20.0 / h) as usize, (8.0 / h) as usize, (3.0 / h) as usize);
        let mut count = 0usize;
        for i in 0..nx {
            let x = 20.0 + (i as f64 + 0.5) * h;
            for j in 0..ny {
                let y = -4.0 + (j as f64 + 0.5) * h;
                for k in 0..nz {
                    let z = 1.0 + (k as f64 + 0.5) * h;
                    if kx * x + ky * y + kz * z < wp.c && !in_tank(x, y, z, zc) {
                        count += 1;
                    }
                }
            }
        }
        let grid = count as f64 * h * h * h;
        let got = v.water_m3(&wp);
        assert!(
            (got - grid).abs() / grid < 3e-3,
            "heel {heel}, trim {trim}: {got} against the grid's {grid}"
        );
    }
}

/// An intermediate stage of a one-sided flood, by added mass: a wing 25 to 35 m, 3 to 5 m from the centre line,
/// keel to deck, permeability 0.95, at half its final water w. The water fills it upright to h = w / (0.95 x 10
/// x 2), at (30, 4, h / 2); its gross free surface is 2 m wide and 10 m long, i = 10 x 2^3 / 12. With
/// mass' = 1200 + w (density 1), T = mass' / (L B), KB = T / 2, BM = B^2 / (12 T), KG' = (1200 KG + w h / 2 + i) /
/// mass' and TCG = 4 w / mass', the wall-sided lever is zero at the heel solved here; the engine's stage must
/// float there.
#[test]
fn an_intermediate_stage_of_a_one_sided_flood_follows_the_wall_sided_closed_form() {
    let t = hull();
    let mass = 1200.0;
    let wing = [FloodedVolume::from_box(
        "WB1S",
        (25.0, 35.0),
        (3.0, 5.0),
        (0.0, 4.0),
        0.95,
    )];
    let s = start(&t, mass);
    let fin = assess(&t, &wing, mass, gravity(), 1.0, s, &DamageCriteria::IGC_2_7);
    assert_eq!(fin.floats, Floats::Yes);
    let final_wp = Waterplane {
        heel_deg: fin.heel_deg,
        trim_deg: fin.trim_deg,
        c: fin.waterplane_c_m,
    };
    let stages = intermediate_stages(
        &t,
        &wing,
        &fin,
        (mass, gravity()),
        1.0,
        s,
        &DamageCriteria::IGC_2_7,
    );
    let half = &stages[1];
    let w = 0.5 * wing[0].water_m3(&final_wp);
    let h = w / (0.95 * 10.0 * 2.0);
    let i = 10.0 * 8.0 / 12.0;
    let m2 = mass + w;
    let tt = m2 / (L * B);
    let gm = tt / 2.0 + B * B / (12.0 * tt) - (mass * KG + w * h / 2.0 + i) / m2;
    let bm = B * B / (12.0 * tt);
    let tcg = 4.0 * w / m2;
    let gz = |deg: f64| {
        let p = deg.to_radians();
        sin(p) * (gm + bm * tan(p) * tan(p) / 2.0) - tcg * cos(p)
    };
    let (mut lo, mut hi) = (0.0, 20.0);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if gz(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let want = 0.5 * (lo + hi);
    assert_eq!(half.floats, Floats::Yes);
    assert!(half.trim_deg.abs() < 1e-6, "{}", half.trim_deg);
    assert!(
        (half.heel_deg - want).abs() < 1e-4,
        "{} against {want}",
        half.heel_deg
    );
}
