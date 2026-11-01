//! The plate model of a hull (issue #742, spec 0013 "Model", `hull_plates` row; D2, D10 to D12).
//!
//! **Concept and preliminary design only (ADR 0011). This is not a structural approval.** The plate
//! model re-cuts the panel model (`hull_panels`): it adds no scantling. Each panel is split into
//! plates by seams across its girth at the stock plate width and butts along its length at the
//! stock plate length, each kept [`WELD_CLEARANCE_M`] clear of a longitudinal or a frame (offset
//! from it, spec 0013 "Model"). Each stiffener is one trace line, one per spacing, as the panel model counts them. Each
//! primary member is one item per frame: a plated one (a floor, a web of the double side) or a
//! profile (a side web, a deck transverse, a vertical web of a bulkhead).
//!
//! Each plate's rule-net thickness is its panel's; the ordered thickness is it rounded up to the
//! `thickness_step_mm` of the `structure` block (D10). The scrap is counted, not factored (D12): each
//! piece is cut from stock plates of the `structure` block's size (like pieces of one panel side by
//! side, as many to a stock plate as fit; no nesting, D3), so the scrap is the stock area bought less
//! the net plate area. Brackets, collars and welds stay the panel model's named
//! allowance (an Estimate, D2). Lightening holes are not deducted (D13): the list errs heavy.
//!
//! Without a Sections Table ([`hull_plates`]) the geometry of a panel is its panel-equivalent
//! rectangle: the panel's area as a length times a girth, placed at the panel's position. It sets
//! each item's centre and the 3D view; the weights are the panel model's whatever the placing.
//!
//! With the hull's Sections Table ([`hull_plates_on`], issue #743, spec 0013 "Ends", D6) the shell
//! follows the table, one strip per Station interval, so the rakes, bilges, sheer, camber and Stern
//! Forms carry real plate, and the end faces of the hull are plated; the transverse bulkheads
//! (aft peak, holds, collision), the floors, the inner bottom and the girders take the hull's
//! section where they stand ([`crate::hull_shell`]). The scantlings stay the panel model's. The plate
//! model then differs from the panel table at the ends: the lightship reads the panel table until
//! spec 0013, T7.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use libm::{ceil, floor};

use crate::hull_panels::{
    HullPanelModel, HullStructureSpec, PanelMember, Section, function_type as ft, region,
};
use crate::mesh::PartSource;
use crate::sections::SectionsTable;
use crate::vessel_model::{SteelGrade, StructureBlock};

/// What the steel list says about the openings (spec 0013, "Openings", D13).
pub const OPENINGS_NOTE: &str = "Lightening holes, access holes and other small openings are not \
deducted: the list errs heavy. The large openings the model holds (the deck cut-outs for the \
tanks) are deducted.";

/// Two lengths closer than this (m) are one.
pub(crate) const EPS: f64 = 1e-9;

/// `a > b`, false for NaN.
fn above(a: f64, b: f64) -> bool {
    a > b
}

/// What a steel list item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    /// A plate of a panel, between seams and butts.
    Plate,
    /// A stiffener: one trace line with a bar section.
    Stiffener,
    /// A primary member: a plated one (a floor, a web of the double side) or a profile.
    Primary,
}

impl ItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plate => "plate",
            Self::Stiffener => "stiffener",
            Self::Primary => "primary",
        }
    }
}

/// A point in hull axes (m): x forward from the aft end, y to starboard, z up from the keel.
pub type Point = (f64, f64, f64);

/// The geometry of an item: a plate's outline in order round it (four corners for a flat plate on a
/// panel-equivalent rectangle; on the Sections Table, issue #743, a curved plate's outline holds the
/// knuckles of the strip too, and a plate cut from a hull section its clipped polygon), or a trace
/// line's two ends (a stiffener, a profile primary member). Outside the results hash.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub points: Vec<Point>,
}

/// One row of the steel list.
#[derive(Debug, Clone, PartialEq)]
pub struct SteelItem {
    /// `PL1`, `ST1`, `PM1`, ... in the order of the panel model's members.
    pub id: String,
    pub kind: ItemKind,
    /// Where the item comes from (spec 0009, "no geometry without a source"): the hull of the Vessel
    /// Model. The 3D Structure layer (spec 0013, T8) may name it finer.
    pub source: PartSource,
    /// The panel's OCX `functionType` and region (`hull_panels`).
    pub member: String,
    pub region: String,
    pub grade: SteelGrade,
    /// The rule-net and ordered plate thickness (mm); 0 for a profile.
    pub rule_net_mm: f64,
    pub ordered_mm: f64,
    /// The bar section of a stiffener or a profile primary member.
    pub section: Option<String>,
    /// A plate's length and width, or a trace line's length (width 0) (m).
    pub length_m: f64,
    pub width_m: f64,
    /// The net plate area (m2); 0 for a profile.
    pub area_m2: f64,
    /// The pieces a plated item is cut in: one for a plate, a plated primary member's pieces.
    pub pieces: usize,
    /// Weight at the rule-net and at the ordered thickness (t); a profile's are equal.
    pub rule_net_t: f64,
    pub ordered_t: f64,
    pub centre_m: Point,
    pub geometry: Geometry,
    /// The rule citation of the panel, its Rule Set, edition and first paragraph, and basis.
    pub rule: String,
    pub rule_set: &'static str,
    pub edition: &'static str,
    pub paragraph: String,
    pub basis: &'static str,
}

/// The weight and centre of a member group (one panel of the panel model).
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub member: String,
    pub region: String,
    pub items: usize,
    pub rule_net_t: f64,
    pub ordered_t: f64,
    /// The centre of the ordered weight.
    pub centre_m: Point,
}

/// Stock plates of one thickness and size bought, and what is cut from them.
#[derive(Debug, Clone, PartialEq)]
pub struct PlatePurchase {
    pub ordered_mm: f64,
    pub stock_width_m: f64,
    pub stock_length_m: f64,
    pub count: usize,
    pub bought_m2: f64,
    pub net_m2: f64,
    pub scrap_m2: f64,
    pub scrap_t: f64,
}

/// The metres of one bar section bought.
#[derive(Debug, Clone, PartialEq)]
pub struct SectionPurchase {
    pub section: String,
    pub length_m: f64,
    pub weight_t: f64,
}

/// The plate model of one hull.
#[derive(Debug, Clone, PartialEq)]
pub struct HullPlateModel {
    pub items: Vec<SteelItem>,
    pub groups: Vec<Group>,
    pub plates: Vec<PlatePurchase>,
    pub sections: Vec<SectionPurchase>,
    pub rule_net_t: f64,
    pub ordered_t: f64,
    /// The panel model's named allowance for brackets, collars and welds, as a share of the
    /// ordered weight (%), and its weight (t).
    pub allowance_pct: f64,
    pub allowance_t: f64,
    /// Ordered weight plus the allowance (t).
    pub total_t: f64,
    pub bought_m2: f64,
    pub net_plate_m2: f64,
    pub scrap_m2: f64,
    pub scrap_t: f64,
    pub structure: StructureBlock,
    pub rule_edition: &'static str,
}

/// The ordered thickness: the rule-net rounded up to the step (D10).
pub fn ordered_mm(rule_net_mm: f64, step_mm: f64) -> f64 {
    if !above(step_mm, 0.0) {
        return rule_net_mm;
    }
    ceil(rule_net_mm / step_mm - 1e-9) * step_mm
}

/// One copy of a panel as a rectangle: `origin`, the `along` direction (unit) over `length`, and
/// the `across` direction (unit) over `girth`. `length x girth` is the panel's area per copy.
#[derive(Debug, Clone, PartialEq)]
struct Face {
    origin: Point,
    along: Point,
    across: Point,
    length: f64,
    girth: f64,
}

impl Face {
    fn at(&self, s: f64, t: f64) -> Point {
        (
            self.origin.0 + self.along.0 * s + self.across.0 * t,
            self.origin.1 + self.along.1 * s + self.across.1 * t,
            self.origin.2 + self.along.2 * s + self.across.2 * t,
        )
    }
}

const X: Point = (1.0, 0.0, 0.0);
const Y: Point = (0.0, 1.0, 0.0);
const Z: Point = (0.0, 0.0, 1.0);

/// How far a seam or a butt is kept from a stiffener or a frame (m). An Estimate: no rule paragraph
/// read sets it; it keeps the plate weld off the stiffener's and the web's welds.
pub const WELD_CLEARANCE_M: f64 = 0.05;

/// Cut points along a run of `total`, at most `stock` apart: each cut at the stock length from the
/// last, pulled back to [`WELD_CLEARANCE_M`] short of a line it would fall on or near. The lines
/// (stiffeners, or frames) sit at `phase + k x spacing`. The ends 0 and `total` are included.
pub(crate) fn cuts(total: f64, stock: f64, lines: Option<(f64, f64)>) -> Vec<f64> {
    let mut out = alloc::vec![0.0];
    if !above(total, EPS) {
        return out;
    }
    if !above(stock, EPS) {
        out.push(total);
        return out;
    }
    let mut at = 0.0;
    while total - at > stock + EPS {
        let mut next = at + stock;
        if let Some((phase, s)) = lines.filter(|l| l.1 > 2.0 * WELD_CLEARANCE_M) {
            // The nearest line at or below the cut, and the one above it.
            let k = floor((next - phase) / s + EPS);
            for line in [phase + k * s, phase + (k + 1.0) * s] {
                if (next - line).abs() < WELD_CLEARANCE_M - EPS {
                    let pulled = line - WELD_CLEARANCE_M;
                    if pulled > at + EPS {
                        next = pulled.min(next);
                    }
                }
            }
        }
        out.push(next);
        at = next;
    }
    out.push(total);
    out
}

/// The x of frames FR1, FR2, ... up to `x1` (FR0 at the aft end).
pub(crate) fn frames(spacing: f64, x1: f64) -> Vec<f64> {
    let mut out = Vec::new();
    if !above(spacing, EPS) {
        return out;
    }
    let mut k = 1.0;
    while k * spacing <= x1 + EPS {
        out.push(k * spacing);
        k += 1.0;
    }
    out
}

/// `n` frame positions up to `x1`: the frames FR1, FR2, ...; spread evenly when fewer frames fit.
pub(crate) fn frames_n(spacing: f64, x1: f64, n: usize) -> Vec<f64> {
    let f = frames(spacing, x1);
    if f.len() >= n {
        f.into_iter().take(n).collect()
    } else {
        spread(n, 0.0, x1)
    }
}

/// `n` positions spread evenly over (a, b), clear of both ends.
pub(crate) fn spread(n: usize, a: f64, b: f64) -> Vec<f64> {
    (1..=n)
        .map(|k| a + (b - a) * k as f64 / (n as f64 + 1.0))
        .collect()
}

/// The panel-equivalent faces of one panel: one list per copy (a side, a floor, a bulkhead), each of
/// its pieces (the two ends of an end panel, or one).
fn faces_of(m: &PanelMember, spec: &HullStructureSpec) -> Vec<Vec<Face>> {
    let l = spec.loa_m;
    let b = spec.beam_m;
    let half_b = b / 2.0;
    let d = spec.depth_m;
    let db = spec.double_bottom_m.clamp(0.0, d);
    let w = spec.wing_width_m.clamp(0.0, half_b);
    let (xa, xf) = (spec.cargo_zone_aft_m, spec.cargo_zone_fwd_m);
    let lc = (xf - xa).max(0.0);
    let web = spec.structure.frame_spacing_m;
    let n = m.count.max(1);
    let each = m.plate_area_m2 / n as f64;
    let z = m.z_m.0;
    // Lengthwise runs of the midship 0.4L and of the two ends.
    let runs: Vec<(f64, f64)> = match m.region.as_str() {
        region::MIDSHIP => alloc::vec![(0.3 * l, 0.7 * l)],
        region::ENDS => alloc::vec![(0.0, 0.3 * l), (0.7 * l, l)],
        _ => alloc::vec![(m.x_m.0, m.x_m.1)],
    };
    let run_len: f64 = runs.iter().map(|r| r.1 - r.0).sum();
    if !above(each, EPS) || !above(run_len, EPS) {
        return Vec::new();
    }
    let lengthwise = |origin: &dyn Fn(f64) -> Point, across: Point, girth: f64| -> Vec<Face> {
        runs.iter()
            .map(|r| Face {
                origin: origin(r.0),
                along: X,
                across,
                length: r.1 - r.0,
                girth,
            })
            .collect()
    };
    let transverse = |x: f64, y0: f64, width: f64, z0: f64, height: f64| {
        alloc::vec![Face {
            origin: (x, y0, z0),
            along: Y,
            across: Z,
            length: width,
            girth: height,
        }]
    };
    let g = each / run_len;
    match m.member.as_str() {
        ft::BOTTOM_SHELL | ft::STRENGTH_DECK | ft::INNER_BOTTOM => {
            alloc::vec![lengthwise(&|x| (x, -g / 2.0, z), Y, g)]
        }
        ft::SIDE_SHELL | ft::INNER_SIDE => {
            let (y, z0) = if m.member == ft::SIDE_SHELL {
                (half_b, 0.0)
            } else {
                (half_b - w, db)
            };
            [-y, y]
                .iter()
                .map(|y| lengthwise(&|x| (x, *y, z0), Z, g))
                .collect()
        }
        // Both sides in one record: half the width each side, from the side shell inboard.
        ft::WING_TANK_TOP => alloc::vec![
            lengthwise(&|x| (x, -half_b, z), Y, g / 2.0),
            lengthwise(&|x| (x, half_b - g / 2.0, z), Y, g / 2.0),
        ],
        ft::CENTRE_GIRDER => alloc::vec![lengthwise(&|x| (x, 0.0, 0.0), Z, g)],
        ft::SIDE_GIRDER => spread(n / 2, 0.0, half_b)
            .iter()
            .flat_map(|y| [-*y, *y])
            .map(|y| lengthwise(&|x| (x, y, 0.0), Z, g))
            .collect(),
        ft::FLOORS => frames_n(web, l, n)
            .into_iter()
            .map(|x| transverse(x, -half_b, b, 0.0, each / b))
            .collect(),
        ft::WING_WEB_FRAMES if w > EPS => frames_n(web, lc, n / 2)
            .into_iter()
            .flat_map(|x| {
                [
                    transverse(xa + x, -half_b, w, db, each / w),
                    transverse(xa + x, half_b - w, w, db, each / w),
                ]
            })
            .collect(),
        ft::TRANSVERSE_BULKHEADS => bulkhead_xs(spec, n)
            .into_iter()
            .map(|x| transverse(x, -half_b, b, db, each / b))
            .collect(),
        _ => Vec::new(),
    }
}

/// The x of the transverse bulkheads of the panel model (Estimates, spec 0013 T5 places them from the
/// Vessel Model): the aft peak bulkhead midway along the stern zone, the hold bulkheads at the ends
/// of the cargo zone and evenly between, and the collision bulkhead midway along the bow zone.
pub(crate) fn bulkhead_xs(spec: &HullStructureSpec, n: usize) -> Vec<f64> {
    let (xa, xf) = (spec.cargo_zone_aft_m, spec.cargo_zone_fwd_m);
    let holds = n.saturating_sub(3);
    let mut out = alloc::vec![xa / 2.0];
    for k in 0..=holds {
        out.push(xa + (xf - xa) * k as f64 / holds.max(1) as f64);
    }
    out.push((xf + spec.loa_m) / 2.0);
    out.truncate(n);
    out
}

/// The positions of the profile primary members of a panel, one per copy, as trace lines.
fn primary_lines(m: &PanelMember, spec: &HullStructureSpec, span: f64) -> Vec<(Point, Point)> {
    let l = spec.loa_m;
    let half_b = spec.beam_m / 2.0;
    let web = spec.structure.frame_spacing_m;
    let (xa, xf) = (spec.cargo_zone_aft_m, spec.cargo_zone_fwd_m);
    let n = m.count;
    match m.member.as_str() {
        ft::SIDE_WEB_FRAMES => {
            let xs: Vec<f64> = if m.region == region::CARGO_ZONE {
                frames(web, xf - xa).iter().map(|x| xa + x).collect()
            } else {
                frames(web, l)
                    .into_iter()
                    .filter(|x| *x < xa - EPS || *x > xf + EPS)
                    .collect()
            };
            let mut xs: Vec<f64> = xs.into_iter().take(n / 2).collect();
            if xs.len() < n / 2 {
                xs = spread(n / 2, 0.0, l);
            }
            xs.iter()
                .flat_map(|x| {
                    [-half_b, half_b].map(|y| ((*x, y, m.z_m.0), (*x, y, m.z_m.0 + span)))
                })
                .collect()
        }
        ft::DECK_TRANSVERSES => frames_n(web, l, n)
            .into_iter()
            .map(|x| ((x, -span / 2.0, m.z_m.1), (x, span / 2.0, m.z_m.1)))
            .collect(),
        _ => Vec::new(),
    }
}

/// The trace lines of the stiffeners of one copy of a panel: each `(section, length)` entry of the
/// panel model (scaled by `share`, the copy's part of the record's copy) is one line per spacing, as
/// many as the length holds runs, and each line is cut at the copy's pieces. A panel's and a
/// bulkhead's stiffeners run along its length; a floor's stand up its depth. The lines sit one
/// spacing apart from the edge.
fn stiffener_lines(
    m: &PanelMember,
    copy: &[Face],
    share: f64,
    spacing_m: f64,
) -> Vec<(Section, Point, Point, f64)> {
    let mut out = Vec::new();
    let upright = m.member == ft::FLOORS;
    let run: f64 = if upright {
        copy.first().map_or(0.0, |f| f.girth)
    } else {
        copy.iter().map(|f| f.length).sum()
    };
    if !above(run, EPS) {
        return out;
    }
    let mut slot = 0usize;
    for (s, len) in &m.stiffeners {
        let len = len * share;
        let lines = libm::round(len / run).max(1.0) as usize;
        let per_line = len / lines as f64;
        for _ in 0..lines {
            slot += 1;
            let at = slot as f64 * spacing_m;
            for f in copy {
                let part = if upright {
                    per_line
                } else {
                    per_line * f.length / run
                };
                let (p0, p1) = if upright {
                    (f.at(at, 0.0), f.at(at, part))
                } else {
                    (f.at(0.0, at), f.at(part, at))
                };
                out.push((s.clone(), p0, p1, part));
            }
        }
    }
    out
}

pub(crate) fn mid(a: Point, b: Point) -> Point {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0)
}

/// The items built so far, with the pieces cut from stock and the item range of each group.
pub(crate) struct Acc {
    pub(crate) items: Vec<SteelItem>,
    /// Every piece cut from stock: (group index, ordered thickness, width, length, net area). A
    /// piece of a hull section is cut from its bounding width x length.
    pub(crate) cut: Vec<(usize, f64, f64, f64, f64)>,
    /// Each group: its member, region and the range of its items.
    pub(crate) groups: Vec<(String, String, usize, usize)>,
    n_pl: usize,
    n_st: usize,
    n_pm: usize,
}

impl Acc {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            cut: Vec::new(),
            groups: Vec::new(),
            n_pl: 0,
            n_st: 0,
            n_pm: 0,
        }
    }

    /// A new item of a panel, with the next id of its kind; its numbers are 0.
    pub(crate) fn item(
        &mut self,
        m: &PanelMember,
        st: &StructureBlock,
        kind: ItemKind,
    ) -> SteelItem {
        let id = match kind {
            ItemKind::Plate => {
                self.n_pl += 1;
                format!("PL{}", self.n_pl)
            }
            ItemKind::Stiffener => {
                self.n_st += 1;
                format!("ST{}", self.n_st)
            }
            ItemKind::Primary => {
                self.n_pm += 1;
                format!("PM{}", self.n_pm)
            }
        };
        SteelItem {
            source: PartSource::Hull,
            id,
            kind,
            member: m.member.clone(),
            region: m.region.clone(),
            grade: st.steel_grade,
            rule_net_mm: 0.0,
            ordered_mm: 0.0,
            section: None,
            length_m: 0.0,
            width_m: 0.0,
            area_m2: 0.0,
            pieces: 0,
            rule_net_t: 0.0,
            ordered_t: 0.0,
            centre_m: (0.0, 0.0, 0.0),
            geometry: Geometry { points: Vec::new() },
            rule: m.rule.clone(),
            rule_set: m.rule_set,
            edition: m.edition,
            paragraph: m.paragraph.clone(),
            basis: m.basis,
        }
    }

    /// A stiffener or a profile primary member: `section` over the line `p0` to `p1`, `len` long.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn line(
        &mut self,
        m: &PanelMember,
        st: &StructureBlock,
        kind: ItemKind,
        s: &Section,
        len: f64,
        rho: f64,
        p0: Point,
        p1: Point,
    ) {
        let mut it = self.item(m, st, kind);
        let t = s.mass_per_m(rho) * len;
        it.section = Some(s.name.clone());
        it.length_m = len;
        it.rule_net_t = t;
        it.ordered_t = t;
        it.centre_m = mid(p0, p1);
        it.geometry.points = alloc::vec![p0, p1];
        self.items.push(it);
    }

    /// Close the group of `m` (or of `m` under another region): the items since `first`.
    pub(crate) fn close(&mut self, member: &str, region: &str, first: usize) {
        self.groups
            .push((member.into(), region.into(), first, self.items.len()));
    }
}

/// The items of one panel laid out on its panel-equivalent rectangles (`faces_of`).
fn box_member(acc: &mut Acc, k: usize, m: &PanelMember, spec: &HullStructureSpec) {
    let st = spec.structure;
    let rho = spec.steel_density;
    let step = st.thickness_step_mm;
    let (stock_w, stock_l) = (st.stock_plate_width_m, st.stock_plate_length_m);
    let spacing = st.longitudinal_spacing_mm / 1000.0;
    let web = st.frame_spacing_m;
    let t_net = m.plate_mm;
    let t_ord = ordered_mm(t_net, step);
    let plated_primary = m.member == ft::FLOORS || m.member == ft::WING_WEB_FRAMES;
    let copies = faces_of(m, spec);
    let share = if copies.is_empty() {
        0.0
    } else {
        m.count.max(1) as f64 / copies.len() as f64
    };
    for copy in &copies {
        for face in copy {
            // Seams across the girth at the stock width, clear of the longitudinals; butts along
            // the length. A transverse face (a floor, a bulkhead) has its butts across the hull.
            let has_stiffeners = !m.stiffeners.is_empty();
            let seams = cuts(
                face.girth,
                stock_w,
                (has_stiffeners && !plated_primary).then_some((0.0, spacing)),
            );
            // Butts of a lengthwise panel clear of the frames (FR0 at x = 0).
            let frames_at = (face.along == X && web > EPS).then(|| {
                let phase = (web - face.origin.0 % web) % web;
                (phase, web)
            });
            let butts = cuts(face.length, stock_l, frames_at);
            if plated_primary {
                // One item per frame, cut in as many pieces as its seams and butts make.
                let area = face.length * face.girth;
                let mut it = acc.item(m, &st, ItemKind::Primary);
                let pieces = (seams.len() - 1) * (butts.len() - 1);
                for bw in butts.windows(2) {
                    for sw in seams.windows(2) {
                        acc.cut.push((
                            k,
                            t_ord,
                            sw[1] - sw[0],
                            bw[1] - bw[0],
                            (sw[1] - sw[0]) * (bw[1] - bw[0]),
                        ));
                    }
                }
                it.rule_net_mm = t_net;
                it.ordered_mm = t_ord;
                it.length_m = face.length;
                it.width_m = face.girth;
                it.area_m2 = area;
                it.pieces = pieces;
                it.rule_net_t = area * t_net / 1000.0 * rho;
                it.ordered_t = area * t_ord / 1000.0 * rho;
                it.centre_m = face.at(face.length / 2.0, face.girth / 2.0);
                it.geometry.points = alloc::vec![
                    face.at(0.0, 0.0),
                    face.at(face.length, 0.0),
                    face.at(face.length, face.girth),
                    face.at(0.0, face.girth),
                ];
                acc.items.push(it);
            } else {
                for bw in butts.windows(2) {
                    for sw in seams.windows(2) {
                        let (len, wid) = (bw[1] - bw[0], sw[1] - sw[0]);
                        let area = len * wid;
                        let mut it = acc.item(m, &st, ItemKind::Plate);
                        it.rule_net_mm = t_net;
                        it.ordered_mm = t_ord;
                        it.length_m = len;
                        it.width_m = wid;
                        it.area_m2 = area;
                        it.pieces = 1;
                        acc.cut.push((k, t_ord, wid, len, area));
                        it.rule_net_t = area * t_net / 1000.0 * rho;
                        it.ordered_t = area * t_ord / 1000.0 * rho;
                        it.centre_m = face.at((bw[0] + bw[1]) / 2.0, (sw[0] + sw[1]) / 2.0);
                        it.geometry.points = alloc::vec![
                            face.at(bw[0], sw[0]),
                            face.at(bw[1], sw[0]),
                            face.at(bw[1], sw[1]),
                            face.at(bw[0], sw[1]),
                        ];
                        acc.items.push(it);
                    }
                }
            }
        }
        for (s, p0, p1, len) in stiffener_lines(m, copy, share, spacing) {
            acc.line(m, &st, ItemKind::Stiffener, &s, len, rho, p0, p1);
        }
    }
    // Profile primary members: one per frame (side webs, deck transverses), and the vertical
    // webs of each bulkhead.
    for (s, len) in &m.primary_members {
        let lines: Vec<(Point, Point)> = if m.member == ft::TRANSVERSE_BULKHEADS {
            // The webs of every bulkhead, spread across it, each the bulkhead's height.
            let n_v = copies
                .first()
                .and_then(|c| c.first())
                .map_or(1, |f| libm::round(len / f.girth.max(EPS)).max(1.0) as usize);
            copies
                .iter()
                .flat_map(|c| c.iter())
                .flat_map(|f| {
                    spread(n_v, 0.0, f.length)
                        .into_iter()
                        .map(|y| (f.at(y, 0.0), f.at(y, f.girth)))
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            primary_lines(m, spec, *len)
        };
        let each = if m.member == ft::TRANSVERSE_BULKHEADS {
            let per = (lines.len() / copies.len().max(1)).max(1);
            len / per as f64
        } else {
            *len
        };
        for (p0, p1) in lines {
            acc.line(m, &st, ItemKind::Primary, s, each, rho, p0, p1);
        }
    }
}

/// The plate model of a hull from its panel model and its `HullStructureSpec`, each panel laid out
/// on its panel-equivalent rectangle. With a Sections Table, [`hull_plates_on`] lays the shell and
/// the end structure out on the hull itself.
pub fn hull_plates(panels: &HullPanelModel, spec: &HullStructureSpec) -> HullPlateModel {
    let mut acc = Acc::new();
    for (k, m) in panels.members.iter().enumerate() {
        let first = acc.items.len();
        box_member(&mut acc, k, m, spec);
        acc.close(&m.member, &m.region, first);
    }
    finish(panels, spec, acc)
}

/// The plate model of a hull laid out on its Sections Table (issue #743, spec 0013 "Ends", D6). The
/// shell (bottom, side and deck, in the midship 0.4L and the ends) follows the table, one strip per
/// Station interval, and the two end faces of the hull are plated ("end plating"). The transverse
/// bulkheads, the floors, the inner bottom and the girders take the hull's section where they stand.
/// The hold bulkheads stand at `hold_bulkheads_m` (the Vessel Model's, from aft) when it holds one
/// per hold end (`holds_along + 1`); else at the panel model's Estimates. The other members (the
/// double side, the side webs, the deck transverses) stand in the cargo zone and keep their
/// panel-equivalent rectangles. See [`crate::hull_shell`].
pub fn hull_plates_on(
    panels: &HullPanelModel,
    spec: &HullStructureSpec,
    hull: &SectionsTable,
    hold_bulkheads_m: &[f64],
) -> HullPlateModel {
    let mut acc = Acc::new();
    let hull = crate::hull_shell::Hull::new(hull, spec);
    for (k, m) in panels.members.iter().enumerate() {
        let first = acc.items.len();
        if !hull.lay_out(&mut acc, k, m, hold_bulkheads_m) {
            box_member(&mut acc, k, m, spec);
        }
        acc.close(&m.member, &m.region, first);
    }
    hull.end_plating(&mut acc, panels);
    finish(panels, spec, acc)
}

/// The groups, the purchase and the totals of the items built.
fn finish(panels: &HullPanelModel, spec: &HullStructureSpec, acc: Acc) -> HullPlateModel {
    let st = spec.structure;
    let rho = spec.steel_density;
    let (stock_w, stock_l) = (st.stock_plate_width_m, st.stock_plate_length_m);
    let Acc {
        items,
        cut,
        groups: ranges,
        ..
    } = acc;
    let groups: Vec<Group> = ranges
        .iter()
        .map(|(member, region, a, b)| {
            let mine = &items[*a..*b];
            let ordered: f64 = mine.iter().map(|i| i.ordered_t).sum();
            let moment = |f: fn(&Point) -> f64| -> f64 {
                if ordered > 0.0 {
                    mine.iter()
                        .map(|i| i.ordered_t * f(&i.centre_m))
                        .sum::<f64>()
                        / ordered
                } else {
                    0.0
                }
            };
            Group {
                member: member.clone(),
                region: region.clone(),
                items: mine.len(),
                rule_net_t: mine.iter().map(|i| i.rule_net_t).sum(),
                ordered_t: ordered,
                centre_m: (moment(|p| p.0), moment(|p| p.1), moment(|p| p.2)),
            }
        })
        .collect();

    // Purchase: stock plates by ordered thickness. Like pieces of one panel (the same thickness
    // and size) are cut side by side from a stock plate, as many as fit in rows and columns, turned
    // whichever way fits more (no nesting beyond that, D3). A full-size plate takes a stock plate.
    let fit = |w: f64, l: f64| -> usize {
        let rows = |a: f64, b: f64| -> usize {
            if a > EPS && b > EPS {
                (floor(stock_w / a + EPS) * floor(stock_l / b + EPS)) as usize
            } else {
                0
            }
        };
        rows(w, l).max(rows(l, w)).max(1)
    };
    let mut kinds: Vec<(usize, f64, f64, f64, usize, f64)> = Vec::new();
    for &(k, t, w, l, a) in &cut {
        let same = |a: f64, b: f64| (a - b).abs() < 1e-6;
        match kinds
            .iter_mut()
            .find(|c| c.0 == k && same(c.1, t) && same(c.2, w) && same(c.3, l))
        {
            Some(c) => {
                c.4 += 1;
                c.5 += a;
            }
            None => kinds.push((k, t, w, l, 1, a)),
        }
    }
    let buys: Vec<(f64, usize, f64)> = kinds
        .iter()
        .map(|&(_, t, w, l, n, a)| (t, n.div_ceil(fit(w, l)), a))
        .collect();
    let mut plates: Vec<PlatePurchase> = Vec::new();
    for (t, count, net) in buys {
        let bought = count as f64 * stock_w * stock_l;
        match plates.iter_mut().find(|p| p.ordered_mm == t) {
            Some(p) => {
                p.count += count;
                p.bought_m2 += bought;
                p.net_m2 += net;
            }
            None => plates.push(PlatePurchase {
                ordered_mm: t,
                stock_width_m: stock_w,
                stock_length_m: stock_l,
                count,
                bought_m2: bought,
                net_m2: net,
                scrap_m2: 0.0,
                scrap_t: 0.0,
            }),
        }
    }
    for p in &mut plates {
        p.scrap_m2 = p.bought_m2 - p.net_m2;
        p.scrap_t = p.scrap_m2 * p.ordered_mm / 1000.0 * rho;
    }
    plates.sort_by(|a, b| a.ordered_mm.total_cmp(&b.ordered_mm));
    let mut sections: Vec<SectionPurchase> = Vec::new();
    for it in &items {
        let Some(name) = it.section.as_ref() else {
            continue;
        };
        match sections.iter_mut().find(|s| &s.section == name) {
            Some(s) => {
                s.length_m += it.length_m;
                s.weight_t += it.ordered_t;
            }
            None => sections.push(SectionPurchase {
                section: name.clone(),
                length_m: it.length_m,
                weight_t: it.ordered_t,
            }),
        }
    }
    let rule_net: f64 = items.iter().map(|i| i.rule_net_t).sum();
    let ordered: f64 = items.iter().map(|i| i.ordered_t).sum();
    let allowance = ordered * panels.allowance_pct / 100.0;
    let bought: f64 = plates.iter().map(|p| p.bought_m2).sum();
    let net_plate: f64 = plates.iter().map(|p| p.net_m2).sum();
    HullPlateModel {
        rule_net_t: rule_net,
        ordered_t: ordered,
        allowance_pct: panels.allowance_pct,
        allowance_t: allowance,
        total_t: ordered + allowance,
        bought_m2: bought,
        net_plate_m2: net_plate,
        scrap_m2: plates.iter().map(|p| p.scrap_m2).sum(),
        scrap_t: plates.iter().map(|p| p.scrap_t).sum(),
        items,
        groups,
        plates,
        sections,
        structure: st,
        rule_edition: panels.rule_edition,
    }
}
