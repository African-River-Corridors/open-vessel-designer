//! Gross tonnage by the International Convention on Tonnage Measurement of Ships, 1969 (ITC 1969),
//! spec 0011 D15 (issue #777).
//!
//! The text implemented is Annex I of the Convention as registered in the United Nations Treaty
//! Series, vol. 1291, No. I-21264 (1982):
//!
//! - Regulation 3, "Gross tonnage", p. 15: "GT = K1 V", where "V = Total volume of all enclosed
//!   spaces of the ship in cubic metres" and "K1 = 0.2 + 0.02 log10 V (or as tabulated in
//!   appendix 2)".
//! - Regulation 2(4), "Enclosed spaces", pp. 13-14: spaces "bounded by the ship's hull, by fixed or
//!   portable partitions or bulkheads, by decks or coverings other than permanent or movable
//!   awnings".
//! - Regulation 6(1), "Calculation of volumes", p. 17: volumes are measured "to the inner side of
//!   the shell or structural boundary plating in ships constructed of metal", which is the moulded
//!   surface OVD's Sections Table describes.
//!
//! US 46 CFR 69.57 (Convention Measurement System) reproduces the same formula and definition of V.
//!
//! What OVD counts in V, and what it leaves out, is in [`EnclosedVolume`]. The Convention does not
//! round GT in Regulation 3, so neither does this module.

use crate::aft_arrangement::AftArrangementResult;
use crate::barge_capacity::{BargeCapacityInputs, BargeCapacityResult};
use crate::sections::{SectionsTable, Waterplane, immersed};

/// The citation of the formula, as the results and the Measure registry give it.
pub const BASIS: &str = "ITC 1969, Annex I, Regulation 3 (UN Treaty Series vol. 1291, No. I-21264, p. 15): GT = K1 V, K1 = 0.2 + 0.02 log10 V; V the total volume of all enclosed spaces (Regulation 2(4)), measured to the inner side of the shell (Regulation 6(1))";

/// What OVD puts in V, and what it leaves out.
pub const VOLUME_BASIS: &str = "V = the moulded hull volume to the deck at the centre line (sheer and camber included) plus the enclosed volume above the main deck that the stern block has today (the 40 ft units and the wheelhouse). The tiers of the deckhouse (spec 0011) join V when the Arrangement Style is deckhouse, and are nothing for the container block. No hatch coamings, no appendages, and no excluded spaces (Regulation 2(5)) are taken off";

/// K1 of Regulation 3: 0.2 + 0.02 log10 V. V in m³; for V <= 0 it is not defined, so 0.
pub fn k1(volume_m3: f64) -> f64 {
    if volume_m3 > 0.0 {
        0.2 + 0.02 * libm::log10(volume_m3)
    } else {
        0.0
    }
}

/// GT of Regulation 3: K1 V. V in m³; 0 for V <= 0.
pub fn gross_tonnage(volume_m3: f64) -> f64 {
    if volume_m3 > 0.0 {
        k1(volume_m3) * volume_m3
    } else {
        0.0
    }
}

/// The enclosed volume V of one hull, by its parts (m³).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnclosedVolume {
    /// The moulded hull volume up to the deck at the centre line: the whole of each section.
    pub hull_m3: f64,
    /// The enclosed spaces above the main deck that the stern block has: 40 ft units and the
    /// wheelhouse. Zero for a hull with no stern block of its own.
    pub stern_block_m3: f64,
    /// The tiers of the deckhouse of spec 0011 (issue #779): zero for the container block.
    pub deckhouse_m3: f64,
}

impl EnclosedVolume {
    /// V, the total (m³).
    pub fn total_m3(&self) -> f64 {
        self.hull_m3 + self.stern_block_m3 + self.deckhouse_m3
    }
}

/// The moulded volume of the whole hull: every section closed from the keel to the deck at the
/// centre line, so sheer and camber are inside it (m³).
pub fn hull_volume_m3(table: &SectionsTable) -> f64 {
    let top = table
        .sections
        .iter()
        .flat_map(|s| s.points.iter().map(|&(_, z)| z))
        .fold(0.0_f64, f64::max);
    // A waterplane above the highest point takes in the whole section.
    immersed(table, &Waterplane::upright(top + 1.0)).volume_m3
}

/// The enclosed volume above the main deck of a stern block (m³): each 40 ft unit (its length along
/// the hull x its width x `unit_height_m`) and the wheelhouse (length x width x height). Each is
/// bounded by fixed walls, a floor and a roof, so each is an enclosed space under Regulation 2(4).
pub fn stern_block_above_deck_m3(res: &AftArrangementResult, unit_height_m: f64) -> f64 {
    let units: f64 = res
        .container_units
        .iter()
        .map(|(_, x_aft, x_fwd, _)| (x_fwd - x_aft) * res.container_unit_width_m * unit_height_m)
        .sum();
    let wheelhouse = (res.wheelhouse_x1_m - res.wheelhouse_x0_m)
        * res.wheelhouse_width_m
        * res.wheelhouse_height_m;
    units + wheelhouse.max(0.0)
}

/// V of a computed design (m³): its hull from the Sections Table, its stern block from the
/// capacity run, and the tiers of its deckhouse when the Arrangement Style is `deckhouse` (issue
/// #779; zero for the container block).
pub fn enclosed_volume(
    capacity: &BargeCapacityInputs,
    result: &BargeCapacityResult,
    hull: &SectionsTable,
) -> EnclosedVolume {
    let unit_height_m = capacity
        .aft
        .as_ref()
        .map_or(capacity.aft_design.container_height_m, |a| {
            a.container_height_m
        });
    EnclosedVolume {
        hull_m3: hull_volume_m3(hull),
        stern_block_m3: result
            .aft_arrangement
            .as_ref()
            .map_or(0.0, |a| stern_block_above_deck_m3(a, unit_height_m)),
        deckhouse_m3: result
            .aft_arrangement
            .as_ref()
            .and_then(|a| a.deckhouse.as_ref())
            .map_or(0.0, |d| d.volume_m3),
    }
}

/// The gross tonnage of a design: the computed value from V, and the typed value that overrides it
/// (spec 0011 §4.2: `gross_tonnage`, none = computed).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrossTonnage {
    pub volume: EnclosedVolume,
    /// K1 of V.
    pub k1: f64,
    /// K1 V.
    pub computed: f64,
    /// The designer's typed GT, when given.
    pub typed: Option<f64>,
}

impl GrossTonnage {
    /// Work out GT from V; a typed value overrides it.
    pub fn of(volume: EnclosedVolume, typed: Option<f64>) -> Self {
        let v = volume.total_m3();
        Self {
            volume,
            k1: k1(v),
            computed: gross_tonnage(v),
            typed,
        }
    }

    /// The GT the design uses: the typed value, else the computed one.
    pub fn value(&self) -> f64 {
        self.typed.unwrap_or(self.computed)
    }

    /// `typed` or `computed`.
    pub fn source(&self) -> &'static str {
        if self.typed.is_some() {
            "typed"
        } else {
            "computed"
        }
    }
}
