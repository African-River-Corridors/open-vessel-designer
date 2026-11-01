//! The steel list of a gas hull's plate model (issue #742, spec 0013 "Results" and "Outputs"): the
//! `structure` results block (`steel_list`, `groups`, `purchase`), the item geometry beside it
//! (`structure_geometry`), and the CSV of `ovd export --format steel-csv`.
//!
//! Both blocks are outside the results hash until the lightship reads the plate model (spec 0013,
//! T7): see `run::UNHASHED_KEYS`. Concept and preliminary design only (ADR 0011).

use ovd_core::hull_plates::{HullPlateModel, OPENINGS_NOTE};
use serde_json::{Map, Value, json};

/// The scope line every steel output carries (ADR 0011).
pub const SCOPE: &str = "Concept and preliminary design only (ADR 0011). Class minimum scantlings re-cut into plates; not a structural approval.";

/// The `structure` results block of a plate model.
pub fn structure_json(m: &HullPlateModel) -> Value {
    let rows: Vec<Value> = m
        .items
        .iter()
        .map(|i| {
            json!({
                "id": i.id,
                "kind": i.kind.as_str(),
                "source": i.source.id(),
                "member": i.member,
                "region": i.region,
                "grade": i.grade.key(),
                "rule_net_mm": i.rule_net_mm,
                "ordered_mm": i.ordered_mm,
                "section": i.section,
                "length_m": i.length_m,
                "width_m": i.width_m,
                "area_m2": i.area_m2,
                "pieces": i.pieces,
                "rule_net_t": i.rule_net_t,
                "ordered_t": i.ordered_t,
                "centre_m": [i.centre_m.0, i.centre_m.1, i.centre_m.2],
                "rule": i.rule,
                "rule_set": i.rule_set,
                "edition": i.edition,
                "paragraph": i.paragraph,
                "basis": i.basis,
            })
        })
        .collect();
    let groups: Vec<Value> = m
        .groups
        .iter()
        .map(|g| {
            json!({
                "member": g.member,
                "region": g.region,
                "items": g.items,
                "rule_net_t": g.rule_net_t,
                "ordered_t": g.ordered_t,
                "centre_m": [g.centre_m.0, g.centre_m.1, g.centre_m.2],
            })
        })
        .collect();
    let plates: Vec<Value> = m
        .plates
        .iter()
        .map(|p| {
            json!({
                "ordered_mm": p.ordered_mm,
                "stock_m": [p.stock_width_m, p.stock_length_m],
                "count": p.count,
                "bought_m2": p.bought_m2,
                "net_m2": p.net_m2,
                "scrap_m2": p.scrap_m2,
                "scrap_t": p.scrap_t,
            })
        })
        .collect();
    let sections: Vec<Value> = m
        .sections
        .iter()
        .map(|s| json!({"section": s.section, "length_m": s.length_m, "weight_t": s.weight_t}))
        .collect();
    let st = &m.structure;
    json!({
        "scope": SCOPE,
        "openings": OPENINGS_NOTE,
        "rule_edition": m.rule_edition,
        "inputs": {
            "frame_spacing_m": st.frame_spacing_m,
            "longitudinal_spacing_mm": st.longitudinal_spacing_mm,
            "stock_plate_m": [st.stock_plate_width_m, st.stock_plate_length_m],
            "thickness_step_mm": st.thickness_step_mm,
            "steel_grade": st.steel_grade.key(),
        },
        "steel_list": rows,
        "groups": {
            "members": groups,
            "rule_net_t": m.rule_net_t,
            "ordered_t": m.ordered_t,
            "allowance_pct": m.allowance_pct,
            "allowance_t": m.allowance_t,
            "allowance_basis": "brackets, collars, welds and small items (an Estimate)",
            "total_t": m.total_t,
            "scrap_m2": m.scrap_m2,
            "scrap_t": m.scrap_t,
        },
        "purchase": {
            "plates": plates,
            "bought_m2": m.bought_m2,
            "net_m2": m.net_plate_m2,
            "sections": sections,
        },
    })
}

/// The geometry of each steel list item, by its id: a plate's outline (its four corners, or on the
/// Sections Table the knuckles of a curved plate and the clipped polygon of a section; issue #743), or a trace line's two
/// ends, in hull axes (m).
pub fn structure_geometry_json(m: &HullPlateModel) -> Value {
    let mut out = Map::new();
    for i in &m.items {
        let pts: Vec<Value> = i
            .geometry
            .points
            .iter()
            .map(|p| json!([p.0, p.1, p.2]))
            .collect();
        out.insert(i.id.clone(), Value::Array(pts));
    }
    Value::Object(out)
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// The steel list of a `structure` results block as CSV: `#` lines with the scope, the openings
/// note and the inputs, then a header and one row per item.
pub fn steel_csv(structure: &Value) -> Result<String, String> {
    let rows = structure["steel_list"]
        .as_array()
        .ok_or("the results have no steel list (a gas hull only, so far)")?;
    let inp = &structure["inputs"];
    let mut out = String::new();
    out.push_str(&format!("# OVD steel list. {SCOPE}\n"));
    out.push_str(&format!(
        "# {}\n",
        structure["openings"].as_str().unwrap_or(OPENINGS_NOTE)
    ));
    out.push_str(&format!(
        "# Rule edition: {}. Thickness step {} mm; stock plate {} x {} m.\n",
        structure["rule_edition"].as_str().unwrap_or(""),
        inp["thickness_step_mm"],
        inp["stock_plate_m"][0],
        inp["stock_plate_m"][1],
    ));
    let cols = [
        "id",
        "kind",
        "member",
        "region",
        "grade",
        "rule_net_mm",
        "ordered_mm",
        "section",
        "length_m",
        "width_m",
        "area_m2",
        "pieces",
        "rule_net_t",
        "ordered_t",
        "x_m",
        "y_m",
        "z_m",
        "rule_set",
        "edition",
        "paragraph",
        "basis",
    ];
    out.push_str(&cols.join(","));
    out.push('\n');
    for r in rows {
        let num = |k: &str, dp: usize| format!("{:.*}", dp, r[k].as_f64().unwrap_or(0.0));
        let text = |k: &str| csv_field(r[k].as_str().unwrap_or(""));
        let c = |i: usize| format!("{:.3}", r["centre_m"][i].as_f64().unwrap_or(0.0));
        let fields = [
            text("id"),
            text("kind"),
            text("member"),
            text("region"),
            text("grade"),
            num("rule_net_mm", 1),
            num("ordered_mm", 1),
            text("section"),
            num("length_m", 3),
            num("width_m", 3),
            num("area_m2", 3),
            r["pieces"].to_string(),
            num("rule_net_t", 4),
            num("ordered_t", 4),
            c(0),
            c(1),
            c(2),
            text("rule_set"),
            text("edition"),
            text("paragraph"),
            text("basis"),
        ];
        out.push_str(&fields.join(","));
        out.push('\n');
    }
    Ok(out)
}
