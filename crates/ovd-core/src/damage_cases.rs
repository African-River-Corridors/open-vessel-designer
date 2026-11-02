//! The damage cases of a hull (spec 0012, issue #757): one generator for every rule whose assumed damage is a
//! box, as Flooded Volumes for [`crate::damage_stability`].
//!
//! - ADN 2025 9.3.2.15 and 9.3.3.15 (issue #341, spec 0006 section 5): [`adn_damage_cases`].
//! - MARPOL Annex I regulation 28 (issue #341): [`marpol_damage_cases`].
//! - IGC Code 2.3 and 2.6 (spec 0012): [`igc_damage_cases`]. The extents ([`BoxExtents`]) and the standard of
//!   damage ([`StandardOfDamage`]) come from the Rule Set's data; `ovd-format` reads them.
//!
//! The cargo zone is the Spaces. On a liquid bulk layout a ballast space (a double bottom or a wing tank) is
//! cut at the transverse bulkheads of the cargo tanks beside it, so a damage floods one compartment of it, not
//! the whole length of the tank block: the cut is an assumption of this model (the Spaces hold each ballast
//! tank as one prism). A gas hull is already cut at its hold bulkheads (spec 0012 D6), so the IGC generator
//! takes its Spaces as they are. A Type C tank reaches across and up the hull as far as its outer diameter. A
//! cofferdam and a pump room are single compartments. A space that is not among the Spaces floods nothing.
//!
//! ADN. The extents are those of the ADN text: a longitudinal extent of 0.10 L but not less than 5.00 m,
//! placed anywhere along the hull; a side damage 0.79 m (type C, 9.3.2.15.1) or 0.59 m (type N, 9.3.3.15.1)
//! inboard of the side, from the base line up without limit; a bottom damage 3.00 m wide and 0.59 m (type C)
//! or 0.49 m (type N) high. A bulkhead within the damage is damaged, so every compartment the extent touches
//! floods, and for a bottom damage the adjacent compartments athwartships flood too. Permeability is 95 % in a
//! space that is empty at the maximum draught (ballast, cofferdams, pump room) and 0 % in a cargo or slop
//! tank, which is full.
//!
//! MARPOL Annex I regulation 28 has its own extents ([`BoxExtents::marpol`]): they reach the cargo tanks. A
//! damaged cargo or slop tank is flooded at 95 % (the oil is lost and sea water takes its place, regulation
//! 28.4.2 "other liquids 0 to 0.95"); the loading condition keeps the mass of the oil, as the lost buoyancy
//! method keeps the mass of the damaged hull.
//!
//! IGC. The extents of IGC 2.3.1 have the shape of MARPOL's (side: (1/3) L^(2/3) or 14.5 m long, B/5 or
//! 11.5 m inboard, without vertical limit; bottom: B/6 or 10 m wide in the forward 0.3 L and B/6 or 5 m
//! elsewhere), but the bottom damage is B/15 or 2 m high (2.3.1.2.3), not 6 m. A type 2G ship is damaged
//! anywhere in its length (2.6.1.2); at 150 m or less, not involving either bulkhead of a machinery space aft
//! (2.6.1.3). `L` is the overall length here, as on the MARPOL path (IGC 1.2.31 names the load line length).
//! The lesser damage of 2.3.2.1 and the local damage of 2.3.2.2 are not generated.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::compartments::{Space, SpaceKind, SpaceShape};
use crate::damage_stability::FloodedVolume;

/// The extents of the assumed damage (ADN 2025 9.3.2.15.1 and 9.3.3.15.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdnExtents {
    /// Side damage: how far inboard of the side, m.
    pub side_inboard_m: f64,
    /// Bottom damage: how wide, m.
    pub bottom_width_m: f64,
    /// Bottom damage: how high above the base line, m.
    pub bottom_height_m: f64,
}

impl AdnExtents {
    /// Type C: 9.3.2.15.1.
    pub const TYPE_C: AdnExtents = AdnExtents {
        side_inboard_m: 0.79,
        bottom_width_m: 3.00,
        bottom_height_m: 0.59,
    };
    /// Type N: 9.3.3.15.1.
    pub const TYPE_N: AdnExtents = AdnExtents {
        side_inboard_m: 0.59,
        bottom_width_m: 3.00,
        bottom_height_m: 0.49,
    };
}

/// The longitudinal extent of a damage: 0.10 L, but not less than 5.00 m.
pub fn longitudinal_extent_m(loa_m: f64) -> f64 {
    let tenth = 0.10 * loa_m;
    if tenth > 5.0 { tenth } else { 5.0 }
}

/// One damage: what it floods.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageCase {
    pub name: String,
    pub flooded: Vec<FloodedVolume>,
}

/// The extents of a box damage with a side and a bottom set, the bottom set in two zones: MARPOL Annex I
/// regulation 28.2 and IGC 2.3.1 have this shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxExtents {
    /// Side damage: longitudinal extent, m (MARPOL 28.2.1.1, IGC 2.3.1.1.1).
    pub side_long_m: f64,
    /// Side damage: how far inboard of the side, m (MARPOL 28.2.1.2, IGC 2.3.1.1.2).
    pub side_inboard_m: f64,
    /// Bottom damage, forward zone: longitudinal extent, m (MARPOL 28.2.2.1, IGC 2.3.1.2.1).
    pub bottom_fwd_long_m: f64,
    /// Bottom damage, forward zone: width, m (MARPOL 28.2.2.2, IGC 2.3.1.2.2).
    pub bottom_fwd_width_m: f64,
    /// Bottom damage, any other part: longitudinal extent, m.
    pub bottom_aft_long_m: f64,
    /// Bottom damage, any other part: width, m.
    pub bottom_aft_width_m: f64,
    /// Bottom damage: height from the moulded line of the bottom shell, m (MARPOL 28.2.2.3, IGC 2.3.1.2.3).
    pub bottom_height_m: f64,
    /// The forward zone of the bottom damage, as a fraction of L from the forward end: 0.3 in both texts.
    pub forward_zone: f64,
}

/// The extents of MARPOL Annex I regulation 28.2 (the name before spec 0012).
pub type MarpolExtents = BoxExtents;

impl BoxExtents {
    /// The extents of MARPOL Annex I regulation 28.2, for a hull of length L and beam B.
    ///
    /// Side: `(1/3) L^(2/3)` or 14.5 m long (28.2.1.1), `B/5` or 11.5 m inboard (28.2.1.2). Bottom, forward
    /// 0.3 L: `(1/3) L^(2/3)` or 14.5 m long, `B/6` or 10 m wide; any other part: `(1/3) L^(2/3)` or 5 m long,
    /// `B/6` or 5 m wide (28.2.2.1, 28.2.2.2); `B/15` or 6 m high (28.2.2.3).
    pub fn marpol(loa_m: f64, beam_m: f64) -> BoxExtents {
        let cbrt = libm::cbrt(loa_m);
        let third_l23 = cbrt * cbrt / 3.0;
        BoxExtents {
            side_long_m: third_l23.min(14.5),
            side_inboard_m: (beam_m / 5.0).min(11.5),
            bottom_fwd_long_m: third_l23.min(14.5),
            bottom_fwd_width_m: (beam_m / 6.0).min(10.0),
            bottom_aft_long_m: third_l23.min(5.0),
            bottom_aft_width_m: (beam_m / 6.0).min(5.0),
            bottom_height_m: (beam_m / 15.0).min(6.0),
            forward_zone: 0.3,
        }
    }
}

/// Where along the hull a box damage may lie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Anywhere in the length: every bulkhead within the damage is damaged.
    Anywhere,
    /// Between adjacent transverse bulkheads (MARPOL 28.1.3): a bulkhead is damaged only where it is next to a
    /// bay shorter than the damage (28.2.5.1).
    BetweenBulkheads,
}

/// The Standard of Damage: where a damage may lie, and the bulkheads it may not involve.
#[derive(Debug, Clone, PartialEq)]
pub struct StandardOfDamage {
    pub placement: Placement,
    /// Transverse bulkheads (x, m) a damage may not involve: no damage spans one.
    pub intact_bulkheads_m: Vec<f64>,
}

impl StandardOfDamage {
    /// MARPOL Annex I regulation 28.1: anywhere in the length above 150 m (28.1.1; the machinery space of 28.1.2
    /// is not modelled), between adjacent transverse bulkheads at 150 m or less (28.1.3).
    pub fn marpol(loa_m: f64) -> StandardOfDamage {
        StandardOfDamage {
            placement: if loa_m <= BETWEEN_BULKHEADS_UP_TO_M {
                Placement::BetweenBulkheads
            } else {
                Placement::Anywhere
            },
            intact_bulkheads_m: Vec::new(),
        }
    }

    /// IGC 2.6.1.2 and 2.6.1.3, a type 2G ship: anywhere in its length; when `machinery_aft_intact` (150 m or
    /// less, a value the Rule Set gives), not involving either bulkhead of a machinery space aft. A machinery
    /// Space is aft when its middle lies in the aft half of the hull.
    pub fn igc_2g(spaces: &[Space], loa_m: f64, machinery_aft_intact: bool) -> StandardOfDamage {
        let mut intact: Vec<f64> = Vec::new();
        if machinery_aft_intact {
            for s in spaces
                .iter()
                .filter(|s| s.kind == SpaceKind::Machinery && s.x_aft + s.x_fwd < loa_m)
            {
                intact.push(s.x_aft);
                intact.push(s.x_fwd);
            }
        }
        intact.sort_by(f64::total_cmp);
        intact.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        StandardOfDamage {
            placement: Placement::Anywhere,
            intact_bulkheads_m: intact,
        }
    }
}

/// A compartment of the cargo zone, with the kind of the Space it comes from.
struct Compartment {
    name: String,
    kind: SpaceKind,
    x: (f64, f64),
    y: (f64, f64),
    z: (f64, f64),
}

/// The 0.95 of ADN 9.3.2.15.1 for a space that is empty at the maximum draught, and 0 for a full one.
fn permeability(kind: SpaceKind) -> f64 {
    match kind {
        SpaceKind::CargoTank | SpaceKind::Slop => 0.0,
        _ => 0.95,
    }
}

/// The extent of a Space across and up the hull: the bounding box of its section, or for a Type C tank the
/// square that holds its outer diameter.
fn bbox(s: &Space) -> ((f64, f64), (f64, f64)) {
    if let SpaceShape::Tank {
        y,
        z,
        outer_diameter,
        ..
    } = &s.shape
    {
        let r = 0.5 * outer_diameter;
        return ((y - r, y + r), (z - r, z + r));
    }
    let (mut y0, mut y1, mut z0, mut z1) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for &(y, z) in s.section() {
        y0 = y0.min(y);
        y1 = y1.max(y);
        z0 = z0.min(z);
        z1 = z1.max(z);
    }
    ((y0, y1), (z0, z1))
}

/// The compartments: every Space, with each ballast space cut at the transverse bulkheads of the cargo
/// tanks when `cut` is set (a liquid bulk layout holds each ballast tank as one prism; a gas hull is already
/// cut at its hold bulkheads, spec 0012 D6).
fn compartments(spaces: &[Space], cut: bool) -> Vec<Compartment> {
    let mut bulkheads: Vec<f64> = spaces
        .iter()
        .filter(|s| matches!(s.kind, SpaceKind::CargoTank | SpaceKind::Slop))
        .flat_map(|s| [s.x_aft, s.x_fwd])
        .collect();
    bulkheads.sort_by(f64::total_cmp);
    bulkheads.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let mut out = Vec::new();
    for s in spaces {
        let (y, z) = bbox(s);
        let cuts = cut
            && matches!(
                s.kind,
                SpaceKind::DoubleBottomBallast | SpaceKind::WingBallast | SpaceKind::PeakBallast
            );
        let mut edges = alloc::vec![s.x_aft];
        if cuts {
            edges.extend(
                bulkheads
                    .iter()
                    .copied()
                    .filter(|&b| b > s.x_aft + 1e-9 && b < s.x_fwd - 1e-9),
            );
        }
        edges.push(s.x_fwd);
        for (i, w) in edges.windows(2).enumerate() {
            out.push(Compartment {
                name: if edges.len() > 2 {
                    format!("{}/{}", s.name, i + 1)
                } else {
                    s.name.clone()
                },
                kind: s.kind,
                x: (w[0], w[1]),
                y,
                z,
            });
        }
    }
    out
}

fn overlaps(a: (f64, f64), b: (f64, f64)) -> bool {
    a.0.max(b.0) + 1e-9 < a.1.min(b.1)
}

/// The damage cases of a layout: the side damage to starboard (the hull is symmetric, so port is its
/// mirror) and the bottom damage, each at every position along the hull that floods a different set of
/// compartments. The damage is placed with an edge on each bulkhead and on each end of the hull.
pub fn adn_damage_cases(
    spaces: &[Space],
    loa_m: f64,
    beam_m: f64,
    extents: &AdnExtents,
) -> Vec<DamageCase> {
    let parts = compartments(spaces, true);
    let len = longitudinal_extent_m(loa_m).min(loa_m);
    let mut edges: Vec<f64> = parts.iter().flat_map(|c| [c.x.0, c.x.1]).collect();
    edges.push(0.0);
    edges.push(loa_m);
    // The damage starts at every edge, ends at every edge, and, to flood the compartments on both sides of a
    // bulkhead, starts or ends just beyond it.
    const BEYOND_M: f64 = 1e-3;
    let mut starts: Vec<f64> = edges
        .iter()
        .flat_map(|&e| [e, e - len, e - BEYOND_M, e - len + BEYOND_M])
        .map(|s| s.clamp(0.0, loa_m - len))
        .collect();
    starts.sort_by(f64::total_cmp);
    starts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let half = 0.5 * beam_m;
    let mut cases: Vec<DamageCase> = Vec::new();
    for &s0 in &starts {
        let window = (s0, s0 + len);
        for side in [true, false] {
            let hit: Vec<&Compartment> = parts
                .iter()
                .filter(|c| overlaps(c.x, window))
                .filter(|c| {
                    if side {
                        // starboard: the strip inboard of the side, from the base line up without limit
                        overlaps(c.y, (half - extents.side_inboard_m, half))
                    } else {
                        // bottom: below the height of the damage; athwartships neighbours flood too
                        c.z.0 + 1e-9 < extents.bottom_height_m
                    }
                })
                .filter(|c| permeability(c.kind) > 0.0)
                .collect();
            if hit.is_empty() {
                continue;
            }
            let mut names: Vec<&str> = hit.iter().map(|c| c.name.as_str()).collect();
            names.sort_unstable();
            if cases.iter().any(|k| {
                let mut other: Vec<&str> = k.flooded.iter().map(|f| f.name.as_str()).collect();
                other.sort_unstable();
                other == names && k.name.starts_with(if side { "side" } else { "bottom" })
            }) {
                continue;
            }
            cases.push(DamageCase {
                name: format!(
                    "{} damage, {:.1} to {:.1} m",
                    if side { "side (starboard)" } else { "bottom" },
                    window.0,
                    window.1
                ),
                flooded: hit
                    .iter()
                    .map(|c| FloodedVolume::from_box(&c.name, c.x, c.y, c.z, permeability(c.kind)))
                    .collect(),
            });
        }
    }
    cases
}

/// The length from which regulation 28.1.1 and 28.1.2 put the damage anywhere in the ship's length (above
/// 150 m). At 150 m or less, regulation 28.1.3 puts it between adjacent transverse bulkheads.
const BETWEEN_BULKHEADS_UP_TO_M: f64 = 150.0;

/// The damage cases of MARPOL Annex I regulation 28 (issue #341): the side damage to starboard and the bottom
/// damage at each position along and across the hull that floods a different set of compartments, every
/// damaged compartment at 95 %. See [`box_damage_cases`] and [`StandardOfDamage::marpol`].
pub fn marpol_damage_cases(spaces: &[Space], loa_m: f64, beam_m: f64) -> Vec<DamageCase> {
    box_damage_cases(
        spaces,
        loa_m,
        beam_m,
        &BoxExtents::marpol(loa_m, beam_m),
        &StandardOfDamage::marpol(loa_m),
        true,
        &|_| 0.95,
    )
}

/// The damage cases of IGC 2.3 under the standard of damage of 2.6.1 (spec 0012): the box generator on the
/// Spaces as they are (a gas hull is cut at its hold bulkheads), with the extents, standard and permeability
/// the caller reads from the Rule Set.
pub fn igc_damage_cases(
    spaces: &[Space],
    loa_m: f64,
    beam_m: f64,
    extents: &BoxExtents,
    standard: &StandardOfDamage,
    permeability: &dyn Fn(SpaceKind) -> f64,
) -> Vec<DamageCase> {
    box_damage_cases(
        spaces,
        loa_m,
        beam_m,
        extents,
        standard,
        false,
        permeability,
    )
}

/// The damage cases of a box damage with a side and a bottom set: the side damage to starboard and the bottom
/// damage at each position along and across the hull that floods a different set of compartments.
///
/// Placement. The damage starts and ends on each bulkhead and each end of the hull, and just beyond each
/// bulkhead, so the compartments on both sides of it flood. [`Placement::BetweenBulkheads`] keeps only the
/// windows whose bulkheads are next to a bay shorter than the damage (MARPOL 28.2.5.1; 28.2.5.2, the step or
/// recess, is not modelled). No window spans an intact bulkhead of the standard. A bottom damage is in the
/// forward set when its centre is in the forward zone, which takes `L` as the overall length. The bottom
/// damage lies across the beam wherever it floods a different set, from the centre line to starboard (the
/// hull is symmetric). `cut` cuts the ballast spaces at the cargo tank bulkheads (a liquid bulk layout).
pub fn box_damage_cases(
    spaces: &[Space],
    loa_m: f64,
    beam_m: f64,
    ext: &BoxExtents,
    standard: &StandardOfDamage,
    cut: bool,
    permeability: &dyn Fn(SpaceKind) -> f64,
) -> Vec<DamageCase> {
    let parts = compartments(spaces, cut);
    let half = 0.5 * beam_m;
    let mut bulkheads: Vec<f64> = parts
        .iter()
        .flat_map(|c| [c.x.0, c.x.1])
        .filter(|&e| e > 1e-9 && e < loa_m - 1e-9)
        .collect();
    bulkheads.sort_by(f64::total_cmp);
    bulkheads.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let mut bounds = alloc::vec![0.0];
    bounds.extend(bulkheads.iter().copied());
    bounds.push(loa_m);
    let between = standard.placement == Placement::BetweenBulkheads;
    let intact = &standard.intact_bulkheads_m;
    // The windows of one longitudinal extent: with an edge on each bulkhead and on each end of the hull, and
    // just beyond each bulkhead, so the compartments on both sides of it flood.
    let windows = |len: f64| -> Vec<(f64, f64)> {
        let len = len.min(loa_m);
        const BEYOND_M: f64 = 1e-3;
        let mut starts: Vec<f64> = bounds
            .iter()
            .flat_map(|&e| [e, e - len, e - BEYOND_M, e - len + BEYOND_M])
            .map(|s| s.clamp(0.0, loa_m - len))
            .collect();
        starts.sort_by(f64::total_cmp);
        starts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        starts
            .into_iter()
            .map(|s| (s, s + len))
            .filter(|&(a, b)| !intact.iter().any(|&k| k > a + 1e-9 && k < b - 1e-9))
            .filter(|&(a, b)| {
                !between
                    || bulkheads
                        .iter()
                        .filter(|&&k| k > a + 1e-9 && k < b - 1e-9)
                        .all(|&k| {
                            // 28.2.5.1: damaged only next to a bay shorter than the damage
                            let i = bounds
                                .iter()
                                .position(|&e| (e - k).abs() < 1e-9)
                                .unwrap_or(0);
                            let before = bounds[i] - bounds[i.saturating_sub(1)];
                            let after = bounds[(i + 1).min(bounds.len() - 1)] - bounds[i];
                            before < len - 1e-9 || after < len - 1e-9
                        })
            })
            .collect()
    };
    let flooded_of = |hit: &[&Compartment]| -> Vec<FloodedVolume> {
        hit.iter()
            .map(|c| FloodedVolume::from_box(&c.name, c.x, c.y, c.z, permeability(c.kind)))
            .collect()
    };
    let mut cases: Vec<DamageCase> = Vec::new();
    let mut seen: Vec<(bool, Vec<String>)> = Vec::new();
    let mut push = |side: bool, name: String, hit: Vec<&Compartment>| {
        if hit.is_empty() {
            return;
        }
        let mut names: Vec<String> = hit.iter().map(|c| c.name.clone()).collect();
        names.sort_unstable();
        if seen.iter().any(|(s, n)| *s == side && *n == names) {
            return;
        }
        seen.push((side, names));
        cases.push(DamageCase {
            name,
            flooded: flooded_of(&hit),
        });
    };
    for (a, b) in windows(ext.side_long_m) {
        let hit: Vec<&Compartment> = parts
            .iter()
            .filter(|c| overlaps(c.x, (a, b)) && overlaps(c.y, (half - ext.side_inboard_m, half)))
            .collect();
        push(
            true,
            format!("side damage (starboard), {a:.1} to {b:.1} m"),
            hit,
        );
    }
    // The forward set and the other set differ in length and width; a window belongs to the set its centre is in.
    let forward_from = (1.0 - ext.forward_zone) * loa_m;
    for (forward, long_m, width_m) in [
        (true, ext.bottom_fwd_long_m, ext.bottom_fwd_width_m),
        (false, ext.bottom_aft_long_m, ext.bottom_aft_width_m),
    ] {
        let mut y_edges: Vec<f64> = parts.iter().flat_map(|c| [c.y.0, c.y.1]).collect();
        y_edges.push(-half);
        y_edges.push(half);
        const BEYOND_M: f64 = 1e-3;
        let mut y_starts: Vec<f64> = y_edges
            .iter()
            .flat_map(|&e| [e, e - width_m, e - BEYOND_M, e - width_m + BEYOND_M])
            .map(|y| y.clamp(-half, half - width_m))
            .filter(|&y| y + 0.5 * width_m >= -1e-9)
            .collect();
        y_starts.sort_by(f64::total_cmp);
        y_starts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        for (a, b) in windows(long_m) {
            if (0.5 * (a + b) >= forward_from - 1e-9) != forward {
                continue;
            }
            for &y0 in &y_starts {
                let y1 = y0 + width_m;
                let hit: Vec<&Compartment> = parts
                    .iter()
                    .filter(|c| {
                        overlaps(c.x, (a, b))
                            && overlaps(c.y, (y0, y1))
                            && c.z.0 + 1e-9 < ext.bottom_height_m
                    })
                    .collect();
                push(
                    false,
                    format!("bottom damage, {a:.1} to {b:.1} m, {y0:.1} to {y1:.1} m across"),
                    hit,
                );
            }
        }
    }
    cases
}
