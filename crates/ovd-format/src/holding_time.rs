//! The `holding_time` block of a gas result (spec 0016, issue #870): the Type C tank stepped through
//! the operation profile by `ovd_core::holding_time`. A new output outside the results hash
//! (`UNHASHED_KEYS`): the model is not wired into the solve yet (D11, D12: issues #871 and #875), so
//! the tank wall, the design pressure and every hashed result are unchanged.
//!
//! One Rule row is evaluated, the design basis (IGC Code 7.2 ambient, the profile of the Option). The
//! ADN and IGF rows wait for the Rule rows of the solve ticket.

use crate::vdesign::DesignOption;
use libm::round;
use ovd_core::holding_profile::{Leg, has_gas_consumer, leg_credit_kg_h, total_days};
use ovd_core::holding_time::{Error, HeatLeak, Inputs, LegInput, Rule, Withdrawal, run};
use serde_json::{Value, json};

const BASIS: &str = "Homogeneous closed-tank model of methane (one saturation state, no thermal stratification: the real pressure can rise faster), stepped every 10 minutes (second order) from the loading limit at the start pressure (IGC Code 15.1.3.1). The heat leak is recomputed each step at the IGC Code 7.2 air ambient (45 C) over the whole tank surface. The gas use credited on a passage leg is the least gas use of the powering model across its service conditions, at its service speed. The leg records its passage speed (the typed one, else the design service speed, as the profile input resolves it), but the credit is not yet worked out at that speed and at the lightest draught of the leg: that link to the powering model belongs to issues #871 and #875. The series is kept every 6 h plus the end of each leg and the last point. Domes, saddles and pipes are not in the heat leak. Methane stands in for the LNG composition.";

/// The step of the stepping (s).
const STEP_S: f64 = 600.0;

/// The series keeps a point every this many hours (h).
const SERIES_EVERY_H: f64 = 6.0;

fn input_row(option: &DesignOption, name: &str) -> Value {
    match option.input(name) {
        Some(i) => json!({ "name": name, "value": i.value, "source": i.source.as_str() }),
        None => {
            json!({ "name": name, "value": Value::Null, "source": "not an input of this Option" })
        }
    }
}

fn not_evaluated(reason: String) -> Value {
    json!({ "evaluated": false, "reason": reason })
}

/// Add the `holding_time` block to the result of a Type C LNG Option that has a tank.
pub(crate) fn add_holding_time(option: &DesignOption, v: &mut Value) {
    let Some(profile) = option.input("holding_profile") else {
        return;
    };
    if option.input("cargo").and_then(|i| i.value.as_str()) != Some("LNG") {
        return;
    }
    let tank = v["drawing"]["tank"].clone();
    let n_tanks = v["geometry"]["n_tanks_total"].as_f64();
    let dims = (
        tank["tank_steel_outer_diameter_m"].as_f64(),
        tank["tank_outer_diameter_m"].as_f64(),
        tank["tank_inner_diameter_m"].as_f64(),
        tank["cylinder_length_m"].as_f64(),
        tank["design_pressure_mpa"].as_f64(),
        n_tanks,
        // The conductivity of the tank design in force (issue #919), not a constant.
        tank["insulation_conductivity_w_per_mk"].as_f64(),
    );
    let (Some(ds), Some(d_o), Some(d_i), Some(len), Some(relief), Some(n), Some(k)) = dims else {
        return;
    };
    if d_o <= ds || n < 1.0 {
        v["holding_time"] =
            not_evaluated("the tank has no insulation or no tank dimensions".into());
        return;
    }
    let legs: Vec<Leg> = match crate::holding_profile::legs_of(&profile.value) {
        Ok(l) => l,
        Err(_) => return,
    };
    // The inside volume of one tank: the cylinder and two heads (the dome is not in it).
    let r = d_i / 2.0;
    let pi = core::f64::consts::PI;
    let head_factor = if tank["head_type"].as_str() == Some("hemisphere") {
        4.0 / 3.0
    } else {
        2.0 / 3.0
    };
    let volume = pi * r * r * len + head_factor * pi * r * r * r;

    // The gas use credited: the least gas use of the powering model's service conditions.
    let envs = v["powering"]["environments"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let use_kg_h = envs
        .iter()
        .filter_map(|e| e["fuel_gas_kg_per_h"].as_f64())
        .reduce(f64::min);
    let design_speed = envs
        .first()
        .and_then(|e| e["service_speed_kn"].as_f64())
        .unwrap_or(0.0);
    let consumer = has_gas_consumer(
        option
            .input("power_source")
            .and_then(|i| i.value.as_str())
            .unwrap_or(""),
    );
    let credits: Vec<f64> = legs
        .iter()
        .map(|l| leg_credit_kg_h(l, design_speed, consumer, use_kg_h))
        .collect();
    let leg_inputs: Vec<LegInput> = legs
        .iter()
        .zip(&credits)
        .map(|(l, c)| LegInput {
            days: l.days,
            withdrawal_kg_s: c / n / 3600.0,
        })
        .collect();
    let start = option
        .input("holding_start_pressure_mpa")
        .and_then(|i| i.value.as_f64())
        .unwrap_or(0.0);
    let typed_fill = option
        .input("holding_fill_ratio")
        .and_then(|i| i.value.as_f64());
    let rule = Rule::Igc;
    let result = run(&Inputs {
        volume_m3: volume,
        start_gauge_mpa: start,
        relief_gauge_mpa: relief,
        fill: typed_fill,
        heat_leak: HeatLeak::conduction(rule, ds / 2.0, d_o / 2.0, len, k),
        withdrawal: Withdrawal::Vapour,
        legs: &leg_inputs,
        step_s: STEP_S,
    });
    let out = match result {
        Ok(o) => o,
        Err(Error::FillAboveLoadingLimit { fill, limit }) => {
            v["holding_time"] = not_evaluated(format!(
                "the typed fill {fill:.4} is above the loading limit {limit:.4} (IGC Code 15.1.3.1, 15.5.1)"
            ));
            return;
        }
        Err(Error::LiquidFull { time_h }) => {
            v["holding_time"] = not_evaluated(format!(
                "the tank is liquid-full {time_h:.1} h into the profile, before the relief setting"
            ));
            return;
        }
        Err(Error::OutOfTable) => {
            v["holding_time"] = not_evaluated(
                "a pressure or state outside the methane saturation table (-170 C to -100 C)"
                    .into(),
            );
            return;
        }
        Err(Error::BadInput) => {
            v["holding_time"] =
                not_evaluated("an input of the holding-time model is not usable".into());
            return;
        }
    };
    let n_f = |x: f64| crate::eval::num(x);
    // The series is decimated: every 6 h, the end of each leg and the last point (the relief
    // setting when it is reached). The stepping itself is hourly.
    let leg_ends: Vec<f64> = out.legs.iter().map(|l| l.end_time_h).collect();
    let last = out.points.len() - 1;
    let series: Vec<Value> = out
        .points
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            *i == 0
                || *i == last
                || ((p.time_h / SERIES_EVERY_H) - round(p.time_h / SERIES_EVERY_H)).abs() < 1.0e-9
                || leg_ends.iter().any(|e| (e - p.time_h).abs() < 1.0e-6)
        })
        .map(|(_, p)| json!([n_f(p.time_h), n_f(p.pressure_gauge_mpa), n_f(p.fill)]))
        .collect();
    let leg_rows: Vec<Value> = legs
        .iter()
        .zip(&out.legs)
        .zip(&credits)
        .map(|((l, r), c)| {
            let speed = ovd_core::holding_profile::passage_speed_kn(l, design_speed);
            json!({
                "state": l.state.key(),
                "passage_speed_kn": speed.map_or(Value::Null, |(k, _)| n_f(k)),
                "passage_speed_is_default": speed.map(|(_, d)| d),
                "days": n_f(l.days),
                "gas_use_credit_kg_h": n_f(*c),
                "end_time_h": n_f(r.end_time_h),
                "end_pressure_mpa_gauge": n_f(r.end_pressure_gauge_mpa),
                "end_fill": n_f(r.end_fill),
                "withdrawn_kg_per_tank": n_f(r.withdrawn_kg),
                "capped_by_vapour": r.capped_by_vapour,
                "held_at_start_pressure": r.held_at_start_pressure,
                "completed": r.completed,
            })
        })
        .collect();
    let required_h = total_days(&legs) * 24.0;
    v["holding_time"] = json!({
        "evaluated": true,
        "basis": BASIS,
        "rows": [{
            "id": "design_basis",
            "ambient_rule": rule.key(),
            "ambient_air_c": n_f(rule.ambient_air_c()),
            "profile_h": n_f(required_h),
            "holding_time_h": out.holding_time_h.map(n_f).unwrap_or(Value::Null),
            "reaches_relief_setting": out.holding_time_h.is_some(),
            "gas_use_credit_basis": if consumer {
                "passage legs: the least gas use of the powering model, shared over the tanks; idle legs: the typed use"
            } else {
                "no gas consumer takes the boil-off: no passage credit; idle legs: the typed use"
            },
            "legs": leg_rows,
        }],
        "tank": {
            "volume_m3_each": n_f(volume),
            "n_tanks": n_f(n),
            "relief_setting_mpa_gauge": n_f(relief),
            "start_pressure_mpa_gauge": n_f(start),
            "filling_limit": n_f(out.filling_limit),
            "loading_limit": n_f(out.loading_limit),
            "start_fill": n_f(out.start_fill),
            "start_mass_kg_each": n_f(out.start_mass_kg),
            "start_temperature_c": n_f(out.start_temperature_c),
            "conductivity_w_per_mk": n_f(k),
        },
        "inputs": [
            input_row(option, "holding_profile"),
            input_row(option, "holding_start_pressure_mpa"),
            input_row(option, "holding_fill_ratio"),
            input_row(option, "power_source"),
            input_row(option, "tank_insulation_thickness_mm"),
            input_row(option, "tank_insulation_conductivity_w_per_mk"),
            input_row(option, "tank_design_pressure_mpa"),
        ],
        "series_columns": ["time_h", "pressure_mpa_gauge", "fill"],
        "series": series,
    });
}
