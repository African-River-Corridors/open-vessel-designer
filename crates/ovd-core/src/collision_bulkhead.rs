//! The collision bulkhead plane (issue #994; ES-TRIN 2025/1 Article 3.03(1)(a) and 1.01, 4.25).
//!
//! The plane stands a distance aft of the forward perpendicular, which is the forward point where the
//! hull meets the water at the maximum draught. On a raked bow the keel ends `rake` aft of the bow and
//! rises `rise` to the deck at the bow, so the hull meets the water at `rake * (1 - T / rise)` aft of
//! the bow. x is metres forward of the stern, as the arrangement reports it.

/// The distance of the collision bulkhead from the forward perpendicular as a fraction of L:
/// the bottom of the band ES-TRIN 3.03(1)(a) gives as a general rule (0.04 L to 0.04 L + 2 m).
pub const DEFAULT_FRACTION_OF_L: f64 = 0.04;

/// Where the collision bulkhead stands on one hull.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionBulkhead {
    /// The forward perpendicular, m forward of the stern.
    pub forward_perpendicular_x_m: f64,
    /// The distance of the plane aft of the forward perpendicular (m).
    pub from_fp_m: f64,
    /// The plane, m forward of the stern.
    pub x_m: f64,
    /// L, the hull's length overall (ES-TRIN 1.01, 4.16), for the band rows.
    pub length_m: f64,
}

/// The forward perpendicular, m forward of the stern, for a bow of rake `rake_m` rising `rise_m` to the
/// deck, at the maximum draught `t_max_m`. At or above the rise, and with no rake, it is the bow.
pub fn forward_perpendicular_x_m(loa_m: f64, rake_m: f64, rise_m: f64, t_max_m: f64) -> f64 {
    if rake_m <= 0.0 || rise_m <= 0.0 {
        return loa_m;
    }
    loa_m - rake_m * (1.0 - t_max_m / rise_m).max(0.0)
}

/// The default distance of the plane from the forward perpendicular (m): 0.04 L.
pub fn default_distance_m(loa_m: f64) -> f64 {
    DEFAULT_FRACTION_OF_L * loa_m
}

/// The plane of a hull: `distance_m` is the typed distance, or `None` for the default.
pub fn locate(
    loa_m: f64,
    rake_m: f64,
    rise_m: f64,
    t_max_m: f64,
    distance_m: Option<f64>,
) -> CollisionBulkhead {
    let fp = forward_perpendicular_x_m(loa_m, rake_m, rise_m, t_max_m);
    let from_fp = distance_m.unwrap_or_else(|| default_distance_m(loa_m));
    CollisionBulkhead {
        forward_perpendicular_x_m: fp,
        from_fp_m: from_fp,
        x_m: fp - from_fp,
        length_m: loa_m,
    }
}
