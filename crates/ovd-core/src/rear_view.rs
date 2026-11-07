//! The view aft from the wings of a forward wheelhouse (issue #994, design D3).
//!
//! A wheelhouse forward of the cargo has the cargo behind the conning position. SOLAS V/22.1.3 asks no
//! view astern from the conning position, but the bridge wings must see aft along the ship's side
//! (V/22.1.4 and 22.1.6, as the research note reads them), and ES-TRIN 7.02(3) may need cameras.
//!
//! The check here is on plan only, and it is against the beam: a wing that reaches out to `wing_y_m`
//! from the centreline sees the full hull side if it reaches the hull side (`wing_y_m` at least the
//! hull half-beam). Until the wing reach is an input, the wing is taken at the side of the forward
//! house, a side deck short of the hull side, so the check reports that shortfall: the honest result.
//! The cargo's outboard edge (the hull half-beam less the side clearance of the tanks) is kept as
//! detail: it says how far the wing is from the nearest thing it must see past. **No camera is credited**: no source for a camera standard was read, so cameras stay an open
//! item for the hazard study.

/// Where the wing stands against the cargo and the hull side. All y are metres from the centreline.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WingView {
    /// The outboard end of the wing: the side of the forward house.
    pub wing_y_m: f64,
    /// The outboard edge of the cargo (detail only: the check is against the hull side).
    pub cargo_outboard_y_m: f64,
    /// The hull side.
    pub hull_half_beam_m: f64,
    /// How far the wing falls short of the hull side (m); zero when it does not.
    pub short_by_m: f64,
    /// The wing sees the full hull side aft.
    pub sees_full_hull_side: bool,
}

/// What a wing at `wing_y_m` sees of the hull side, on a hull of half-beam `hull_half_beam_m`, with
/// cargo whose outboard edge is at `cargo_outboard_y_m` (detail only).
pub fn wing_view(wing_y_m: f64, cargo_outboard_y_m: f64, hull_half_beam_m: f64) -> WingView {
    // A wing level with the hull side sees along it: a rounding step is not a failure.
    let short = hull_half_beam_m - wing_y_m;
    let short_by = if short > 1e-9 { short } else { 0.0 };
    WingView {
        wing_y_m,
        cargo_outboard_y_m,
        hull_half_beam_m,
        short_by_m: short_by,
        sees_full_hull_side: short_by == 0.0,
    }
}

/// The note the results carry with the wing view: cameras are an open item.
pub const CAMERAS_OPEN_ITEM: &str = "ES-TRIN 7.02(3) may need cameras for the view aft; no camera standard has been read, so none is credited and the hazard study must settle it.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wing_at_the_hull_side_sees_it() {
        let w = wing_view(14.5, 13.7, 14.5);
        assert!(w.sees_full_hull_side && w.short_by_m == 0.0);
    }

    #[test]
    fn a_wing_short_of_the_hull_side_does_not_whatever_the_cargo_does() {
        // Level with the cargo's edge, a side deck inside the hull side: short by the side deck.
        let w = wing_view(13.7, 13.7, 14.5);
        assert!(!w.sees_full_hull_side);
        assert!((w.short_by_m - 0.8).abs() < 1e-12);
        // No cargo at all (the edge at the centreline) changes nothing.
        assert!(!wing_view(13.7, 0.0, 14.5).sees_full_hull_side);
    }
}
