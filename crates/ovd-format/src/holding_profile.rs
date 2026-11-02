//! The `holding_profile` input at the format's edge (spec 0016, issue #869): the legs a Type C tank
//! goes through, as rows of a state, a length in days and, by state, a passage speed or a typed gas
//! use. The engine's holding-time model (#870) reads them; here they are read and refused.

use ovd_core::holding_profile::{Leg, LegState};
use ovd_core::text;
use ovd_core::text::Text;
use serde_json::{Value, json};

/// The keys a row may hold.
pub const KEYS: [&str; 4] = ["state", "days", "passage_speed_kn", "gas_use_kg_h"];

/// The default profile as a stored value: 20 days laden passage, then 10 days idle.
pub fn default_value() -> Value {
    Value::Array(
        ovd_core::holding_profile::default_design_basis()
            .iter()
            .map(|l| json!({ "state": l.state.key(), "days": l.days }))
            .collect(),
    )
}

fn finite(row: &Value, key: &str) -> Result<Option<f64>, ()> {
    match row.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => match v.as_f64() {
            Some(x) if x.is_finite() => Ok(Some(x)),
            _ => Err(()),
        },
    }
}

/// The legs of a `holding_profile` value, or the first thing wrong with it. A profile is a
/// non-empty list of rows. A row is `{state, days}` and may carry `passage_speed_kn` (a laden
/// passage) or `gas_use_kg_h` (an idle leg, zero unless typed). A state other than `laden_passage`
/// or `idle`, a length that is not above zero, a negative speed or use, a key the row does not
/// take, and a profile with no leg are refused (spec 0016 D9, section 7 "Profiles").
pub fn legs_of(v: &Value) -> Result<Vec<Leg>, Text> {
    let Some(rows) = v.as_array().filter(|r| !r.is_empty()) else {
        return Err(text!(
            "msg-profile-empty",
            "the holding-time profile must be a list of at least one leg"
        ));
    };
    let mut legs = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        let row = i + 1;
        let Some(o) = r.as_object() else {
            return Err(text!(
                "msg-profile-row-shape",
                "leg {row} must be an object with `state` and `days`",
                row = row
            ));
        };
        if let Some(k) = o.keys().find(|k| !KEYS.contains(&k.as_str())) {
            return Err(text!(
                "msg-profile-key",
                "leg {row} has `{key}`, which a leg does not take (state, days, passage_speed_kn, \
                 gas_use_kg_h)",
                row = row,
                key = k.as_str()
            ));
        }
        let state_text = o.get("state").and_then(Value::as_str).unwrap_or("");
        let Some(state) = LegState::from_key(state_text) else {
            return Err(text!(
                "msg-profile-state",
                "leg {row}: the state `{state}` is not known; the states are laden_passage and \
                 idle (loading, discharge and ballast are not modelled)",
                row = row,
                state = state_text
            ));
        };
        let days = match finite(r, "days") {
            Ok(Some(d)) if d > 0.0 => d,
            _ => {
                return Err(text!(
                    "msg-profile-days",
                    "leg {row}: `days` must be a number above zero",
                    row = row
                ));
            }
        };
        let speed = match finite(r, "passage_speed_kn") {
            Ok(s) => s,
            Err(()) => {
                return Err(text!(
                    "msg-profile-speed",
                    "leg {row}: `passage_speed_kn` must be a number above zero, or left out",
                    row = row
                ));
            }
        };
        if speed.is_some_and(|s| s <= 0.0) {
            return Err(text!(
                "msg-profile-speed",
                "leg {row}: `passage_speed_kn` must be a number above zero, or left out",
                row = row
            ));
        }
        let use_kg_h = match finite(r, "gas_use_kg_h") {
            Ok(u) => u,
            Err(()) => {
                return Err(text!(
                    "msg-profile-use",
                    "leg {row}: `gas_use_kg_h` must be a number of zero or more, or left out",
                    row = row
                ));
            }
        };
        if use_kg_h.is_some_and(|u| u < 0.0) {
            return Err(text!(
                "msg-profile-use",
                "leg {row}: `gas_use_kg_h` must be a number of zero or more, or left out",
                row = row
            ));
        }
        match state {
            LegState::Idle if speed.is_some() => {
                return Err(text!(
                    "msg-profile-speed-idle",
                    "leg {row} is idle and has a passage speed; an idle leg has none",
                    row = row
                ));
            }
            LegState::LadenPassage if use_kg_h.is_some() => {
                return Err(text!(
                    "msg-profile-use-passage",
                    "leg {row} is a laden passage and has a typed gas use; its credit is the \
                     powering model's gas use at the passage speed",
                    row = row
                ));
            }
            _ => {}
        }
        legs.push(Leg {
            state,
            days,
            passage_speed_kn: speed,
            gas_use_kg_h: use_kg_h.unwrap_or(0.0),
        });
    }
    Ok(legs)
}
