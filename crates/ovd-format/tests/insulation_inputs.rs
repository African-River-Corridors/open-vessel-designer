//! Seam: the insulation's conductivity and density are inputs (issue #919). The heat leak (and so
//! the boil-off) reads the conductivity, the insulation mass reads the density, and both default by
//! the Cargo, as the insulation type does. A file saved before them keeps the values the engine
//! used.
//!
//! The example is the 140 x 29 m coastal LNG reference design, Option "A basis".
//!
//! Label: property (a typed value scales the result it should scale), except where a test says
//! `sourced`.

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check};
use serde_json::{Value, json};

const CONDUCTIVITY: &str = "tank_insulation_conductivity_w_per_mk";
const DENSITY: &str = "tank_insulation_density_kg_m3";

fn example_text() -> String {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn example() -> DesignSession {
    DesignSession::open(&example_text(), Profile::built_in())
        .unwrap()
        .0
}

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn heat_leak_kw(r: &Value) -> f64 {
    r["boil_off"]["heat_leak_kw"].as_f64().unwrap()
}

fn insulation_t(r: &Value) -> f64 {
    r["cost_masses"]["masses"]["insulation_t"].as_f64().unwrap()
}

#[test]
fn a_typed_conductivity_scales_the_heat_leak() {
    let base = results(&example());
    let mut s = example();
    s.set_input("A basis", CONDUCTIVITY, json!(0.052)).unwrap();
    let typed = results(&s);
    let ratio = heat_leak_kw(&typed) / heat_leak_kw(&base);
    assert!(
        (ratio - 2.0).abs() < 1e-9,
        "the heat leak is linear in k: {ratio}"
    );
    assert_eq!(typed["boil_off"]["conductivity_w_per_mk"], json!(0.052));
    // The holding time reads the same conductivity.
    assert_eq!(
        typed["holding_time"]["tank"]["conductivity_w_per_mk"],
        json!(0.052)
    );
    // The mass does not read it.
    assert_eq!(insulation_t(&typed), insulation_t(&base));
}

#[test]
fn a_typed_density_scales_the_insulation_mass() {
    let base = results(&example());
    let mut s = example();
    let before = s.design().options[0].input(DENSITY).unwrap().value.clone();
    s.set_input("A basis", DENSITY, json!(2.0 * before.as_f64().unwrap()))
        .unwrap();
    let typed = results(&s);
    let ratio = insulation_t(&typed) / insulation_t(&base);
    assert!((ratio - 2.0).abs() < 1e-9, "{ratio}");
    // The heat leak does not read it.
    assert_eq!(heat_leak_kw(&typed), heat_leak_kw(&base));
}

/// Label: sourced. Lee et al., J. Mar. Sci. Eng. 2021, 9, 36, Table 3 (k = 0.026 W/m.K for foam of
/// 96 kg/m3 at about 215 K), and the hand check of the example's tank in issue #918: OD 11.0 m, steel
/// OD 10.34 m, 330 mm, cylinder 37.904 m, dT 207 K, giving 26.54 kW per tank and 746.85 kg/h for four.
#[test]
fn the_examples_heat_leak_and_boil_off_are_the_hand_check() {
    let r = results(&example());
    let n = r["geometry"]["n_tanks_total"].as_f64().unwrap();
    assert_eq!(n, 4.0);
    let per_tank = heat_leak_kw(&r) / n;
    assert!((per_tank - 26.54).abs() < 0.01, "{per_tank}");
    let boil = r["boil_off"]["boil_off_kg_per_h"].as_f64().unwrap();
    assert!((boil - 746.85).abs() < 0.01, "{boil}");
}

#[test]
fn the_defaults_are_inputs_marked_default_with_a_basis() {
    let s = example();
    for name in [CONDUCTIVITY, DENSITY] {
        let i = s.design().options[0].input(name).expect(name);
        assert!(i.value.as_f64().is_some(), "{name}");
        assert_eq!(i.source, Source::Default, "{name}");
    }
}

#[test]
fn a_file_saved_before_the_inputs_keeps_the_values_the_engine_used() {
    // A version 3 gas file, written before these inputs: 0.026 W/m.K and 60 kg/m3, marked default,
    // not the profile's 96 kg/m3 (the maintainer's decision on #919 moves new Designs only).
    let path = format!(
        "{}/tests/legacy/v3_gas_before_739.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let text = ovd_format::with_service_area(&text, "river-only").unwrap();
    let (s, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert!(opened.report.errors.is_empty(), "{:?}", opened.report);
    let o = &s.design().options[0];
    for (name, want) in [(CONDUCTIVITY, 0.026), (DENSITY, 60.0)] {
        let i = o.input(name).unwrap();
        assert_eq!(
            (i.value.clone(), i.source),
            (json!(want), Source::Default),
            "{name}"
        );
    }
}

/// Label: snapshot (printed by the code). The default density is 96 kg/m3 (the maintainer's decision
/// on #919; the foam of the conductivity): the example's insulation, 133.8 t at 60 kg/m3, is 214.2 t
/// (#918: about +80 t).
#[test]
fn the_examples_insulation_mass_is_214_t_at_the_default_density() {
    let s = example();
    assert_eq!(
        s.design().options[0].input(DENSITY).unwrap().value,
        json!(96.0)
    );
    let m = insulation_t(&results(&s));
    assert!((m - 214.157).abs() < 0.01, "{m}");
    assert!((m - 133.848 - 80.3).abs() < 0.1, "about +80 t: {m}");
}

fn with_typed(f: impl Fn(&mut Value)) -> Value {
    let mut d: Value = serde_json::from_str(&example_text()).unwrap();
    f(&mut d["options"][0]["inputs"]);
    d
}

fn warned(d: &Value) -> bool {
    let c = check(&d.to_string(), &Profile::built_in());
    assert!(c.report.errors.is_empty(), "{:?}", c.report.errors);
    c.report
        .warnings
        .iter()
        .any(|m| m.contains("still hold the defaults for panel"))
}

#[test]
fn a_typed_type_other_than_the_default_warns_until_the_values_are_typed_too() {
    let typed = |v: Value| json!({ "value": v, "source": "typed" });
    // The Cargo's default type is `panel`: the same type typed gives no warning.
    assert!(!warned(&with_typed(
        |i| i["tank_insulation_type"] = typed(json!("panel"))
    )));
    // Another type, with k and rho still the defaults of `panel`.
    assert!(warned(&with_typed(
        |i| i["tank_insulation_type"] = typed(json!("pu_foam"))
    )));
    // One of the two typed: the other is still the default.
    assert!(warned(&with_typed(|i| {
        i["tank_insulation_type"] = typed(json!("pu_foam"));
        i[CONDUCTIVITY] = typed(json!(0.03));
    })));
    // Both typed: the designer has given the values of the typed type.
    assert!(!warned(&with_typed(|i| {
        i["tank_insulation_type"] = typed(json!("pu_foam"));
        i[CONDUCTIVITY] = typed(json!(0.03));
        i[DENSITY] = typed(json!(40.0));
    })));
}

/// Label: sourced. The hand check of the example's tank in issue #918: U = 0.0759 W/m2.K on the outer
/// area of 1,690 m2 (cylinder 37.904 m, OD 11.0 m, two heads as one sphere), 26.54 kW per tank;
/// boil-off 0.27 to 0.29 %/day of the cargo.
#[test]
fn the_results_carry_the_u_value_and_the_heat_leak_per_tank() {
    let r = results(&example());
    let b = &r["boil_off_detail"];
    let get = |k: &str| b[k].as_f64().unwrap_or_else(|| panic!("{k}: {b}"));
    assert!((get("u_value_w_per_m2k") - 0.0759).abs() < 1e-4);
    assert!((get("heat_leak_per_tank_kw") - 26.54).abs() < 0.01);
    assert!((get("tank_outer_area_m2") - 1690.0).abs() < 1.0);
    let pct = get("boil_off_pct_per_day");
    assert!((0.27..0.29).contains(&pct), "{pct}");
    // The per-tank figure and the total agree.
    assert!((get("heat_leak_per_tank_kw") * 4.0 - heat_leak_kw(&r)).abs() < 1e-9);
}

#[test]
fn the_u_value_does_not_depend_on_the_conductivity_it_is_made_of_but_follows_it() {
    let mut s = example();
    let base = results(&s)["boil_off_detail"]["u_value_w_per_m2k"]
        .as_f64()
        .unwrap();
    s.set_input("A basis", CONDUCTIVITY, json!(0.052)).unwrap();
    let doubled = results(&s)["boil_off_detail"]["u_value_w_per_m2k"]
        .as_f64()
        .unwrap();
    assert!((doubled / base - 2.0).abs() < 1e-9);
}

/// The U-value and the other figures are outside the results hash: the example's stored hashes still
/// match, and the hash does not read them.
#[test]
fn the_new_figures_are_unhashed() {
    let (_, opened) = DesignSession::open(&example_text(), Profile::built_in()).unwrap();
    assert!(
        opened
            .hash_status
            .iter()
            .all(|(_, h)| *h == HashStatus::Matches),
        "{:?}",
        opened.hash_status
    );
    let mut r = results(&example());
    let before = ovd_format::results_hash(&r);
    r["boil_off_detail"]["u_value_w_per_m2k"] = json!(123.0);
    assert_eq!(ovd_format::results_hash(&r), before);
    r["boil_off"]["heat_leak_kw"] = json!(1.0);
    assert_ne!(ovd_format::results_hash(&r), before);
}
