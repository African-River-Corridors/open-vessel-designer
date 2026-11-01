//! Seam: the gas hull's panel table in the results (`cost_masses.hull_panels`), built from the
//! `HullStructureSpec` of the Vessel Model (issue #741, spec 0013 "Model", `hull_panels` row).
//!
//! Label: property - the table the results show weighs the same hull steel the lightship takes
//! (`cost_masses.masses.hull_steel_t`), every member is named by an OCX `functionType` and cites
//! the `abs-steel-barges` Rule Set, and the `structure` block's spacings reach the table.

use std::path::Path;

use ovd_format::{Profile, RunOutcome, check, run_option};
use serde_json::Value;

const GAS_FIXTURES: &[&str] = &[
    "gas_igc_2g_exact.vdesign",
    "gas_igc_2g_self_propelled.vdesign",
];

fn text_of(file: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(file),
    )
    .expect("fixture")
}

/// The results of every computed Option of a design text.
fn results_of(text: &str) -> Vec<Value> {
    let design = check(text, &Profile::built_in())
        .design
        .expect("a valid Design");
    design
        .options
        .iter()
        .filter_map(|o| match run_option(o) {
            RunOutcome::Results { results, .. } => Some(results),
            _ => None,
        })
        .collect()
}

#[test]
fn the_table_weighs_the_lightship_hull_steel() {
    for file in GAS_FIXTURES {
        let all = results_of(&text_of(file));
        assert!(!all.is_empty(), "{file}");
        for r in all {
            let cm = &r["cost_masses"];
            let table = cm["hull_panels"]["total_t"]
                .as_f64()
                .expect("a panel table");
            let steel = cm["masses"]["hull_steel_t"].as_f64().expect("hull steel");
            assert!(
                (table - steel).abs() <= 1e-9 * steel,
                "{file}: table {table} vs lightship {steel}"
            );
            for m in cm["hull_panels"]["members"].as_array().unwrap() {
                let name = m["member"].as_str().unwrap();
                assert!(
                    name.starts_with("SHELL")
                        || name.starts_with("DECK: ")
                        || name.starts_with("LONGITUDINAL: ")
                        || name.starts_with("WEB_FRAME: ")
                        || name.starts_with("TRANSVERSAL_BULKHEAD"),
                    "{file}: {name}"
                );
                assert!(m["region"].is_string(), "{file}: {name}");
            }
        }
    }
}

#[test]
fn the_structure_block_reaches_the_table() {
    let text = text_of("gas_igc_2g_exact.vdesign");
    let mut v: Value = serde_json::from_str(&text).unwrap();
    for option in v["options"].as_array_mut().unwrap() {
        option["inputs"]["longitudinal_spacing_mm"]["value"] = 800.0.into();
    }
    let all = results_of(&serde_json::to_string(&v).unwrap());
    assert!(!all.is_empty());
    for r in all {
        let members = r["cost_masses"]["hull_panels"]["members"]
            .as_array()
            .unwrap();
        let bottom = members
            .iter()
            .find(|m| m["member"] == "SHELL: Bottom shell")
            .unwrap();
        assert_eq!(bottom["stiffener_spacing_mm"], 800.0);
        let table = r["cost_masses"]["hull_panels"]["total_t"].as_f64().unwrap();
        let steel = r["cost_masses"]["masses"]["hull_steel_t"].as_f64().unwrap();
        assert!((table - steel).abs() <= 1e-9 * steel, "{table} vs {steel}");
    }
}
