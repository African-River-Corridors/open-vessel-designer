//! The damage data a Rule Set gives the damage case generator (spec 0012, issue #757): the IGC 2.3.1 extents
//! and the IGC 2.6.1 standard of damage, read from the set's tables, so the numbers live in the Rule Set and
//! not in code.

use arc_rules::eval::{MeasureValue, table_value};
use arc_rules::model::RuleSet;
use ovd_core::damage_cases::BoxExtents;

/// The IGC damage data of a hull of length `loa_m` and beam `beam_m`: the extents of IGC 2.3.1, and whether
/// the bulkheads of a machinery space aft stay intact (IGC 2.6.1.3, a type 2G ship of 150 m or less).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IgcDamageData {
    pub extents: BoxExtents,
    pub machinery_aft_intact: bool,
}

/// The table ids of a set that holds IGC damage data.
pub const IGC_DAMAGE_TABLES: &[&str] = &[
    "igc-damage-side-long",
    "igc-damage-side-inboard",
    "igc-damage-bottom-long",
    "igc-damage-bottom-width-forward",
    "igc-damage-bottom-width",
    "igc-damage-bottom-height",
    "igc-damage-forward-zone",
    "igc-damage-machinery-aft-intact",
];

/// Read the IGC damage data from a Rule Set's tables. The two longitudinal extents are keyed on `L^(2/3)`;
/// the widths and the height on `B`; the forward zone and the standard of damage on `L`. An error names a
/// table the set does not hold.
pub fn igc_damage_data(set: &RuleSet, loa_m: f64, beam_m: f64) -> Result<IgcDamageData, String> {
    let at = |id: &str, v: f64| -> Result<f64, String> {
        let table = set
            .tables
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("{} holds no table `{id}`", set.key()))?;
        table_value(table, &MeasureValue::Number(v))
    };
    let cbrt = libm::cbrt(loa_m);
    let l23 = cbrt * cbrt;
    Ok(IgcDamageData {
        extents: BoxExtents {
            side_long_m: at("igc-damage-side-long", l23)?,
            side_inboard_m: at("igc-damage-side-inboard", beam_m)?,
            bottom_fwd_long_m: at("igc-damage-bottom-long", l23)?,
            bottom_fwd_width_m: at("igc-damage-bottom-width-forward", beam_m)?,
            bottom_aft_long_m: at("igc-damage-bottom-long", l23)?,
            bottom_aft_width_m: at("igc-damage-bottom-width", beam_m)?,
            bottom_height_m: at("igc-damage-bottom-height", beam_m)?,
            forward_zone: at("igc-damage-forward-zone", loa_m)?,
        },
        machinery_aft_intact: at("igc-damage-machinery-aft-intact", loa_m)? != 0.0,
    })
}
