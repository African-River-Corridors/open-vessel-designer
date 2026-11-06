//! Seam: one insulation thickness on a Design path (issue #919). A thickness typed on the 140 x 29 m
//! coastal LNG reference design (123 mm) is the one the steel diameter, the capacity result, the
//! heat leak and the insulation mass use. A 350 mm anywhere would fail these.
//!
//! Label: property

use ovd_core::gas_fuel::{UPPER_AMBIENT_AIR_C, tank_heat_leak_kw};
use ovd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

fn results_with(thickness_mm: f64) -> Value {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let mut s = DesignSession::open(&text, Profile::built_in()).unwrap().0;
    s.set_input("A basis", "tank_insulation_thickness_mm", json!(thickness_mm))
        .unwrap();
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_typed_thickness_is_the_one_every_result_uses() {
    let r = results_with(123.0);
    let t = &r["drawing"]["tank"];
    let num = |v: &Value| v.as_f64().unwrap();
    assert_eq!(t["insulation_thickness_mm"], json!(123.0));
    assert_eq!(r["geometry"]["insulation_thickness_mm"], json!(123.0));
    let od = num(&t["tank_outer_diameter_m"]);
    let steel = num(&t["tank_steel_outer_diameter_m"]);
    assert!((steel - (od - 2.0 * 0.123)).abs() < 1e-12, "{steel}");

    // The heat leak: the same radii and the conductivity in force.
    let (len, k) = (
        num(&t["cylinder_length_m"]),
        num(&t["insulation_conductivity_w_per_mk"]),
    );
    let n = num(&r["geometry"]["n_tanks_total"]);
    let delta = UPPER_AMBIENT_AIR_C - num(&r["boil_off"]["cargo_temperature_c"]);
    let want = n * tank_heat_leak_kw(steel / 2.0, od / 2.0, len, delta, k);
    let got = num(&r["boil_off"]["heat_leak_kw"]);
    assert!((got - want).abs() < 1e-9 * want, "{got} vs {want}");

    // The insulation mass: the outer surface (a cylinder and two hemispheres) times the thickness.
    assert_eq!(t["head_type"], "hemisphere");
    let ro = od / 2.0;
    let area = std::f64::consts::PI * (2.0 * ro * len + 4.0 * ro * ro);
    let rho = num(&t["insulation_density_kg_m3"]);
    let want_t = n * area * 0.123 * rho / 1000.0;
    let got_t = num(&r["cost_masses"]["masses"]["insulation_t"]);
    assert!((got_t - want_t).abs() < 1e-9 * want_t, "{got_t} vs {want_t}");
}

#[test]
fn a_thicker_layer_leaks_less_and_weighs_more() {
    let (thin, thick) = (results_with(100.0), results_with(200.0));
    let q = |r: &Value| r["boil_off"]["heat_leak_kw"].as_f64().unwrap();
    let m = |r: &Value| r["cost_masses"]["masses"]["insulation_t"].as_f64().unwrap();
    assert!(q(&thick) < q(&thin));
    assert!(m(&thick) > m(&thin));
}
