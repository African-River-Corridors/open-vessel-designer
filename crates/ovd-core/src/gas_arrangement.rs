//! Best gas-tank arrangement for a hull: search, do not trust the single AUTO answer.
//! Port of `core/gas_arrangement.py`. The reasoning behind the search lives in the Python file.
//!
//! Only IGC rules apply to the gas tanks (issue #175): `IGC_2G`, the per-tank rule of IGC 2.4.1.1
//! with the datum of 2.4.2 (issue #237). Two tanks lie
//! along the hull. Every tank has to keep to the outer diameter cap (`FitOptions::max_tank_od_m`)
//! and to length over inner diameter at most [`MAX_TANK_LENGTH_OVER_OD`] (the diameter of a tank
//! shell is its internal diameter, ADN 2025 1.2.1; issue #416). A candidate over that is not
//! feasible: it gets the failed check `tank_length_over_diameter`, so it can never win while a
//! compliant one exists, and a hull with none is reported infeasible, not given a payload.
//!
//! Behaviour is copied exactly, including the errors and their message text. Python's `x or 0.0`
//! on a float is `truthy_or_zero`, `x or y` on an optional float is `or_else`, a strict `>` with
//! the `1e-6` margin picks the winner (so the first of two near-equal candidates stays), and the
//! loops run in Python's order so the tie-breaks agree. Python's keyword defaults become
//! [`FitOptions`].
//!
//! Python's `ArrangementFit` also carries the engine's own input and result; so does
//! [`ArrangementFit`], because the Python bridge returns them and `gas_stability` reads the result.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::aft_arrangement::AftDesign;
use crate::barge_capacity::{
    BargeCapacityInputs, BargeCapacityResult, DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
    DEFAULT_MAX_TANK_OD_M, DEFAULT_PUSHED_STERN_VOID_M, DEFAULT_TANK_DOME_DIAMETER_M,
    DEFAULT_TANK_DOME_HEIGHT_M, DEFAULT_TANK_SUPPORT_HEIGHT_M, DEFAULT_TANK_WALL_THICKNESS_MM,
    DEFAULT_TANKS_IN_LINE, DEFAULT_WEATHER_SHIELD_CLEARANCE_M, DEFAULT_WEATHER_SHIELD_THICKNESS_MM,
    DEFAULT_WING_TANK_GAP_M, DEFAULT_WING_TANK_HEIGHT_RATIO, TankDesign, compute as capacity,
    dome_dimensions, dome_volume_m3,
};
use crate::checks::{Check, all_ok};
use crate::gas_damage::{self, DamageRunner, SerialRunner};
use crate::gas_stability::{self as gs, DEFAULT_WATER_DENSITY, GasCapacityResult};
use crate::journey_derating::{JourneyDeratingInputs, JourneyDeratingResult, compute as derate};
use crate::loading_conditions as lc;
use crate::measured::{Measured, deadweight_band};
use crate::models::DEFAULT_TANK_GAP_TRANSVERSE_M;
use crate::pressure_vessel::PyError;
use crate::protective_location::d_vc_exact;
use crate::pyfmt::{fixed, py_str, round_to};
use crate::rule_trace::{
    Failure, MeasureInfo, MeasureSource, MeasureValue, ObjectType, Outcome, Trace, evaluate,
    failures, measure_def,
};
use crate::rules::{ArgValue, MeasureRef, RuleSet};
use crate::tanks::INT_OVERFLOW_MESSAGE;
use crate::text;
use crate::text::{Arg, Text};

/// The rulesets `fit` accepts.
pub const RULESETS: [&str; 1] = ["IGC_2G"];
/// Largest overall tank length (heads included) over the inner shell diameter (issue #416), for
/// every pressure tank. A design limit of this tool. The figure has two sources, read in the
/// library: ADN 2025 9.3.1.11.1(b) ("Pressure tanks whose ratio of length to diameter exceeds 7
/// are prohibited") and the inland class rules Pt 4 Ch 4 3.5.1, now the class Rule
/// `lr-inland/3.5.1/tank-l-over-d` (issue #424). No IGC Code paragraph gives it.
pub const MAX_TANK_LENGTH_OVER_OD: f64 = 7.0;
/// `round(3.5 + 0.25 * i, 2)` for `i` in `0..15`: every value is exact in binary, so the
/// rounding changes nothing.
pub const DEFAULT_DEPTHS_M: [f64; 15] = [
    3.5, 3.75, 4.0, 4.25, 4.5, 4.75, 5.0, 5.25, 5.5, 5.75, 6.0, 6.25, 6.5, 6.75, 7.0,
];
pub const DEFAULT_TANKS_WIDE: [i64; 3] = [1, 2, 3];
pub const DEFAULT_EXTRA_CLEARANCE_M: [f64; 4] = [0.5, 1.0, 1.5, 2.0];
/// The default least freeboard (m), the one `BargeCapacityInputs` uses.
pub const DEFAULT_REQUIRED_FREEBOARD_M: f64 = 0.3;
/// Issue #935: the most damage runs one search works out (the maintainer's choice at the gate).
pub const DEFAULT_DAMAGE_RUN_CAP: usize = 24;
/// Issue #935: the numbers of tanks along the search chooses from.
pub const TANKS_ALONG_CHOICES: [i64; 3] = [2, 3, 4];
/// Issue #935: the diameter the search refines a failing size to, between it and the next size down (m).
const DAMAGE_REFINE_STEP_M: f64 = 0.1;
/// Issue #992: with the tanks in hold (a standing height of 0), the gap between the tank top and
/// the underside of the main deck (m). Built practice as the research for #993 gave it; an estimate.
pub const TANKS_IN_HOLD_DECK_ALLOWANCE_M: f64 = 0.3;
/// Issue #992: how many times the build is repeated to settle a depth that follows the tank.
const DEPTH_SETTLE_ROUNDS: usize = 8;

/// Issue #992: the moulded depth that puts the tank top `standing_m` above the main deck (m), to the
/// millimetre. `tank_top_above_keel_m` is the double bottom, the support and the tank outer diameter.
/// At 0 the tanks are in hold, and the deck stands [`TANKS_IN_HOLD_DECK_ALLOWANCE_M`] above the tank
/// top; above 0 the deck cuts the tank `standing_m` below its top.
pub fn depth_for_standing_height(tank_top_above_keel_m: f64, standing_m: f64) -> f64 {
    let gap = if standing_m <= 0.0 {
        TANKS_IN_HOLD_DECK_ALLOWANCE_M
    } else {
        0.0
    };
    libm::round((tank_top_above_keel_m - standing_m + gap) * 1000.0) / 1000.0
}

/// Issue #992: what stands above the main deck over the cargo tanks, for the windage and the sightline.
/// All estimates; there is no wind heel model yet, and nothing here is a freeboard or hull girder credit.
#[derive(Debug, Clone, PartialEq)]
pub struct AboveDeck {
    /// A weather shield is fitted (its mass is in the lightship).
    pub shield_fitted: bool,
    /// The highest of the shield top, the dome top and the tank top above the main deck (m); 0 when
    /// nothing stands above it. It is the obstruction for the sightline.
    pub top_above_deck_m: f64,
    /// The dome top above the deck (m); the domes pass through the shield and the deck.
    pub dome_top_above_deck_m: f64,
    /// The shield's own top above the deck (m); 0 with no shield.
    pub shield_top_above_deck_m: f64,
    /// The projected side area above the deck (m2): the cargo zone length times the height of what
    /// stands. An upper bound for bare cylinders.
    pub side_area_m2: f64,
    /// The projected front area above the deck (m2): the beam times the height for a shield; the
    /// tanks across times their diameter times the height for bare tanks.
    pub front_area_m2: f64,
    /// The mass of the shield (t), already in the lightship.
    pub shield_mass_t: f64,
}

/// What stands above the deck of a gas arrangement of moulded depth `depth` (issue #992).
pub fn above_deck(r: &BargeCapacityResult, depth: f64) -> Option<AboveDeck> {
    let g = r.gas_basis.as_ref()?;
    let inp = &g.inputs;
    let od = g.tanks.outer_diameter;
    let bottom = r.tank_rules.as_ref().map_or(0.0, |t| t.bottom_gap_m);
    let tank_top = bottom + od;
    let zone = crate::tanks::CargoZone::new(
        inp.loa,
        inp.bow_non_tank_length.unwrap_or(0.0),
        inp.stern_non_tank_length.unwrap_or(0.0),
    );
    let zone_len = (zone.x_fwd_m - zone.x_aft_m).max(0.0);
    let fitted = g.weights.weather_shield > 0.0;
    let shield_top = tank_top + inp.weather_shield_clearance - depth;
    let (shield_top_above, height, front) = if fitted && shield_top > 0.0 {
        let wing_top = bottom + inp.wing_tank_height_ratio * od;
        let foot = wing_top.max(depth);
        let h = (tank_top + inp.weather_shield_clearance - foot).max(0.0);
        (shield_top, h, inp.beam * h)
    } else {
        let h = (tank_top - depth).max(0.0);
        (0.0, h, g.tanks.n_tanks_wide as f64 * od * h)
    };
    let dome_top = (tank_top + inp.dome_height - depth).max(0.0);
    Some(AboveDeck {
        shield_fitted: fitted,
        top_above_deck_m: shield_top_above
            .max((tank_top - depth).max(0.0))
            .max(dome_top),
        dome_top_above_deck_m: dome_top,
        shield_top_above_deck_m: shield_top_above,
        side_area_m2: zone_len * height,
        front_area_m2: front,
        shield_mass_t: if g.weights.weather_shield > 0.0 {
            g.weights.weather_shield
        } else {
            0.0
        },
    })
}

/// The top of the tank shell above the keel (m): the double bottom, the support and the outer
/// diameter (`Detail::tank_top_above_keel_m` of the capacity run). Dome and shield not counted.
pub fn tank_top_above_keel_m(r: &BargeCapacityResult) -> f64 {
    r.tank_rules.as_ref().map_or(0.0, |t| t.bottom_gap_m) + opt_or_zero(r.tank_outer_diameter_m)
}

/// The keyword arguments of `fit`. `Default` is the Python defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct FitOptions {
    pub ruleset: String,
    pub fill_ratio: f64,
    /// Cargo density (kg/m^3) in place of the cargo data's (issue #315); `None`: the cargo data's.
    pub cargo_density_kg_m3: Option<f64>,
    /// Issue #348: `Some` puts the stern block's mass into the lightship of a self-propelled hull.
    pub stern_block_mass: Option<crate::aft_arrangement::SternMassInputs>,
    /// Issue #350: the Stern Form, when it is not today's square stern.
    pub stern_geometry: Option<crate::stern_form::SternGeometry>,
    /// Issue #352: the machinery of a specified Propulsor (`BargeCapacityInputs::machinery`).
    pub machinery: Option<crate::machinery::MachineryInputs>,
    /// Issue #354: the Blind Distance limit (`BargeCapacityInputs::sightline_limit_m`). Issue #989: read
    /// from the Rule Sets of the stack by [`blind_limits`], the limit the Rule Trace checks.
    pub sightline_limit_m: Option<f64>,
    /// Issue #989: the laden limit (`BargeCapacityInputs::sightline_limit_laden_m`).
    pub sightline_limit_laden_m: Option<f64>,
    /// Issue #989: the light limit a fixed deckhouse is sized to (`BargeCapacityInputs::sightline_sizing_limit_m`).
    pub sightline_sizing_limit_m: Option<f64>,
    /// Issue #989: the id of the Rule that gives the light limit.
    pub sightline_limit_source: Option<String>,
    pub self_propelled: bool,
    /// Issue #992: the height of the tank top above the main deck (m). `Some`: the depth of each
    /// candidate follows its tank ([`depth_for_standing_height`]) and `depths` only seeds the build
    /// (the tank depends on the depth only a little, through the bow rake). `Some(0.0)` is tanks in hold. `None`: depth is
    /// the one asked for, as before.
    pub tank_top_above_deck_m: Option<f64>,
    /// Issue #992: the draught the waterway or the user allows (m), without the hull's own limit.
    /// With a standing height the depth is never less than this plus the least freeboard plus
    /// `depth_margin_m`.
    pub draught_limit_m: Option<f64>,
    /// Issue #992: the structure margin of that floor (m). An estimate; none is sourced.
    pub depth_margin_m: f64,
    pub depths: Vec<f64>,
    pub tanks_wide: Vec<i64>,
    pub extra_clearance_m: Vec<f64>,
    /// Issue #675: the least side space of every candidate (m), on top of the IGC 2.4 minimum.
    /// 0: the IGC minimum alone. The wing tank is this less [`FitOptions::wing_tank_gap_m`].
    pub min_side_space_m: f64,
    /// Issue #448: Cb only for the IGC 4.28.2 vertical acceleration; the hydrostatics come from
    /// each candidate's sections.
    pub block_coefficient: f64,
    /// Cap on a tank's outer diameter (m).
    pub max_tank_od_m: f64,
    /// Bow rake as a multiple of the moulded depth of each candidate.
    pub bow_rake_to_depth_ratio: f64,
    /// Issue #661: the longest bow rake the ratio gives (m); `None`: no cap.
    pub bow_rake_max_length_m: Option<f64>,
    /// Issue #446: a typed bow rake length (m) for every candidate; `None`: the ratio.
    pub bow_rake_length_m: Option<f64>,
    /// Issue #446: the keel height at the bow (m); `None`: each candidate's moulded depth.
    pub bow_rake_rise_m: Option<f64>,
    /// Issue #449: bilge radius, sheer and camber of every candidate.
    pub hull_shape: crate::hull::hull_form::HullShape,
    /// Issue #282: the angle of down-flooding the designer gives (degrees); `None`: none.
    pub downflooding_angle_deg: Option<f64>,
    /// Issue #592: the deck openings the angle is worked from when none is typed.
    pub downflooding_openings: Vec<crate::hydrostatics::Opening>,
    /// Aft void of a pushed barge (m).
    pub pushed_stern_void_m: f64,
    /// Tanks along the hull, for every candidate.
    pub tanks_in_line: i64,
    /// Height of the tank saddles above the inner bottom (m).
    pub tank_support_height_m: f64,
    /// Dome height above the tank top (m).
    pub tank_dome_height_m: f64,
    /// Dome diameter (m): with the dome height it gives the dome's volume, which counts in Vc
    /// only (IGC 2.4.1.1). A nominal value; no published source yet.
    pub tank_dome_diameter_m: f64,
    /// Issue #316: `true` takes the dome diameter, or height, from the dome rule of each tank
    /// (`barge_capacity::dome_dimensions`) instead of the value above.
    pub tank_dome_diameter_by_rule: bool,
    pub tank_dome_height_by_rule: bool,
    /// Gap between the wing tank and the cargo tank, per side (m).
    pub wing_tank_gap_m: f64,
    /// Wing tank height as a fraction of the cargo tank outer diameter.
    pub wing_tank_height_ratio: f64,
    /// Plate thickness of the weather shield over the tanks (mm); 0: no shield (issue #410).
    pub weather_shield_thickness_mm: f64,
    /// Clearance from the tank top to the weather shield top (m, issue #410).
    pub weather_shield_clearance_m: f64,
    /// The `structure` block of the Hull (spec 0013, issue #741), for the panel model of each
    /// arrangement tried.
    pub structure: crate::vessel_model::StructureBlock,
    /// Forming allowance on the tank plate, % of the required thickness (issue #412).
    pub tank_forming_allowance_pct: f64,
    /// Wall thickness of each tank's dome (mm); 0: no dome mass (issue #412).
    pub tank_dome_wall_thickness_mm: f64,
    /// The end-bulkhead space aft and forward of the tanks (m): the type G rule for the tank end
    /// shape (`gas_separation::default_end_space`), or the Option's own values (issue #280).
    pub aft_end_space_m: f64,
    pub fwd_end_space_m: f64,
    /// Issue #381: where the opening setback was measured from. Since spec 0011 (D17) the hull keeps
    /// no length for the setback, so no result reads it; kept so saved files still build.
    pub opening_setback_from_cargo_area: bool,
    /// The IGC 3.1.2 cofferdam between the hold and the stern block (m), chosen in place of the
    /// gastight A-60 bulkhead; `0`: the bulkhead (spec 0011 D18).
    pub hold_cofferdam_m: f64,
    /// Fixed tank wall thickness (mm); `None`: the pressure-vessel formula gives it (issue #292).
    pub tank_wall_thickness_mm: Option<f64>,
    /// The Cargo-dependent tank design values: pressure, steel, insulation, structure factor.
    pub tank_design: TankDesign,
    /// Water density for the draught of every candidate and its journey de-rating (t/m3).
    pub water_density_t_per_m3: f64,
    /// Least freeboard at the carried draught (m): a candidate that carries its payload deeper
    /// than the hull depth less this fails `freeboard_at_payload`.
    pub required_freeboard_m: f64,
    /// Air draft limit of the light condition (m), as the capacity check reads it. `None`: none.
    pub air_draft_limit_m: Option<f64>,
    /// Issue #991: the bridge clearance of the route and where it comes from (`BargeCapacityInputs`):
    /// an air draft over it warns.
    pub bridge_clearance_m: Option<f64>,
    pub bridge_clearance_source: Option<String>,
    /// The stern block's design values for every candidate (a self-propelled hull only).
    pub aft_design: AftDesign,
    /// Issue #822: the cargo vent mast of every candidate (see the fields of the same names in
    /// [`BargeCapacityInputs`]).
    pub river_only: bool,
    pub vent_stub_m: f64,
    pub vent_mast_lowerable: bool,
    pub entrance_distance_m: f64,
    pub engine_room_vent_inlet_distance_m: f64,
    pub funnel_outlet_distance_m: f64,
    /// The selected Rule Sets, in selection order (spec 0004, "How `fit` uses it", issue #277). A
    /// candidate passes when its Rule Trace has no `fail` row. Empty: the search judges by its own
    /// check list (see "THE CHECKS THE SEARCH USES"), as the Python API does.
    pub rules: Vec<RuleSet>,
    /// The Option's inputs, for the input Measures and the Applicability of [`FitOptions::rules`].
    pub rule_inputs: RuleInputs,
    /// Issue #426: `Some` checks every candidate's stability in the IS Code 2008 loading
    /// conditions (`loading_conditions`) in place of the legacy laden, half laden and light ones,
    /// and ranks it by the cargo it carries with its stores and fuel aboard. `None`: legacy.
    pub is_code: Option<IsCodeConditions>,
    /// Issue #935: the most damage runs one search works out. On reaching it the search returns the best candidate
    /// that passed damage so far and says it stopped (not wall time, which would give different answers on
    /// different machines).
    pub damage_run_cap: usize,
    /// Issue #935: the search chooses the number of tanks along from 2, 3 and 4 (screened by damage) when the stack
    /// screens by IGC 2.7 damage and the run does not skip damage. `false`: the number is [`FitOptions::tanks_in_line`]
    /// (a typed value pins it). The first evaluation (damage not run) keeps `tanks_in_line`.
    pub choose_tanks_along: bool,
}

/// What the IS Code 2008 loading conditions of a candidate need beyond its own result (issue #426).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IsCodeConditions {
    /// The stores and fuel of the full load (t). 0 for a barge.
    pub consumables_t: f64,
    pub may_sail_without_ballast: bool,
    /// The user's ballast fills by tank group, departure then arrival; `None`: the least ballast.
    pub user_ballast: Option<[lc::GroupFills; 2]>,
}

/// The Option's inputs as the Rule Trace reads them: each enumerated input's text value (for
/// Applicability), and each input Measure with its registry entry and value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RuleInputs {
    pub applicability: Vec<(String, String)>,
    pub measures: Vec<(String, MeasureInfo, MeasureValue)>,
}

/// The Measures of one candidate of the search.
struct CandidateSource<'a> {
    measured: &'a Measured,
    inputs: &'a RuleInputs,
}

impl MeasureSource for CandidateSource<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        if let Some(d) = measure_def(id) {
            return Some(d.info());
        }
        self.inputs
            .measures
            .iter()
            .find(|(n, _, _)| n == id)
            .map(|(_, info, _)| info.clone())
    }
    fn objects(&self, object: &str) -> Vec<String> {
        self.measured.objects_by_key(object, "Candidate")
    }
    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        // The Option's own values: each input Measure, and the Measures its typed inputs give
        // (issue #417).
        let own = self
            .inputs
            .measures
            .iter()
            .find(|(n, _, _)| *n == m.id)
            .map(|(_, _, v)| v.clone());
        if m.id.starts_with("input.") {
            return own.ok_or_else(|| format!("the Option has no `{}`", m.id));
        }
        if let Some(v) = own {
            return Ok(v);
        }
        self.measured.value(m, index)
    }
    fn input(&self, name: &str) -> Option<String> {
        if name == "deadweight_band" {
            return self
                .measured
                .deadweight_t
                .map(|dw| deadweight_band(dw).to_string());
        }
        if name == "downflooding_angle" {
            return Some(
                crate::measured::downflooding_angle_value(
                    self.measured.downflooding_deg,
                    self.measured.downflooding_openings,
                )
                .to_string(),
            );
        }
        self.inputs
            .applicability
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    }
    fn object_input(&self, object: &str, index: usize, name: &str) -> Option<String> {
        self.measured
            .object_input(object, index, name)
            .or_else(|| self.input(name))
    }
    fn has(&self, m: &MeasureRef) -> bool {
        self.measured.has(m)
    }
}

/// Issue #989: the Blind Distance limits of a design, read from the Rule Sets of its stack. A Rule gives
/// a limit when it measures `wheelhouse.blind_distance`, blocks, and applies to the design (its
/// Applicability and its set's, as the Rule Trace reads them); the limit is the Rule's own, worked from
/// the design's inputs (SOLAS V/22.1.1 reads the length overall). The least limit of each condition
/// binds, and the id of its Rule says where it came from. A stack with no such Rule gives no limit, and
/// a Rule that only warns gives no blocking limit, but it sizes a fixed deckhouse (`sizing_light_m`).
///
/// This is the one reading the arrangement search sizes the wheelhouse to and the Rule Trace checks the
/// result against: the Trace evaluates the same Rules, so the limit it shows is the one used here. The
/// limit is worked by evaluating the Rules with a Blind Distance of 0, which passes, so only the limit
/// is read from the row.
pub fn blind_limits(rules: &[RuleSet], inputs: &RuleInputs) -> crate::wheelhouse::BlindLimits {
    use crate::rules::Consequence;
    let sets: Vec<RuleSet> = rules
        .iter()
        .filter_map(|set| {
            let mut one = set.clone();
            one.rules
                .retain(|r| r.measure.id == "wheelhouse.blind_distance");
            (!one.rules.is_empty()).then_some(one)
        })
        .collect();
    let mut out = crate::wheelhouse::BlindLimits::default();
    if sets.is_empty() {
        return out;
    }
    let measured = Measured {
        blind_distance_light_m: Some(0.0),
        blind_distance_laden_m: Some(0.0),
        ..Measured::default()
    };
    let refs: Vec<&RuleSet> = sets.iter().collect();
    let trace = evaluate(
        &refs,
        &CandidateSource {
            measured: &measured,
            inputs,
        },
    );
    for row in &trace.rows {
        let (Some(measure), Some(limit), Some(rule)) = (&row.measure, row.limit, &row.rule) else {
            continue;
        };
        if !matches!(row.outcome, Outcome::Pass | Outcome::Fail) {
            continue;
        }
        let blocks = row.consequence == Some(Consequence::Block);
        if !blocks && row.consequence != Some(Consequence::Warn) {
            continue;
        }
        let condition = measure
            .args
            .iter()
            .find_map(|(k, v)| match (k.as_str(), v) {
                ("condition", ArgValue::Text(t)) => Some(t.as_str()),
                _ => None,
            });
        if condition == Some("light") && out.sizing_light_m.is_none_or(|least| limit < least) {
            out.sizing_light_m = Some(limit);
        }
        if !blocks {
            continue;
        }
        let (slot, source) = match condition {
            Some("light") => (&mut out.light_m, &mut out.light_rule),
            Some("laden") => (&mut out.laden_m, &mut out.laden_rule),
            _ => continue,
        };
        if slot.is_none_or(|least| limit < least) {
            *slot = Some(limit);
            *source = Some(rule.clone());
        }
    }
    out
}

/// What the Rule Trace of a candidate reads: its checks (named as a result lists them), its
/// stability conditions (none before the costly stage) and its tank and separation values.
fn candidate_measured(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    depth: f64,
    checks: &[Check],
    conditions: Vec<gs::Condition>,
) -> Measured {
    let plane = inp.separation.as_ref().is_some_and(|s| s.plane_tank_ends);
    Measured {
        conditions,
        deadweight_t: Some(j.achievable_tonnage_t),
        // The same Freeboard `freeboard_at_payload` checks (issue #659).
        freeboard_laden_m: Some(depth - j.draft_at_achievable_m),
        downflooding_deg: inp.downflooding_angle_deg,
        downflooding_openings: !inp.downflooding_openings.is_empty(),
        tank: r.tank_rules.clone(),
        vent_mast: r.vent_mast.clone(),
        separation: r.separation.clone().map(|s| (s, plane)),
        checks: checks
            .iter()
            .map(|c| (c.name.replace('_', " "), c.ok))
            .collect(),
        // The wheelhouse and the powering are worked out for the answer, not for a candidate.
        ..Measured::default()
    }
}

/// The whole Rule Trace of a candidate. Built once, for the answer.
fn candidate_trace(measured: &Measured, o: &FitOptions) -> Trace {
    let sets: Vec<&RuleSet> = o.rules.iter().collect();
    evaluate(
        &sets,
        &CandidateSource {
            measured,
            inputs: &o.rule_inputs,
        },
    )
}

/// The failing rows of a candidate's Rule Trace, without building the trace: the search calls this
/// on every candidate.
fn candidate_failures(measured: &Measured, o: &FitOptions) -> Vec<Failure> {
    let sets: Vec<&RuleSet> = o.rules.iter().collect();
    failures(
        &sets,
        &CandidateSource {
            measured,
            inputs: &o.rule_inputs,
        },
    )
}

/// The failing rows of a trace, grouped as the check list groups them: the rows of one loading
/// condition count once (one stability check per condition), the Rules on one engine check count
/// once (a class Rule and a tool limit can both report it, #424), each other failing Rule once.
fn failing_groups(fails: &[Failure]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for f in fails {
        let check = match (f.measure.id.as_str(), f.measure.args.as_slice()) {
            ("check.ok", [(_, ArgValue::Text(name))]) => Some(name),
            _ => None,
        };
        let key = match (
            f.object_type.as_deref().and_then(ObjectType::from_key),
            check,
        ) {
            (Some(ObjectType::LoadingCondition), _) => {
                format!(
                    "Stability \u{2014} {}",
                    f.object.clone().unwrap_or_default()
                )
            }
            (_, Some(name)) => format!("check {name}"),
            _ => f.rule.clone(),
        };
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

impl Default for FitOptions {
    fn default() -> Self {
        Self {
            ruleset: "IGC_2G".to_string(),
            fill_ratio: 0.95,
            cargo_density_kg_m3: None,
            stern_block_mass: None,
            stern_geometry: None,
            machinery: None,
            sightline_limit_m: None,
            sightline_limit_laden_m: None,
            sightline_sizing_limit_m: None,
            sightline_limit_source: None,
            self_propelled: true,
            tank_top_above_deck_m: None,
            draught_limit_m: None,
            depth_margin_m: 0.0,
            depths: DEFAULT_DEPTHS_M.to_vec(),
            tanks_wide: DEFAULT_TANKS_WIDE.to_vec(),
            extra_clearance_m: DEFAULT_EXTRA_CLEARANCE_M.to_vec(),
            min_side_space_m: 0.0,
            block_coefficient: 0.85,
            max_tank_od_m: DEFAULT_MAX_TANK_OD_M,
            bow_rake_to_depth_ratio: DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
            bow_rake_max_length_m: None,
            bow_rake_length_m: None,
            bow_rake_rise_m: None,
            hull_shape: crate::hull::hull_form::HullShape::default(),
            downflooding_angle_deg: None,
            downflooding_openings: Vec::new(),
            pushed_stern_void_m: DEFAULT_PUSHED_STERN_VOID_M,
            tanks_in_line: DEFAULT_TANKS_IN_LINE,
            tank_support_height_m: DEFAULT_TANK_SUPPORT_HEIGHT_M,
            tank_dome_height_m: DEFAULT_TANK_DOME_HEIGHT_M,
            tank_dome_diameter_m: DEFAULT_TANK_DOME_DIAMETER_M,
            tank_dome_diameter_by_rule: false,
            tank_dome_height_by_rule: false,
            wing_tank_gap_m: DEFAULT_WING_TANK_GAP_M,
            wing_tank_height_ratio: DEFAULT_WING_TANK_HEIGHT_RATIO,
            weather_shield_thickness_mm: DEFAULT_WEATHER_SHIELD_THICKNESS_MM,
            weather_shield_clearance_m: DEFAULT_WEATHER_SHIELD_CLEARANCE_M,
            structure: crate::vessel_model::StructureBlock::default(),
            // As the legacy capacity inputs: neither is weighed unless a Design asks (issue #412).
            tank_forming_allowance_pct: 0.0,
            tank_dome_wall_thickness_mm: 0.0,
            aft_end_space_m: crate::gas_separation::default_end_space(false),
            fwd_end_space_m: crate::gas_separation::default_end_space(false),
            opening_setback_from_cargo_area: true,
            hold_cofferdam_m: 0.0,
            tank_wall_thickness_mm: Some(DEFAULT_TANK_WALL_THICKNESS_MM),
            tank_design: TankDesign::default(),
            water_density_t_per_m3: DEFAULT_WATER_DENSITY,
            required_freeboard_m: DEFAULT_REQUIRED_FREEBOARD_M,
            air_draft_limit_m: None,
            bridge_clearance_m: None,
            bridge_clearance_source: None,
            aft_design: AftDesign::default(),
            river_only: false,
            vent_stub_m: crate::vent_mast::DEFAULT_VENT_STUB_M,
            vent_mast_lowerable: false,
            entrance_distance_m: 5.0,
            engine_room_vent_inlet_distance_m: 5.0,
            funnel_outlet_distance_m: 2.0,
            rules: Vec::new(),
            rule_inputs: RuleInputs::default(),
            is_code: None,
            damage_run_cap: DEFAULT_DAMAGE_RUN_CAP,
            choose_tanks_along: false,
        }
    }
}

/// The winning arrangement and its payload.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrangementFit {
    pub loa_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
    pub cargo: String,
    pub ruleset: String,
    pub fill_ratio: f64,
    pub self_propelled: bool,
    pub t_max_m: f64,
    /// Per journey, after the draught cap.
    pub payload_t: f64,
    pub volume_capacity_t: f64,
    /// `"volume"`, `"draft"` or `"no cargo area"`.
    pub binding: String,
    pub draft_at_payload_m: f64,
    pub cargo_zone_m: f64,
    pub stern_void_m: f64,
    pub bow_void_m: f64,
    pub tank_od_m: f64,
    pub n_tanks: i64,
    pub n_tanks_wide: i64,
    pub n_tanks_long: i64,
    pub lightship_t: f64,
    pub air_draft_light_m: f64,
    pub side_clearance_m: f64,
    pub tank_inner_diameter_m: f64,
    pub containment_inner_volume_m3: f64,
    pub per_tank_volume_m3: f64,
    pub cargo_volume_m3: f64,
    pub density_t_m3: f64,
    pub failed_checks: Vec<String>,
    /// False: no candidate passed its error checks; see `failed_checks`.
    pub feasible: bool,
    pub candidates_tried: i64,
    /// Issue #675: every arrangement the search tried, in the order it built them, and what became
    /// of each.
    pub tried: Vec<TriedArrangement>,
    pub engine_inputs: BargeCapacityInputs,
    pub engine_result: BargeCapacityResult,
    /// Issue #935: how many damage runs the search worked out (0: it did not screen by damage).
    pub damage_runs: usize,
    /// Issue #935: whether the pick was screened by damage, and what the search without damage would have picked.
    pub damage_screen: DamageScreen,
    /// Issue #935: the most damage runs the search expects (0: nothing to screen by), for the estimate before a
    /// damage run.
    pub damage_planned_runs: usize,
    /// Issue #935: the numbers of tanks along the search tried (one: pinned, or damage not screened).
    pub tanks_along_tried: Vec<i64>,
}

/// Issue #935: how the arrangement search treated IGC 2.7 damage survival.
#[derive(Debug, Clone, PartialEq)]
pub enum DamageScreen {
    /// The stack does not evaluate IGC 2.7 on this design: there is nothing to screen by.
    NotApplicable,
    /// The stack evaluates IGC 2.7 but the run skips damage: the pick is the unscreened one, and may change when
    /// damage runs.
    Skipped,
    /// The candidates were screened. `unscreened` is the pick the search without damage makes, when it is not the
    /// pick the screen made.
    Screened {
        unscreened: Option<UnscreenedPick>,
        /// The search reached its damage-run limit and stopped: the pick is the best that passed damage until then
        /// (or, with none, the closest).
        cap_hit: bool,
    },
}

/// The arrangement the search makes without the damage screen, for the note that says damage changed the pick.
#[derive(Debug, Clone, PartialEq)]
pub struct UnscreenedPick {
    pub depth_m: f64,
    pub n_wide: i64,
    pub n_long: i64,
    pub tank_od_m: f64,
    pub cargo_volume_m3: f64,
}

/// One arrangement the search tried (issue #675): one depth and number of tanks across. It is the
/// best of the side spaces tried for it: the chosen one, else the one with the most cargo that
/// passes the cheap checks, else the one closest to passing: the fewest failed checks, then the
/// smallest tank. More side space only makes the tank smaller, so the smallest tank comes closest
/// on the height checks (blind distance, air draft) that most often turn an arrangement down.
#[derive(Debug, Clone, PartialEq)]
pub struct TriedArrangement {
    pub depth_m: f64,
    pub n_wide: i64,
    pub n_long: i64,
    /// 0 when no tank was built.
    pub tank_od_m: f64,
    pub side_clearance_m: f64,
    /// The side space less the wing tank gap, and not less than zero.
    pub wing_tank_width_m: f64,
    pub cargo_volume_m3: f64,
    /// What the search ranks by (t): the payload, or with the IS Code conditions the cargo of
    /// their full loads.
    pub cargo_t: f64,
    /// How many side spaces the search built for it.
    pub variants: i64,
    pub outcome: TriedOutcome,
}

/// What became of a [`TriedArrangement`].
#[derive(Debug, Clone, PartialEq)]
pub enum TriedOutcome {
    /// The answer: it passes every check.
    Chosen,
    /// It passes the cheap checks but carries less than the answer, so the search did not assess
    /// its stability (or it passed and lost a tie).
    LessCargo,
    /// It fails these error checks (the answer, when no arrangement passes, is one of these).
    Fails(Vec<Check>),
    /// No candidate could be built; the reason.
    NotBuilt(Text),
}

impl FitOptions {
    /// The defaults of `smallest_viable_loa`: as `fit`, but `depths` is the first default depth.
    pub fn for_smallest_viable_loa() -> Self {
        Self {
            depths: vec![DEFAULT_DEPTHS_M[0]],
            ..Self::default()
        }
    }
}

fn value_error(message: String) -> PyError {
    PyError {
        kind: "ValueError",
        message,
    }
}

/// Python `x or 0.0` on a float: zero is false, so it gives `0.0`; NaN is true.
fn truthy_or_zero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}

/// Python `x or 0.0` on an optional float: `None` and zero give `0.0`.
fn opt_or_zero(x: Option<f64>) -> f64 {
    truthy_or_zero(x.unwrap_or(0.0))
}

/// Python `int(x or 0)`.
fn opt_count(x: Option<i64>) -> i64 {
    x.unwrap_or(0)
}

/// Python `max(r.n_tanks_total or 1, 1)` as the float the divisions turn it into.
fn tank_divisor(n_tanks_total: Option<i64>) -> f64 {
    let n = match n_tanks_total {
        None | Some(0) => 1,
        Some(n) => n,
    };
    n.max(1) as f64
}

/// Python `a // b` on ints with `b != 0`. The one result no 64-bit field holds
/// (`i64::MIN // -1`) is the fixed `OverflowError`.
fn floor_div(a: i64, b: i64) -> Result<i64, PyError> {
    let q = a.checked_div(b).ok_or_else(|| PyError {
        kind: "OverflowError",
        message: INT_OVERFLOW_MESSAGE.to_string(),
    })?;
    Ok(if a % b != 0 && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    })
}

/// The code minimum for the engine's clearance datum (outer face of the insulation): the d(Vc)
/// distance of IGC Code 2.4.1.1 (see `protective_location::d_vc_exact`) per individual tank, less
/// the insulation, because IGC 2.4.2 measures d plate to plate, from the moulded line to the
/// moulded line, excluding insulation.
///
/// `per_tank_m3` is Vc of one tank, dome included. `total_m3` is not used (the stepped reading,
/// which took the total volume, is removed); it stays so the signature is unchanged.
///
/// `Err` is the `ValueError` for a ruleset that is not a vessel type.
pub fn required_side_clearance(
    ruleset: &str,
    _beam: f64,
    _total_m3: f64,
    per_tank_m3: f64,
    insulation_mm: f64,
) -> Result<f64, PyError> {
    // An unknown name is the `ValueError` of the vessel types; of the known ones only the gas
    // rule set applies here.
    crate::protective_location::VesselType::from_value(ruleset).map_err(value_error)?;
    if !RULESETS.contains(&ruleset) {
        return Err(value_error(format!(
            "ruleset must be one of ('IGC_2G',), not {}",
            crate::pystr::repr_str(ruleset)
        )));
    }
    // IGC Code 2.4.1.1, per tank; 2.4.2, measured to the tank steel: the insulation sits inside d.
    Ok(d_vc_exact(per_tank_m3) - insulation_mm / 1000.0)
}

/// The length over diameter check of a candidate's tank: the overall tank length, heads included,
/// over the inner diameter of the steel shell. ADN 2025 9.3.1.11.1 (b) and the inland class rules
/// Pt 4 Ch 4 3.5.1 set the ratio at 7; ADN 1.2.1 defines the diameter of a tank shell as its
/// internal diameter (maintainer decision on issue #416; it was the insulated outer diameter).
/// `None` when there is no tank (a bulk result, or a hull with no cargo area), which the other
/// checks already report.
pub fn length_over_diameter_check(r: &BargeCapacityResult) -> Option<Check> {
    let tanks = &r.gas_basis.as_ref()?.tanks;
    if tanks.outer_diameter <= 0.0 || tanks.inner_diameter <= 0.0 {
        return None;
    }
    let ratio = tanks.overall_tank_length / tanks.inner_diameter;
    Some(Check::new(
        "tank_length_over_diameter",
        ratio <= MAX_TANK_LENGTH_OVER_OD,
        text!(
            "check-tank-length-over-inner-diameter",
            "tank length {length} m / inner diameter {diameter} m = {ratio} <= {max}",
            length = Arg::fixed(tanks.overall_tank_length, 2),
            diameter = Arg::fixed(tanks.inner_diameter, 2),
            ratio = Arg::fixed(ratio, 2),
            max = Arg::py(MAX_TANK_LENGTH_OVER_OD),
        ),
    ))
}

// ---------------------------------------------------------------------------------------------
// THE CHECKS THE SEARCH USES (issue #264). This is the one list. `fit` and `smallest_viable_loa`
// keep a candidate only if every error-severity check below passes, then rank the passing ones by
// cargo. When the rules engine (#176) lands, it replaces this list with the Design's selected rules.
//
// Cheap checks (run first, from numbers the search has already computed):
//   - the capacity result's own checks: feasible geometry, cargo zone, the aft arrangement
//     (thrusters, containers, wheelhouse), the gas separation (the cofferdam, the opening
//     setback and the hold end space of the inland class rules; the IGC protective location
//     clearances are tank Rules, not checks of this list),
//     and the air draft limit of the light condition when one is set. Not `not_overloaded` or
//     `freeboard_ok` (`FULL_VOLUME_CHECKS`): they test the full tank volume, and a candidate is
//     de-rated to its payload, so they do not gate (maintainer decision on issue #418);
//   - the journey checks: the channel draught limit leaves a payload that floats;
//   - `tank_length_over_diameter`: overall tank length over inner shell diameter, at most 7;
//   - `freeboard_at_payload`: the hull depth less the carried draught, at least the required freeboard.
// Costly check (run only for a candidate that passed the cheap ones and could still win):
//   - stability: IS Code 2008 Part A 2.2, each loading condition: the legacy ones
//     (`gas_stability::assess`), or with `FitOptions::is_code` the Part B 3.4.1.2 ones
//     (`is_code_conditions`), with their ballast plan and air draft (issue #426).
// A check of warning severity is advisory: it never gates (`all_ok`), as everywhere else.
// ---------------------------------------------------------------------------------------------

/// The capacity checks that test the full tank volume, not the carried payload. They are
/// information: the search does not gate on them (issue #418). The carried draught is gated by
/// `freeboard_at_payload` and the journey's `floats` check.
pub const FULL_VOLUME_CHECKS: &[&str] = &["not_overloaded", "freeboard_ok"];

/// The cheap checks of a candidate: see the list above.
fn cheap_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    depth: f64,
    o: &FitOptions,
) -> Vec<Check> {
    let mut all: Vec<Check> = r
        .checks
        .iter()
        .filter(|c| !FULL_VOLUME_CHECKS.contains(&c.name.as_str()))
        .cloned()
        .collect();
    all.extend(j.checks.iter().cloned());
    all.extend(length_over_diameter_check(r));
    let freeboard = depth - j.draft_at_achievable_m;
    all.push(Check::new(
        "freeboard_at_payload",
        freeboard + 1e-9 >= o.required_freeboard_m,
        text!(
            "check-freeboard-at-payload",
            "freeboard {freeboard} m at the carried draught {draught} m >= required {required} m",
            freeboard = Arg::fixed(freeboard, 2),
            draught = Arg::fixed(j.draft_at_achievable_m, 2),
            required = Arg::py(o.required_freeboard_m),
        ),
    ));
    all
}

/// The criterion name of IS Code 2008 Part A 2.2.3, the angle of maximum GZ.
const ANGLE_CRITERION: &str = "Angle of max GZ";

/// Whether the condition is one of the two with no ballast that OVD adds or keeps: "Lightship, no
/// ballast" of the IS Code conditions and the legacy "Light" (issue #498).
fn is_light_without_ballast(name: &str) -> bool {
    name == "Lightship, no ballast" || name == "Light"
}

/// One stability condition as a check, named and worded as the simulation lists it.
pub fn stability_check(c: &gs::Condition) -> Check {
    stability_check_with(c, false)
}

/// [`stability_check`] for a gas Design.
///
/// Issue #498: in a light condition with no ballast, a failure where the angle of maximum GZ
/// (2.2.3) is the only failing criterion is a warning, not a failure: the bilge emerges early on a
/// wide, shallow hull, and the vessel can sail light with ballast. IS Code 2.2.3 allows alternative
/// criteria only with the approval of the Administration, which OVD does not model, so the
/// alternative is not taken as a pass. A dry-bulk Design keeps its checks as they were (all its
/// stability checks are warnings already).
pub fn gas_stability_check(c: &gs::Condition, warns: bool) -> Check {
    stability_check_with(c, warns)
}

/// The Rule that fails the angle of max GZ of every condition in the IS Code Rule Sets of 1.0.0.
const ANGLE_RULE: &str = "is2008/A.2.2.3/angle-max-gz";

/// The Rule that sizes the double-hull space of a gas hull: ADN 9.3.1.11.2 (a), at least 0.80 m (issue #850).
pub const ADN_DOUBLE_SIDE_RULE: &str = "adn/9.3.1.11.2(a)/double-hull-width";
/// The Rule that sizes the double bottom of a gas hull: ADN 9.3.1.11.2 (a), at least 0.60 m (issue #850).
pub const ADN_DOUBLE_BOTTOM_RULE: &str = "adn/9.3.1.11.2(a)/double-bottom-height";

/// The constant lower limit of an evaluated Rule that sizes the hull, read from the stack, m; 0 when
/// no selected set holds the Rule. The arrangement takes the limit from the Rule, so the stack's
/// version decides the size (issue #850).
pub fn sizing_limit(rules: &[RuleSet], id: &str) -> f64 {
    rules
        .iter()
        .filter_map(|s| s.rule(id))
        .find_map(|r| match (&r.limit, r.compare) {
            (
                Some(crate::rules::Limit::Constant { value, .. }),
                Some(crate::rules::Compare::Ge),
            ) if r.evaluated && r.sizes => Some(*value),
            _ => None,
        })
        .unwrap_or(0.0)
}

/// Whether the angle of max GZ alone in a light condition with no ballast is a warning for these
/// Rule Sets (issue #498). It is when the Rule Sets say so: the IS Code Rule that holds the angle
/// names the kind of loading condition it applies to. A stack whose Rule applies to every
/// condition (`is2008-a` 1.0.0) keeps the failure, so a file saved before keeps its results. No
/// Rule Sets (the Python API) means the warning.
pub fn light_angle_warns(rules: &[RuleSet]) -> bool {
    let mut found = rules
        .iter()
        .flat_map(|set| set.rules.iter())
        .filter(|r| r.id == ANGLE_RULE)
        .peekable();
    if found.peek().is_none() {
        return true;
    }
    found.all(|r| {
        r.applies_when
            .conditions
            .iter()
            .any(|(input, _)| input == "loading_condition_kind")
    })
}

fn stability_check_with(c: &gs::Condition, gas: bool) -> Check {
    let bad: Vec<&gs::Criterion> = c.criteria.iter().filter(|x| !x.ok).collect();
    let gm = Arg::fixed(c.gm_fluid_m, 2);
    let max_gz = Arg::fixed(c.max_gz_m, 2);
    let angle = Arg::fixed(f64::from(c.max_gz_deg), 0);
    if gas && is_light_without_ballast(&c.name) && bad.len() == 1 && bad[0].name == ANGLE_CRITERION
    {
        let value = match bad[0].value {
            gs::PyNum::Int(i) => i as f64,
            gs::PyNum::Float(v) => v,
        };
        return Check::with_severity(
            format!("Stability \u{2014} {}", c.name.to_lowercase()),
            false,
            text!(
                "check-stability-light-angle",
                "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0} \u{2014} warning, not a failure: \
                 angle of max GZ {value} < {limit} (IS Code 2.2.3) with no ballast; every other criterion holds. \
                 Sail light with ballast",
                gm = gm,
                max_gz = max_gz,
                angle = angle,
                value = Arg::fixed(value, 0),
                limit = Arg::fixed(bad[0].limit, 0),
            ),
            "warning",
        );
    }
    let message = if bad.is_empty() {
        text!(
            "check-stability",
            "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0}",
            gm = gm,
            max_gz = max_gz,
            angle = angle,
        )
    } else {
        let parts: Vec<String> = bad
            .iter()
            .map(|x| {
                let v = match x.value {
                    gs::PyNum::Int(i) => i as f64,
                    gs::PyNum::Float(v) => v,
                };
                format!("{} {} < {}", x.name, fixed(v, 3), py_str(x.limit))
            })
            .collect();
        // The failed criteria are one English list (criterion names, values and limits).
        text!(
            "check-stability-fails",
            "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0} \u{2014} fails: {fails}",
            gm = gm,
            max_gz = max_gz,
            angle = angle,
            fails = parts.join(", "),
        )
    };
    Check::new(
        format!("Stability \u{2014} {}", c.name.to_lowercase()),
        bad.is_empty(),
        message,
    )
}

/// The checks of one IS Code loading condition (issue #426): its stability, and when they fail,
/// its ballast plan (none meets every criterion) and its air draft.
/// The checks of every IS Code condition of a Design, in order. The "Lightship, no ballast" warning of
/// issue #498 also names the ballast the "Ballast, departure" condition needs to meet every criterion.
pub fn loading_condition_checks_of(conds: &[lc::LoadedCondition], warns: bool) -> Vec<Check> {
    let ballast = conds
        .iter()
        .find(|c| c.role == lc::Role::BallastDeparture && c.ok() && c.ballast_t > 0.0)
        .map(|c| c.ballast_t);
    let mut out = Vec::new();
    for c in conds {
        let mut checks = loading_condition_checks_with(c, warns);
        if let (Some(t), Some(first)) = (ballast, checks.first_mut())
            && c.role == lc::Role::LightshipNoBallast
            && first.severity == "warning"
        {
            let (gm, max_gz, angle) = (
                Arg::fixed(c.condition.gm_fluid_m, 2),
                Arg::fixed(c.condition.max_gz_m, 2),
                Arg::fixed(f64::from(c.condition.max_gz_deg), 0),
            );
            let name = first.name.clone();
            *first = Check::with_severity(
                name,
                false,
                text!(
                    "check-stability-light-angle-ballast",
                    "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0} \u{2014} warning, not a failure: \
                 angle of max GZ {angle} < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. \
                 To sail light, take {ballast} t of ballast (the Ballast, departure condition meets every criterion)",
                    gm = gm,
                    max_gz = max_gz,
                    angle = angle,
                    ballast = Arg::fixed(t, 0),
                ),
                "warning",
            );
        }
        out.extend(checks);
    }
    out
}

pub fn loading_condition_checks(c: &lc::LoadedCondition) -> Vec<Check> {
    loading_condition_checks_with(c, false)
}

fn loading_condition_checks_with(c: &lc::LoadedCondition, gas: bool) -> Vec<Check> {
    let mut out = vec![stability_check_with(&c.condition, gas)];
    if c.plan_found == Some(false) {
        out.push(Check::new(
            "Ballast plan",
            false,
            text!(
                "check-no-ballast-plan",
                "{condition}: no ballast plan meets every criterion, even with every ballast tank full.",
                condition = c.condition.name.clone(),
            ),
        ));
    }
    if c.air_draft_ok == Some(false) {
        out.push(Check::new(
            "Air draft",
            false,
            text!(
                "check-condition-air-draft",
                "{condition}: air draft {air} m is above the limit.",
                condition = c.condition.name.clone(),
                air = Arg::fixed(c.air_draft_m, 2),
            ),
        ));
    }
    out
}

/// The cargo of the IS Code full-load conditions (issue #426): the payload, but no more than the
/// displacement at the draught cap less the lightship and the stores and fuel. A draught-bound
/// cargo is that much less; a volume-bound one does not change.
pub fn is_code_cargo(
    r: &BargeCapacityResult,
    payload_t: f64,
    loa: f64,
    beam: f64,
    t_max: f64,
    o: &FitOptions,
    ic: &IsCodeConditions,
) -> f64 {
    // Issue #448: the hull's displacement at the draught cap, from its sections.
    let _ = (loa, beam);
    let displaced = r
        .hull_spec
        .as_ref()
        .and_then(|spec| crate::hydrostatics::Hull::build(spec).ok())
        .map(|h| h.volume_at(t_max))
        .unwrap_or(0.0);
    let room = displaced * o.water_density_t_per_m3 - r.lightship_t - ic.consumables_t;
    let room = if room > 0.0 { room } else { 0.0 };
    if room < payload_t { room } else { payload_t }
}

/// The IS Code loading conditions of a gas arrangement with `cargo_t` in the full loads (issue
/// #426): its lightship and cargo model, its ballast tanks and the stores and fuel of `ic`.
/// Empty for a result with no gas basis.
pub fn is_code_conditions(
    r: &BargeCapacityResult,
    inp: &BargeCapacityInputs,
    cargo_t: f64,
    o: &FitOptions,
    ic: &IsCodeConditions,
) -> Result<Vec<lc::LoadedCondition>, gs::PyError> {
    let Ok(basis) = GasCapacityResult::try_from(r) else {
        return Ok(Vec::new());
    };
    let Some((lightship, cargo)) = lc::gas_model(&basis) else {
        return Ok(Vec::new());
    };
    let cargo_at = |m: f64| cargo.at(m);
    let Some(spec) = r.hull_spec.as_ref() else {
        return Ok(Vec::new());
    };
    let hull = crate::hydrostatics::Hull::build(spec)
        .map_err(|e| gs::PyError::Value(alloc::format!("{e}")))?;
    // Issue #455: the ballast tanks as the hull clips them.
    let tanks: Vec<_> = crate::compartments::ballast_tanks(inp, r)
        .iter()
        .map(|t| crate::compartments::clip(t, &hull.table))
        .collect();
    let ballast_plans = match &ic.user_ballast {
        Some([departure, arrival]) => [departure.plan(&tanks), arrival.plan(&tanks)],
        None => [lc::BallastPlan::Auto, lc::BallastPlan::Auto],
    };
    // The consumables tanks sit on the bottom at the aft end, in the stern block.
    let consumables = lc::Consumables {
        full_t: ic.consumables_t,
        tanks: crate::compartments::consumables_tanks(inp.beam, 0.0, ic.consumables_t, &hull.table),
        density: 1.0,
    };
    lc::assess(&lc::Inputs {
        loa: inp.loa,
        beam: inp.beam,
        depth: inp.depth,
        hull: &hull,
        water_density: o.water_density_t_per_m3,
        criteria_set: gs::criteria_set_for(&o.ruleset),
        lightship: &lightship,
        full_cargo_t: cargo_t,
        cargo_at: &cargo_at,
        consumables,
        ballast_tanks: &tanks,
        ballast_density: o.water_density_t_per_m3,
        ballast_plans,
        may_sail_without_ballast: ic.may_sail_without_ballast,
        air_draft_top_m: r.air_draft_top_above_keel_m,
        air_draft_limit_m: o.air_draft_limit_m,
    })
}

/// What the search ranks a candidate by: its payload, or with the IS Code conditions the cargo of
/// their full loads.
fn ranked_cargo(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    o: &FitOptions,
) -> f64 {
    match &o.is_code {
        Some(ic) => is_code_cargo(
            r,
            j.achievable_tonnage_t,
            inp.loa,
            inp.beam,
            j.t_max_m,
            o,
            ic,
        ),
        None => j.achievable_tonnage_t,
    }
}

/// The stability checks of a candidate, one per loading condition. None when it carries nothing or
/// has no gas tanks (the cheap checks report that). An assessment that cannot be made is one failed
/// check, so that a candidate the engine cannot judge never passes and never stops the search.
fn stability_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    o: &FitOptions,
) -> Vec<Check> {
    stability_assessment(r, j, inp, o).0
}

/// The stability checks of a candidate and the loading conditions they come from (none when the
/// candidate carries nothing or the assessment failed).
fn stability_assessment(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    o: &FitOptions,
) -> (Vec<Check>, Vec<gs::Condition>) {
    if j.achievable_tonnage_t <= 0.0 || r.gas_basis.is_none() {
        return (Vec::new(), Vec::new());
    }
    let assessed = match &o.is_code {
        Some(ic) => is_code_conditions(r, inp, ranked_cargo(r, j, inp, o), o, ic)
            .map(|conds| {
                (
                    loading_condition_checks_of(&conds, light_angle_warns(&o.rules)),
                    conds.into_iter().map(|c| c.condition).collect(),
                )
            })
            .map_err(|e| e.message().to_string()),
        None => GasCapacityResult::try_from(r)
            .map_err(|m| m.to_string())
            .and_then(|basis| {
                gs::assess(
                    &basis,
                    j.achievable_tonnage_t,
                    o.water_density_t_per_m3,
                    gs::criteria_set_for(&o.ruleset),
                )
                .map_err(|e| e.message().to_string())
            })
            .map(|a| {
                (
                    a.conditions
                        .iter()
                        .map(|c| gas_stability_check(c, light_angle_warns(&o.rules)))
                        .collect(),
                    a.conditions,
                )
            }),
    };
    match assessed {
        Ok(a) => a,
        Err(m) => (
            vec![Check::new(
                "Stability \u{2014} not assessed",
                false,
                text!(
                    "check-stability-not-assessed",
                    "stability could not be assessed: {reason}",
                    reason = m,
                ),
            )],
            Vec::new(),
        ),
    }
}

/// Whether a candidate passes its costly stage: every check, stability included, or, with Rule Sets
/// selected, no `fail` row in its Rule Trace.
fn passes_all(c: &Candidate, o: &FitOptions) -> bool {
    if o.rules.is_empty() {
        return all_ok(&stability_checks(&c.result, &c.journey, &c.inputs, o));
    }
    candidate_failures(&full_measured(c, o), o).is_empty()
}

/// What the whole Rule Trace of a candidate reads: every check and the stability conditions.
fn full_measured(c: &Candidate, o: &FitOptions) -> Measured {
    let (stab, conditions) = stability_assessment(&c.result, &c.journey, &c.inputs, o);
    let mut checks = cheap_checks(&c.result, &c.journey, c.depth, o);
    checks.extend(stab);
    let mut m = candidate_measured(
        &c.result, &c.journey, &c.inputs, c.depth, &checks, conditions,
    );
    // Issue #935: the damage the search worked out; its rows read as the answer's do.
    if let Some(verdicts) = &c.damage {
        m.damage = verdicts.clone();
        m.damage_status = crate::measured::DamageStatus::Run;
    }
    m
}

/// The `igc-2g` Rule Set of the stack when it evaluates IGC 2.7 damage survival (the set the results' damage block
/// is read from), and the search is to screen by it.
fn igc_damage_set(o: &FitOptions) -> Option<&RuleSet> {
    o.rules.iter().find(|s| {
        s.id == "igc-2g"
            && s.rule("igc/2.7/damage-survival")
                .is_some_and(|r| r.evaluated)
    })
}

/// Issue #935: work out the IGC damage of a candidate that passed the cheap checks and stability, as the answer's
/// run does (same conditions, same cargo, same spaces), and keep the verdicts on it. Does nothing when the stack
/// does not evaluate IGC 2.7, the runner is told to skip, or the candidate has no loading conditions with weights.
#[derive(PartialEq)]
enum Screened {
    /// The damage ran (or the candidate has nothing to run on): the candidate's `damage` is as far as it goes.
    Done,
    /// The search has used its damage runs.
    CapReached,
}

fn screen_damage(s: &mut Search<'_>, c: &mut Candidate) -> Result<Screened, PyError> {
    let o = s.o;
    let (Some(set), Some(ic)) = (igc_damage_set(o), o.is_code.as_ref()) else {
        return Ok(Screened::Done);
    };
    if !s.runner.enabled() || c.result.gas_basis.is_none() {
        return Ok(Screened::Done);
    }
    let Some(spec) = c.result.hull_spec.as_ref() else {
        return Ok(Screened::Done);
    };
    let cargo_t = ranked_cargo(&c.result, &c.journey, &c.inputs, o);
    let conditions =
        is_code_conditions(&c.result, &c.inputs, cargo_t, o, ic).map_err(|e| PyError {
            kind: e.type_name(),
            message: e.message().to_string(),
        })?;
    if conditions.is_empty() {
        return Ok(Screened::Done);
    }
    let hull = crate::hydrostatics::Hull::build(spec).map_err(|e| value_error(format!("{e}")))?;
    let model = crate::vessel_model::assemble(&crate::vessel_model::VesselModelInputs {
        capacity: c.inputs.clone(),
        result: c.result.clone(),
        hull: hull.table.clone(),
        structure: crate::vessel_model::StructureBlock::default(),
    });
    let spaces: Vec<crate::compartments::Space> =
        model.spaces.iter().map(|k| k.space.clone()).collect();
    // A Rule Set version without the tables gives no damage run, as the answer's run does.
    let Ok(plan) = gas_damage::gas_damage_plan(set, &spaces, &hull) else {
        return Ok(Screened::Done);
    };
    if s.damage_runs >= o.damage_run_cap {
        s.cap_hit = true;
        return Ok(Screened::CapReached);
    }
    let run = s
        .runner
        .run(&plan, &spaces, &hull, &conditions, o.water_density_t_per_m3)
        .map_err(|gas_damage::Cancelled| PyError {
            kind: "Cancelled",
            message: gas_damage::DAMAGE_CANCELLED.to_string(),
        })?;
    s.damage_runs += 1;
    c.damage = Some(run.verdicts);
    Ok(Screened::Done)
}

/// Every check of a candidate: the cheap ones, then stability.
fn search_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    depth: f64,
    o: &FitOptions,
) -> Vec<Check> {
    let mut all = cheap_checks(r, j, depth, o);
    all.extend(stability_checks(r, j, inp, o));
    all
}

/// The checks the search applies to the answer beyond its capacity result's own checks and its
/// stability: the journey checks, tank length over diameter and freeboard at the payload. A
/// result lists them, so every check that can remove a candidate is visible (issue #418).
pub fn answer_search_checks(f: &ArrangementFit, o: &FitOptions) -> Result<Vec<Check>, PyError> {
    let r = &f.engine_result;
    let j = derate_candidate(
        r,
        f.loa_m,
        f.beam_m,
        f.depth_m,
        f.t_max_m,
        o.water_density_t_per_m3,
    )?;
    let own: Vec<&str> = r.checks.iter().map(|c| c.name.as_str()).collect();
    Ok(cheap_checks(r, &j, f.depth_m, o)
        .into_iter()
        .filter(|c| !own.contains(&c.name.as_str()))
        .collect())
}

/// One built candidate: the inputs and the capacity result they gave.
type Built = (BargeCapacityInputs, BargeCapacityResult);

/// One scored candidate, as Python's `(payload, depth, result, journey, inputs)` tuple.
#[derive(Clone)]
struct Candidate {
    /// How many cheap checks fail (see "THE CHECKS THE SEARCH USES"), counted as
    /// `closest_to_passing` counts: never more than the candidate fails in all.
    cheap_failed: usize,
    tonnage: f64,
    /// The moulded depth of the candidate. With a standing height (issue #992) it follows the tank.
    depth: f64,
    /// The depth of the search step that built it (the same as `depth` without a standing height).
    search_depth: f64,
    result: BargeCapacityResult,
    journey: JourneyDeratingResult,
    inputs: BargeCapacityInputs,
    /// Issue #935: the damage verdicts of the candidate, when the search ran its damage (only candidates that
    /// passed the cheap checks and stability get it).
    damage: Option<Vec<crate::measured::DamageVerdict>>,
}

fn derate_candidate(
    r: &BargeCapacityResult,
    loa: f64,
    beam: f64,
    depth: f64,
    t_max: f64,
    water_density: f64,
) -> Result<JourneyDeratingResult, PyError> {
    let mut ji = JourneyDeratingInputs::new(r, loa, beam, depth);
    ji.water_density = water_density;
    ji.design_depth_m = t_max;
    ji.ukc_m = 0.0;
    derate(&ji).map_err(value_error)
}

/// Python's `fit.build`, with its `nonlocal tried` as `&mut tried`.
struct Search<'a> {
    loa: f64,
    beam: f64,
    cargo: &'a str,
    o: &'a FitOptions,
    tried: i64,
    /// Issue #675: the arrangements `build` turned down, as (depth, tanks across, reason).
    refused: Vec<(f64, i64, Text)>,
    /// Issue #935: works out the damage of a candidate (threads, progress and cancel are the caller's).
    runner: &'a dyn DamageRunner,
    /// Issue #935: the damage runs worked out so far.
    damage_runs: usize,
    /// Issue #935: the search reached `damage_run_cap` and stopped screening.
    cap_hit: bool,
    /// Issue #935: the numbers of tanks along the search tries, the profile's first (just `tanks_in_line` when it
    /// is pinned, or when damage is not screened).
    along: Vec<i64>,
    /// Issue #992: stop after the cheap stage and take the most cargo among the candidates that pass
    /// it (the probe of [`probe_depth`]); the costly stability stage does not run.
    cheap_only: bool,
}

impl Search<'_> {
    fn base_inputs(&self, depth: f64, nw: i64, nl: i64) -> BargeCapacityInputs {
        let mut inp = BargeCapacityInputs::new(self.loa, self.beam, depth);
        inp.cargo_type = self.cargo.to_string();
        inp.self_propelled = self.o.self_propelled;
        inp.vessel_type = self.o.ruleset.clone();
        inp.fill_ratio = self.o.fill_ratio;
        inp.cargo_density_kg_m3 = self.o.cargo_density_kg_m3;
        inp.stern_block_mass = self.o.stern_block_mass.clone();
        inp.stern_geometry = self.o.stern_geometry.clone();
        inp.machinery = self.o.machinery.clone();
        inp.sightline_limit_m = self.o.sightline_limit_m;
        inp.sightline_limit_laden_m = self.o.sightline_limit_laden_m;
        inp.sightline_sizing_limit_m = self.o.sightline_sizing_limit_m;
        inp.sightline_limit_source = self.o.sightline_limit_source.clone();
        inp.block_coefficient = self.o.block_coefficient;
        inp.n_tanks_wide = Some(nw);
        inp.n_tanks_long = nl;
        inp.max_tank_od_m = self.o.max_tank_od_m;
        inp.bow_rake_to_depth_ratio = self.o.bow_rake_to_depth_ratio;
        inp.bow_rake_max_length_m = self.o.bow_rake_max_length_m;
        inp.bow_rake_length_m = self.o.bow_rake_length_m;
        inp.bow_rake_rise_m = self.o.bow_rake_rise_m;
        inp.hull_shape = self.o.hull_shape.clone();
        inp.downflooding_angle_deg = self.o.downflooding_angle_deg;
        inp.downflooding_openings = self.o.downflooding_openings.clone();
        inp.pushed_stern_void_m = self.o.pushed_stern_void_m;
        inp.tank_support_height = self.o.tank_support_height_m;
        inp.dome_height = self.o.tank_dome_height_m;
        inp.dome_diameter = self.o.tank_dome_diameter_m;
        inp.dome_diameter_by_rule = self.o.tank_dome_diameter_by_rule;
        inp.dome_height_by_rule = self.o.tank_dome_height_by_rule;
        inp.wing_tank_gap_m = self.o.wing_tank_gap_m;
        // Issue #850: the hull limits of ADN 9.3.1.11.2 (a), when the stack holds them.
        inp.double_side_min_m = sizing_limit(&self.o.rules, ADN_DOUBLE_SIDE_RULE);
        inp.double_bottom_min_m = sizing_limit(&self.o.rules, ADN_DOUBLE_BOTTOM_RULE);
        inp.wing_tank_height_ratio = self.o.wing_tank_height_ratio;
        // Issue #992: with the tanks in hold (a standing height of 0) nothing stands over the deck
        // but the domes, so there is no shield, whatever its thickness is set to.
        inp.weather_shield_thickness_mm = if self.o.tank_top_above_deck_m == Some(0.0) {
            0.0
        } else {
            self.o.weather_shield_thickness_mm
        };
        inp.weather_shield_clearance_m = self.o.weather_shield_clearance_m;
        inp.structure = self.o.structure;
        inp.tank_forming_allowance_pct = self.o.tank_forming_allowance_pct;
        inp.tank_dome_wall_thickness_mm = self.o.tank_dome_wall_thickness_mm;
        // Issue #280: end spaces of the Option's own, when they are not the engine's default.
        let default_space = crate::gas_separation::default_end_space(false);
        if self.o.aft_end_space_m != default_space
            || self.o.fwd_end_space_m != default_space
            || !self.o.opening_setback_from_cargo_area
            || self.o.hold_cofferdam_m != 0.0
        {
            inp.separation = Some(crate::gas_separation::GasSeparationInputs {
                cofferdam_m: self.o.aft_end_space_m,
                fwd_end_bulkhead_m: self.o.fwd_end_space_m,
                setback_from_cargo_area: self.o.opening_setback_from_cargo_area,
                hold_cofferdam_m: self.o.hold_cofferdam_m,
                ..Default::default()
            });
        }
        inp.tank_wall_thickness = self.o.tank_wall_thickness_mm;
        inp.tank_design = self.o.tank_design.clone();
        inp.water_density = self.o.water_density_t_per_m3;
        inp.air_draft_limit_m = self.o.air_draft_limit_m;
        inp.tank_top_above_deck_m = self.o.tank_top_above_deck_m;
        inp.bridge_clearance_m = self.o.bridge_clearance_m;
        inp.bridge_clearance_source = self.o.bridge_clearance_source.clone();
        inp.aft_design = self.o.aft_design.clone();
        inp.river_only = self.o.river_only;
        inp.vent_stub_m = self.o.vent_stub_m;
        inp.vent_mast_lowerable = self.o.vent_mast_lowerable;
        inp.entrance_distance_m = self.o.entrance_distance_m;
        inp.engine_room_vent_inlet_distance_m = self.o.engine_room_vent_inlet_distance_m;
        inp.funnel_outlet_distance_m = self.o.funnel_outlet_distance_m;
        // The user's freeboard, so the full-volume `freeboard_ok` the result shows uses it (#418).
        inp.required_freeboard = self.o.required_freeboard_m;
        inp
    }

    fn od_unusable(&self, r: &BargeCapacityResult) -> bool {
        let od = opt_or_zero(r.tank_outer_diameter_m);
        od == 0.0 || od > self.o.max_tank_od_m + 1e-6
    }

    /// Issue #992: the depth for a tank whose top is `top` above the keel and which is to stand `h`
    /// above the deck: the tank-driven depth, but not less than the draught limit, the least
    /// freeboard and the margin (D1a of the design).
    fn target_depth(&self, top: f64, h: f64) -> f64 {
        let tank = depth_for_standing_height(top, h);
        match self.o.draught_limit_m {
            Some(t) => {
                let floor = t + self.o.required_freeboard_m + self.o.depth_margin_m;
                tank.max(libm::ceil(floor * 1000.0) / 1000.0)
            }
            None => tank,
        }
    }

    fn build(&mut self, d: f64, nw: i64, nl: i64) -> Result<Option<Built>, PyError> {
        self.tried += 1;
        let mut built = self.build_or_refuse(d, nw, nl)?;
        // Issue #992: the depth follows the tank. The tank depends a little on the depth (the bow
        // rake is a share of it, so the cargo zone and the tank length move), so the depth is
        // settled by repeating the build at the depth the last tank gave, to the millimetre.
        if let (Ok((_, r)), Some(h)) = (&built, self.o.tank_top_above_deck_m) {
            let od = opt_or_zero(r.tank_outer_diameter_m);
            if h > od + 1e-9 {
                built = Err(text!(
                    "fit-refused-standing-height",
                    "The tank top is to stand {h} m above the deck, more than the tank diameter {od} m.",
                    h = Arg::fixed(h, 2),
                    od = Arg::fixed(od, 2),
                ));
            } else {
                let mut dd = self.target_depth(tank_top_above_keel_m(r), h);
                let mut settled = false;
                for _ in 0..DEPTH_SETTLE_ROUNDS {
                    let b = self.build_or_refuse(dd, nw, nl)?;
                    let next = match &b {
                        Ok((_, r2)) => self.target_depth(tank_top_above_keel_m(r2), h),
                        Err(_) => dd,
                    };
                    built = b;
                    if (next - dd).abs() < 0.0015 {
                        settled = true;
                        break;
                    }
                    dd = next;
                }
                if built.is_ok() && !settled {
                    built = Err(text!(
                        "fit-refused-depth-not-settled",
                        "The depth that follows the tank does not settle, near {depth} m.",
                        depth = Arg::fixed(dd, 2),
                    ));
                }
            }
        }
        Ok(match built {
            Ok(b) => Some(b),
            Err(reason) => {
                self.refused.push((d, nw, reason));
                None
            }
        })
    }

    fn build_or_refuse(&self, d: f64, nw: i64, nl: i64) -> Result<Result<Built, Text>, PyError> {
        let mut inp = self.base_inputs(d, nw, nl);
        let mut r = capacity(&inp)?;
        // Issue #675: a side space under the user's least one is widened to it.
        let min_side = self.o.min_side_space_m;
        if min_side > 0.0 && opt_or_zero(r.side_clearance_m) + 1e-9 < min_side {
            inp.side_clearance_override_m = Some(min_side);
            r = capacity(&inp)?;
        }
        if self.od_unusable(&r) {
            // Full-beam tank infeasible or over the cap: put the tank ON the cap and widen the
            // clearance: a legitimate design only if that clearance meets the rule.
            // The n - 1 gaps between tanks side by side take beam too (issue #422), as the tank
            // layout counts them; without them a capped tank came out below the cap.
            let cs = (self.beam
                - nw as f64 * self.o.max_tank_od_m
                - (nw - 1) as f64 * DEFAULT_TANK_GAP_TRANSVERSE_M)
                / 2.0;
            // Mutation survivor (#59), `cs < 0.0` to `cs == 0.0`: equivalent. `cs` is negative only
            // when the beam is under `nw` tank diameters at the cap. A full-beam tank over the cap
            // needs a beam over `nw` caps, so a negative `cs` is reached only with an unusable
            // full-beam tank of zero diameter (the tanks do not fit the hull). Under the mutant
            // the negative clearance goes on to `capacity`, which cannot make a tank over the cap
            // (the diameter is then at most the cap) and a candidate with it is turned down by
            // the rule check below: the rule needs at least 0.45 m (`d(V)` is at least 0.8 m, the
            // insulation is the tank design's thickness: 350 mm when this was searched, and the profile range runs to 1000 mm, which this argument does not cover), and `cs + 1e-9 < need` holds for every negative
            // `cs`. So `None` comes back either way. Searched with the mutant built in: 15 000
            // random inputs (both rulesets of the time, caps 0.2 to 8.2 m, beams 0.3 to 14.3 m, 1 to 5 tanks
            // wide), the branch reached in at least 10 068 of them, no difference in any field.
            // `a_hull_too_narrow_for_tanks_at_the_cap_gets_no_candidate` and
            // `the_side_clearance_rule_is_never_below_0_45_m` guard the two facts this rests on.
            if cs < 0.0 {
                return Ok(Err(text!(
                    "fit-refused-cap-too-wide",
                    "{n} across at the diameter cap {cap} m do not fit in the beam.",
                    n = nw,
                    cap = Arg::fixed(self.o.max_tank_od_m, 2),
                )));
            }
            if cs + 1e-9 < min_side {
                return Ok(Err(text!(
                    "fit-refused-min-side-space",
                    "No tank fits across the beam with the minimum side space {min} m.",
                    min = Arg::fixed(min_side, 2),
                )));
            }
            inp.side_clearance_override_m = Some(cs);
            r = capacity(&inp)?;
            if self.od_unusable(&r) {
                return Ok(Err(text!(
                    "fit-refused-no-tank-at-cap",
                    "No tank fits at the diameter cap {cap} m.",
                    cap = Arg::fixed(self.o.max_tank_od_m, 2),
                )));
            }
            let total = opt_or_zero(r.containment_inner_volume_m3);
            // Mutation survivor (#146), `total / n` to `total % n` below: equivalent. The per-tank
            // volume matters only through the rule d(V) - insulation, which is non-decreasing
            // in V (the dome adds the same volume to every tank), and continuous, so the loop that sets the clearance settles. We are here only when the full-beam tank is over the cap by more than the
            // 1e-6 of `od_unusable`, so the clearance `cs` is at least 5e-7 above the one the
            // full-beam attempt settled on, and that settled clearance is already at least the
            // rule for the full-beam tank. The cap tank is smaller, so its rule is smaller still,
            // and the check below passes whatever per-tank volume is given. The other way in, a
            // full-beam tank of zero diameter, needs a beam under 0.9 m, where `cs` is under the
            // 0.45 m floor of the rule for every volume the engine can build. Searched with the
            // mutant built in: 180 000 random inputs, 240 000 more at hull sizes up to 4 km, none
            // differ. Not a gap in the tests.
            let need = required_side_clearance(
                &self.o.ruleset,
                self.beam,
                total,
                // Vc of one tank: the tank plus its dome (IGC 2.4.1.1).
                total / tank_divisor(r.n_tanks_total) + {
                    let (d, h) = dome_dimensions(
                        opt_or_zero(r.tank_outer_diameter_m)
                            - 2.0 * opt_or_zero(r.insulation_thickness_mm) / 1000.0,
                        (!self.o.tank_dome_diameter_by_rule).then_some(self.o.tank_dome_diameter_m),
                        (!self.o.tank_dome_height_by_rule).then_some(self.o.tank_dome_height_m),
                    );
                    dome_volume_m3(d, h)
                },
                opt_or_zero(r.insulation_thickness_mm),
            )?;
            // Killed by `a_cap_clearance_a_hair_under_the_rule_is_accepted_not_refused` (`+` to `-`).
            if cs + 1e-9 < need {
                return Ok(Err(text!(
                    "fit-refused-cap-side-space",
                    "At the diameter cap {cap} m the side space is {side} m, under the IGC 2.4 minimum {need} m.",
                    cap = Arg::fixed(self.o.max_tank_od_m, 2),
                    side = Arg::fixed(cs, 2),
                    need = Arg::fixed(need, 2),
                )));
            }
        }
        Ok(Ok((inp, r)))
    }
}

/// Every candidate the search built, and the index of the best one that passes every check.
struct Pool {
    all: Vec<Candidate>,
    best: Option<usize>,
    /// The candidates whose costly stage ran and failed (issue #675).
    assessed_failed: Vec<usize>,
    /// Issue #935: the best candidate before the damage screen: the pick of a search without damage.
    unscreened_best: Option<usize>,
    /// Issue #935: the most damage runs the search expects (0: the stack has no IGC damage to screen by). An upper
    /// bound: the candidates that pass the cheap checks and three refinement steps for each depth and number of
    /// tanks across, within the cap.
    planned_runs: usize,
}

/// Build every candidate and judge it by the whole check list. Every candidate gets the cheap
/// checks. The stability assessment, the costly check, runs only for the candidates that passed
/// the cheap ones, from the most cargo down, and stops at the first that passes it: a candidate
/// with less cargo cannot win, so its stability is not needed. Among candidates within 1e-6 t of
/// the winner the first built stays, as before. With `stop_at_first_pass` the search ends at the
/// first passing candidate (for a yes or no answer).
fn search(s: &mut Search<'_>, t_max: f64, stop_at_first_pass: bool) -> Result<Pool, PyError> {
    let (loa, beam, o) = (s.loa, s.beam, s.o);
    let mut pool = Pool {
        all: Vec::new(),
        best: None,
        assessed_failed: Vec::new(),
        unscreened_best: None,
        planned_runs: 0,
    };
    for along in s.along.clone() {
        for &d in &o.depths {
            for &nw in &o.tanks_wide {
                let mut cands: Vec<Built> = Vec::new();
                if let Some(x) = s.build(d, nw, along)? {
                    cands.push(x);
                }
                let mut more: Vec<Built> = Vec::new();
                for (inp0, r0) in &cands {
                    if opt_or_zero(r0.tank_outer_diameter_m) == 0.0 {
                        continue;
                    }
                    let Some(side) = r0.side_clearance_m else {
                        continue;
                    };
                    for &extra in &o.extra_clearance_m {
                        s.tried += 1;
                        let mut i2 = inp0.clone();
                        i2.side_clearance_override_m = Some(side + extra);
                        let mut r2 = capacity(&i2)?;
                        // Issue #992: a wider side space gives a smaller tank, so the depth that
                        // follows the tank is settled again for this variant.
                        if let Some(h) = o.tank_top_above_deck_m {
                            let mut settled = false;
                            for _ in 0..DEPTH_SETTLE_ROUNDS {
                                let want = s.target_depth(tank_top_above_keel_m(&r2), h);
                                if (want - i2.depth).abs() < 0.0015 {
                                    settled = true;
                                    break;
                                }
                                i2.depth = want;
                                r2 = capacity(&i2)?;
                            }
                            let od2 = opt_or_zero(r2.tank_outer_diameter_m);
                            let why = if !settled {
                                Some(text!(
                                    "fit-refused-depth-not-settled",
                                    "The depth that follows the tank does not settle, near {depth} m.",
                                    depth = Arg::fixed(i2.depth, 2),
                                ))
                            } else if h > od2 + 1e-9 {
                                Some(text!(
                                    "fit-refused-standing-height",
                                    "The tank top is to stand {h} m above the deck, more than the tank diameter {od} m.",
                                    h = Arg::fixed(h, 2),
                                    od = Arg::fixed(od2, 2),
                                ))
                            } else {
                                None
                            };
                            if let Some(why) = why {
                                s.refused.push((d, nw, why));
                                continue;
                            }
                        }
                        // Mutation survivors (#58), `&&` to `||` and `>` to `>=` below: equivalent. The
                        // two differ from the real test only for a candidate with a tank but zero
                        // capacity (a fill ratio of 0, or a product that underflows to 0); a tank
                        // diameter of 0 comes with zero capacity, so `||` adds nothing else. Such a
                        // candidate joins the search under the mutants and can never win: its payload is
                        // 0.0 (never above the 1e-6 margin over a payload that is 0.0 or more), and
                        // its `feasible_geometry` check fails (cargo volume must be positive), so it
                        // is never the best passing candidate either. It is also built last, after
                        // the base candidate, so it cannot win a tie. Searched with each mutant built
                        // in: 12 000 random inputs (rulesets, fill ratios from 0, tank caps, depths,
                        // clearances, 9 969 infeasible), and 175 824 direct capacity runs, none differ
                        // in any field. `a_zero_capacity_arrangement_is_infeasible_and_carries_nothing`
                        // guards the two facts this rests on. Not a gap in the tests.
                        if opt_or_zero(r2.tank_outer_diameter_m) != 0.0 && r2.cargo_capacity_t > 0.0
                        {
                            more.push((i2, r2));
                        }
                    }
                }
                for (inp, r) in cands.into_iter().chain(more) {
                    // With a standing height (issue #992) the depth is the candidate's own, and the
                    // draught cap is the less of the limit and its own depth less the freeboard.
                    let dep = inp.depth;
                    let t_cap = match o.tank_top_above_deck_m {
                        Some(_) => t_max.min(dep - o.required_freeboard_m),
                        None => t_max,
                    };
                    let j = derate_candidate(&r, loa, beam, dep, t_cap, o.water_density_t_per_m3)?;
                    let cheap = cheap_checks(&r, &j, dep, o);
                    // With Rule Sets: the trace before stability (its stability Rules are not yet
                    // evaluated, so they cannot fail here).
                    let cheap_failed = if o.rules.is_empty() {
                        cheap.iter().filter(|k| k.is_error() && !k.ok).count()
                    } else {
                        failing_groups(&candidate_failures(
                            &candidate_measured(&r, &j, &inp, dep, &cheap, Vec::new()),
                            o,
                        ))
                        .len()
                    };
                    pool.all.push(Candidate {
                        cheap_failed,
                        tonnage: ranked_cargo(&r, &j, &inp, o),
                        depth: dep,
                        search_depth: d,
                        result: r,
                        journey: j,
                        inputs: inp,
                        damage: None,
                    });
                }
            }
        }
    }
    // The costly check, from the most cargo down (a stable sort keeps the build order of equals).
    let mut order: Vec<usize> = (0..pool.all.len())
        .filter(|&i| pool.all[i].cheap_failed == 0)
        .collect();
    order.sort_by(|&a, &b| pool.all[b].tonnage.total_cmp(&pool.all[a].tonnage));
    if s.cheap_only {
        pool.best = order.first().copied();
        return Ok(pool);
    }
    // Issue #935: tell the runner how many damage runs to plan for, so its progress is a fixed total that shrinks at
    // the end (the fraction done never falls), and keep it for the results (the estimate before a damage run).
    // Over every number of tanks along the search tries: the candidates that pass the cheap checks and three
    // refinement steps for each depth, number across and number along, within the cap, which bounds the search.
    let screening = igc_damage_set(o).is_some() && o.is_code.is_some();
    if screening {
        let mut groups: Vec<(f64, i64, i64)> = Vec::new();
        for &i in &order {
            let g = (
                pool.all[i].search_depth,
                pool.all[i].inputs.n_tanks_wide.unwrap_or(0),
                pool.all[i].inputs.n_tanks_long,
            );
            if !groups.contains(&g) {
                groups.push(g);
            }
        }
        pool.planned_runs = (order.len() + 3 * groups.len()).clamp(1, o.damage_run_cap.max(1));
        if s.runner.enabled() {
            s.runner.plan(pool.planned_runs);
        }
    }
    let mut top = 0.0;
    let mut unscreened_top = 0.0;
    'counts: for along in s.along.clone() {
        // The candidates of this number of tanks along, from the most cargo down.
        let in_count: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| pool.all[i].inputs.n_tanks_long == along)
            .collect();
        for i in in_count {
            // A candidate with less cargo than the best pass cannot win, whatever number along it has.
            if pool.best.is_some() && pool.all[i].tonnage < top - 1e-6 {
                break;
            }
            // Issue #935: damage is the slowest part, so it runs last, on a candidate that passed everything else.
            let mut ok = passes_all(&pool.all[i], o);
            // What a search without damage would pick: the same rule as the pick below, on the candidates of the
            // profile's number of tanks along that passed before damage (the first evaluation keeps that number).
            if ok && along == o.tanks_in_line {
                match pool.unscreened_best {
                    None => {
                        unscreened_top = pool.all[i].tonnage;
                        pool.unscreened_best = Some(i);
                    }
                    Some(w) if i < w && pool.all[i].tonnage >= unscreened_top - 1e-6 => {
                        pool.unscreened_best = Some(i);
                    }
                    Some(_) => {}
                }
            }
            if ok && screening {
                if screen_damage(s, &mut pool.all[i])? == Screened::CapReached {
                    break 'counts;
                }
                ok = passes_all(&pool.all[i], o);
            }
            if !ok {
                pool.assessed_failed.push(i);
            } else {
                match pool.best {
                    None => {
                        top = pool.all[i].tonnage;
                        pool.best = Some(i);
                        if stop_at_first_pass {
                            break 'counts;
                        }
                    }
                    Some(w) if better_pick(&pool.all, i, w) => {
                        top = top.max(pool.all[i].tonnage);
                        pool.best = Some(i);
                    }
                    Some(_) => {}
                }
            }
        }
        // Issue #935: a size that fails damage is refined between it and the next size down.
        if screening && s.runner.enabled() && !s.cap_hit {
            refine_failed_groups(s, &mut pool, t_max, along)?;
            if let Some(b) = pool.best {
                top = top.max(pool.all[b].tonnage);
            }
        }
    }
    Ok(pool)
}

/// What the pick compares of a candidate (issue #935, addendum D3).
#[derive(Debug, Clone, Copy, PartialEq)]
struct PickKey {
    tonnage: f64,
    tanks: i64,
    along: i64,
    od_m: f64,
    index: usize,
}

/// Whether `a` is a better pick than `b`: the most cargo, and where two carry the same (to 1e-6 t) and have a
/// different number of tanks along, the fewest tanks, then the fewest along, then the larger diameter. Within one
/// number along, the first built stays, as before.
fn better_key(a: &PickKey, b: &PickKey) -> bool {
    if a.tonnage > b.tonnage + 1e-6 {
        return true;
    }
    if a.tonnage < b.tonnage - 1e-6 {
        return false;
    }
    if a.along == b.along {
        return a.index < b.index;
    }
    (a.tanks, a.along, -a.od_m) < (b.tanks, b.along, -b.od_m)
}

/// [`better_key`] of candidates `i` and `w` of `all`.
fn better_pick(all: &[Candidate], i: usize, w: usize) -> bool {
    let key = |k: usize| PickKey {
        tonnage: all[k].tonnage,
        tanks: opt_count(all[k].result.n_tanks_total),
        along: all[k].inputs.n_tanks_long,
        od_m: opt_or_zero(all[k].result.tank_outer_diameter_m),
        index: k,
    };
    better_key(&key(i), &key(w))
}

/// Issue #935: the sizes the grid offers are one clearance step apart (about 1 m of diameter for two across), and
/// damage survival is a step function of the diameter, so the best tank can lie between two sizes: one fails, the
/// next down passes with less cargo than a candidate of another number of tanks across.
///
/// For each depth and number of tanks across that had a size fail damage, and that could still beat the best pass
/// (its failing size carries more cargo), bisect the diameter between its smallest failing size and the next size
/// down, to [`DAMAGE_REFINE_STEP_M`]. Each step builds the candidate through the side clearance (diameter
/// `(B - 2 cs - (n - 1) gap) / n`), checks it before damage and runs its damage: a pass moves the lower bound up
/// and joins the pool, a fail moves the upper bound down. The pass is not monotone (research section 4a), so this
/// finds a pass between the two bounds, not the largest possible. Every damage run counts against the cap; on
/// reaching it the search keeps the best pass so far.
fn refine_failed_groups(
    s: &mut Search<'_>,
    pool: &mut Pool,
    t_max: f64,
    along: i64,
) -> Result<(), PyError> {
    let o = s.o;
    // (depth, tanks across): the smallest failing diameter of the group and the candidate that has it.
    let mut groups: Vec<(f64, i64, usize)> = Vec::new();
    for &i in &pool.assessed_failed {
        let c = &pool.all[i];
        // Only this number of tanks along; a candidate that failed before damage (stability) has no damage and
        // bounds nothing.
        if c.inputs.n_tanks_long != along || c.damage.is_none() {
            continue;
        }
        // The search step it was built at (the depth follows the tank with a standing height, issue #992).
        let (d, nw) = (c.search_depth, c.inputs.n_tanks_wide.unwrap_or(0));
        let od = opt_or_zero(c.result.tank_outer_diameter_m);
        match groups.iter_mut().find(|g| g.0 == d && g.1 == nw) {
            Some(g) => {
                if od < opt_or_zero(pool.all[g.2].result.tank_outer_diameter_m) {
                    g.2 = i;
                }
            }
            None => groups.push((d, nw, i)),
        }
    }
    for (depth, nw, hi_idx) in groups {
        if nw < 1 {
            continue;
        }
        let hi_cand = pool.all[hi_idx].clone();
        // It cannot help if the failing size carries no more than the best pass.
        if let Some(b) = pool.best
            && hi_cand.tonnage <= pool.all[b].tonnage + 1e-6
        {
            continue;
        }
        let mut hi = opt_or_zero(hi_cand.result.tank_outer_diameter_m);
        let Some(cs_hi) = hi_cand.result.side_clearance_m else {
            continue;
        };
        // The next size down: the largest diameter below `hi` in the group that passed before damage.
        let mut lo = pool
            .all
            .iter()
            .filter(|c| {
                c.search_depth == depth
                    && c.inputs.n_tanks_long == along
                    && c.inputs.n_tanks_wide == Some(nw)
                    && c.cheap_failed == 0
                    && opt_or_zero(c.result.tank_outer_diameter_m) < hi - 1e-9
            })
            .map(|c| opt_or_zero(c.result.tank_outer_diameter_m))
            .fold(0.0, f64::max);
        if lo <= 0.0 {
            continue;
        }
        while hi - lo > DAMAGE_REFINE_STEP_M + 1e-9 {
            if s.damage_runs >= o.damage_run_cap {
                s.cap_hit = true;
                return Ok(());
            }
            let mid = 0.5 * (hi + lo);
            let mut inp = hi_cand.inputs.clone();
            // The side clearance that gives diameter `mid` across `nw` tanks, from `hi`'s.
            inp.side_clearance_override_m = Some(cs_hi + 0.5 * nw as f64 * (hi - mid));
            let mut r = capacity(&inp)?;
            // Issue #992: with a standing height the depth follows the tank, settled again for this diameter.
            let mut refused = false;
            if let Some(h) = o.tank_top_above_deck_m {
                let mut settled = false;
                for _ in 0..DEPTH_SETTLE_ROUNDS {
                    let want = s.target_depth(tank_top_above_keel_m(&r), h);
                    if (want - inp.depth).abs() < 0.0015 {
                        settled = true;
                        break;
                    }
                    inp.depth = want;
                    r = capacity(&inp)?;
                }
                refused = !settled || h > opt_or_zero(r.tank_outer_diameter_m) + 1e-9;
            }
            let got = opt_or_zero(r.tank_outer_diameter_m);
            if refused || got <= 0.0 || r.cargo_capacity_t <= 0.0 || (got - mid).abs() > 1e-6 {
                // The capacity gave another diameter (a cap or a limit): this size is not offered.
                hi = mid;
                continue;
            }
            // The depth and the draught cap are the candidate's own, as the search builds them.
            let dep = inp.depth;
            let t_cap = match o.tank_top_above_deck_m {
                Some(_) => t_max.min(dep - o.required_freeboard_m),
                None => t_max,
            };
            let j = derate_candidate(&r, s.loa, s.beam, dep, t_cap, o.water_density_t_per_m3)?;
            let cheap = cheap_checks(&r, &j, dep, o);
            let cheap_failed = failing_groups(&candidate_failures(
                &candidate_measured(&r, &j, &inp, dep, &cheap, Vec::new()),
                o,
            ))
            .len();
            let mut cand = Candidate {
                cheap_failed,
                tonnage: ranked_cargo(&r, &j, &inp, o),
                depth: dep,
                search_depth: hi_cand.search_depth,
                result: r,
                journey: j,
                inputs: inp,
                damage: None,
            };
            let idx = pool.all.len();
            let mut passes = cand.cheap_failed == 0 && passes_all(&cand, o);
            if passes && screen_damage(s, &mut cand)? == Screened::CapReached {
                return Ok(());
            }
            passes = passes && passes_all(&cand, o);
            pool.all.push(cand);
            if passes {
                lo = mid;
                // The most cargo among the passes (the tie-break of `better_pick`).
                if pool.best.is_none_or(|b| better_pick(&pool.all, idx, b)) {
                    pool.best = Some(idx);
                }
            } else {
                pool.assessed_failed.push(idx);
                hi = mid;
            }
        }
    }
    Ok(())
}

/// The closest candidate to passing, when none passes: the fewest failed error checks, then the
/// most cargo (the first of two near-equal ones stays).
fn closest_to_passing(pool: &[Candidate], o: &FitOptions) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for (i, c) in pool.iter().enumerate() {
        // The cheap failures are a floor on all of them: skip the costly stage when even the floor
        // cannot beat the best so far. The answer is the one a full count of each gives.
        if let Some((bi, bf)) = best
            && (c.cheap_failed > bf
                || (c.cheap_failed == bf && c.tonnage <= pool[bi].tonnage + 1e-6))
        {
            continue;
        }
        let failed = if o.rules.is_empty() {
            search_checks(&c.result, &c.journey, &c.inputs, c.depth, o)
                .iter()
                .filter(|k| k.is_error() && !k.ok)
                .count()
        } else {
            failing_groups(&candidate_failures(&full_measured(c, o), o)).len()
        };
        let better = match best {
            None => true,
            Some((bi, bf)) => failed < bf || (failed == bf && c.tonnage > pool[bi].tonnage + 1e-6),
        };
        if better {
            best = Some((i, failed));
        }
    }
    best.map(|(i, _)| i)
}

/// The error checks a candidate fails, for the list of arrangements tried (issue #675): the cheap
/// ones, then (when its costly stage ran) stability, then, with Rule Sets, the failing Rules no
/// check reports, each as a check named by its Rule id.
fn failing_checks(c: &Candidate, o: &FitOptions, assessed: bool) -> Vec<Check> {
    let mut checks = cheap_checks(&c.result, &c.journey, c.depth, o);
    if assessed {
        checks.extend(stability_checks(&c.result, &c.journey, &c.inputs, o));
    }
    let mut out: Vec<Check> = checks
        .iter()
        .filter(|k| k.is_error() && !k.ok)
        .cloned()
        .collect();
    if !o.rules.is_empty() {
        let measured = if assessed {
            full_measured(c, o)
        } else {
            candidate_measured(
                &c.result,
                &c.journey,
                &c.inputs,
                c.depth,
                &checks,
                Vec::new(),
            )
        };
        for row in candidate_trace(&measured, o)
            .rows
            .iter()
            .filter(|r| r.outcome == Outcome::Fail)
        {
            let mirrored = row
                .measure
                .as_ref()
                .is_some_and(|m| m.id == "check.ok" || m.id.starts_with("separation."))
                || row.object_type.as_deref() == Some(ObjectType::LoadingCondition.key());
            let name = row.rule.clone().unwrap_or_default();
            if !mirrored && !out.iter().any(|k| k.name == name) {
                out.push(Check::new(
                    name,
                    false,
                    format!("{} ({})", row.title, row.object.clone().unwrap_or_default()),
                ));
            }
        }
    }
    out
}

/// One row per depth and number of tanks across that the search tried, in its order (issue #675).
fn tried_arrangements(
    pool: &Pool,
    pick: Option<usize>,
    refused: &[(f64, i64, Text)],
    o: &FitOptions,
    counts: &[i64],
) -> Vec<TriedArrangement> {
    let nw_of = |c: &Candidate| c.inputs.n_tanks_wide.unwrap_or(0);
    let mut out = Vec::new();
    // Issue #935: one row per depth, number across and number of tanks along the search tried.
    for (k, &nl) in counts.iter().enumerate() {
        for &d in &o.depths {
            for &nw in &o.tanks_wide {
                let group: Vec<usize> = (0..pool.all.len())
                    .filter(|&i| {
                        pool.all[i].search_depth == d
                            && nw_of(&pool.all[i]) == nw
                            && pool.all[i].inputs.n_tanks_long == nl
                    })
                    .collect();
                if group.is_empty() {
                    // A size that could not be built is reported once, not for each number along.
                    if k == 0
                        && let Some((_, _, why)) =
                            refused.iter().find(|(rd, rn, _)| *rd == d && *rn == nw)
                    {
                        out.push(TriedArrangement {
                            depth_m: d,
                            n_wide: nw,
                            n_long: nl,
                            tank_od_m: 0.0,
                            side_clearance_m: 0.0,
                            wing_tank_width_m: 0.0,
                            cargo_volume_m3: 0.0,
                            cargo_t: 0.0,
                            variants: 0,
                            outcome: TriedOutcome::NotBuilt(why.clone()),
                        });
                    }
                    continue;
                }
                let failed = |i: &usize| pool.assessed_failed.contains(i);
                // The first of the most cargo stays (a strict comparison), as the search ranks.
                let most_cargo = |ids: &mut dyn Iterator<Item = usize>| {
                    ids.fold(None, |best: Option<usize>, i| match best {
                        Some(b) if pool.all[i].tonnage <= pool.all[b].tonnage + 1e-6 => Some(b),
                        _ => Some(i),
                    })
                };
                let chosen = pick.filter(|p| group.contains(p));
                let passing = most_cargo(
                    &mut group
                        .iter()
                        .copied()
                        .filter(|i| pool.all[*i].cheap_failed == 0 && !failed(i)),
                );
                let stability_failed = most_cargo(
                    &mut group
                        .iter()
                        .copied()
                        .filter(|i| pool.all[*i].cheap_failed == 0 && failed(i)),
                );
                let od = |i: usize| opt_or_zero(pool.all[i].result.tank_outer_diameter_m);
                let closest =
                    group
                        .iter()
                        .copied()
                        .fold(None, |best: Option<usize>, i| match best {
                            Some(b)
                                if pool.all[i].cheap_failed > pool.all[b].cheap_failed
                                    || (pool.all[i].cheap_failed == pool.all[b].cheap_failed
                                        && od(i) >= od(b) - 1e-9) =>
                            {
                                Some(b)
                            }
                            _ => Some(i),
                        });
                let (i, outcome) = match (chosen, passing, stability_failed, closest) {
                    (Some(i), ..) if pool.best == Some(i) => (i, TriedOutcome::Chosen),
                    (Some(i), ..) => (
                        i,
                        TriedOutcome::Fails(failing_checks(&pool.all[i], o, true)),
                    ),
                    (None, Some(i), ..) => (i, TriedOutcome::LessCargo),
                    (None, None, Some(i), _) => (
                        i,
                        TriedOutcome::Fails(failing_checks(&pool.all[i], o, true)),
                    ),
                    (None, None, None, Some(i)) => (
                        i,
                        TriedOutcome::Fails(failing_checks(&pool.all[i], o, false)),
                    ),
                    (None, None, None, None) => continue,
                };
                let c = &pool.all[i];
                let r = &c.result;
                out.push(TriedArrangement {
                    depth_m: c.depth,
                    n_wide: nw,
                    n_long: c.inputs.n_tanks_long,
                    tank_od_m: opt_or_zero(r.tank_outer_diameter_m),
                    side_clearance_m: opt_or_zero(r.side_clearance_m),
                    wing_tank_width_m: wing_tank_width(r),
                    cargo_volume_m3: r.cargo_volume_m3,
                    cargo_t: c.tonnage,
                    variants: group.len() as i64,
                    outcome,
                });
            }
        }
    }
    out
}

/// The wing ballast tank width of a gas result (m): the side space less the wing tank gap, not
/// less than zero (issue #283). 0 with no gas basis.
pub fn wing_tank_width(r: &BargeCapacityResult) -> f64 {
    r.gas_basis
        .as_ref()
        .and_then(|g| g.inputs.wing_tank_width)
        .unwrap_or(0.0)
}

/// The arrangement that meets every check of the search, with the most cargo per journey at draught
/// cap `t_max` over depth x arrangement.
///
/// A candidate must pass every error-severity check (see "THE CHECKS THE SEARCH USES"), stability
/// included, to win. If none passes, the closest candidate to passing comes back with
/// `feasible == false` and its failing checks in `failed_checks`; it is not an answer, so check
/// `feasible` before using the payload.
pub fn fit(
    loa: f64,
    beam: f64,
    cargo: &str,
    t_max: f64,
    o: &FitOptions,
) -> Result<ArrangementFit, PyError> {
    fit_with(loa, beam, cargo, t_max, o, &SerialRunner)
}

/// Issue #935: the numbers of tanks along the search tries: the profile's number first, then the others of 2, 3 and
/// 4, when the search screens by IGC 2.7 damage and the number is not pinned (`choose_tanks_along`); otherwise just
/// the profile's number.
fn tanks_along_to_try(o: &FitOptions, runner: &dyn DamageRunner) -> Vec<i64> {
    let mut out = vec![o.tanks_in_line];
    if o.choose_tanks_along
        && igc_damage_set(o).is_some()
        && o.is_code.is_some()
        && runner.enabled()
    {
        out.extend(
            TANKS_ALONG_CHOICES
                .iter()
                .copied()
                .filter(|&n| n != o.tanks_in_line),
        );
    }
    out
}

/// [`fit`], with the damage of a candidate worked out by `runner` (issue #935): the caller's threads, progress
/// and cancel. A cancelled run is `Err` with kind `Cancelled` and the message [`gas_damage::DAMAGE_CANCELLED`].
pub fn fit_with(
    loa: f64,
    beam: f64,
    cargo: &str,
    t_max: f64,
    o: &FitOptions,
    runner: &dyn DamageRunner,
) -> Result<ArrangementFit, PyError> {
    if !RULESETS.contains(&o.ruleset.as_str()) {
        return Err(value_error(
            "ruleset must be one of ('IGC_2G',)".to_string(),
        ));
    }
    let mut s = Search {
        loa,
        beam,
        cargo,
        o,
        tried: 0,
        refused: Vec::new(),
        runner,
        damage_runs: 0,
        cap_hit: false,
        along: tanks_along_to_try(o, runner),
        cheap_only: false,
    };
    let pool = search(&mut s, t_max, false)?;
    let pick = pool.best.or_else(|| closest_to_passing(&pool.all, o));
    let tried = tried_arrangements(&pool, pick, &s.refused, o, &s.along);
    // Issue #935: damage runs ran out before any candidate passed: the closest is shown, and it is not an answer.
    let cap_blocked = s.cap_hit && pool.best.is_none();
    let pool_planned_runs = pool.planned_runs;
    // Issue #935: the search's pick without damage, when the screen changed it.
    let damage_screen = if igc_damage_set(o).is_none() || o.is_code.is_none() {
        DamageScreen::NotApplicable
    } else if !runner.enabled() {
        DamageScreen::Skipped
    } else {
        let unscreened = pool
            .unscreened_best
            .filter(|&u| pick != Some(u))
            .map(|u| &pool.all[u])
            .map(|c| UnscreenedPick {
                depth_m: c.depth,
                n_wide: opt_count(c.result.n_tanks_wide),
                n_long: c.inputs.n_tanks_long,
                tank_od_m: opt_or_zero(c.result.tank_outer_diameter_m),
                cargo_volume_m3: c.result.cargo_volume_m3,
            });
        DamageScreen::Screened {
            unscreened,
            cap_hit: s.cap_hit,
        }
    };
    let mut all = pool.all;
    // Nothing passes: report the closest failing candidate, flagged infeasible.
    let best = match pick {
        Some(i) => all.swap_remove(i),
        None => {
            // Nothing fitted: report the plain single-tank attempt, which carries nothing.
            let d = *o.depths.first().ok_or_else(|| PyError {
                kind: "IndexError",
                message: "list index out of range".to_string(),
            })?;
            // The first number of tanks across the search tried (issue #675: it may be fixed).
            let nw = o.tanks_wide.first().copied().unwrap_or(1);
            let inp = s.base_inputs(d, nw, o.tanks_in_line);
            let r = capacity(&inp)?;
            let t_cap = match o.tank_top_above_deck_m {
                Some(_) => t_max.min(d - o.required_freeboard_m),
                None => t_max,
            };
            let j = derate_candidate(&r, loa, beam, d, t_cap, o.water_density_t_per_m3)?;
            Candidate {
                cheap_failed: 1,
                tonnage: 0.0,
                depth: d,
                search_depth: d,
                result: r,
                journey: j,
                inputs: inp,
                damage: None,
            }
        }
    };

    let checks = search_checks(&best.result, &best.journey, &best.inputs, best.depth, o);
    let mut failed: Vec<String> = checks
        .iter()
        .filter(|c| c.is_error() && !c.ok)
        .map(|c| format!("{}: {}", c.name, c.message))
        .collect();
    // Issue #992: nothing was built because every tank stood higher than its diameter allows (or
    // none floated): the plain attempt below is not an answer.
    let nothing_built = pick.is_none() && o.tank_top_above_deck_m.is_some();
    if nothing_built {
        failed.push("Arrangement: no tank was built at the standing height asked for; see the arrangements tried".to_string());
    }
    let feasible = if nothing_built {
        false
    } else if o.rules.is_empty() {
        all_ok(&checks)
    } else {
        // The failing rows no check of the list reports (a tank Rule, a Rule of a set loaded at
        // run time) join the failed checks, as "rule id: title (object)".
        let trace = candidate_trace(&full_measured(&best, o), o);
        for row in trace.rows.iter().filter(|r| r.outcome == Outcome::Fail) {
            let mirrored = row
                .measure
                .as_ref()
                .is_some_and(|m| m.id == "check.ok" || m.id.starts_with("separation."))
                || row.object_type.as_deref() == Some(ObjectType::LoadingCondition.key());
            let line = format!(
                "{}: {} ({})",
                row.rule.clone().unwrap_or_default(),
                row.title,
                row.object.clone().unwrap_or_default()
            );
            if !mirrored && !failed.contains(&line) {
                failed.push(line);
            }
        }
        trace.rows.iter().all(|r| r.outcome != Outcome::Fail)
    };
    let feasible = feasible && !cap_blocked;
    if cap_blocked {
        failed.push(
            "damage: the search stopped at the damage-run limit before a candidate passed"
                .to_string(),
        );
    }
    let Candidate {
        depth: d,
        result: r,
        journey: j,
        inputs: inp,
        ..
    } = best;
    let n_total = opt_count(r.n_tanks_total);
    let n_wide = opt_count(r.n_tanks_wide);
    let n_tanks_long = if n_wide != 0 {
        floor_div(n_total, n_wide)?
    } else {
        0
    };
    let containment = opt_or_zero(r.containment_inner_volume_m3);
    let tanks_for_division = n_total.max(1) as f64;
    Ok(ArrangementFit {
        loa_m: loa,
        beam_m: beam,
        depth_m: d,
        cargo: cargo.to_string(),
        ruleset: o.ruleset.clone(),
        fill_ratio: o.fill_ratio,
        self_propelled: o.self_propelled,
        t_max_m: t_max,
        payload_t: j.achievable_tonnage_t,
        volume_capacity_t: r.cargo_capacity_t,
        binding: if r.cargo_capacity_t <= 0.0 {
            "no cargo area".to_string()
        } else {
            j.binding_limit.clone()
        },
        draft_at_payload_m: j.draft_at_achievable_m,
        cargo_zone_m: opt_or_zero(r.cargo_zone_length_m),
        stern_void_m: opt_or_zero(r.stern_void_length_m),
        bow_void_m: opt_or_zero(r.bow_void_length_m),
        tank_od_m: opt_or_zero(r.tank_outer_diameter_m),
        n_tanks: n_total,
        n_tanks_wide: n_wide,
        n_tanks_long,
        lightship_t: r.lightship_t,
        air_draft_light_m: match r.air_draft_light_m {
            // Python `r.air_draft_light_m or r.air_draft_m`. Mutation survivor (#146), the guard
            // `x != 0.0` always true: no input found that tells it apart, so it is recorded as
            // equivalent. It differs only when the light air draft is exactly 0.0 and the loaded
            // air draft is not: the tank top level with the light waterline to the last bit, with
            // the loaded waterline above it. When the draught is clamped at the depth (a lightship
            // over the hull's displacement) both air drafts are the tank top less the depth, so
            // the two sides agree. Hill-climbing the light air draft down over length, beam,
            // depth, tank cap and draught cap, and 240 000 random inputs, found nothing under
            // 0.0146 m, and that only for a hull 15 mm deep. The guard stays as the port of
            // Python's `or`.
            Some(x) if x != 0.0 => x,
            _ => r.air_draft_m,
        },
        side_clearance_m: opt_or_zero(r.side_clearance_m),
        tank_inner_diameter_m: opt_or_zero(r.tank_inner_diameter_m),
        containment_inner_volume_m3: containment,
        per_tank_volume_m3: containment / tanks_for_division,
        cargo_volume_m3: r.cargo_volume_m3,
        density_t_m3: r.cargo_density_t_per_m3,
        failed_checks: failed,
        feasible,
        candidates_tried: s.tried,
        tried,
        engine_inputs: inp,
        damage_runs: s.damage_runs,
        damage_screen,
        damage_planned_runs: pool_planned_runs,
        tanks_along_tried: s.along.clone(),
        engine_result: r,
    })
}

/// Whether some arrangement of this hull meets every check of the search (the first one found
/// ends the search).
fn has_passing_arrangement(
    loa: f64,
    beam: f64,
    cargo: &str,
    t_max: f64,
    o: &FitOptions,
    runner: &dyn DamageRunner,
) -> Result<bool, PyError> {
    if !RULESETS.contains(&o.ruleset.as_str()) {
        return Err(value_error(
            "ruleset must be one of ('IGC_2G',)".to_string(),
        ));
    }
    let mut s = Search {
        loa,
        beam,
        cargo,
        o,
        tried: 0,
        refused: Vec::new(),
        runner,
        damage_runs: 0,
        cap_hit: false,
        along: vec![o.tanks_in_line],
        cheap_only: false,
    };
    Ok(search(&mut s, t_max, true)?.best.is_some())
}

/// Issue #992: the depth the search would settle on with a standing height, found without the costly
/// stability stage: the depth of the candidate with the most cargo among those that pass the cheap
/// checks. The stability stage can turn that candidate down and the full search then picks another,
/// so a caller checks the depth `fit` gives against this one. `None` when no candidate passes the
/// cheap checks.
pub fn probe_depth(
    loa: f64,
    beam: f64,
    cargo: &str,
    t_max: f64,
    o: &FitOptions,
) -> Result<Option<f64>, PyError> {
    if !RULESETS.contains(&o.ruleset.as_str()) {
        return Err(value_error(
            "ruleset must be one of ('IGC_2G',)".to_string(),
        ));
    }
    let mut s = Search {
        loa,
        beam,
        cargo,
        o,
        tried: 0,
        refused: Vec::new(),
        runner: &SerialRunner,
        damage_runs: 0,
        cap_hit: false,
        along: vec![o.tanks_in_line],
        cheap_only: true,
    };
    let pool = search(&mut s, t_max, false)?;
    Ok(pool.best.map(|i| pool.all[i].depth))
}

/// Step of the upward scan of `smallest_viable_loa` (m).
const VIABLE_LOA_SCAN_STEP_M: f64 = 2.0;

/// Shortest LOA (to 0.1 m) at which some arrangement of the hull meets every check of the search
/// (the same list as `fit`, stability included). Infinity when no LOA from `lo` to `hi` has one.
///
/// Whether a hull is viable is not monotone in its length: a long hull can fail length over
/// diameter, and stability moves with length. So the search scans upward from `lo` in steps of
/// [`VIABLE_LOA_SCAN_STEP_M`] to the first viable length, then halves the last step to 0.1 m. The
/// length it returns always has a passing arrangement. Python defaults `depths` to the first
/// default depth only for this function: start from [`FitOptions::for_smallest_viable_loa`].
pub fn smallest_viable_loa(
    beam: f64,
    cargo: &str,
    t_max: f64,
    lo: f64,
    hi: f64,
    o: &FitOptions,
) -> Result<f64, PyError> {
    smallest_viable_loa_with(beam, cargo, t_max, lo, hi, o, &SerialRunner)
}

/// [`smallest_viable_loa`] with the damage of each candidate worked out by `runner` (issue #935): the caller's
/// threads, progress and cancel. A cancelled run is `Err` with kind `Cancelled`.
pub fn smallest_viable_loa_with(
    beam: f64,
    cargo: &str,
    t_max: f64,
    lo: f64,
    hi: f64,
    o: &FitOptions,
    runner: &dyn DamageRunner,
) -> Result<f64, PyError> {
    let mut below = lo;
    let mut at = lo;
    loop {
        if has_passing_arrangement(at, beam, cargo, t_max, o, runner)? {
            break;
        }
        if at >= hi {
            return Ok(f64::INFINITY);
        }
        below = at;
        at = (at + VIABLE_LOA_SCAN_STEP_M).min(hi);
    }
    let (mut lo, mut hi) = (below, at);
    while hi - lo > 0.1 {
        let mid = 0.5 * (lo + hi);
        if has_passing_arrangement(mid, beam, cargo, t_max, o, runner)? {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    // The answer is on the 0.1 m grid, and a length is only returned once it has been tested
    // (issue #422): rounding to nearest could go below `hi`, where nothing was tested.
    let nearest = round_to(hi, 1);
    if nearest == hi || has_passing_arrangement(nearest, beam, cargo, t_max, o, runner)? {
        return Ok(nearest);
    }
    let up = libm::ceil(hi * 10.0) / 10.0;
    if has_passing_arrangement(up, beam, cargo, t_max, o, runner)? {
        return Ok(up);
    }
    Ok(hi)
}

#[cfg(test)]
mod pick_tests {
    use super::{PickKey, better_key};

    fn key(tonnage: f64, tanks: i64, along: i64, od_m: f64, index: usize) -> PickKey {
        PickKey {
            tonnage,
            tanks,
            along,
            od_m,
            index,
        }
    }

    /// Issue #935, addendum D3. Label: rule (the owner's aim is cargo; ties go to the simpler hull).
    #[test]
    fn more_cargo_wins_and_equal_cargo_goes_to_the_fewest_tanks_then_the_fewest_along() {
        // More cargo wins, whatever else.
        assert!(better_key(
            &key(101.0, 8, 4, 9.0, 5),
            &key(100.0, 4, 2, 12.0, 1)
        ));
        assert!(!better_key(
            &key(100.0, 4, 2, 12.0, 1),
            &key(101.0, 8, 4, 9.0, 5)
        ));
        // Equal cargo (to 1e-6 t), another number along: the fewest tanks.
        assert!(better_key(
            &key(100.0, 4, 2, 12.0, 1),
            &key(100.0, 6, 3, 10.0, 2)
        ));
        assert!(!better_key(
            &key(100.0, 6, 3, 10.0, 2),
            &key(100.0, 4, 2, 12.0, 1)
        ));
        // The same number of tanks: the fewest along.
        assert!(better_key(
            &key(100.0, 6, 2, 10.0, 7),
            &key(100.0, 6, 3, 10.0, 2)
        ));
        // Within one number along the first built stays: the diameter does not decide.
        assert!(better_key(
            &key(100.0, 6, 3, 10.0, 1),
            &key(100.0, 6, 3, 11.0, 2)
        ));
        assert!(!better_key(
            &key(100.0, 6, 3, 11.0, 3),
            &key(100.0, 6, 3, 10.0, 2)
        ));
    }
}
