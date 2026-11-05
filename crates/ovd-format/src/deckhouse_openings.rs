//! The `deckhouse_openings` input at the format's edge (issue #912): a typed list of the openings of
//! a deckhouse stern, which stands in for the default layout the engine places. A row is
//! `{kind, space, face, x_m, y_m, z_m}` and may carry `openable`, `gas_tight`, `tier` and a `name`.
//! x is metres forward of the stern, y metres to starboard of the centreline, z metres above the keel.

use ovd_core::deckhouse_openings::{Face, Kind, Opening, Space};
use ovd_core::text;
use ovd_core::text::Text;
use serde_json::Value;

/// The numbers a row holds.
pub const NUMBERS: [&str; 3] = ["x_m", "y_m", "z_m"];
/// The texts a row holds.
pub const TEXTS: [&str; 3] = ["kind", "space", "face"];
const OPTIONAL: [&str; 4] = ["openable", "gas_tight", "tier", "name"];

/// Whether `row` is an object of the three texts and three numbers, and only the optional keys
/// besides (the texts being values of their kinds).
pub fn is_row(row: &Value) -> bool {
    row_error(row).is_none()
}

fn row_error(row: &Value) -> Option<&'static str> {
    let Some(o) = row.as_object() else {
        return Some("is not an object");
    };
    for k in NUMBERS {
        if !o.get(k).and_then(Value::as_f64).is_some_and(f64::is_finite) {
            return Some("needs the numbers x_m, y_m and z_m");
        }
    }
    let text_of = |k: &str| o.get(k).and_then(Value::as_str);
    if !text_of("kind").is_some_and(|s| Kind::from_key(s).is_some()) {
        return Some("needs a kind: entrance, window, air_inlet or air_intake");
    }
    if !text_of("space").is_some_and(|s| Space::from_key(s).is_some()) {
        return Some(
            "needs a space: accommodation, service, machinery, control_station or wheelhouse",
        );
    }
    if !text_of("face").is_some_and(|s| Face::from_key(s).is_some()) {
        return Some("needs a face: front, side or aft");
    }
    for (k, v) in o {
        let known = NUMBERS.contains(&k.as_str())
            || TEXTS.contains(&k.as_str())
            || OPTIONAL.contains(&k.as_str());
        if !known {
            return Some("has a key it does not take");
        }
        let ok = match k.as_str() {
            "openable" | "gas_tight" => v.is_boolean(),
            "tier" => v.as_i64().is_some_and(|t| t >= 0),
            "name" => v.is_string(),
            _ => true,
        };
        if !ok {
            return Some("has a value of the wrong kind (openable and gas_tight are true or false, tier a whole number from 0)");
        }
    }
    None
}

/// The openings of a `deckhouse_openings` value: `null` is none (the default layout stands); an empty
/// list is a deckhouse with no opening.
pub fn openings_of(v: &Value) -> Result<Option<Vec<Opening>>, Text> {
    let Some(list) = v.as_array() else {
        return Ok(None);
    };
    list.iter()
        .enumerate()
        .map(|(i, r)| {
            if let Some(why) = row_error(r) {
                return Err(text!(
                    "msg-deckhouse-openings-bad-row",
                    "deckhouse opening {row} {why}",
                    row = i + 1,
                    why = why
                ));
            }
            let o = r.as_object().expect("a row checked as an object");
            let num = |k: &str| o[k].as_f64().expect("a number checked");
            let kind = Kind::from_key(o["kind"].as_str().expect("a text")).expect("a kind");
            let face = Face::from_key(o["face"].as_str().expect("a text")).expect("a face");
            let space = Space::from_key(o["space"].as_str().expect("a text")).expect("a space");
            let flag = |k: &str, default: bool| o.get(k).and_then(Value::as_bool).unwrap_or(default);
            Ok(Opening {
                id: o
                    .get("name")
                    .and_then(Value::as_str)
                    .map_or_else(|| format!("typed opening {}", i + 1), str::to_string),
                kind,
                space,
                face,
                x_m: num("x_m"),
                y_m: num("y_m"),
                z_m: num("z_m"),
                // A window is fixed unless the row says it opens; a door or a vent opens.
                openable: flag("openable", kind != Kind::Window),
                gas_tight: flag("gas_tight", false),
                tier: o
                    .get("tier")
                    .and_then(Value::as_i64)
                    .unwrap_or(if space == Space::Wheelhouse { 0 } else { 1 }),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
