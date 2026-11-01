//! The plate model on the hull's Sections Table (issue #743, spec 0013 "Model", `sections` / `mesh`
//! row, "Ends", D6).
//!
//! **Concept and preliminary design only (ADR 0011). This is not a structural approval.** The
//! scantlings are the panel model's (`hull_panels`); this module only lays the plates out on the
//! hull itself, so the rakes, bilges, sheer, camber and Stern Forms carry real plate.
//!
//! - **Shell.** One strip per Station interval, the same ruled quads the hull mesh builds
//!   (`mesh::hull_mesh`): between point `i` and point `i + 1` of two neighbouring Sections, both
//!   sides, and the slot wall of a slotted pair. The points of a half-section fall into runs of one
//!   member each, read on the Section with the lowest keel: the deck from the deck edge to the
//!   centre line, and below it each edge is bottom where it runs flatter than 45 degrees and side
//!   where it runs steeper. A plate is a piece of one run between seams (along the girth, at the
//!   stock width, clear of the longitudinals) and butts (along the strip, at the stock length, clear
//!   of the frames): flat on one quad, single-curved over a bilge or a camber. Its area and centre
//!   are integrated on the ruled surface by Gauss-Legendre quadrature (4 x 4 points per quad), exact
//!   on a flat quad. The strips are split at 0.3L and 0.7L (the panel model's midship 0.4L) and at
//!   the ends of the cargo zone. In the cargo zone the deck cut-outs for the tanks are left out: the
//!   panel model's total cut-out width, one cut-out per tank abreast, centred on the tanks spread
//!   evenly over the beam (an Estimate of where they are).
//! - **End plating.** The first and the last Section, where they enclose an area (a transom, a
//!   square bow), are plated at the side shell's end thickness, with horizontal stiffeners one
//!   spacing apart.
//! - **End structure.** Each transverse bulkhead is the hull's section where it stands, above the
//!   inner bottom: the aft peak and collision bulkheads at the panel model's positions (midway along
//!   the stern and bow zones; Estimates), the hold bulkheads at the Vessel Model's. Each floor is the
//!   section below the inner bottom at its frame, and none where the keel stands above the inner
//!   bottom. The inner bottom stops where the keel rises through it; each girder runs from the
//!   bottom to the inner bottom. The centre girder takes the 85 % of ABS 5-1-2/5.1.2 over the
//!   panel model's ends, 0.125L at each end (the panel model weighs 0.25L of it thinner).
//!
//! A plate cut from a planar part (a bulkhead, a floor, the inner bottom, a girder, an end plate) is
//! the part's polygon clipped to the cell between two seams and two butts.

use alloc::vec::Vec;

use libm::{ceil, fabs, sqrt};

use crate::hull_panels::{
    HullPanelModel, HullStructureSpec, PanelMember, Section as Bar, function_type as ft, region,
};
use crate::hull_plates::{
    Acc, EPS, ItemKind, Point, bulkhead_xs, cuts, frames_n, ordered_mm, spread,
};
use crate::sections::{Section, SectionsTable};

/// What member a run of a half-section is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Bottom,
    Side,
    Deck,
}

/// A point `(s, t)` in the plane of a planar part.
type P2 = (f64, f64);

/// The plane of a planar part, and which hull axes its `(s, t)` are.
#[derive(Debug, Clone, Copy)]
enum Plane {
    /// A transverse plane at x: `s = y`, `t = z`.
    X(f64),
    /// A longitudinal plane at y: `s = x`, `t = z`.
    Y(f64),
    /// A horizontal plane at z: `s = x`, `t = y`.
    Z(f64),
}

impl Plane {
    fn at(self, (s, t): P2) -> Point {
        match self {
            Plane::X(x) => (x, s, t),
            Plane::Y(y) => (s, y, t),
            Plane::Z(z) => (s, t, z),
        }
    }
}

/// Signed area and centroid of a polygon (shoelace).
fn area_centroid(poly: &[P2]) -> (f64, P2) {
    let n = poly.len();
    let (mut a, mut cs, mut ct) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let c = p.0 * q.1 - q.0 * p.1;
        a += c;
        cs += (p.0 + q.0) * c;
        ct += (p.1 + q.1) * c;
    }
    a /= 2.0;
    if fabs(a) > 0.0 {
        (a, (cs / (6.0 * a), ct / (6.0 * a)))
    } else {
        (0.0, (0.0, 0.0))
    }
}

/// The part of a polygon where the linear function `f` is not negative (one Sutherland-Hodgman
/// step). The area of the result is exact for any simple polygon.
fn clip_by(poly: &[P2], f: impl Fn(P2) -> f64) -> Vec<P2> {
    let n = poly.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let (fp, fq) = (f(p), f(q));
        if fp >= 0.0 {
            out.push(p);
        }
        if (fp >= 0.0) != (fq >= 0.0) {
            let u = fp / (fp - fq);
            out.push((p.0 + u * (q.0 - p.0), p.1 + u * (q.1 - p.1)));
        }
    }
    out
}

/// A polygon clipped to the box `s0..s1`, `t0..t1`.
fn clip_box(poly: &[P2], (s0, s1): (f64, f64), (t0, t1): (f64, f64)) -> Vec<P2> {
    let a = clip_by(poly, |p| p.0 - s0);
    let a = clip_by(&a, |p| s1 - p.0);
    let a = clip_by(&a, |p| p.1 - t0);
    clip_by(&a, |p| t1 - p.1)
}

/// The pieces of the line `t = c` (`along_s`) or `s = c` inside a polygon, as `(from, to)`.
fn scan(poly: &[P2], along_s: bool, c: f64) -> Vec<(f64, f64)> {
    let n = poly.len();
    let mut xs: Vec<f64> = Vec::new();
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let (a, b, pa, qa) = if along_s {
            (p.1, q.1, p.0, q.0)
        } else {
            (p.0, q.0, p.1, q.1)
        };
        if (a <= c) != (b <= c) {
            xs.push(pa + (c - a) / (b - a) * (qa - pa));
        }
    }
    xs.sort_by(f64::total_cmp);
    xs.as_chunks::<2>().0.iter().map(|w| (w[0], w[1])).collect()
}

fn extent(poly: &[P2], f: fn(&P2) -> f64) -> (f64, f64) {
    poly.iter()
        .map(f)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
            (a.min(v), b.max(v))
        })
}

/// `v > 0`, false for NaN.
fn positive(v: f64) -> bool {
    v > 0.0
}

fn slotted(s: &Section) -> bool {
    s.points[0].0 > 0.0
}

/// The full section as its polygons `(y, z)`: the half and its mirror back to the keel, or for a
/// slotted Section the half and the mirrored half (as `mesh::hull_mesh` closes its ends).
fn rings(s: &Section) -> Vec<Vec<P2>> {
    let h = &s.points;
    let n = h.len();
    if slotted(s) {
        alloc::vec![h.clone(), h.iter().rev().map(|&(y, z)| (-y, z)).collect()]
    } else {
        let mut full = h.clone();
        full.extend(h[1..n - 1].iter().rev().map(|&(y, z)| (-y, z)));
        alloc::vec![full]
    }
}

/// The half breadth of a section at height z: the largest |y| where the plane cuts it; 0 where it
/// misses.
fn half_breadth(s: &Section, z: f64) -> f64 {
    rings(s)
        .iter()
        .flat_map(|r| scan(r, true, z))
        .fold(0.0, |a, (y0, y1)| a.max(fabs(y0)).max(fabs(y1)))
}

/// The lowest point of a section at y (the bottom there); `None` where the line misses it.
fn bottom_at(s: &Section, y: f64) -> Option<f64> {
    rings(s)
        .iter()
        .flat_map(|r| scan(r, false, y))
        .map(|(z0, _)| z0)
        .reduce(f64::min)
}

fn keel_of(s: &Section) -> f64 {
    s.points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min)
}

/// The section `(1 - u) a + u b` of a ruled interval.
fn between(a: &Section, b: &Section, u: f64) -> Section {
    Section {
        x: a.x + u * (b.x - a.x),
        points: a
            .points
            .iter()
            .zip(&b.points)
            .map(|(p, q)| (p.0 + u * (q.0 - p.0), p.1 + u * (q.1 - p.1)))
            .collect(),
    }
}

fn dist(a: Point, b: Point) -> f64 {
    let (dx, dy, dz) = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
    sqrt(dx * dx + dy * dy + dz * dz)
}

fn lerp3(a: Point, b: Point, u: f64) -> Point {
    (
        a.0 + u * (b.0 - a.0),
        a.1 + u * (b.1 - a.1),
        a.2 + u * (b.2 - a.2),
    )
}

/// Gauss-Legendre nodes and weights on [0, 1], 4 points.
const GAUSS: [(f64, f64); 4] = [
    (
        0.5 - 0.5 * 0.861_136_311_594_052_6,
        0.5 * 0.347_854_845_137_453_8,
    ),
    (
        0.5 - 0.5 * 0.339_981_043_584_856_3,
        0.5 * 0.652_145_154_862_546_1,
    ),
    (
        0.5 + 0.5 * 0.339_981_043_584_856_3,
        0.5 * 0.652_145_154_862_546_1,
    ),
    (
        0.5 + 0.5 * 0.861_136_311_594_052_6,
        0.5 * 0.347_854_845_137_453_8,
    ),
];

/// Where a cut lands: a phase and spacing in the coordinate of a run that starts at `origin`.
fn rel(lines: Option<(f64, f64)>, origin: f64) -> Option<(f64, f64)> {
    lines.map(|(phase, sp)| {
        let r = (phase - origin) / sp;
        ((r - libm::floor(r)) * sp, sp)
    })
}

/// One strip of the shell: a run of points on one side of one Station interval, between `u0` and
/// `u1` of the interval.
struct Strip<'a> {
    a: &'a Section,
    b: &'a Section,
    idx: &'a [usize],
    side: f64,
    /// The end of the interval whose run has the larger girth: the girths are read there.
    reference: &'a Section,
    /// Girth of each edge, and to the start of each, at the reference end of the interval.
    len: Vec<f64>,
    from: Vec<f64>,
    girth: f64,
}

impl<'a> Strip<'a> {
    fn new(a: &'a Section, b: &'a Section, idx: &'a [usize], side: f64) -> Self {
        let girth_at = |s: &Section| -> Vec<f64> {
            idx.windows(2)
                .map(|w| {
                    let (p, q) = (s.points[w[0]], s.points[w[1]]);
                    sqrt((q.0 - p.0) * (q.0 - p.0) + (q.1 - p.1) * (q.1 - p.1))
                })
                .collect()
        };
        let (la, lb) = (girth_at(a), girth_at(b));
        let (len, reference) = if lb.iter().sum::<f64>() > la.iter().sum::<f64>() {
            (lb, b)
        } else {
            (la, a)
        };
        let mut from = Vec::with_capacity(len.len());
        let mut g = 0.0;
        for l in &len {
            from.push(g);
            g += l;
        }
        Self {
            a,
            b,
            idx,
            side,
            reference,
            len,
            from,
            girth: g,
        }
    }

    /// Point `i` of the run at `u` along the interval, in hull axes.
    fn point(&self, i: usize, u: f64) -> Point {
        let (p, q) = (self.a.points[self.idx[i]], self.b.points[self.idx[i]]);
        (
            self.a.x + u * (self.b.x - self.a.x),
            self.side * (p.0 + u * (q.0 - p.0)),
            p.1 + u * (q.1 - p.1),
        )
    }

    /// The edge and the share along it of girth `g` (at the reference end).
    fn locate(&self, g: f64) -> (usize, f64) {
        let n = self.len.len();
        for j in 0..n {
            if g <= self.from[j] + self.len[j] + EPS || j == n - 1 {
                let t = if self.len[j] > 0.0 {
                    ((g - self.from[j]) / self.len[j]).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                return (j, t);
            }
        }
        (0, 0.0)
    }

    fn at(&self, u: f64, g: f64) -> Point {
        let (j, t) = self.locate(g);
        lerp3(self.point(j, u), self.point(j + 1, u), t)
    }

    /// The outline of the piece `u0..u1`, `g0..g1`: along the girth at `u0`, back at `u1`.
    fn outline(&self, (u0, u1): (f64, f64), (g0, g1): (f64, f64)) -> Vec<Point> {
        let inner: Vec<f64> = self
            .from
            .iter()
            .copied()
            .filter(|f| *f > g0 + EPS && *f < g1 - EPS)
            .collect();
        let mut pts = alloc::vec![self.at(u0, g0)];
        pts.extend(inner.iter().map(|g| self.at(u0, *g)));
        pts.push(self.at(u0, g1));
        pts.push(self.at(u1, g1));
        pts.extend(inner.iter().rev().map(|g| self.at(u1, *g)));
        pts.push(self.at(u1, g0));
        pts
    }

    /// The girths of the piece's boundary lines (its seams and the knuckles inside it).
    fn lines_in(&self, (g0, g1): (f64, f64)) -> Vec<f64> {
        let mut v = alloc::vec![g0];
        v.extend(
            self.from
                .iter()
                .copied()
                .filter(|f| *f > g0 + EPS && *f < g1 - EPS),
        );
        v.push(g1);
        v
    }

    /// The longest of the piece's boundary lines along the strip, and its widest girth at either end.
    fn size(&self, u: (f64, f64), g: (f64, f64)) -> (f64, f64) {
        let lines = self.lines_in(g);
        let length = lines
            .iter()
            .map(|gi| dist(self.at(u.0, *gi), self.at(u.1, *gi)))
            .fold(0.0, f64::max);
        let width = [u.0, u.1]
            .iter()
            .map(|uu| {
                lines
                    .windows(2)
                    .map(|w| dist(self.at(*uu, w[0]), self.at(*uu, w[1])))
                    .sum::<f64>()
            })
            .fold(0.0, f64::max);
        (length, width)
    }

    /// Area and centre of the piece `u0..u1`, `g0..g1` on the ruled surface.
    fn area(&self, (u0, u1): (f64, f64), (g0, g1): (f64, f64)) -> (f64, Point) {
        let (mut a, mut c) = (0.0, (0.0, 0.0, 0.0));
        for j in 0..self.len.len() {
            if self.len[j] <= 0.0 {
                continue;
            }
            let t0 = ((g0 - self.from[j]) / self.len[j]).clamp(0.0, 1.0);
            let t1 = ((g1 - self.from[j]) / self.len[j]).clamp(0.0, 1.0);
            if t1 - t0 <= 0.0 {
                continue;
            }
            let (p0, p1) = (self.point(j, 0.0), self.point(j, 1.0));
            let (q0, q1) = (self.point(j + 1, 0.0), self.point(j + 1, 1.0));
            let dp = (p1.0 - p0.0, p1.1 - p0.1, p1.2 - p0.2);
            let dq = (q1.0 - q0.0, q1.1 - q0.1, q1.2 - q0.2);
            let scale = (u1 - u0) * (t1 - t0);
            for (gu, wu) in GAUSS {
                let u = u0 + gu * (u1 - u0);
                let (p, q) = (lerp3(p0, p1, u), lerp3(q0, q1, u));
                let e = (q.0 - p.0, q.1 - p.1, q.2 - p.2);
                for (gt, wt) in GAUSS {
                    let t = t0 + gt * (t1 - t0);
                    let du = (
                        dp.0 + t * (dq.0 - dp.0),
                        dp.1 + t * (dq.1 - dp.1),
                        dp.2 + t * (dq.2 - dp.2),
                    );
                    let n = (
                        du.1 * e.2 - du.2 * e.1,
                        du.2 * e.0 - du.0 * e.2,
                        du.0 * e.1 - du.1 * e.0,
                    );
                    let w = wu * wt * scale * sqrt(n.0 * n.0 + n.1 * n.1 + n.2 * n.2);
                    let at = lerp3(p, q, t);
                    a += w;
                    c = (c.0 + w * at.0, c.1 + w * at.1, c.2 + w * at.2);
                }
            }
        }
        if a > 0.0 {
            (a, (c.0 / a, c.1 / a, c.2 / a))
        } else {
            (0.0, c)
        }
    }
}

/// A hull's Sections Table, ready to lay the plate model out on.
pub(crate) struct Hull<'a> {
    table: &'a SectionsTable,
    spec: &'a HullStructureSpec,
    /// The runs of a half-section: a member and its point indices.
    runs: Vec<(Class, Vec<usize>)>,
}

impl<'a> Hull<'a> {
    pub(crate) fn new(table: &'a SectionsTable, spec: &'a HullStructureSpec) -> Self {
        Self {
            table,
            spec,
            runs: runs_of(table),
        }
    }

    fn x0(&self) -> f64 {
        self.table.sections[0].x
    }

    /// Lay the panel `m` out on the hull; false when it keeps its panel-equivalent rectangles.
    pub(crate) fn lay_out(&self, acc: &mut Acc, k: usize, m: &PanelMember, holds: &[f64]) -> bool {
        let shell =
            |c: Class| matches!(m.region.as_str(), region::MIDSHIP | region::ENDS).then_some(c);
        let class = match m.member.as_str() {
            ft::BOTTOM_SHELL => shell(Class::Bottom),
            ft::SIDE_SHELL => shell(Class::Side),
            ft::STRENGTH_DECK => shell(Class::Deck),
            _ => None,
        };
        if let Some(c) = class {
            self.shell(acc, k, m, c);
            return true;
        }
        match m.member.as_str() {
            ft::INNER_BOTTOM => self.inner_bottom(acc, k, m),
            ft::CENTRE_GIRDER => self.girders(acc, k, m, &[0.0]),
            ft::SIDE_GIRDER => {
                let ys: Vec<f64> = spread(m.count / 2, 0.0, self.spec.beam_m / 2.0)
                    .into_iter()
                    .flat_map(|y| [-y, y])
                    .collect();
                self.girders(acc, k, m, &ys);
            }
            ft::FLOORS => self.floors(acc, k, m),
            ft::TRANSVERSE_BULKHEADS => self.bulkheads(acc, k, m, holds),
            _ => return false,
        }
        true
    }

    /// The shell plates and longitudinals of one panel: its run of every strip in its region.
    fn shell(&self, acc: &mut Acc, k: usize, m: &PanelMember, class: Class) {
        let spec = self.spec;
        let l = spec.loa_m;
        let x0 = self.x0();
        let midship = m.region == region::MIDSHIP;
        let breaks = [
            x0 + 0.3 * l,
            x0 + 0.7 * l,
            spec.cargo_zone_aft_m,
            spec.cargo_zone_fwd_m,
        ];
        let ss = &self.table.sections;
        let n = ss[0].points.len();
        let slot_wall = alloc::vec![n - 1, 0];
        for w in ss.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            let dx = b.x - a.x;
            let mut us = alloc::vec![0.0, 1.0];
            if dx > 0.0 {
                us.extend(
                    breaks
                        .iter()
                        .filter(|x| **x > a.x + EPS && **x < b.x - EPS)
                        .map(|x| (x - a.x) / dx),
                );
            }
            us.sort_by(f64::total_cmp);
            let mut runs: Vec<&[usize]> = self
                .runs
                .iter()
                .filter(|r| r.0 == class)
                .map(|r| r.1.as_slice())
                .collect();
            if class == Class::Side && (slotted(a) || slotted(b)) {
                runs.push(&slot_wall);
            }
            for uw in us.windows(2) {
                let u = (uw[0], uw[1]);
                let xm = a.x + 0.5 * (u.0 + u.1) * dx - x0;
                let in_mid = xm >= 0.3 * l - EPS && xm <= 0.7 * l + EPS;
                if in_mid != midship {
                    continue;
                }
                let xh = xm + x0;
                let cut_outs = class == Class::Deck
                    && xh > spec.cargo_zone_aft_m
                    && xh < spec.cargo_zone_fwd_m;
                for idx in &runs {
                    for side in [1.0, -1.0] {
                        let strip = Strip::new(a, b, idx, side);
                        self.strip(acc, k, m, class, &strip, u, cut_outs);
                    }
                }
            }
        }
    }

    /// The girth intervals of a deck strip that are plated: all of it, or outside the cut-outs.
    fn kept(&self, strip: &Strip, cut_outs: bool) -> Vec<(f64, f64)> {
        let spec = self.spec;
        let g = strip.girth;
        if !cut_outs || spec.deck_opening_width_m <= 0.0 {
            return alloc::vec![(0.0, g)];
        }
        let b = spec.beam_m;
        let n = spec.tanks_abreast.max(1);
        let w = spec.deck_opening_width_m / n as f64;
        // |y| intervals of the cut-outs (the layout is symmetric), then girth along the run from
        // its first point. The deck run's |y| falls from the deck edge to the centre line.
        let ys: Vec<f64> = strip
            .idx
            .iter()
            .map(|i| strip.reference.points[*i].0)
            .collect();
        let girth_of = |y: f64| -> f64 {
            for j in 0..ys.len() - 1 {
                let (p, q) = (ys[j], ys[j + 1]);
                if (p - y) * (q - y) <= 0.0 && fabs(p - q) > 0.0 {
                    return strip.from[j] + strip.len[j] * (p - y) / (p - q);
                }
            }
            if y >= ys[0] { 0.0 } else { g }
        };
        let mut holes: Vec<(f64, f64)> = (0..n)
            .map(|k| -b / 2.0 + (k as f64 + 0.5) * b / n as f64)
            .filter_map(|c| {
                let (y0, y1) = ((c - w / 2.0).max(0.0), c + w / 2.0);
                (y1 > y0).then(|| {
                    let (g0, g1) = (girth_of(y1), girth_of(y0));
                    (g0.min(g1), g0.max(g1))
                })
            })
            .collect();
        holes.sort_by(|p, q| p.0.total_cmp(&q.0));
        let mut out = Vec::new();
        let mut at = 0.0;
        for (h0, h1) in holes {
            if h0 > at + EPS {
                out.push((at, h0));
            }
            at = at.max(h1);
        }
        if g > at + EPS {
            out.push((at, g));
        }
        out
    }

    /// The longitudinals of a strip, as their number and girth: one per spacing from the keel (the
    /// bottom), from the centre line (the deck) or from the run's first point (the side, and a run
    /// off the centre line). A centre line longitudinal goes on the starboard strip only.
    fn longitudinals(&self, class: Class, strip: &Strip, sp: f64) -> Vec<(usize, f64)> {
        let g = strip.girth;
        let pts = &strip.reference.points;
        let n = pts.len();
        let mut out = Vec::new();
        if !positive(sp) {
            return out;
        }
        let centre_start = strip.idx[0] == 0 && pts[0].0 == 0.0;
        let centre_end = strip.idx[strip.idx.len() - 1] == n - 1 && pts[n - 1].0 == 0.0;
        let from_end = class == Class::Deck && centre_end;
        let on_centre = match class {
            Class::Bottom => centre_start,
            Class::Deck => centre_end,
            Class::Side => false,
        };
        let start = if on_centre && strip.side > 0.0 { 0 } else { 1 };
        for kk in start.. {
            let d = kk as f64 * sp;
            if kk > 0 && d >= g - EPS {
                break;
            }
            out.push((kk, if from_end { g - d } else { d }));
        }
        out
    }

    /// The plates and longitudinals of one strip.
    #[allow(clippy::too_many_arguments)]
    fn strip(
        &self,
        acc: &mut Acc,
        k: usize,
        m: &PanelMember,
        class: Class,
        strip: &Strip,
        u: (f64, f64),
        cut_outs: bool,
    ) {
        let spec = self.spec;
        let st = spec.structure;
        let rho = spec.steel_density;
        let sp = st.longitudinal_spacing_mm / 1000.0;
        let web = st.frame_spacing_m;
        if strip.girth <= EPS {
            return;
        }
        let kept = self.kept(strip, cut_outs);
        // The longest point line of the run sets the butts, along it.
        let lmax = (0..strip.idx.len())
            .map(|i| dist(strip.point(i, u.0), strip.point(i, u.1)))
            .fold(0.0, f64::max);
        if lmax <= EPS {
            return;
        }
        let (xa, xb) = (strip.point(0, u.0).0, strip.point(0, u.1).0);
        let frames = (xb - xa > EPS && web > EPS).then(|| {
            let first = self.x0() + ceil((xa - self.x0()) / web - EPS) * web;
            let k = lmax / (xb - xa);
            ((first - xa) * k, web * k)
        });
        // The seams keep clear of the longitudinals, which stand one spacing apart from the keel or
        // the run's first point, or (the deck) from the centre line at its end.
        let lines = self.longitudinals(class, strip, sp);
        let phase = lines.first().map_or(0.0, |l| l.1);
        let seam_lines = (!m.stiffeners.is_empty() && sp > 0.0).then_some((phase, sp));
        let t_net = m.plate_mm;
        let t_ord = ordered_mm(t_net, st.thickness_step_mm);
        let (mut w_eff, mut l_eff) = (st.stock_plate_width_m, st.stock_plate_length_m);
        let mut pieces: Vec<((f64, f64), (f64, f64))> = Vec::new();
        for _ in 0..60 {
            pieces.clear();
            let butts: Vec<f64> = cuts(lmax, l_eff, frames)
                .into_iter()
                .map(|c| u.0 + (u.1 - u.0) * c / lmax)
                .collect();
            for &(g0, g1) in &kept {
                let seams: Vec<f64> = cuts(g1 - g0, w_eff, rel(seam_lines, g0))
                    .into_iter()
                    .map(|c| g0 + c)
                    .collect();
                for bw in butts.windows(2) {
                    for sw in seams.windows(2) {
                        pieces.push(((bw[0], bw[1]), (sw[0], sw[1])));
                    }
                }
            }
            let (mut wide, mut long) = (false, false);
            for (pu, pg) in &pieces {
                let (len, wid) = strip.size(*pu, *pg);
                wide |= wid > st.stock_plate_width_m + 1e-9;
                long |= len > st.stock_plate_length_m + 1e-9;
            }
            if !wide && !long {
                break;
            }
            if wide {
                w_eff *= 0.97;
            }
            if long {
                l_eff *= 0.97;
            }
        }
        for (pu, pg) in &pieces {
            let (area, centre) = strip.area(*pu, *pg);
            if area <= EPS * EPS {
                continue;
            }
            let (len, wid) = strip.size(*pu, *pg);
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
            it.centre_m = centre;
            it.geometry.points = strip.outline(*pu, *pg);
            acc.items.push(it);
        }
        // The longitudinals of the strip, in the plated part of its girth.
        if m.stiffeners.is_empty() {
            return;
        }
        for (n_line, g) in lines {
            if !kept.iter().any(|(g0, g1)| g >= *g0 - EPS && g <= *g1 + EPS) {
                continue;
            }
            let s = pick(
                &m.stiffeners,
                if class == Class::Side {
                    n_line.saturating_sub(1)
                } else {
                    0
                },
            );
            let (p0, p1) = (strip.at(u.0, g), strip.at(u.1, g));
            acc.line(m, &st, ItemKind::Stiffener, s, dist(p0, p1), rho, p0, p1);
        }
    }

    /// Plates cut from planar polygons: butts along s at the stock length, seams along t at the
    /// stock width, each clear of its lines (`(phase, spacing)` in s or t). As one primary member
    /// (a floor) when `primary`. `region` renames the items' region.
    #[allow(clippy::too_many_arguments)]
    fn flat(
        &self,
        acc: &mut Acc,
        k: usize,
        m: &PanelMember,
        t_net: f64,
        plane: Plane,
        polys: &[Vec<P2>],
        butt_lines: Option<(f64, f64)>,
        seam_lines: Option<(f64, f64)>,
        primary: bool,
        region: Option<&str>,
    ) -> f64 {
        let st = self.spec.structure;
        let rho = self.spec.steel_density;
        let t_ord = ordered_mm(t_net, st.thickness_step_mm);
        let all: Vec<P2> = polys.iter().flatten().copied().collect();
        if all.is_empty() {
            return 0.0;
        }
        let (s0, s1) = extent(&all, |p| p.0);
        let (t0, t1) = extent(&all, |p| p.1);
        let butts: Vec<f64> = cuts(s1 - s0, st.stock_plate_length_m, rel(butt_lines, s0))
            .into_iter()
            .map(|c| s0 + c)
            .collect();
        let seams: Vec<f64> = cuts(t1 - t0, st.stock_plate_width_m, rel(seam_lines, t0))
            .into_iter()
            .map(|c| t0 + c)
            .collect();
        let mut total = 0.0;
        let mut moment = (0.0, 0.0);
        let mut n_pieces = 0;
        let mut outline: Vec<P2> = Vec::new();
        for bw in butts.windows(2) {
            for sw in seams.windows(2) {
                for poly in polys {
                    let c = clip_box(poly, (bw[0], bw[1]), (sw[0], sw[1]));
                    if c.len() < 3 {
                        continue;
                    }
                    let (a, cen) = area_centroid(&c);
                    let a = fabs(a);
                    if a <= EPS * EPS {
                        continue;
                    }
                    let (ls, le) = extent(&c, |p| p.0);
                    let (ws, we) = extent(&c, |p| p.1);
                    let (len, wid) = (le - ls, we - ws);
                    acc.cut.push((k, t_ord, wid, len, a));
                    total += a;
                    moment = (moment.0 + a * cen.0, moment.1 + a * cen.1);
                    n_pieces += 1;
                    if primary {
                        continue;
                    }
                    let mut it = acc.item(m, &st, ItemKind::Plate);
                    if let Some(r) = region {
                        it.region = r.into();
                    }
                    it.rule_net_mm = t_net;
                    it.ordered_mm = t_ord;
                    it.length_m = len;
                    it.width_m = wid;
                    it.area_m2 = a;
                    it.pieces = 1;
                    it.rule_net_t = a * t_net / 1000.0 * rho;
                    it.ordered_t = a * t_ord / 1000.0 * rho;
                    it.centre_m = plane.at(cen);
                    it.geometry.points = c.iter().map(|p| plane.at(*p)).collect();
                    acc.items.push(it);
                }
            }
        }
        if primary && total > EPS * EPS {
            for poly in polys {
                if fabs(area_centroid(poly).0) > fabs(area_centroid(&outline).0) {
                    outline = poly.clone();
                }
            }
            let mut it = acc.item(m, &st, ItemKind::Primary);
            if let Some(r) = region {
                it.region = r.into();
            }
            it.rule_net_mm = t_net;
            it.ordered_mm = t_ord;
            it.length_m = s1 - s0;
            it.width_m = t1 - t0;
            it.area_m2 = total;
            it.pieces = n_pieces;
            it.rule_net_t = total * t_net / 1000.0 * rho;
            it.ordered_t = total * t_ord / 1000.0 * rho;
            it.centre_m = plane.at((moment.0 / total, moment.1 / total));
            it.geometry.points = outline.iter().map(|p| plane.at(*p)).collect();
            acc.items.push(it);
        }
        total
    }

    /// Stiffeners on planar polygons: one line per position `cs`, along s (`along_s`) or t, each
    /// piece inside the polygons one item; the bar of line `i` is `bar(i)`.
    #[allow(clippy::too_many_arguments)]
    fn flat_lines<'b>(
        &self,
        acc: &mut Acc,
        m: &'b PanelMember,
        kind: ItemKind,
        plane: Plane,
        polys: &[Vec<P2>],
        along_s: bool,
        cs: &[f64],
        bar: impl Fn(usize) -> &'b Bar,
        region: Option<&str>,
    ) {
        let st = self.spec.structure;
        let rho = self.spec.steel_density;
        for (i, c) in cs.iter().enumerate() {
            for poly in polys {
                for (a, b) in scan(poly, along_s, *c) {
                    if b - a <= EPS {
                        continue;
                    }
                    let (p0, p1) = if along_s {
                        (plane.at((a, *c)), plane.at((b, *c)))
                    } else {
                        (plane.at((*c, a)), plane.at((*c, b)))
                    };
                    let first = acc.items.len();
                    acc.line(m, &st, kind, bar(i), b - a, rho, p0, p1);
                    if let Some(r) = region {
                        acc.items[first].region = r.into();
                    }
                }
            }
        }
    }

    /// The section of the hull at x (on the ruled surface; at a step, the first Section).
    fn section_at(&self, x: f64) -> Section {
        self.table.section_at(x).0
    }

    /// The x of the sample points along the hull where `f(section) - level` changes sign, with the
    /// Stations: each Station interval split where `f` (linear along it) crosses the level.
    fn samples(&self, f: impl Fn(&Section) -> Option<f64>, level: f64) -> Vec<Section> {
        let ss = &self.table.sections;
        let mut out: Vec<Section> = Vec::new();
        for (i, w) in ss.windows(2).enumerate() {
            let (a, b) = (&w[0], &w[1]);
            if i == 0 {
                out.push(a.clone());
            }
            if b.x > a.x {
                let (fa, fb) = (f(a).unwrap_or(f64::INFINITY), f(b).unwrap_or(f64::INFINITY));
                if fa.is_finite() && fb.is_finite() && (fa - level) * (fb - level) < 0.0 {
                    let u = (level - fa) / (fb - fa);
                    out.push(between(a, b, u));
                }
            }
            out.push(b.clone());
        }
        out
    }

    /// The inner bottom: at z = the double bottom height, across the hull where its keel is below.
    fn inner_bottom(&self, acc: &mut Acc, k: usize, m: &PanelMember) {
        let db = self.spec.double_bottom_m;
        let st = self.spec.structure;
        let sp = st.longitudinal_spacing_mm / 1000.0;
        let width = |s: &Section| -> f64 {
            if keel_of(s) < db - 1e-9 {
                half_breadth(s, db)
            } else {
                // Where the keel is at the inner bottom: its edge from the wet side.
                0.0f64.max(if fabs(keel_of(s) - db) <= 1e-9 {
                    half_breadth(s, db + 1e-9)
                } else {
                    0.0
                })
            }
        };
        let samples = self.samples(|s| Some(keel_of(s)), db);
        // At each crossing the inner bottom starts or stops: a step between the wet side and 0.
        let mut chain: Vec<P2> = Vec::new();
        for (i, s) in samples.iter().enumerate() {
            let w = width(s);
            let at_level = fabs(keel_of(s) - db) <= 1e-9;
            if at_level {
                // Which side is wet: the sample before or after.
                let before = i > 0 && keel_of(&samples[i - 1]) < db - 1e-9;
                if before {
                    chain.push((s.x, w));
                    chain.push((s.x, 0.0));
                } else {
                    chain.push((s.x, 0.0));
                    chain.push((s.x, w));
                }
            } else {
                chain.push((s.x, w));
            }
        }
        let mut poly: Vec<P2> = chain.iter().map(|&(x, w)| (x, -w)).collect();
        poly.extend(chain.iter().rev().map(|&(x, w)| (x, w)));
        let polys = alloc::vec![poly];
        let web = st.frame_spacing_m;
        let x0 = self.x0();
        self.flat(
            acc,
            k,
            m,
            m.plate_mm,
            Plane::Z(db),
            &polys,
            Some((x0, web)),
            Some((0.0, sp)),
            false,
            None,
        );
        if m.stiffeners.is_empty() || !positive(sp) {
            return;
        }
        let hb = self.spec.beam_m / 2.0;
        let n = libm::floor(hb / sp + EPS) as i64;
        let ys: Vec<f64> = (-n..=n).map(|i| i as f64 * sp).collect();
        self.flat_lines(
            acc,
            m,
            ItemKind::Stiffener,
            Plane::Z(db),
            &polys,
            true,
            &ys,
            |_| &m.stiffeners[0].0,
            None,
        );
    }

    /// Girders at each y: from the bottom up to the inner bottom, where the bottom is below it.
    fn girders(&self, acc: &mut Acc, k: usize, m: &PanelMember, ys: &[f64]) {
        let db = self.spec.double_bottom_m;
        let st = self.spec.structure;
        let x0 = self.x0();
        let l = self.spec.loa_m;
        for &y in ys {
            let samples = self.samples(|s| bottom_at(s, y), db);
            let mut poly: Vec<P2> = samples
                .iter()
                .map(|s| (s.x, bottom_at(s, y).unwrap_or(db).min(db)))
                .collect();
            let (first, last) = (poly[0].0, poly[poly.len() - 1].0);
            poly.push((last, db));
            poly.push((first, db));
            if m.member == ft::CENTRE_GIRDER {
                // 85 % of the thickness over 0.125L at each end (ABS 5-1-2/5.1.2).
                let ends = crate::scantling_rules::coefficient(
                    "abs-steel-barges/5-1-2.5.1.2/center-girder-ends",
                    "value",
                );
                let (xa, xf) = (x0 + 0.125 * l, x0 + 0.875 * l);
                for (lo, hi, t) in [
                    (f64::NEG_INFINITY, xa, ends * m.plate_mm),
                    (xa, xf, m.plate_mm),
                    (xf, f64::INFINITY, ends * m.plate_mm),
                ] {
                    let part = clip_by(&clip_by(&poly, |p| p.0 - lo), |p| hi - p.0);
                    if part.len() >= 3 {
                        let polys = alloc::vec![part];
                        self.flat(
                            acc,
                            k,
                            m,
                            t,
                            Plane::Y(y),
                            &polys,
                            Some((x0, st.frame_spacing_m)),
                            None,
                            false,
                            None,
                        );
                    }
                }
            } else {
                let polys = alloc::vec![poly];
                self.flat(
                    acc,
                    k,
                    m,
                    m.plate_mm,
                    Plane::Y(y),
                    &polys,
                    Some((x0, st.frame_spacing_m)),
                    None,
                    false,
                    None,
                );
            }
        }
    }

    /// Floors: the hull's section below the inner bottom at each frame.
    fn floors(&self, acc: &mut Acc, k: usize, m: &PanelMember) {
        let db = self.spec.double_bottom_m;
        let st = self.spec.structure;
        let sp = st.longitudinal_spacing_mm / 1000.0;
        let x0 = self.x0();
        for xf in frames_n(st.frame_spacing_m, self.spec.loa_m, m.count) {
            let x = x0 + xf;
            let polys: Vec<Vec<P2>> = rings(&self.section_at(x))
                .iter()
                .map(|r| clip_by(r, |p| db - p.1))
                .filter(|r| r.len() >= 3)
                .collect();
            let area = self.flat(
                acc,
                k,
                m,
                m.plate_mm,
                Plane::X(x),
                &polys,
                None,
                None,
                true,
                None,
            );
            if area <= EPS * EPS || m.stiffeners.is_empty() || !positive(sp) {
                continue;
            }
            let hb = self.spec.beam_m / 2.0;
            let n = libm::floor(hb / sp + EPS) as i64;
            let ys: Vec<f64> = (-n..=n).map(|i| i as f64 * sp).collect();
            self.flat_lines(
                acc,
                m,
                ItemKind::Stiffener,
                Plane::X(x),
                &polys,
                false,
                &ys,
                |_| &m.stiffeners[0].0,
                None,
            );
        }
    }

    /// The transverse bulkheads: the aft peak, the holds and the collision bulkhead, each the hull's
    /// section above the inner bottom.
    fn bulkheads(&self, acc: &mut Acc, k: usize, m: &PanelMember, holds: &[f64]) {
        let spec = self.spec;
        let st = spec.structure;
        let db = spec.double_bottom_m;
        let sp = st.longitudinal_spacing_mm / 1000.0;
        let n = m.count;
        let xs: Vec<f64> = if n >= 2 && holds.len() + 2 == n {
            let mut v = alloc::vec![spec.cargo_zone_aft_m / 2.0];
            v.extend_from_slice(holds);
            v.push((spec.cargo_zone_fwd_m + spec.loa_m) / 2.0);
            v
        } else {
            bulkhead_xs(spec, n)
        };
        let hb_box = (spec.depth_m - db).max(EPS);
        for x in xs {
            let polys: Vec<Vec<P2>> = rings(&self.section_at(x))
                .iter()
                .map(|r| clip_by(r, |p| p.1 - db))
                .filter(|r| r.len() >= 3)
                .collect();
            let seams = (!m.stiffeners.is_empty() && sp > 0.0).then_some((db, sp));
            let area = self.flat(
                acc,
                k,
                m,
                m.plate_mm,
                Plane::X(x),
                &polys,
                None,
                seams,
                false,
                None,
            );
            if area <= EPS * EPS {
                continue;
            }
            let all: Vec<P2> = polys.iter().flatten().copied().collect();
            let (_, top) = extent(&all, |p| p.1);
            if sp > 0.0 && !m.stiffeners.is_empty() {
                let mut zs = Vec::new();
                let mut z = db + sp;
                while z < top - EPS {
                    zs.push(z);
                    z += sp;
                }
                self.flat_lines(
                    acc,
                    m,
                    ItemKind::Stiffener,
                    Plane::X(x),
                    &polys,
                    true,
                    &zs,
                    |i| pick(&m.stiffeners, i),
                    None,
                );
            }
            for (bar, len) in &m.primary_members {
                let n_v = (libm::round(len / hb_box) as usize).max(1);
                let (s0, s1) = extent(&all, |p| p.0);
                let ys: Vec<f64> = spread(n_v, s0, s1);
                self.flat_lines(
                    acc,
                    m,
                    ItemKind::Primary,
                    Plane::X(x),
                    &polys,
                    false,
                    &ys,
                    |_| bar,
                    None,
                );
            }
        }
    }

    /// The plating of the hull's two end faces (a transom, a square bow), at the side shell's end
    /// thickness, with horizontal stiffeners one spacing apart; its own group.
    pub(crate) fn end_plating(&self, acc: &mut Acc, panels: &HullPanelModel) {
        let Some(m) = panels
            .members
            .iter()
            .find(|p| p.member == ft::SIDE_SHELL && p.region == region::ENDS)
        else {
            return;
        };
        let st = self.spec.structure;
        let sp = st.longitudinal_spacing_mm / 1000.0;
        let k = panels.members.len();
        let first = acc.items.len();
        let ss = &self.table.sections;
        for s in [&ss[0], &ss[ss.len() - 1]] {
            let polys = rings(s);
            let seams = (!m.stiffeners.is_empty() && sp > 0.0).then_some((0.0, sp));
            let area = self.flat(
                acc,
                k,
                m,
                m.plate_mm,
                Plane::X(s.x),
                &polys,
                None,
                seams,
                false,
                Some(region::END_PLATING),
            );
            if area <= EPS * EPS || m.stiffeners.is_empty() || !positive(sp) {
                continue;
            }
            let all: Vec<P2> = polys.iter().flatten().copied().collect();
            let (_, top) = extent(&all, |p| p.1);
            let mut zs = Vec::new();
            let mut z = sp;
            while z < top - EPS {
                zs.push(z);
                z += sp;
            }
            self.flat_lines(
                acc,
                m,
                ItemKind::Stiffener,
                Plane::X(s.x),
                &polys,
                true,
                &zs,
                |i| pick(&m.stiffeners, i),
                Some(region::END_PLATING),
            );
        }
        acc.close(ft::SIDE_SHELL, region::END_PLATING, first);
    }
}

/// Entry `i` of a panel's stiffeners, or its last.
fn pick(stiffeners: &[(Bar, f64)], i: usize) -> &Bar {
    &stiffeners[i.min(stiffeners.len() - 1)].0
}

/// The runs of a half-section: read on the Section with the lowest keel, the deck from the first
/// point at the deck edge height to the end, and below it each edge bottom (flatter than 45
/// degrees) or side (steeper). An edge of no length takes the member of the edge before it.
fn runs_of(t: &SectionsTable) -> Vec<(Class, Vec<usize>)> {
    let (ri, r) = t
        .sections
        .iter()
        .enumerate()
        .fold((0, &t.sections[0]), |(bi, b), (i, s)| {
            if keel_of(s) < keel_of(b) {
                (i, s)
            } else {
                (bi, b)
            }
        });
    let pts = &r.points;
    let n = pts.len();
    let edge = t.depth_at_side[ri];
    let k_e = pts.iter().position(|p| p.1 >= edge - 1e-9).unwrap_or(n - 1);
    let mut classes: Vec<Class> = Vec::with_capacity(n - 1);
    for j in 0..n - 1 {
        let c = if j >= k_e {
            Class::Deck
        } else {
            let (dy, dz) = (fabs(pts[j + 1].0 - pts[j].0), fabs(pts[j + 1].1 - pts[j].1));
            if dy + dz <= 0.0 {
                classes.last().copied().unwrap_or(Class::Bottom)
            } else if dz <= dy {
                Class::Bottom
            } else {
                Class::Side
            }
        };
        classes.push(c);
    }
    let mut runs: Vec<(Class, Vec<usize>)> = Vec::new();
    for (j, c) in classes.iter().enumerate() {
        match runs.last_mut() {
            Some((rc, idx)) if rc == c => idx.push(j + 1),
            _ => runs.push((*c, alloc::vec![j, j + 1])),
        }
    }
    runs
}
