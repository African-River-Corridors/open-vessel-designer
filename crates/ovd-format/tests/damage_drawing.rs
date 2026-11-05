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

/// A gas Option of the given length in a Service Area, with the `damage` block of the coastal stack and the
/// `damage_by_rule` sets the stack evaluates (ES-TRIN 28.03 runs over 110 m only).
fn gas_results(loa: f64, beam: f64, depth: f64, area: &str) -> Value {
    let inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "service_area": { "value": area, "source": "typed" },
        "moulded_depth_m": { "value": depth, "source": "typed" },
        "loading_conditions": { "value": "is_code_2008", "source": "typed" }
    });
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

/// Every sheet of the Option, in the order the app and `ovd draw` give them: the `damage` block's, then each
/// `damage_by_rule` set's, with the key of the set (none for the `damage` block).
fn all_sheets(r: &Value) -> Vec<(Option<String>, String, Value)> {
    let mut out: Vec<_> = ovd_format::damage_drawings_for(r, Some("Option 1"))
        .unwrap_or_default()
        .into_iter()
        .map(|(s, d)| (None, s, d))
        .collect();
    for (key, sheets) in ovd_format::damage_drawings_by_rule(r, Some("Option 1")).unwrap() {
        out.extend(sheets.into_iter().map(|(s, d)| (Some(key.clone()), s, d)));
    }
    out
}

fn text_of(doc: &Value) -> String {
    doc.to_string()
}

/// Issue #928: the sheets of every damage set of an Option have one document number each and one sheet
/// count, across the sets, on a river-only hull (ADN and ES-TRIN) and on a coastal one (IGC and ES-TRIN).
#[test]
fn no_two_damage_sheets_of_an_option_share_a_document_number() {
    for (area, first_set) in [("river-only", "adn-gas"), ("coastal-and-river", "igc-2g")] {
        let r = gas_results(130.0, 16.0, 7.5, area);
        let by_rule = r["damage_by_rule"].as_object().unwrap();
        let es = by_rule
            .keys()
            .find(|k| k.starts_with("es-trin"))
            .unwrap_or_else(|| panic!("{area}: no ES-TRIN damage set over 110 m: {by_rule:?}"));
        let sheets = all_sheets(&r);
        let mut numbers: Vec<String> = Vec::new();
        for (set, stem, doc) in &sheets {
            let f = &doc["title_block"]["fields"];
            numbers.push(f["document_number"].as_str().unwrap().to_string());
            assert_eq!(f["sheet_count"], sheets.len(), "{area} {stem}");
            // what is drawn: the case and the Rule Set the sheet is of
            let text = text_of(doc);
            let rule_set = match set {
                Some(k) => by_rule[k]["rule_set"].as_str().unwrap().to_string(),
                None => r["damage"]["rule_set"].as_str().unwrap().to_string(),
            };
            assert!(
                text.contains(&rule_set.replace('@', " ")),
                "{area} {stem}: no {rule_set}"
            );
        }
        let mut unique = numbers.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), numbers.len(), "{area}: {numbers:?}");
        assert!(
            sheets.iter().any(|(s, _, _)| s.as_deref() == Some(es)),
            "{area}: no ES-TRIN sheet"
        );
        assert!(
            area == "river-only"
                && sheets
                    .iter()
                    .any(|(s, _, _)| s.as_deref() == Some(first_set))
                || area != "river-only" && sheets.iter().any(|(s, _, _)| s.is_none()),
            "{area}: no {first_set} sheet"
        );
    }
}

/// Issue #928: the sheet of a set names the case the engine names for that set, in its own notes.
#[test]
fn each_sheet_draws_the_governing_case_of_its_own_set() {
    let r = gas_results(130.0, 16.0, 7.5, "river-only");
    for (key, block) in r["damage_by_rule"].as_object().unwrap() {
        let sheets = ovd_format::damage_drawings_by_rule(&r, None).unwrap();
        let mine = &sheets.iter().find(|(k, _)| k == key).unwrap().1;
        let governing = block["governing"].as_array().unwrap();
        assert_eq!(mine.len(), governing.len(), "{key}");
        for ((_, doc), g) in mine.iter().zip(governing) {
            let want = format!(
                "{}: {}",
                g["condition"].as_str().unwrap(),
                g["case"].as_str().unwrap()
            )
            .to_uppercase();
            assert!(text_of(doc).contains(&want), "{key}: {want}");
        }
    }
}

/// Issue #928: each governing case carries its extent as fields (`x_m`, and `y_m` for a bottom damage that has
/// a width), the drawing is made from them, and not from the case name.
#[test]
fn each_governing_case_carries_its_extent_and_the_sheet_dimensions_it() {
    let (mut side, mut bottom, mut across) = (0, 0, 0);
    for area in ["river-only", "coastal-and-river"] {
        let r = gas_results(130.0, 16.0, 7.5, area);
        let mut blocks: Vec<&Value> = vec![&r["damage"]];
        blocks.extend(r["damage_by_rule"].as_object().unwrap().values());
        for block in blocks.into_iter().filter(|b| b.is_object()) {
            for g in block["governing"].as_array().unwrap() {
                let name = g["case"].as_str().unwrap();
                let x = g["x_m"]
                    .as_array()
                    .unwrap_or_else(|| panic!("{name}: no x_m"));
                assert!(
                    x[0].as_f64().unwrap() < x[1].as_f64().unwrap(),
                    "{name}: {x:?}"
                );
                // a case that names a width across has `y_m`, and no other has
                let wide = name.contains("m across");
                assert_eq!(g.get("y_m").is_some(), wide, "{name}");
                side += usize::from(name.starts_with("side"));
                bottom += usize::from(name.starts_with("bottom"));
                across += usize::from(wide);
            }
        }
        for (_, stem, doc) in all_sheets(&r) {
            assert!(
                entities(&doc).iter().any(|e| e["type"] == "dimension"),
                "{area} {stem}: no dimension for the damage extent"
            );
        }
    }
    assert!(side > 0 && bottom > 0, "side {side}, bottom {bottom}");
    println!("governing cases: side {side}, bottom {bottom}, with a width {across}");
}
