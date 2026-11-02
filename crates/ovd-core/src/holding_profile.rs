//! The operation profile of a Type C tank's holding time (spec 0016, issue #869): the legs the
//! tank goes through (D9: a laden passage or idle only, the tank closed and starting full), the
//! default design basis (D2, D3), and the gas-use credit of a leg (D7).
//!
//! This is input only. The holding-time model that steps the tank through the legs is a later
//! ticket (#870); nothing here reads a result or moves one.

use alloc::vec::Vec;

/// A leg of laden passage.
pub const LADEN_PASSAGE: &str = "laden_passage";
/// A leg with the vessel idle: no propulsion.
pub const IDLE: &str = "idle";
/// The leg states (D9). Loading, discharge and ballast are out of scope: a closed tank that starts
/// full has no meaning for them.
pub const LEG_STATES: &[&str] = &[LADEN_PASSAGE, IDLE];

/// The default design basis (D2, D3): 30 days, the first 20 of them laden passage, the last 10
/// idle. The idle block is last because it is the worst case: the tank is warmest when the gas use
/// stops. IGC 7.5 sets no figure, so this is a design basis, never a compliance figure.
pub const DEFAULT_PASSAGE_DAYS: f64 = 20.0;
pub const DEFAULT_IDLE_DAYS: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegState {
    LadenPassage,
    Idle,
}

impl LegState {
    pub fn key(self) -> &'static str {
        match self {
            LegState::LadenPassage => LADEN_PASSAGE,
            LegState::Idle => IDLE,
        }
    }

    pub fn from_key(key: &str) -> Option<LegState> {
        match key {
            LADEN_PASSAGE => Some(LegState::LadenPassage),
            IDLE => Some(LegState::Idle),
            _ => None,
        }
    }
}

/// One leg of the profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leg {
    pub state: LegState,
    /// The length of the leg (days).
    pub days: f64,
    /// A laden passage leg: the typed passage speed (knots), the speed at which the powering model
    /// gives the calm-water gas use (D7). `None`: not typed, so no passage credit.
    pub passage_speed_kn: Option<f64>,
    /// An idle leg: the typed gas use (kg/h) the tank gives up. Zero unless typed (D7).
    pub gas_use_kg_h: f64,
}

/// The default design basis as legs: 20 days of laden passage, then 10 days idle. No passage speed
/// is typed, so the default credits no passage gas use until one is.
pub fn default_design_basis() -> Vec<Leg> {
    alloc::vec![
        Leg {
            state: LegState::LadenPassage,
            days: DEFAULT_PASSAGE_DAYS,
            passage_speed_kn: None,
            gas_use_kg_h: 0.0,
        },
        Leg {
            state: LegState::Idle,
            days: DEFAULT_IDLE_DAYS,
            passage_speed_kn: None,
            gas_use_kg_h: 0.0,
        },
    ]
}

/// The length of the profile (days).
pub fn total_days(legs: &[Leg]) -> f64 {
    legs.iter().map(|l| l.days).sum()
}

/// Whether the Option has a gas consumer that takes the boil-off (D7): the Power Source burns gas.
/// A barge with no engines of its own has none; the case of an articulated tug and barge waits for
/// issue #793, so it counts as none here.
pub fn has_gas_consumer(power_source: &str) -> bool {
    power_source == "gas_fuelled"
}

/// The gas a laden passage credits (kg/h), D7: the calm-water gas use the powering model gives at
/// the typed passage speed and the lightest draught in the leg. **Zero when the Option has no gas
/// consumer that takes the boil-off**, or when no use could be worked out.
pub fn passage_credit_kg_h(has_consumer: bool, calm_water_use_kg_h: Option<f64>) -> f64 {
    match calm_water_use_kg_h {
        Some(u) if has_consumer && u.is_finite() && u > 0.0 => u,
        _ => 0.0,
    }
}

/// The gas a leg credits for the design basis row (kg/h): a passage leg its passage credit (zero
/// with no typed passage speed), an idle leg the typed use (zero unless typed).
pub fn leg_credit_kg_h(leg: &Leg, has_consumer: bool, calm_water_use_kg_h: Option<f64>) -> f64 {
    match leg.state {
        LegState::LadenPassage if leg.passage_speed_kn.is_some() => {
            passage_credit_kg_h(has_consumer, calm_water_use_kg_h)
        }
        LegState::LadenPassage => 0.0,
        LegState::Idle => leg.gas_use_kg_h,
    }
}
