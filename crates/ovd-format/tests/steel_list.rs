//! Seam: the steel list of a gas hull's plate model in the results (`structure`,
//! `structure_geometry`) and as `ovd export --format steel-csv` (issue #742, spec 0013 "Results" and
//! "Outputs").
//!
//! Label: property - the steel list re-cuts the panel table (the members the ends do not lay out on
//! the hull exactly; issue #743), the CSV has one row per item
//! and says that lightening holes are not deducted, every item has its geometry, and the steel list
//! is outside the results hash.

use std::path::Path;

use ovd_format::export::option_export;
use ovd_format::{DesignOption, Profile, RunOutcome, check, results_hash, run_option};
use serde_json::Value;

fn options_of(file: &str) -> Vec<DesignOption> {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(file),
    )
    .expect("fixture");
    check(&text, &Profile::built_in())
        .design
        .expect("a valid Design")
        .options
}

fn results(o: &DesignOption) -> (Value, String) {
    match run_option(o) {
        RunOutcome::Results { results, hash } => (results, hash),
        _ => panic!("{} has no results", o.name),
    }
}

#[test]
fn the_steel_list_re_cuts_the_panel_table() {
    for o in options_of("gas_igc_2g_exact.vdesign") {
        let (r, _) = results(&o);
        let st = &r["structure"];
        let rows = st["steel_list"].as_array().expect("a steel list");
        let net: f64 = rows.iter().map(|i| i["rule_net_t"].as_f64().unwrap()).sum();
        let panel = r["cost_masses"]["hull_panels"]["net_t"].as_f64().unwrap();
        assert!((st["groups"]["rule_net_t"].as_f64().unwrap() - net).abs() <= 1e-9 * net);
        // Issue #743: the shell, the double bottom and the bulkheads follow the hull's Sections
        // Table, and the end faces are plated, so the steel list differs from the panel table
        // there. Every other member re-cuts its panel exactly.
        let laid_out = [
            "SHELL: Bottom shell",
            "SHELL",
            "DECK: Strength deck",
            "SHELL: Inner bottom shell",
            "LONGITUDINAL: Centerline girder",
            "LONGITUDINAL: Side girder",
            "WEB_FRAME: Floor frame",
            "TRANSVERSAL_BULKHEAD",
        ];
        let members = r["cost_masses"]["hull_panels"]["members"]
            .as_array()
            .unwrap();
        let groups = st["groups"]["members"].as_array().unwrap();
        let mut same = 0;
        for (g, p) in groups.iter().zip(members) {
            assert_eq!(g["member"], p["member"]);
            assert_eq!(g["region"], p["region"]);
            if laid_out.contains(&p["member"].as_str().unwrap()) {
                continue;
            }
            let (a, b) = (
                g["rule_net_t"].as_f64().unwrap(),
                p["total_t"].as_f64().unwrap(),
            );
            assert!(
                (a - b).abs() <= 1e-9 * b.max(1.0),
                "{}: {a} vs {b}",
                p["member"]
            );
            same += 1;
        }
        assert!(same > 0);
        assert_eq!(
            groups.len(),
            members.len() + 1,
            "the end plating is its own group"
        );
        // The ends follow the hull, within a few per cent of the box the panel table weighs.
        assert!((net - panel).abs() <= 0.1 * panel, "{net} vs {panel}");
        let ordered = st["groups"]["ordered_t"].as_f64().unwrap();
        assert!(ordered >= net);
        let geometry = r["structure_geometry"].as_object().unwrap();
        assert_eq!(geometry.len(), rows.len());
        for i in rows {
            let pts = geometry[i["id"].as_str().unwrap()].as_array().unwrap();
            let n = if i["kind"] == "plate"
                || i["kind"] == "primary" && i["area_m2"].as_f64().unwrap() > 0.0
            {
                3
            } else {
                2
            };
            assert!(pts.len() >= n, "{}", i["id"]);
        }
        let p = &st["purchase"];
        let bought = p["bought_m2"].as_f64().unwrap();
        let net_m2 = p["net_m2"].as_f64().unwrap();
        assert!(((bought - net_m2) - st["groups"]["scrap_m2"].as_f64().unwrap()).abs() < 1e-6);
        assert!(st["openings"].as_str().unwrap().contains("not deducted"));
    }
}

#[test]
fn the_steel_csv_has_one_row_per_item_and_the_openings_note() {
    let o = &options_of("gas_igc_2g_exact.vdesign")[0];
    let (r, _) = results(o);
    let bytes = option_export("steel-csv", o, None).unwrap();
    let csv = String::from_utf8(bytes).unwrap();
    assert!(
        csv.lines()
            .any(|l| l.starts_with('#') && l.contains("not deducted"))
    );
    let body: Vec<&str> = csv.lines().filter(|l| !l.starts_with('#')).collect();
    assert!(body[0].starts_with("id,kind,member,region,"));
    assert_eq!(
        body.len(),
        r["structure"]["steel_list"].as_array().unwrap().len() + 1
    );
    // Commas inside a quoted field do not count.
    let fields = |row: &str| {
        let mut quoted = false;
        1 + row
            .chars()
            .filter(|c| {
                if *c == '"' {
                    quoted = !quoted;
                }
                *c == ',' && !quoted
            })
            .count()
    };
    let cols = fields(body[0]);
    for row in &body[1..] {
        assert_eq!(fields(row), cols, "{row}");
    }
}

#[test]
fn a_hull_with_no_plate_model_says_so() {
    let o = &options_of("dry_bulk_box_hold.vdesign")[0];
    let (r, _) = results(o);
    assert!(r["structure"].is_null());
    assert!(option_export("steel-csv", o, None).is_err());
}

#[test]
fn the_steel_list_is_outside_the_results_hash() {
    let o = &options_of("gas_igc_2g_exact.vdesign")[0];
    let (mut r, hash) = results(o);
    assert_eq!(results_hash(&r), hash);
    r["structure"] = Value::Null;
    r["structure_geometry"] = Value::Null;
    assert_eq!(results_hash(&r), hash);
}
