//! Seam: the `ovd` 3.2.0 reference row (issue #926). On a river-only Design the IGC 4.23.1.2 minimum
//! is shown beside the design pressure as a note: it never blocks, and on a coastal Design the IGC
//! row is the only one.
//!
//! Label: sourced - IGC Code 4.23.1.2 (MSC.370(93)); the limit is the hand-worked formula of
//! `.agents/artifacts/926/02-research.md` section 5, applied to the tank block of the run.

use ovd_format::{DesignSession, Profile, RunOutcome, Source};
use serde_json::{Value, json};

const ROW: &str = "ovd/4.23.1.2/min-design-pressure-reference";
const IGC_ROW: &str = "igc/4.23.1.2/min-design-pressure";

fn example() -> DesignSession {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    DesignSession::open(&text, Profile::built_in()).unwrap().0
}

/// The same hull as the reference design on a river-only Service Area, with the default stack of
/// today (`ovd` at its highest version).
fn river_twin() -> DesignSession {
    let reference = example();
    let option = &reference.design().options[0];
    let mut s = DesignSession::new_design_in(Profile::built_in(), "river-only").unwrap();
    for (name, stored) in &option.inputs {
        let skip = matches!(
            name.as_str(),
            "service_area" | "rule_sets" | "formation" | "cargo_containment"
        );
        if stored.source == Source::Typed && !skip {
            s.set_input("Option 1", name, stored.value.clone()).unwrap();
        }
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn results_of(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn rows<'a>(r: &'a Value, rule: &str) -> Vec<&'a Value> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["rule"] == rule && x["outcome"] != "not applicable")
        .collect()
}

/// Po of the tank a run built, worked by hand from its tank block (IGC 4.23.1.2): an oracle that
/// shares no code with the engine.
fn po_of(r: &Value) -> f64 {
    let t = &r["drawing"]["tank"];
    let (od, len) = (
        t["tank_outer_diameter_m"].as_f64().unwrap(),
        t["overall_tank_length_m"].as_f64().unwrap(),
    );
    let sigma = t["allowable_stress_mpa"].as_f64().unwrap();
    let rho = t["liquid_pressure_density_kg_m3"].as_f64().unwrap() / 1000.0;
    let c = od.max(0.75 * od).max(0.45 * len);
    let x = sigma / 55.0;
    // rho^1.5 as rho times its square root: no platform-dependent pow.
    0.2 + 0.00185 * x * x * c * rho * rho.sqrt()
}

/// Label: sourced - IGC 4.23.1.2, Po worked by hand from the tank block (research section 5)
#[test]
fn a_river_only_lng_tank_shows_the_igc_minimum_as_a_note_beside_its_pressure() {
    let r = results_of(&river_twin());
    let found = rows(&r, ROW);
    assert!(!found.is_empty(), "no reference row in the trace");
    for row in found {
        assert_eq!(row["value"], json!(0.4));
        // The limit is Po of the tank the run built.
        let t = &r["drawing"]["tank"];
        assert!((row["limit"].as_f64().unwrap() - po_of(&r)).abs() < 1e-9);
        assert_eq!(t["design_pressure_basis"], json!("default"));
        // A `warn` Rule that fails is reported as a warning and never blocks.
        assert_eq!(row["consequence"], json!("warn"));
        assert_eq!(row["outcome"], json!("warn"));
    }
    assert!(
        rows(&r, IGC_ROW).is_empty(),
        "the IGC row is not on river-only"
    );
    // The failed row does not change the verdict: the same Design with a pressure that meets the
    // minimum has the same failing rows and the same pass or fail.
    let mut met = river_twin();
    met.set_input("Option 1", "tank_design_pressure_mpa", json!(0.5))
        .unwrap();
    let (a, b) = (
        &r["rule_trace"]["verdict"],
        &results_of(&met)["rule_trace"]["verdict"],
    );
    assert_eq!(a["failing_rows"], b["failing_rows"]);
    assert_eq!(a["passes"], b["passes"]);
}

/// Label: property. On a coastal Design the reference row is not applicable; the IGC row is the one.
#[test]
fn on_a_coastal_design_the_reference_row_is_not_applicable() {
    let mut s = river_twin();
    s.set_input("Option 1", "service_area", json!("coastal-and-river"))
        .unwrap();
    let r = results_of(&s);
    assert!(rows(&r, ROW).is_empty());
    assert!(!rows(&r, IGC_ROW).is_empty());
}

/// Label: property. A river-only tank whose typed pressure meets the minimum passes the note.
#[test]
fn a_river_only_pressure_at_or_above_the_minimum_passes_the_note() {
    let mut s = river_twin();
    s.set_input("Option 1", "tank_design_pressure_mpa", json!(0.5))
        .unwrap();
    for row in rows(&results_of(&s), ROW) {
        assert_eq!(row["outcome"], json!("pass"));
    }
}

/// Label: sourced - IGC 4.23.1.2, Po from the hand formula; a boundary pair one step either side
#[test]
fn the_note_fails_a_millipascal_under_the_minimum_and_passes_a_millipascal_over_it() {
    let po = po_of(&results_of(&river_twin()));
    for (typed, outcome) in [(po - 0.001, "warn"), (po + 0.001, "pass")] {
        let mut s = river_twin();
        s.set_input("Option 1", "tank_design_pressure_mpa", json!(typed))
            .unwrap();
        let r = results_of(&s);
        // The tank is the same size whatever the pressure, so Po is the same.
        assert!((po_of(&r) - po).abs() < 1e-9);
        let found = rows(&r, ROW);
        assert!(!found.is_empty());
        for row in found {
            assert_eq!(row["outcome"], json!(outcome), "typed {typed}");
        }
    }
}
