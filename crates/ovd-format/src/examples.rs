//! The examples the app offers when a designer starts a Design (spec 0017, D6): data in one place,
//! so the desktop app and the web build cannot differ. An example is typed inputs only; the
//! session call `DesignSession::from_typed` makes the Design from them.

use serde_json::{Map, Value, json};

/// One example: an id, a title, one line of text, and the typed inputs of its one Option.
pub struct Example {
    pub id: &'static str,
    pub title: &'static str,
    pub text: &'static str,
    inputs: fn() -> Value,
}

impl Example {
    /// The typed inputs, as the input name and value pairs the session call takes.
    pub fn inputs(&self) -> Map<String, Value> {
        match (self.inputs)() {
            Value::Object(map) => map,
            _ => Map::new(),
        }
    }
}

/// Every example, in the order the app lists them.
pub const EXAMPLES: &[Example] = &[Example {
    id: "coastal-lng-barge",
    title: "Coastal LNG barge, 140 × 29 m",
    text: "A self-propelled LNG barge with Type C tanks, for coastal and river service.",
    inputs: coastal_lng_barge,
}];

fn coastal_lng_barge() -> Value {
    json!({
        "cargo": "LNG",
        "hull_form": "raked_box",
        "formation": { "kind": "self_propelled" },
        "length_overall_m": 140.0,
        "beam_m": 29.0,
        "service_area": "coastal-and-river",
        "moulded_depth_m": 12.0,
        "max_speed_kn": 12.0,
        "max_tank_od_m": 12.0,
        "tanks_in_line": 2,
        "min_freeboard_m": 1.81,
    })
}

/// The list as JSON: each example's `id`, `title`, `text` and `inputs`.
pub fn examples_json() -> Value {
    Value::Array(
        EXAMPLES
            .iter()
            .map(|e| json!({ "id": e.id, "title": e.title, "text": e.text, "inputs": e.inputs() }))
            .collect(),
    )
}
