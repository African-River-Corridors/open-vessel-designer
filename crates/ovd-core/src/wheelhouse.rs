//! The wheelhouse sightline as a distance (issue #354, spec 0007 "Wheelhouse, sightline and air
//! draft"): the Blind Distance ahead of the bow, as ES-TRIN 7.02(2) and SOLAS V/22.1.1 state the rule,
//! and the lift that brings it within a limit. The limit is not in this file: the Rule Sets of the stack
//! give it ([`BlindLimits`], issue #989).
//!
//! The ship frame: x forward from the aft end, z up from the keel. The water is the straight line
//! through the draughts aft (x = 0) and forward (x = L), so a trimmed waterline counts (#349). The
//! sightline from the eye over the top of an obstruction meets the water at x_hit; the Blind Distance
//! is the largest x_hit - x_bow of the obstructions. On an even keel this is the research note's
//! similar triangles (sec. 9.2, derived): x_o z_e / (z_e - z_o) - x_b with x from the eye and z above
//! the water. The lift needed works the same line backwards: the eye that sees the water at
//! x_bow + limit over every obstruction.

use alloc::string::String;
use alloc::vec::Vec;

/// The Blind Distance limits that bind a design (m), one for each loading condition, and the Rule that
/// sets each (issue #989). They come from the Rule Sets of the stack, never from a constant: a Rule that
/// measures `wheelhouse.blind_distance`, blocks, and applies to the design gives a limit, and the least
/// limit of a condition binds. A condition with no such Rule has no limit. See
/// [`crate::gas_arrangement::blind_limits`], which reads them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlindLimits {
    /// The limit in the light condition, and the id of the Rule that gives it.
    pub light_m: Option<f64>,
    pub light_rule: Option<String>,
    /// The limit in the laden condition, and the id of the Rule that gives it.
    pub laden_m: Option<f64>,
    pub laden_rule: Option<String>,
    /// The least light limit of the Rules that block or warn. A fixed deckhouse wheelhouse is sized to it
    /// (issue #779, ADR 0007): a stack that holds SOLAS V/22.1.1 as a warning (`solas-v22` 1.x) still
    /// sizes the house to two ship lengths, and only warns, as before. A lifting wheelhouse is sized to
    /// the blocking limit alone. With a blocking Rule only, it is the same as `light_m`.
    pub sizing_light_m: Option<f64>,
}

impl BlindLimits {
    /// Whether no Rule of the stack limits the Blind Distance.
    pub fn is_none(&self) -> bool {
        self.light_m.is_none() && self.laden_m.is_none()
    }
}

/// Something forward of the eye: its top above the keel at its forward edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Obstruction {
    pub label: &'static str,
    /// The forward edge, from the aft end (m).
    pub x_m: f64,
    pub top_above_keel_m: f64,
}

/// The water in the ship frame: the draughts at the aft end and at the forward end of length `l`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Waterline {
    pub draught_aft_m: f64,
    pub draught_fwd_m: f64,
    pub length_m: f64,
}

impl Waterline {
    pub fn even(draught_m: f64, length_m: f64) -> Self {
        Self {
            draught_aft_m: draught_m,
            draught_fwd_m: draught_m,
            length_m,
        }
    }

    /// The water's height above the keel at x.
    pub fn at(&self, x: f64) -> f64 {
        self.draught_aft_m + (self.draught_fwd_m - self.draught_aft_m) * x / self.length_m
    }

    fn slope(&self) -> f64 {
        (self.draught_fwd_m - self.draught_aft_m) / self.length_m
    }
}

/// The Blind Distance ahead of the bow (m) from the eye at (`eye_x`, `eye_z`), over the obstructions
/// forward of it. Infinite when an obstruction hides the water altogether; 0 with none forward.
pub fn blind_distance(
    eye_x: f64,
    eye_z: f64,
    obstructions: &[Obstruction],
    bow_x: f64,
    w: &Waterline,
) -> f64 {
    let mut worst: f64 = 0.0;
    for o in obstructions.iter().filter(|o| o.x_m > eye_x) {
        let ray = (o.top_above_keel_m - eye_z) / (o.x_m - eye_x);
        // The ray must fall faster than the water rises to meet it ahead.
        let closing = w.slope() - ray;
        if closing <= 0.0 {
            return f64::INFINITY;
        }
        let x_hit = eye_x + (eye_z - w.at(eye_x)) / closing;
        worst = worst.max(x_hit - bow_x);
    }
    worst
}

/// The lowest eye above the keel at `eye_x` that keeps the Blind Distance within `limit_m`; `None`
/// when an obstruction stands at or beyond the point it must see.
pub fn eye_needed(
    eye_x: f64,
    obstructions: &[Obstruction],
    bow_x: f64,
    w: &Waterline,
    limit_m: f64,
) -> Option<f64> {
    let x_seen = bow_x + limit_m;
    let z_seen = w.at(x_seen);
    let mut need = f64::NEG_INFINITY;
    for o in obstructions.iter().filter(|o| o.x_m > eye_x) {
        if o.x_m >= x_seen {
            return None;
        }
        // The line from the water at x_seen over the obstruction's top, back to the eye.
        let z =
            o.top_above_keel_m + (o.top_above_keel_m - z_seen) * (o.x_m - eye_x) / (x_seen - o.x_m);
        need = need.max(z);
    }
    Some(need)
}

/// The sightline of one loading condition, with the lift that meets the limit.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sightline {
    pub limit_m: f64,
    /// The lift the limit needs (m); infinite if no eye height meets it.
    pub lift_needed_m: f64,
    /// The lift used: the lift needed, at most the maximum lift.
    pub lift_used_m: f64,
    pub eye_raised_m: f64,
    pub floor_raised_m: f64,
    pub blind_distance_m: f64,
    /// The maximum lift is enough.
    pub enough: bool,
}

/// The lift that keeps the Blind Distance within `limit_m`, from the eye lowered at `eye_lowered_m`
/// above the keel (the floor `eye_height_m` below it), with at most `max_lift_m`.
#[allow(clippy::too_many_arguments)] // the eye, the column, the hull and the limit
pub fn assess(
    eye_x: f64,
    eye_lowered_m: f64,
    eye_height_m: f64,
    max_lift_m: f64,
    obstructions: &[Obstruction],
    bow_x: f64,
    w: &Waterline,
    limit_m: f64,
) -> Sightline {
    let needed = match eye_needed(eye_x, obstructions, bow_x, w, limit_m) {
        Some(z) => (z - eye_lowered_m).max(0.0),
        None => f64::INFINITY,
    };
    let cap = max_lift_m.max(0.0);
    let mut used = needed.min(cap);
    let mut eye = eye_lowered_m + used;
    let mut blind = blind_distance(eye_x, eye, obstructions, bow_x, w);
    // The solve meets the limit exactly, so rounding can leave the blind distance a hair above it.
    // Raise the lift by the smallest doubling step that brings it to the limit or below, within the
    // maximum lift. A fixed step sequence keeps native and WebAssembly bit-identical.
    let mut step = 1e-12 * eye.abs().max(1.0);
    for _ in 0..64 {
        if !(blind > limit_m && used < cap) {
            break;
        }
        used = (used + step).min(cap);
        eye = eye_lowered_m + used;
        blind = blind_distance(eye_x, eye, obstructions, bow_x, w);
        step *= 2.0;
    }
    Sightline {
        limit_m,
        lift_needed_m: needed,
        lift_used_m: used,
        eye_raised_m: eye,
        floor_raised_m: eye - eye_height_m,
        blind_distance_m: blind,
        enough: blind <= limit_m + 1e-9,
    }
}

/// [`assess`] in the light condition, and in the laden condition too when it has a limit (issue #989).
/// The lift is the larger of the two the limits need, within the maximum lift; the Blind Distance of
/// each condition is then worked at that eye. With no laden limit the answer is [`assess`] alone.
#[allow(clippy::too_many_arguments)] // the eye, the column, the hull and the two limits
pub fn assess_conditions(
    eye_x: f64,
    eye_lowered_m: f64,
    eye_height_m: f64,
    max_lift_m: f64,
    obstructions: &[Obstruction],
    bow_x: f64,
    light: (&Waterline, f64),
    laden: Option<(&Waterline, f64)>,
) -> Sightline {
    let s = assess(
        eye_x,
        eye_lowered_m,
        eye_height_m,
        max_lift_m,
        obstructions,
        bow_x,
        light.0,
        light.1,
    );
    let Some((laden_water, laden_limit)) = laden else {
        return s;
    };
    let l = assess(
        eye_x,
        eye_lowered_m,
        eye_height_m,
        max_lift_m,
        obstructions,
        bow_x,
        laden_water,
        laden_limit,
    );
    let used = s.lift_used_m.max(l.lift_used_m);
    let eye = eye_lowered_m + used;
    let blind_light = blind_distance(eye_x, eye, obstructions, bow_x, light.0);
    let blind_laden = blind_distance(eye_x, eye, obstructions, bow_x, laden_water);
    Sightline {
        limit_m: light.1,
        lift_needed_m: s.lift_needed_m.max(l.lift_needed_m),
        lift_used_m: used,
        eye_raised_m: eye,
        floor_raised_m: eye - eye_height_m,
        blind_distance_m: blind_light,
        enough: blind_light <= light.1 + 1e-9 && blind_laden <= laden_limit + 1e-9,
    }
}

/// The obstructions of a hull forward of its wheelhouse: the deck units, the cargo or tank top at the
/// forward end of the cargo zone, and the deck at the bow.
pub fn obstructions(
    units_top_m: Option<(f64, f64)>,
    cargo_top_m: (f64, f64),
    bow: (f64, f64),
) -> Vec<Obstruction> {
    let mut out = Vec::new();
    if let Some((x, z)) = units_top_m {
        out.push(Obstruction {
            label: "deck units",
            x_m: x,
            top_above_keel_m: z,
        });
    }
    out.push(Obstruction {
        label: "cargo top",
        x_m: cargo_top_m.0,
        top_above_keel_m: cargo_top_m.1,
    });
    out.push(Obstruction {
        label: "bow",
        x_m: bow.0,
        top_above_keel_m: bow.1,
    });
    out
}
