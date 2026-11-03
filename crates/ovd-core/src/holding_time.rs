//! The pressure holding time of a Type C methane tank (spec 0016 section 4, issue #870).
//!
//! A closed tank of volume V holds a mass m of methane in one saturation state (temperature T,
//! quality x): the homogeneous model of D4. It ignores thermal stratification, which can make the
//! real pressure rise faster. Over a step of length dt, with a heat leak Q and a vapour withdrawal
//! (the gas use credited for the leg, D7):
//!
//! - U(t + dt) = U(t) + Q dt - mdot h_out dt
//! - m(t + dt) = m(t) - mdot dt
//! - the new state is the (T, x) whose specific internal energy is U / m and whose specific volume
//!   is V / m, solved together on the saturation table ([`state_from_u_v`]).
//!
//! `h_out` is the saturated vapour enthalpy when vapour is burned, or the saturated liquid enthalpy
//! when liquid goes to a forced vaporiser. Both are read at the state at the start of the step.
//!
//! The withdrawal is capped at the vapour available (x m) and is floored: it stops when the pressure
//! is at the start pressure, and a step that would take the pressure below it takes only as much as
//! holds it there. The tank is never drawn below the start pressure.
//!
//! The heat leak is recomputed at each step from the cargo temperature (D13): the caller's
//! [`HeatLeak::Conduction`] takes `delta_k` = the air ambient of the governing rule less the cargo
//! temperature, over the whole tank surface (maintainer, 2026-11-02), for [`tank_heat_leak_kw`].
//! [`HeatLeak::Constant`] is the test mode of spec 0016 section 7.
//!
//! The loading limit is LL = FL x rhoR / rhoL (IGC Code 15.1.3.1 and 15.5.1, always, D14): rhoR is
//! the liquid density at the saturation temperature at the relief setting, rhoL at the loading
//! temperature (the saturation temperature at the start pressure). A typed fill above LL is refused.
//!
//! The holding time is the time from the start state to the relief setting. The crossing inside the
//! last step is found by bisection on the step, so it is exact for the model, not a straight line.
//! Pressures are gauge in and out (the rules); the table is absolute.
//!
//! Properties are the NIST methane saturation table of [`crate::methane_saturation`]. Methane
//! stands in for the real LNG composition. OVD is for concept and preliminary design only, never
//! for approval (ADR 0011).

use alloc::vec::Vec;

use crate::gas_fuel::tank_heat_leak_kw;
use crate::methane_saturation::{
    OutOfTable, Pressure, State, liquid_density_kg_m3, liquid_enthalpy_kj_per_kg,
    liquid_internal_energy_kj_per_kg, saturation_pressure_mpa, saturation_temperature_c,
    state_from_u_v, vapour_density_kg_m3, vapour_enthalpy_kj_per_kg,
    vapour_internal_energy_kj_per_kg,
};
use crate::tanks::DEFAULT_FILLING_LIMIT;

/// The rule whose upper ambient design temperature sets the heat leak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// IGC Code 7.2 (the design basis and IGF): air 45 C, sea 32 C.
    Igc,
    /// ADN 9.3.1.24.2: air 30 C, water 20 C.
    Adn,
}

impl Rule {
    /// The air ambient (C). It applies over the whole tank surface (D13): conservative and simple.
    pub fn ambient_air_c(self) -> f64 {
        match self {
            Rule::Igc => 45.0,
            Rule::Adn => 30.0,
        }
    }
    /// The sea or water ambient (C). Not used: the air ambient governs the whole surface.
    pub fn ambient_water_c(self) -> f64 {
        match self {
            Rule::Igc => 32.0,
            Rule::Adn => 20.0,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Rule::Igc => "igc_7_2",
            Rule::Adn => "adn_9_3_1_24_2",
        }
    }
}

/// How the heat leak into the tank is found at a step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatLeak {
    /// A fixed heat leak (kW): the constant-heat-leak test mode.
    Constant { kw: f64 },
    /// Conduction through the insulation, [`tank_heat_leak_kw`], for `delta_k` = the ambient less
    /// the cargo temperature at the start of the step.
    Conduction {
        r_steel_m: f64,
        r_outer_m: f64,
        cylinder_m: f64,
        conductivity_w_per_mk: f64,
        ambient_c: f64,
    },
}

impl HeatLeak {
    /// Conduction at the air ambient of `rule`.
    pub fn conduction(
        rule: Rule,
        r_steel_m: f64,
        r_outer_m: f64,
        cylinder_m: f64,
        conductivity_w_per_mk: f64,
    ) -> Self {
        HeatLeak::Conduction {
            r_steel_m,
            r_outer_m,
            cylinder_m,
            conductivity_w_per_mk,
            ambient_c: rule.ambient_air_c(),
        }
    }

    /// The heat leak (kW) with the cargo at `cargo_c`.
    pub fn kw(&self, cargo_c: f64) -> f64 {
        match *self {
            HeatLeak::Constant { kw } => kw,
            HeatLeak::Conduction {
                r_steel_m,
                r_outer_m,
                cylinder_m,
                conductivity_w_per_mk,
                ambient_c,
            } => tank_heat_leak_kw(
                r_steel_m,
                r_outer_m,
                cylinder_m,
                ambient_c - cargo_c,
                conductivity_w_per_mk,
            ),
        }
    }
}

/// What leaves the tank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Withdrawal {
    /// Vapour, at the saturated vapour enthalpy (the default).
    Vapour,
    /// Liquid to a forced vaporiser, at the saturated liquid enthalpy.
    Liquid,
}

/// One leg: its length and the gas use credited for it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegInput {
    pub days: f64,
    /// The withdrawal (kg/s). Zero for none.
    pub withdrawal_kg_s: f64,
}

/// A holding-time run.
#[derive(Debug, Clone, PartialEq)]
pub struct Inputs<'a> {
    /// The inside volume of the tank (m3).
    pub volume_m3: f64,
    /// The start pressure (MPa gauge) and the floor of the withdrawal.
    pub start_gauge_mpa: f64,
    /// The relief setting (MPa gauge).
    pub relief_gauge_mpa: f64,
    /// A typed fill (volume fraction). `None`: loaded to the loading limit.
    pub fill: Option<f64>,
    pub heat_leak: HeatLeak,
    pub withdrawal: Withdrawal,
    pub legs: &'a [LegInput],
    /// The step (s).
    pub step_s: f64,
}

/// A point of the pressure and fill series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub time_h: f64,
    pub pressure_gauge_mpa: f64,
    pub temperature_c: f64,
    /// The liquid volume over the tank volume.
    pub fill: f64,
}

/// The state at the end of a leg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegResult {
    pub end_time_h: f64,
    pub end_pressure_gauge_mpa: f64,
    pub end_fill: f64,
    /// The vapour or liquid taken out in the leg (kg).
    pub withdrawn_kg: f64,
    /// A step of the leg asked for more than the vapour available.
    pub capped_by_vapour: bool,
    /// A step of the leg was held at the start pressure.
    pub held_at_start_pressure: bool,
    /// The leg ran to its end. False for the leg in which the relief setting was reached, and for
    /// any leg after it.
    pub completed: bool,
}

/// The result of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub filling_limit: f64,
    pub loading_limit: f64,
    pub start_fill: f64,
    pub start_mass_kg: f64,
    pub start_temperature_c: f64,
    /// The time from the start to the relief setting (h); `None` if the profile ended first.
    pub holding_time_h: Option<f64>,
    pub points: Vec<Point>,
    pub legs: Vec<LegResult>,
}

/// Why a run was refused.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    /// A pressure or state outside the saturation table.
    OutOfTable,
    /// A typed fill above the loading limit.
    FillAboveLoadingLimit { fill: f64, limit: f64 },
    /// The tank went liquid-full before the relief setting.
    LiquidFull { time_h: f64 },
    /// An input that is not usable (volume, pressures, step, fill, legs).
    BadInput,
}

impl From<OutOfTable> for Error {
    fn from(_: OutOfTable) -> Self {
        Error::OutOfTable
    }
}

/// The loading limit of a tank loaded at `start_gauge_mpa` with the relief setting
/// `relief_gauge_mpa` (IGC Code 15.1.3.1 and 15.5.1): LL = FL rhoR / rhoL, not above FL.
pub fn loading_limit(start_gauge_mpa: f64, relief_gauge_mpa: f64) -> Result<f64, OutOfTable> {
    let t_load =
        saturation_temperature_c(Pressure::from_gauge_mpa(start_gauge_mpa).absolute_mpa())?;
    let t_ref =
        saturation_temperature_c(Pressure::from_gauge_mpa(relief_gauge_mpa).absolute_mpa())?;
    let ll = DEFAULT_FILLING_LIMIT * liquid_density_kg_m3(t_ref)? / liquid_density_kg_m3(t_load)?;
    Ok(ll.min(DEFAULT_FILLING_LIMIT))
}

/// A closed tank at an instant.
#[derive(Clone, Copy)]
struct Tank {
    u_kj: f64,
    m_kg: f64,
    state: State,
}

fn pressure_abs(s: &State) -> Result<f64, OutOfTable> {
    saturation_pressure_mpa(s.temperature_c)
}

fn fill_of(s: &State, m_kg: f64, volume_m3: f64) -> Result<f64, OutOfTable> {
    Ok((1.0 - s.quality) * m_kg / liquid_density_kg_m3(s.temperature_c)? / volume_m3)
}

/// Advance `tank` by `tau` seconds at heat leak `q_kw` and `take_kg` taken at `h_out` (kJ/kg).
fn advance(
    tank: &Tank,
    volume_m3: f64,
    tau_s: f64,
    q_kw: f64,
    take_kg: f64,
    h_out: f64,
) -> Result<Tank, OutOfTable> {
    let u = tank.u_kj + q_kw * tau_s - take_kg * h_out;
    let m = tank.m_kg - take_kg;
    if m <= 0.0 {
        return Err(OutOfTable);
    }
    let state = state_from_u_v(u / m, volume_m3 / m)?;
    Ok(Tank {
        u_kj: u,
        m_kg: m,
        state,
    })
}

/// A step that leaves the dome from a state with a quality below this is a tank filling with liquid.
const LIQUID_FULL_QUALITY: f64 = 0.5;

fn is_pos(x: f64) -> bool {
    x.is_finite() && x > 0.0
}

/// Run the tank through the legs from the start state to the end of the profile or the relief
/// setting, whichever comes first.
pub fn run(inp: &Inputs) -> Result<Outcome, Error> {
    if !is_pos(inp.volume_m3)
        || !is_pos(inp.step_s)
        || !inp.start_gauge_mpa.is_finite()
        || inp.relief_gauge_mpa.is_nan()
        || inp.start_gauge_mpa < 0.0
        || inp.relief_gauge_mpa <= inp.start_gauge_mpa
        || inp.legs.is_empty()
        || inp
            .legs
            .iter()
            .any(|l| !is_pos(l.days) || !l.withdrawal_kg_s.is_finite() || l.withdrawal_kg_s < 0.0)
    {
        return Err(Error::BadInput);
    }
    let p_start = Pressure::from_gauge_mpa(inp.start_gauge_mpa).absolute_mpa();
    let p_relief = Pressure::from_gauge_mpa(inp.relief_gauge_mpa).absolute_mpa();
    let ll = loading_limit(inp.start_gauge_mpa, inp.relief_gauge_mpa)?;
    let fill = match inp.fill {
        None => ll,
        Some(f) if !(f.is_finite() && f > 0.0 && f < 1.0) => return Err(Error::BadInput),
        Some(f) if f > ll + 1.0e-12 => {
            return Err(Error::FillAboveLoadingLimit { fill: f, limit: ll });
        }
        Some(f) => f,
    };

    // The start state: saturated at the start pressure, the liquid and vapour at their densities.
    let t0 = saturation_temperature_c(p_start)?;
    let (rl, rv) = (liquid_density_kg_m3(t0)?, vapour_density_kg_m3(t0)?);
    let (ul, uv) = (
        liquid_internal_energy_kj_per_kg(t0)?,
        vapour_internal_energy_kj_per_kg(t0)?,
    );
    let v = inp.volume_m3;
    let m0 = v * (fill * rl + (1.0 - fill) * rv);
    let u0 = v * (fill * rl * ul + (1.0 - fill) * rv * uv);
    let mut tank = Tank {
        u_kj: u0,
        m_kg: m0,
        state: State {
            temperature_c: t0,
            quality: (1.0 - fill) * v * rv / m0,
        },
    };

    let mut points = alloc::vec![Point {
        time_h: 0.0,
        pressure_gauge_mpa: Pressure::from_absolute_mpa(p_start).gauge_mpa(),
        temperature_c: t0,
        fill,
    }];
    let mut legs: Vec<LegResult> = Vec::with_capacity(inp.legs.len());
    let mut time_s = 0.0_f64;
    let mut holding: Option<f64> = None;
    let gauge = |p_abs: f64| Pressure::from_absolute_mpa(p_abs).gauge_mpa();

    'legs: for leg in inp.legs {
        let leg_end_s = time_s + leg.days * 86_400.0;
        let mut withdrawn = 0.0;
        let mut capped = false;
        let mut held = false;
        while leg_end_s - time_s > 1.0e-6 {
            let dt = inp.step_s.min(leg_end_s - time_s);
            let p_now = pressure_abs(&tank.state)?;
            let q = inp.heat_leak.kw(tank.state.temperature_c);
            let t_now = tank.state.temperature_c;
            let h_out = match inp.withdrawal {
                Withdrawal::Vapour => vapour_enthalpy_kj_per_kg(t_now)?,
                Withdrawal::Liquid => liquid_enthalpy_kj_per_kg(t_now)?,
            };
            // The withdrawal asked for, capped at the vapour available, and none at or below the
            // start pressure.
            let asked = leg.withdrawal_kg_s * dt;
            let available = match inp.withdrawal {
                Withdrawal::Vapour => tank.state.quality * tank.m_kg,
                Withdrawal::Liquid => (1.0 - tank.state.quality) * tank.m_kg,
            };
            let above_floor = p_now > p_start + 1.0e-9;
            let mut take = if asked > 0.0 && above_floor {
                asked.min(available)
            } else {
                0.0
            };
            if asked > 0.0 && above_floor && take < asked {
                capped = true;
            }
            let mut next = match advance(&tank, v, dt, q, take, h_out) {
                Ok(n) => n,
                // The state left the dome: the liquid filled the tank (the vapour was the smaller
                // part of the mass at the start of the step), or the table ended.
                Err(OutOfTable) if tank.state.quality < LIQUID_FULL_QUALITY => {
                    return Err(Error::LiquidFull {
                        time_h: (time_s + dt) / 3600.0,
                    });
                }
                Err(e) => return Err(e.into()),
            };
            // The floor: a step that would end below the start pressure takes only what holds it.
            if take > 0.0 && pressure_abs(&next.state)? < p_start {
                let (mut lo, mut hi) = (0.0, take);
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    let n = advance(&tank, v, dt, q, mid, h_out)?;
                    if pressure_abs(&n.state)? < p_start {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                take = lo;
                next = advance(&tank, v, dt, q, take, h_out)?;
                held = true;
            }
            if pressure_abs(&next.state)? >= p_relief {
                // The relief setting is reached inside this step: bisect on the time in it, with
                // the withdrawal of the step's rate.
                let rate = take / dt;
                let (mut lo, mut hi) = (0.0, dt);
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    let n = advance(&tank, v, mid, q, rate * mid, h_out)?;
                    if pressure_abs(&n.state)? >= p_relief {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                let tau = 0.5 * (lo + hi);
                let at = advance(&tank, v, tau, q, rate * tau, h_out)?;
                withdrawn += rate * tau;
                time_s += tau;
                points.push(Point {
                    time_h: time_s / 3600.0,
                    pressure_gauge_mpa: inp.relief_gauge_mpa,
                    temperature_c: at.state.temperature_c,
                    fill: fill_of(&at.state, at.m_kg, v)?,
                });
                legs.push(LegResult {
                    end_time_h: time_s / 3600.0,
                    end_pressure_gauge_mpa: inp.relief_gauge_mpa,
                    end_fill: fill_of(&at.state, at.m_kg, v)?,
                    withdrawn_kg: withdrawn,
                    capped_by_vapour: capped,
                    held_at_start_pressure: held,
                    completed: false,
                });
                holding = Some(time_s / 3600.0);
                break 'legs;
            }
            if next.state.quality <= 1.0e-9 {
                return Err(Error::LiquidFull {
                    time_h: (time_s + dt) / 3600.0,
                });
            }
            withdrawn += take;
            tank = next;
            time_s += dt;
            points.push(Point {
                time_h: time_s / 3600.0,
                pressure_gauge_mpa: gauge(pressure_abs(&tank.state)?),
                temperature_c: tank.state.temperature_c,
                fill: fill_of(&tank.state, tank.m_kg, v)?,
            });
        }
        legs.push(LegResult {
            end_time_h: time_s / 3600.0,
            end_pressure_gauge_mpa: gauge(pressure_abs(&tank.state)?),
            end_fill: fill_of(&tank.state, tank.m_kg, v)?,
            withdrawn_kg: withdrawn,
            capped_by_vapour: capped,
            held_at_start_pressure: held,
            completed: true,
        });
    }
    Ok(Outcome {
        filling_limit: DEFAULT_FILLING_LIMIT,
        loading_limit: ll,
        start_fill: fill,
        start_mass_kg: m0,
        start_temperature_c: t0,
        holding_time_h: holding,
        points,
        legs,
    })
}
