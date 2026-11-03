//! The hull views: the GA documents of an Option (spec 0003), drawn to the Drawing Format Standard by
//! `ovd-draw`, as SVG for the main panel.
//!
//! This module has no Tauri types. The front end only places and zooms what it returns: it holds
//! no geometry, no scale and no dimension.

use ovd_draw::render::svg::{SvgOptions, to_svg};
use ovd_draw::{NoFiles, RunResult, run_value};
use ovd_format::{NO_DAMAGE_DRAWING, RunOutcome, damage_drawings_named, ga_views_named};
use serde_json::{Value, json};

/// The three views of a computed Option as SVG (`side`, `plan`, `midship`), or the reason there are
/// none (`message`). The font is not embedded: the front end loads it once.
pub fn hull_views(outcome: &RunOutcome, option: &str) -> Value {
    let views = match ga_views_named(outcome, Some(option)) {
        Ok(v) => v,
        Err(message) => return json!({ "message": message }),
    };
    hull_views_of(&[
        ("side", &views.side_elevation),
        ("plan", &views.plan),
        ("midship", &views.midship_section),
    ])
}

/// The damage case drawings of a computed Option as SVG, for the Damage stability view: `sheets`, one per
/// governing case, each with its `condition` and `case` as the results name them, and `message` when a
/// sheet is left out. `None` for an Option whose results hold no drawable damage cases, so the front end
/// is sent nothing then. The font is not embedded: the front end loads it once.
pub fn damage_views(outcome: &RunOutcome, option: &str) -> Option<Value> {
    let sheets = match damage_drawings_named(outcome, Some(option)) {
        Ok(s) => s,
        Err(message) if message == NO_DAMAGE_DRAWING => return None,
        Err(message) => return Some(json!({ "sheets": [], "message": message })),
    };
    let results = match outcome {
        RunOutcome::Results { results, .. } => results,
        _ => return None,
    };
    let governing = results["damage"]["governing"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    let mut out = Vec::new();
    let mut refused = Vec::new();
    for (i, (stem, document)) in sheets.iter().enumerate() {
        match run_value(document, &NoFiles) {
            RunResult::Ok(norm) => out.push(json!({
                "stem": stem,
                "condition": governing.get(i).map_or(Value::Null, |g| g["condition"].clone()),
                "case": governing.get(i).map_or(Value::Null, |g| g["case"].clone()),
                "svg": to_svg(&norm, &SvgOptions { embed_font: false }),
            })),
            RunResult::Errors(clauses) => {
                let list: Vec<String> = clauses.into_iter().collect();
                refused.push(format!("{stem} (clause {})", list.join(", ")));
            }
        }
    }
    let mut v = json!({ "sheets": out });
    if !refused.is_empty() {
        v["message"] = Value::String(format!(
            "These damage drawings break the Drawing Format Standard: {}. This is a fault in OVD, not in the Design.",
            refused.join("; ")
        ));
    }
    Some(v)
}

/// The SVG of each view the standard accepts. A view it refuses does not hide the others (issue
/// #660): it is left out, and `message` names it and the clauses.
fn hull_views_of(documents: &[(&str, &Value)]) -> Value {
    let mut out = serde_json::Map::new();
    let mut refused = Vec::new();
    for (key, document) in documents {
        match run_value(document, &NoFiles) {
            RunResult::Ok(norm) => {
                out.insert(
                    (*key).to_string(),
                    Value::String(to_svg(&norm, &SvgOptions { embed_font: false })),
                );
            }
            RunResult::Errors(clauses) => {
                let list: Vec<String> = clauses.into_iter().collect();
                refused.push(format!("{key} (clause {})", list.join(", ")));
            }
        }
    }
    if !refused.is_empty() {
        out.insert(
            "message".to_string(),
            Value::String(format!(
                "These hull views break the Drawing Format Standard: {}. This is a fault in OVD, not in the Design.",
                refused.join("; ")
            )),
        );
    }
    Value::Object(out)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ovd_format::{CONVOY_NO_VIEWS, Profile, check, fill_as_new, run_design, run_option};
    use std::path::Path;

    fn outcome(fixture: &str) -> RunOutcome {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ovd-format/tests/fixtures")
            .join(fixture);
        let checked = check(
            &std::fs::read_to_string(path).unwrap(),
            &Profile::built_in(),
        );
        run_design(&checked.design.unwrap()).remove(0).outcome
    }

    #[test]
    fn a_computed_option_gets_three_svg_views() {
        for fixture in [
            "dry_bulk_box_hold.vdesign",
            "gas_igc_2g_self_propelled.vdesign",
        ] {
            let v = hull_views(&outcome(fixture), "Option 1");
            for key in ["side", "plan", "midship"] {
                let svg = v[key]
                    .as_str()
                    .unwrap_or_else(|| panic!("{fixture}: no {key}: {v}"));
                assert!(svg.starts_with("<?xml"), "{fixture} {key}");
                assert!(
                    svg.contains("viewBox=\"0 0 420 297\""),
                    "an A3 landscape sheet"
                );
                assert!(
                    !svg.contains("base64"),
                    "the font is loaded once by the front end, not in each SVG"
                );
            }
        }
    }

    #[test]
    fn the_dimensions_on_the_views_are_the_numbers_of_the_results() {
        let v = hull_views(&outcome("dry_bulk_box_hold.vdesign"), "Option 1");
        // LOA 76.5 m, beam 11.4 m, depth 3.5 m
        assert!(v["side"].as_str().unwrap().contains(">76.50<"));
        assert!(v["plan"].as_str().unwrap().contains(">76.50<"));
        assert!(v["plan"].as_str().unwrap().contains(">11.40<"));
        assert!(v["midship"].as_str().unwrap().contains(">11.40<"));
        assert!(v["midship"].as_str().unwrap().contains(">3.50<"));
    }

    #[test]
    fn a_convoy_has_no_views_and_the_message_of_the_outputs_panel() {
        let v = hull_views(&outcome("pushed_convoy.vdesign"), "Option 1");
        assert_eq!(v, json!({ "message": CONVOY_NO_VIEWS }));
    }

    #[test]
    fn a_wide_option_name_gets_its_three_views() {
        // Issue #660: "140 x 29 x 9m LNG" was too wide for the document number cell, so the
        // standard refused all three views.
        for name in ["140 x 29 x 9m LNG", "Coastal LNG barge, 140 x 29 x 9 m"] {
            let v = hull_views(&outcome("gas_igc_2g_self_propelled.vdesign"), name);
            assert!(v.get("message").is_none(), "{name}: {v}");
            for key in ["side", "plan", "midship"] {
                assert!(
                    v[key].as_str().is_some_and(|s| s.starts_with("<?xml")),
                    "{name} {key}"
                );
            }
        }
    }

    #[test]
    fn a_name_too_wide_for_the_title_block_says_what_fits_not_a_fault() {
        let v = hull_views(
            &outcome("gas_igc_2g_self_propelled.vdesign"),
            "A very long descriptive Option name that no title block can hold",
        );
        let message = v["message"].as_str().unwrap();
        assert!(
            message.contains("too wide for the drawing title block"),
            "{message}"
        );
        assert!(!message.contains("fault in OVD"), "{message}");
    }

    #[test]
    fn one_view_the_standard_refuses_does_not_hide_the_others() {
        let o = outcome("gas_igc_2g_self_propelled.vdesign");
        let views = ovd_format::ga_views_named(&o, Some("Option 1")).unwrap();
        // A plan the standard refuses: its document number is too wide for its cell (clause 14.6).
        let mut plan = views.plan.clone();
        plan["title_block"]["fields"]["document_number"] = json!("W".repeat(40));
        let v = hull_views_of(&[
            ("side", &views.side_elevation),
            ("plan", &plan),
            ("midship", &views.midship_section),
        ]);
        assert!(v["side"].as_str().is_some_and(|s| s.starts_with("<?xml")));
        assert!(
            v["midship"]
                .as_str()
                .is_some_and(|s| s.starts_with("<?xml"))
        );
        assert!(v.get("plan").is_none());
        let message = v["message"].as_str().unwrap();
        assert!(message.contains("plan (clause 14.6)"), "{message}");
        assert!(
            !message.contains("side") && !message.contains("midship"),
            "{message}"
        );
    }

    /// The text of a filled Design for a gas hull checked in the IS Code 2008 conditions on the
    /// default Rule Sets, which evaluate the IGC damage cases.
    pub(crate) fn gas_damage_design_text() -> String {
        let file = json!({
            "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
            "options": [{ "name": "Option 1", "inputs": {
                "cargo": { "value": "LNG", "source": "typed" },
                "hull_form": { "value": "raked_box", "source": "typed" },
                "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
                "length_overall_m": { "value": 110.0, "source": "typed" },
                "beam_m": { "value": 11.4, "source": "typed" },
                "moulded_depth_m": { "value": 6.0, "source": "typed" },
                "loading_conditions": { "value": "is_code_2008", "source": "typed" }
            } }]
        });
        let filled = fill_as_new(&file.to_string(), &Profile::built_in());
        filled.design.unwrap().to_text()
    }

    fn damage_outcome() -> RunOutcome {
        let filled = fill_as_new(&gas_damage_design_text(), &Profile::built_in());
        run_option(&filled.design.unwrap().options[0])
    }

    #[test]
    fn a_run_with_damage_cases_sends_one_damage_svg_per_governing_case() {
        let o = damage_outcome();
        let RunOutcome::Results { results, .. } = &o else {
            panic!("no results")
        };
        let governing = results["damage"]["governing"].as_array().unwrap();
        let v = damage_views(&o, "Option 1").expect("damage views");
        assert!(v.get("message").is_none(), "{v}");
        let sheets = v["sheets"].as_array().unwrap();
        assert_eq!(sheets.len(), governing.len());
        for (sheet, g) in sheets.iter().zip(governing) {
            let svg = sheet["svg"].as_str().unwrap();
            assert!(svg.starts_with("<?xml"));
            assert!(svg.contains("viewBox=\"0 0 420 297\""));
            assert!(!svg.contains("base64"), "the font is loaded once");
            assert_eq!(sheet["condition"], g["condition"]);
            assert_eq!(sheet["case"], g["case"]);
        }
    }

    #[test]
    fn a_run_without_damage_cases_sends_no_damage_svg() {
        // legacy gas conditions, a liquid hull (its damage is a list, not drawn) and a dry hull
        for fixture in [
            "gas_igc_2g_exact.vdesign",
            "crude_integral_tanks_self_propelled.vdesign",
            "dry_bulk_box_hold.vdesign",
        ] {
            assert!(
                damage_views(&outcome(fixture), "Option 1").is_none(),
                "{fixture}"
            );
        }
    }

    #[test]
    fn the_governing_case_of_each_condition_is_the_trace_row_the_view_reads() {
        // The view marks the engine's governing case of each condition. It must be a row of the trace,
        // with the verdict the results hold, and the worst case of its condition.
        let o = damage_outcome();
        let RunOutcome::Results { results, .. } = &o else {
            panic!("no results")
        };
        let rows = results["rule_trace"]["rows"].as_array().unwrap();
        let damage_rows: Vec<&Value> = rows
            .iter()
            .filter(|r| r["rule"] == "igc/2.7/damage-survival")
            .collect();
        let cases = results["damage"]["cases"].as_array().unwrap();
        assert_eq!(damage_rows.len(), cases.len(), "one row per case");
        for g in results["damage"]["governing"].as_array().unwrap() {
            let object = format!(
                "{}: {}",
                g["condition"].as_str().unwrap(),
                g["case"].as_str().unwrap()
            );
            let row = damage_rows
                .iter()
                .find(|r| r["object"] == object.as_str())
                .unwrap_or_else(|| panic!("no trace row for {object}"));
            assert_eq!(row["outcome"] == "pass", g["ok"] == true, "{object}");
            // The governing case is the worst of its condition: a failing case beats a passing one and,
            // among those, the least maximum righting lever wins (a tie keeps the earlier case).
            let mut worst: Option<&Value> = None;
            for c in cases.iter().filter(|c| c["condition"] == g["condition"]) {
                let worse = worst.is_none_or(|w| {
                    (w["ok"] == true && c["ok"] == false)
                        || (w["ok"] == c["ok"]
                            && c["max_gz_m"].as_f64().unwrap() < w["max_gz_m"].as_f64().unwrap())
                });
                if worse {
                    worst = Some(c);
                }
            }
            assert_eq!(worst.unwrap()["name"], g["case"], "{object}");
        }
    }

    #[test]
    fn the_same_results_give_the_same_svg() {
        let o = outcome("gas_igc_2g_exact.vdesign");
        assert_eq!(hull_views(&o, "Option 1"), hull_views(&o, "Option 1"));
    }
}
