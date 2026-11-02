//! The damage data a Rule Set gives the damage case generator (spec 0012): the IGC 2.3.1 extents and the IGC
//! 2.6.1 standard of damage (issue #757), and the Permeability Tables (issue #758), read from the set's tables,
//! so the numbers live in the Rule Set and not in code.

use arc_rules::eval::{MeasureValue, table_value};
use arc_rules::model::RuleSet;
use ovd_core::compartments::SpaceKind;
use ovd_core::damage_cases::{BoxExtents, Permeability, PermeabilityTable};

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

/// Read a Permeability Table from a Rule Set (spec 0012 D9). Table `table_id` gives the permeability of each
/// `SpaceKind` by its key (a liquid space: the value when it is empty). The kinds that table `<table_id>-by-filling`
/// names (if the set has it) take their value times the empty fraction of the condition's filling. An error
/// names a missing table or a kind the table does not give.
pub fn permeability_table(set: &RuleSet, table_id: &str) -> Result<PermeabilityTable, String> {
    let find = |id: &str| set.tables.iter().find(|t| t.id == id);
    let table =
        find(table_id).ok_or_else(|| format!("{} holds no table `{table_id}`", set.key()))?;
    let by_filling = find(&format!("{table_id}-by-filling"));
    let mut rows = Vec::new();
    for kind in SpaceKind::ALL {
        let key = MeasureValue::Text(kind.key().to_string());
        let value = table_value(table, &key).map_err(|e| {
            format!(
                "{}: `{table_id}` gives no value for `{}`: {e}",
                set.key(),
                kind.key()
            )
        })?;
        let liquid = by_filling.is_some_and(|t| table_value(t, &key).is_ok_and(|v| v != 0.0));
        rows.push((
            kind,
            if liquid {
                Permeability::ByFilling(value)
            } else {
                Permeability::Fixed(value)
            },
        ));
    }
    Ok(PermeabilityTable { rows })
}

/// The Permeability Table `table_id` of the highest built-in version of set `set_id` that holds it. A run reads
/// the liquid permeabilities this way whatever version of the set the design pins: the older versions hold no
/// table, and the values were in code before.
pub fn built_in_permeability(set_id: &str, table_id: &str) -> Result<PermeabilityTable, String> {
    let lib = crate::rule_set::built_in();
    let mut sets: Vec<&RuleSet> = lib
        .sets()
        .iter()
        .filter(|s| s.id == set_id && s.tables.iter().any(|t| t.id == table_id))
        .collect();
    sets.sort_by(|a, b| {
        if crate::rule_set::is_newer(&a.version, &b.version) {
            core::cmp::Ordering::Greater
        } else if crate::rule_set::is_newer(&b.version, &a.version) {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Equal
        }
    });
    let set = sets
        .last()
        .ok_or_else(|| format!("no built-in `{set_id}` holds table `{table_id}`"))?;
    permeability_table(set, table_id)
}
