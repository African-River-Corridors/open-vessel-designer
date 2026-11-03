//! Seam: `ovd_format::damage_drawings_for`, the damage case drawing (issue #761, spec 0012 D12 and D19): one
//! sheet per governing case (per Rule Set and loading condition), the flooded Spaces hatched and the damaged
//! waterline drawn.
//!
//! Label: property

use ovd_format::{
    NO_DAMAGE_DRAWING, Profile, RunOutcome, damage_drawings_for, fill_as_new, run_option,
};
use serde_json::{Value, json};

fn results(loading: Option<&str>) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        // Issue #850: the IGC rows are in the coastal-and-river stack; a river-only gas Option takes ADN 9.3.1.
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    if let Some(l) = loading {
        inputs["loading_conditions"] = json!({ "value": l, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn entities(doc: &Value) -> &Vec<Value> {
    doc["viewports"][0]["entities"].as_array().unwrap()
}

/// One sheet per governing case, not one per case; every sheet meets the drawing standard.
#[test]
fn one_sheet_per_governing_case() {
    let r = results(Some("is_code_2008"));
    let governing = r["damage"]["governing"].as_array().unwrap().len();
    let cases = r["damage"]["cases"].as_array().unwrap().len();
    assert!(
        governing >= 1 && governing < cases,
        "{governing} of {cases}"
    );
    let sheets = damage_drawings_for(&r, Some("Option 1")).unwrap();
    assert_eq!(sheets.len(), governing);
    for (i, (stem, doc)) in sheets.iter().enumerate() {
        assert_eq!(stem, &format!("damage-{}", i + 1));
        match ovd_draw::run_value(doc, &ovd_draw::NoFiles) {
            ovd_draw::RunResult::Ok(_) => {}
            ovd_draw::RunResult::Errors(set) => panic!("{stem} breaks the standard: {set:?}"),
        }
    }
}

/// The flooded Spaces are hatched (in plan, and in the section where they cross it), and a floating case
/// has its damaged waterline.
#[test]
fn flooded_spaces_are_hatched_and_the_damaged_waterline_is_drawn() {
    let r = results(Some("is_code_2008"));
    let governing = r["damage"]["governing"].as_array().unwrap();
    let sheets = damage_drawings_for(&r, Some("Option 1")).unwrap();
    for (g, (_, doc)) in governing.iter().zip(&sheets) {
        let flooded = g["flooded"].as_array().unwrap().len();
        let hatches = entities(doc)
            .iter()
            .filter(|e| e["type"] == "hatch" && e["layer"] == "HATCH")
            .count();
        assert!(
            hatches >= flooded,
            "{hatches} hatches for {flooded} flooded"
        );
        let water = entities(doc)
            .iter()
            .filter(|e| e["layer"] == "WATER")
            .count();
        if g["floats"] == "yes" {
            assert_eq!(water, 1, "{}", g["case"]);
        } else {
            assert_eq!(water, 0);
        }
    }
}

/// An Option with no damage cases (here, the legacy gas conditions) has no damage drawing, and says so.
#[test]
fn an_option_without_damage_cases_has_no_damage_drawing() {
    let r = results(Some("legacy"));
    assert!(r["damage"].is_null());
    assert_eq!(
        damage_drawings_for(&r, None).unwrap_err(),
        NO_DAMAGE_DRAWING
    );
}
