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

/// The attribute `name="value"` of an SVG tag.
fn attr(tag: &str, name: &str) -> Option<f64> {
    let key = format!(" {name}=\"");
    let at = tag.find(&key)? + key.len();
    tag[at..].split('"').next()?.parse().ok()
}

/// The leader lines of a sheet as it renders: (label end, target tip), in sheet millimetres.
fn leader_lines(doc: &Value) -> Vec<([f64; 2], [f64; 2])> {
    let norm = match ovd_draw::run_value(doc, &ovd_draw::NoFiles) {
        ovd_draw::RunResult::Ok(n) => n,
        ovd_draw::RunResult::Errors(set) => panic!("breaks the standard: {set:?}"),
    };
    let svg = ovd_draw::render::svg::to_svg(
        &norm,
        &ovd_draw::render::svg::SvgOptions { embed_font: false },
    );
    svg.lines()
        .filter(|l| l.contains("viewport.main.leader") && l.starts_with("<line"))
        .map(|l| {
            let g = |n| attr(l, n).unwrap_or_else(|| panic!("{n} in {l}"));
            ([g("x1"), g("y1")], [g("x2"), g("y2")])
        })
        .collect()
}

/// How far the point `p` is from the segment `a`-`b`.
fn dist_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dy * dy;
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0);
    libm::hypot(p[0] - a[0] - t * dx, p[1] - a[1] - t * dy)
}

/// Do two segments cross each other (a proper crossing)?
fn cross(a: ([f64; 2], [f64; 2]), b: ([f64; 2], [f64; 2])) -> bool {
    let side = |o: [f64; 2], p: [f64; 2], q: [f64; 2]| {
        (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0])
    };
    side(b.0, b.1, a.0) * side(b.0, b.1, a.1) < 0.0
        && side(a.0, a.1, b.0) * side(a.0, a.1, b.1) < 0.0
}

/// Issue #957: on a damage sheet no two leaders cross, and no arrowhead lands on another leader. Measured on
/// the rendered SVG of every sheet of the 130 m river-only (ADN, ES-TRIN) and coastal (IGC, ES-TRIN) hulls.
#[test]
fn no_two_leaders_of_a_damage_sheet_cross() {
    let mut bad = Vec::new();
    let mut sheets = 0;
    for area in ["river-only", "coastal-and-river"] {
        let r = gas_results(130.0, 16.0, 7.5, area);
        for (set, stem, doc) in all_sheets(&r) {
            sheets += 1;
            let ls = leader_lines(&doc);
            assert!(ls.len() >= 2, "{area} {stem}: {} leaders", ls.len());
            for i in 0..ls.len() {
                for j in 0..ls.len() {
                    if i == j {
                        continue;
                    }
                    if i < j && cross(ls[i], ls[j]) {
                        bad.push(format!("{area} {set:?} {stem}: leaders {i} and {j} cross"));
                    }
                    // the tip of leader i on leader j
                    if dist_to_segment(ls[i].1, ls[j].0, ls[j].1) < 0.3 {
                        bad.push(format!(
                            "{area} {set:?} {stem}: the arrowhead of {i} is on leader {j}"
                        ));
                    }
                }
            }
        }
    }
    assert!(sheets >= 4, "{sheets} sheets");
    assert!(bad.is_empty(), "{} faults:\n{}", bad.len(), bad.join("\n"));
}

/// Issue #957: a governing case with no `x_m` is an error that names the case, not a sheet without its extent
/// dimension. The same for the sheets of a Rule Set.
#[test]
fn a_governing_case_without_x_m_is_an_error() {
    let mut r = gas_results(130.0, 16.0, 7.5, "coastal-and-river");
    let case = r["damage"]["governing"][0]["case"]
        .as_str()
        .unwrap()
        .to_string();
    r["damage"]["governing"][0]
        .as_object_mut()
        .unwrap()
        .remove("x_m");
    let err = damage_drawings_for(&r, Some("Option 1")).unwrap_err();
    assert!(err.contains("x_m") && err.contains(&case), "{err}");
    let mut r = gas_results(130.0, 16.0, 7.5, "river-only");
    let sets = r["damage_by_rule"].as_object_mut().unwrap();
    let block = sets.values_mut().find(|b| b.is_object()).unwrap();
    block["governing"][0].as_object_mut().unwrap().remove("x_m");
    let err = ovd_format::damage_drawings_by_rule(&r, Some("Option 1")).unwrap_err();
    assert!(err.contains("x_m"), "{err}");
}

/// Issue #957: the leaders of the 110 m hull's sheets (the `damage` block, two Spaces to a stack) do not cross.
#[test]
fn the_leaders_of_the_110_m_sheets_do_not_cross() {
    let r = results(Some("is_code_2008"));
    for (stem, doc) in damage_drawings_for(&r, Some("Option 1")).unwrap() {
        let ls = leader_lines(&doc);
        for i in 0..ls.len() {
            for j in 0..ls.len() {
                assert!(i == j || !cross(ls[i], ls[j]), "{stem}: {i} x {j}");
                assert!(
                    i == j || dist_to_segment(ls[i].1, ls[j].0, ls[j].1) >= 0.3,
                    "{stem}: arrowhead {i} on {j}"
                );
            }
        }
    }
}
