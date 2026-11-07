//! The damage cases of a hull (spec 0012, issue #757): one generator for every rule whose assumed damage is a
//! box, as Flooded Volumes for [`crate::damage_stability`].
//!
//! - ADN 2025 9.3.2.15 and 9.3.3.15 (issue #341, spec 0006 section 5): [`adn_damage_cases`].
//! - MARPOL Annex I regulation 28 (issue #341): [`marpol_damage_cases`].
//! - IGC Code 2.3 and 2.6 (spec 0012): [`igc_damage_cases`]. The extents ([`BoxExtents`]) and the standard of
//!   damage ([`StandardOfDamage`]) come from the Rule Set's data; `ovd-format` reads them.
//! - ADN 2025 9.3.1.15 (type G, spec 0012 D3), ES-TRIN 2025/1 28.03(4) (craft over 110 m, D2): the ADN generator
//!   with their extents ([`AdnExtents::TYPE_G`], [`AdnExtents::ES_TRIN_28_03`]) and the one-compartment standard
//!   of the main engine room ([`machinery_bulkheads_intact`]), as [`adn_damage_cases_with`].
//! - NORMAM-202/DPC 5.23.8 (spec 0012 D2, issue #763): [`normam_damage_cases`], the MARPOL extents with the
//!   standard of 5.23.8.
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
//! floods, and for a bottom damage the adjacent compartments athwartships flood too. Permeability comes from the
//! caller, from the `adn-tank` Permeability Table: 95 % in a space that is empty at the maximum draught (ballast,
//! cofferdams, pump room) and 0 % in a cargo or slop tank, which is full. A space at 0 % floods nothing and is
//! left out of the case.
//!
//! MARPOL Annex I regulation 28 has its own extents ([`BoxExtents::marpol`]): they reach the cargo tanks. A
//! damaged cargo or slop tank is flooded at 95 % (the oil is lost and sea water takes its place, regulation
//! 28.4.2 "other liquids 0 to 0.95"). Its oil leaves the loading condition (28.4.1, "any outflow of liquids
//! from damaged compartments"; spec 0012 D20, issue #764): `ovd-format` takes it out with
//! [`without_breached_cargo`].
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
    /// Type G: 9.3.1.15.1 (a) and (b), the numbers of type C. A side damage reaches 0.79 m "or, when applicable,
    /// the distance allowed by section 9.3.4, reduced by 0.01 m"; 9.3.4 is not modelled.
    pub const TYPE_G: AdnExtents = AdnExtents {
        side_inboard_m: 0.79,
        bottom_width_m: 3.00,
        bottom_height_m: 0.59,
    };
    /// ES-TRIN 2025/1 28.03(4)(a) and (b): side 0.59 m inboard, bottom 3.00 m wide and 0.39 m high.
    pub const ES_TRIN_28_03: AdnExtents = AdnExtents {
        side_inboard_m: 0.59,
        bottom_width_m: 3.00,
        bottom_height_m: 0.39,
    };
}

/// The longitudinal extent of a damage: 0.10 L, but not less than 5.00 m.
pub fn longitudinal_extent_m(loa_m: f64) -> f64 {
    let tenth = 0.10 * loa_m;
    if tenth > 5.0 { tenth } else { 5.0 }
}

/// The longitudinal extent of an ES-TRIN 28.03(4) damage: "at least 0.10 L", with no minimum in metres.
pub fn es_trin_longitudinal_extent_m(loa_m: f64) -> f64 {
    0.10 * loa_m
}

/// One damage: what it floods.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageCase {
    pub name: String,
    pub flooded: Vec<FloodedVolume>,
    /// The longitudinal extent of the damage, m from the stern (aft, forward). Zero length where a case has none.
    pub x_m: (f64, f64),
    /// The athwartships extent of a bottom damage, m (to starboard positive); none for a side damage.
    pub y_m: Option<(f64, f64)>,
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
        StandardOfDamage {
            placement: Placement::Anywhere,
            intact_bulkheads_m: if machinery_aft_intact {
                machinery_bulkheads_intact(spaces, loa_m)
            } else {
                Vec::new()
            },
        }
    }

    /// NORMAM-202/DPC 5.23.8 (c) and (d) (issue #763): anywhere in the length, every bulkhead within the damage
    /// damaged, except the end bulkheads of the main engine room, which count as one compartment.
    pub fn normam(spaces: &[Space], loa_m: f64) -> StandardOfDamage {
        StandardOfDamage {
            placement: Placement::Anywhere,
            intact_bulkheads_m: machinery_bulkheads_intact(spaces, loa_m),
        }
    }
}

/// The end bulkheads (x, m) of the main engine room, which a damage may not involve: the one-compartment
/// standard of ADN 2025 9.3.1.15.1 (c), ES-TRIN 2025/1 28.03(4)(c) and NORMAM-202/DPC 5.23.8 (d), and the
/// machinery space aft of IGC 2.6.1.3. A machinery Space is aft when its middle lies in the aft half of the hull.
pub fn machinery_bulkheads_intact(spaces: &[Space], loa_m: f64) -> Vec<f64> {
    let mut intact: Vec<f64> = Vec::new();
    for s in spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::Machinery && s.x_aft + s.x_fwd < loa_m)
    {
        intact.push(s.x_aft);
        intact.push(s.x_fwd);
    }
    intact.sort_by(f64::total_cmp);
    intact.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    intact
}

/// A compartment of the cargo zone, with the kind of the Space it comes from.
struct Compartment {
    name: String,
    kind: SpaceKind,
    x: (f64, f64),
    y: (f64, f64),
    z: (f64, f64),
    /// A Type C tank's steel in the (y, z) plane: the centre and the radius of the steel, inside the
    /// insulation. A damage breaches the tank only if it reaches this circle (issue #987).
    steel: Option<(f64, f64, f64)>,
}

impl Compartment {
    /// Whether a damage over `y` across the hull and `z` up it reaches the compartment. A plain compartment
    /// is reached when the ranges overlap its extent; a Type C tank is reached only when the damage reaches
    /// its steel, so the insulation does not count as the tank's boundary (IGC 2.4.2 measures to the shell).
    fn reached_by(&self, y: (f64, f64), z: (f64, f64)) -> bool {
        match self.steel {
            None => overlaps(self.y, y) && overlaps(self.z, z),
            Some((cy, cz, r)) => {
                let dy = (y.0 - cy).max(0.0).max(cy - y.1);
                let dz = (z.0 - cz).max(0.0).max(cz - z.1);
                dy * dy + dz * dz + 1e-9 < r * r
            }
        }
    }
}

const ANY: (f64, f64) = (f64::NEG_INFINITY, f64::INFINITY);

/// The permeability of one `SpaceKind` in a [`PermeabilityTable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Permeability {
    /// The same whatever the Loading Condition holds.
    Fixed(f64),
    /// A liquid space: the value when the space is empty, times the empty fraction of the condition's filling
    /// (IGC 2.5.2 note 2, "consistent with the amount of liquid carried"; ADN and ES-TRIN "0 or 95 %" as the
    /// space is full or empty).
    ByFilling(f64),
    /// A space whose water is the greater of `net` times its net volume (its Intact Inclusions taken out) and
    /// `floor` times its gross volume (ES-TRIN 2025/1 28.03(4)(d): a cargo hold at 95 %, not less than 70 %).
    WithGrossFloor { net: f64, floor: f64 },
}

/// A Permeability Table (spec 0012 D9): the permeability of a damaged Space by its `SpaceKind`, from a Rule
/// Set's data (`ovd-format` reads it). A complete table names every kind.
#[derive(Debug, Clone, PartialEq)]
pub struct PermeabilityTable {
    pub rows: Vec<(SpaceKind, Permeability)>,
}

impl PermeabilityTable {
    /// The permeability of a Space of `kind` filled to `filling` (0 empty, 1 full; ignored by a fixed row).
    /// `None` when the table does not name the kind.
    pub fn of(&self, kind: SpaceKind, filling: f64) -> Option<f64> {
        self.rows
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, p)| match *p {
                Permeability::Fixed(v) => v,
                Permeability::ByFilling(v) => v * (1.0 - filling.clamp(0.0, 1.0)),
                Permeability::WithGrossFloor { net, .. } => net,
            })
    }

    /// The floor on the water as a fraction of the gross volume of a Space of `kind`, where the table gives one.
    pub fn gross_floor(&self, kind: SpaceKind) -> Option<f64> {
        self.rows
            .iter()
            .find(|(k, _)| *k == kind)
            .and_then(|(_, p)| match *p {
                Permeability::WithGrossFloor { floor, .. } => Some(floor),
                _ => None,
            })
    }
}

/// The extent of a Space across and up the hull: the bounding box of its section, or for a Type C tank the
/// square that holds its outer diameter. A tank's steel is in [`Compartment::steel`].
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
        let steel = match &s.shape {
            SpaceShape::Tank {
                y,
                z,
                outer_diameter,
                insulation_m,
                ..
            } => Some((*y, *z, (0.5 * outer_diameter - insulation_m).max(0.0))),
            SpaceShape::Prism { .. } => None,
        };
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
                steel,
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
/// compartments. The damage is placed with an edge on each bulkhead and on each end of the hull. `permeability`
/// gives each compartment's permeability by its kind (ADN 9.3.2.15.1 (c), from the Rule Set's table).
pub fn adn_damage_cases(
    spaces: &[Space],
    loa_m: f64,
    beam_m: f64,
    extents: &AdnExtents,
    permeability: &dyn Fn(SpaceKind) -> f64,
) -> Vec<DamageCase> {
    adn_damage_cases_with(
        spaces,
        loa_m,
        beam_m,
        extents,
        &CaseOptions {
            long_m: longitudinal_extent_m(loa_m),
            cut: true,
            intact_bulkheads_m: &[],
        },
        permeability,
    )
}

/// How the ADN-style generator places and cuts the damage (issue #763).
#[derive(Debug, Clone, Copy)]
pub struct CaseOptions<'a> {
    /// The longitudinal extent of the damage, m: ADN 0.10 L but at least 5.00 m ([`longitudinal_extent_m`]),
    /// ES-TRIN 0.10 L ([`es_trin_longitudinal_extent_m`]).
    pub long_m: f64,
    /// Cut the ballast spaces at the cargo tank bulkheads (a liquid bulk layout); a gas hull is cut already.
    pub cut: bool,
    /// Transverse bulkheads (x, m) a damage may not involve: the end bulkheads of the main engine room.
    pub intact_bulkheads_m: &'a [f64],
}

/// [`adn_damage_cases`] with the longitudinal extent, the cut and the intact bulkheads given: the generator of
/// ADN 2025 9.3.1.15.1 (gas, type G) and ES-TRIN 2025/1 28.03(4), which differ from the liquid ADN paragraphs in
/// the extents, the longitudinal minimum and the one-compartment engine room. No window of damage spans an
/// intact bulkhead, as in [`box_damage_cases`].
pub fn adn_damage_cases_with(
    spaces: &[Space],
    loa_m: f64,
    beam_m: f64,
    extents: &AdnExtents,
    options: &CaseOptions<'_>,
    permeability: &dyn Fn(SpaceKind) -> f64,
) -> Vec<DamageCase> {
    let parts = compartments(spaces, options.cut);
    let len = options.long_m.min(loa_m);
    let intact = options.intact_bulkheads_m;
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
        if intact
            .iter()
            .any(|&k| k > window.0 + 1e-9 && k < window.1 - 1e-9)
        {
            continue;
        }
        for side in [true, false] {
            let hit: Vec<&Compartment> = parts
                .iter()
                .filter(|c| overlaps(c.x, window))
                .filter(|c| {
                    if side {
                        // starboard: the strip inboard of the side, from the base line up without limit
                        c.reached_by((half - extents.side_inboard_m, half), ANY)
                    } else {
                        // bottom: below the height of the damage; athwartships neighbours flood too
                        c.reached_by(ANY, (f64::NEG_INFINITY, extents.bottom_height_m))
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
                x_m: window,
                y_m: None,
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

/// The damage cases of NORMAM-202/DPC 5.23.8 on a liquid bulk hull (spec 0012 D2, issue #763): the box
/// generator with the extents of 5.23.8 (a) and (b), which are those of MARPOL Annex I regulation 28.2
/// ([`BoxExtents::marpol`]), under the standard of 5.23.8 (c) and (d) ([`StandardOfDamage::normam`]): anywhere in
/// the length, the end bulkheads of the main engine room intact. The permeability is the caller's, from the Rule
/// Set's table (5.23.8 (f)): a full cargo tank is 0 % and floods nothing, so it is left out of the case. The
/// lesser damage of 5.23.8 (g) is not generated.
pub fn normam_damage_cases(
    spaces: &[Space],
    loa_m: f64,
    beam_m: f64,
    permeability: &dyn Fn(SpaceKind) -> f64,
) -> Vec<DamageCase> {
    box_damage_cases(
        spaces,
        loa_m,
        beam_m,
        &BoxExtents::marpol(loa_m, beam_m),
        &StandardOfDamage::normam(spaces, loa_m),
        true,
        permeability,
    )
    .into_iter()
    .filter_map(|mut c| {
        c.flooded.retain(|f| f.permeability > 0.0);
        (!c.flooded.is_empty()).then_some(c)
    })
    .fold(Vec::<DamageCase>::new(), |mut out, c| {
        // Leaving the full tanks out can make two cases alike: keep the first.
        let key = |c: &DamageCase| {
            let mut names: Vec<&str> = c.flooded.iter().map(|f| f.name.as_str()).collect();
            names.sort_unstable();
            (c.name.starts_with("side"), names.join("\u{1}"))
        };
        let k = key(&c);
        if !out.iter().any(|o| key(o) == k) {
            out.push(c);
        }
        out
    })
}

/// NORMAM-202/DPC 5.23.9 (c): a final heel of at most 25 degrees, "podendo ser aceita uma inclinação de até 30°,
/// caso não ocorra imersão do convés" (up to 30 degrees accepted when the deck is not immersed). `ok_to_30` is the
/// verdict with the 30 degree limit ([`crate::damage_stability::DamageCriteria::NORMAM_5_23_9`]).
pub fn normam_final_heel_ok(ok_to_30: bool, heel_deg: f64, deck_immersed: bool) -> bool {
    ok_to_30 && (heel_deg.abs() <= 25.0 || !deck_immersed)
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
    let mut push = |side: bool,
                    name: String,
                    x_m: (f64, f64),
                    y_m: Option<(f64, f64)>,
                    hit: Vec<&Compartment>| {
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
            x_m,
            y_m,
        });
    };
    for (a, b) in windows(ext.side_long_m) {
        let hit: Vec<&Compartment> = parts
            .iter()
            .filter(|c| {
                overlaps(c.x, (a, b)) && c.reached_by((half - ext.side_inboard_m, half), ANY)
            })
            .collect();
        push(
            true,
            format!("side damage (starboard), {a:.1} to {b:.1} m"),
            (a, b),
            None,
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
                            && c.reached_by((y0, y1), (f64::NEG_INFINITY, ext.bottom_height_m))
                    })
                    .collect();
                push(
                    false,
                    format!("bottom damage, {a:.1} to {b:.1} m, {y0:.1} to {y1:.1} m across"),
                    (a, b),
                    Some((y0, y1)),
                    hit,
                );
            }
        }
    }
    cases
}

/// The Flooded Volumes of one IGC damage case on a gas hull, and the Type C tanks it breaches (spec 0012, D7 to
/// D9).
#[derive(Debug, Clone, PartialEq)]
pub struct GasFlooding {
    pub flooded: Vec<FloodedVolume>,
    /// The names of the breached Type C tanks: their cargo leaves the condition (IGC 2.5.3, D8).
    pub breached: Vec<String>,
}

/// The Flooded Volumes of `case` on the Spaces of a gas hull.
///
/// Each flooded Space floods at its own shape. A hold keeps every Type C tank inside it as an Intact Inclusion
/// (D7): the hold's water does not fill the tank. A breached tank also floods on its own: its contents are lost
/// and sea water takes their place (IGC 2.5.3), so it takes the table's value for an empty space. Any other space
/// takes the table's value at its filling (`filling` by Space name, 0 empty to 1 full; IGC 2.5.2 note 2). A
/// kind the table does not name floods at 0.95, the IGC value for a void.
pub fn gas_flooding(
    case: &DamageCase,
    spaces: &[Space],
    table: &PermeabilityTable,
    filling: &dyn Fn(&str) -> f64,
) -> GasFlooding {
    const UNNAMED: f64 = 0.95;
    let tanks: Vec<&Space> = spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::CargoTank && matches!(s.shape, SpaceShape::Tank { .. }))
        .collect();
    let mut flooded = Vec::new();
    let mut breached = Vec::new();
    for f in &case.flooded {
        let Some(space) = spaces.iter().find(|s| s.name == f.name) else {
            continue;
        };
        if space.kind == SpaceKind::CargoTank {
            breached.push(space.name.clone());
            let p = table.of(space.kind, 0.0).unwrap_or(UNNAMED);
            flooded.push(FloodedVolume::of_space(space, p));
            continue;
        }
        let p = table
            .of(space.kind, filling(&space.name))
            .unwrap_or(UNNAMED);
        let mut v = FloodedVolume::of_space(space, p);
        if space.kind == SpaceKind::CargoHold {
            v.inclusions = tanks
                .iter()
                .filter(|t| t.x_aft >= space.x_aft - 1e-9 && t.x_fwd <= space.x_fwd + 1e-9)
                .map(|t| crate::damage_stability::IntactInclusion::of_space(t))
                .collect();
            v.gross_floor = table.gross_floor(space.kind);
        }
        flooded.push(v);
    }
    GasFlooding { flooded, breached }
}

/// The mass and centre of gravity of a condition at the fraction `fraction` (0 to 1) of the flood, when the
/// breached tanks have let out that fraction of their cargo (issue #988, spec 0012 D18): [`without_breached_cargo`]
/// with `fraction` of the cargo. At 0 nothing has left; at 1 it is [`without_breached_cargo`]. A breached tank
/// then holds sea water to the same fraction of its flood and its cargo for the rest.
pub fn breached_cargo_at_stage(
    fraction: f64,
    mass_t: f64,
    g: crate::sections::Gravity,
    (cargo_t, cargo_kg_m): (f64, f64),
    spaces: &[Space],
    breached: &[String],
) -> (f64, crate::sections::Gravity) {
    without_breached_cargo(
        mass_t,
        g,
        (cargo_t * fraction.clamp(0.0, 1.0), cargo_kg_m),
        spaces,
        breached,
    )
}

/// The mass and centre of gravity of a condition after the cargo of the breached tanks leaves it (IGC 2.5.3,
/// spec 0012 D8; MARPOL Annex I 28.4.1, D20). The cargo of `cargo_t` at height `cargo_kg_m` is shared between
/// the cargo tanks by their volume (a Type C tank's interior volume, a prism tank's volume: the tanks are
/// filled alike); each breached tank's share leaves from the middle of its length, at the centre of its
/// section across the hull.
pub fn without_breached_cargo(
    mass_t: f64,
    g: crate::sections::Gravity,
    (cargo_t, cargo_kg_m): (f64, f64),
    spaces: &[Space],
    breached: &[String],
) -> (f64, crate::sections::Gravity) {
    let volume = |s: &Space| match s.shape {
        SpaceShape::Tank {
            inner_volume_m3, ..
        } => inner_volume_m3,
        SpaceShape::Prism { .. } => s.volume_m3(),
    };
    let total: f64 = spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::CargoTank)
        .map(volume)
        .sum();
    if total <= 0.0 || cargo_t <= 0.0 {
        return (mass_t, g);
    }
    let (mut m, mut mx, mut my, mut mz) =
        (mass_t, mass_t * g.lcg_m, mass_t * g.tcg_m, mass_t * g.kg_m);
    for s in spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::CargoTank && breached.contains(&s.name))
    {
        let share = cargo_t * volume(s) / total;
        let y = match &s.shape {
            SpaceShape::Tank { y, .. } => *y,
            SpaceShape::Prism { section } => crate::box_stability::area_centroid(section).1,
        };
        m -= share;
        mx -= share * 0.5 * (s.x_aft + s.x_fwd);
        my -= share * y;
        mz -= share * cargo_kg_m;
    }
    if m <= 0.0 {
        return (mass_t, g);
    }
    (
        m,
        crate::sections::Gravity {
            lcg_m: mx / m,
            tcg_m: my / m,
            kg_m: mz / m,
        },
    )
}

/// Whether the deck edge is under the waterplane anywhere along `x` (m): the deck at `depth_m`, at the side
/// `half_beam_m` from the centre line, at both ends and both sides (the deck-edge test of spec 0012 D11).
pub fn deck_edge_immersed(
    wp: &crate::sections::Waterplane,
    x: (f64, f64),
    half_beam_m: f64,
    depth_m: f64,
) -> bool {
    let (kx, ky, kz) = wp.normal();
    [x.0, x.1].iter().any(|&xx| {
        [-half_beam_m, half_beam_m]
            .iter()
            .any(|&y| kx * xx + ky * y + kz * depth_m < wp.c)
    })
}
