//! Seam: the `wheelhouse_forward` Arrangement Style through a Design (issue #994).
//!
//! Label: property. Each test states a rule: the house stands forward of the cargo zone, the
//! machinery and accommodation stay aft, the other styles keep their results, and a forepeak that is
//! too short makes the house take cargo length.

use ovd_format::{Profile, RunOutcome, fill_as_new, results_hash, run_option};
use serde_json::{Value, json};

fn design(loa: f64, beam: f64, depth: f64, extra: Value) -> String {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "moulded_depth_m": { "value": depth, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
    .to_string()
}

fn results_of(text: &str) -> Value {
    let d = fill_as_new(text, &Profile::built_in()).design.unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

#[test]
fn a_river_vessel_has_its_house_forward_of_the_cargo_and_its_machinery_aft() {
    let loa = 110.0;
    let r = results_of(&design(
        loa,
        11.4,
        6.0,
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }),
    ));
    let aft = &r["drawing"]["aft_arrangement"];
    let house = &aft["forward_house"];
    assert!(house.is_object(), "{aft}");
    // Forward of the cargo zone: inside the bow void.
    let bow_void = f(&r["geometry"]["bow_void_length_m"]);
    assert!(f(&house["x_aft_m"]) >= loa - bow_void - 1e-9);
    assert!(f(&house["x_fwd_m"]) < loa);
    // Aft: the deckhouse over the machinery space, at the stern.
    assert_eq!(f(&aft["deckhouse"]["x_aft_m"]), 0.0);
    assert!(f(&aft["deckhouse"]["x_fwd_m"]) < f(&house["x_aft_m"]));
    // The wheelhouse is on the forward house and fixed.
    assert!(f(&aft["wheelhouse_x0_m"]) >= f(&house["x_aft_m"]));
    assert_eq!(f(&aft["wheelhouse_lift_m"]), 0.0);
    let s = &r["stern_arrangement"];
    assert_eq!(s["style"], json!("wheelhouse_forward"));
    assert_eq!(s["wheelhouse"], json!("fixed"));
    // No cargo length was lost on a hull whose forepeak holds the house.
    assert_eq!(f(&house["cargo_length_lost_m"]), 0.0);
}

#[test]
fn the_forward_house_gives_the_same_cargo_zone_as_the_deckhouse_when_the_forepeak_holds_it() {
    let base = json!({ "arrangement_style": "deckhouse", "bow_rake_length_m": 20.0 });
    let fwd = json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 });
    let a = results_of(&design(110.0, 11.4, 6.0, base));
    let b = results_of(&design(110.0, 11.4, 6.0, fwd));
    assert_eq!(
        a["geometry"]["cargo_zone_length_m"],
        b["geometry"]["cargo_zone_length_m"]
    );
    assert_ne!(results_hash(&a), results_hash(&b));
}

#[test]
fn a_short_forepeak_makes_the_house_take_cargo_length_and_the_results_say_so() {
    // A short rake: the house (8 m) and its clear deck (3 m) do not fit a 6 m forepeak.
    let short = json!({
        "arrangement_style": "wheelhouse_forward",
        "bow_rake_length_m": 6.0
    });
    let r = results_of(&design(110.0, 11.4, 6.0, short));
    let lost = f(&r["drawing"]["aft_arrangement"]["forward_house"]["cargo_length_lost_m"]);
    assert!(lost > 0.0, "{lost}");
    assert!(
        r["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().is_some_and(|w| w.contains("takes") && w.contains("of cargo length"))),
        "{}",
        r["warnings"]
    );
    // The same hull on the deckhouse style keeps the longer cargo zone.
    let d = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "deckhouse", "bow_rake_length_m": 6.0 }),
    ));
    let cargo = |v: &Value| f(&v["geometry"]["cargo_zone_length_m"]);
    assert!((cargo(&d) - cargo(&r) - lost).abs() < 1e-9);
}

#[test]
fn the_other_styles_have_no_forward_house() {
    for style in ["container_block", "deckhouse"] {
        let r = results_of(&design(110.0, 11.4, 6.0, json!({ "arrangement_style": style })));
        assert!(r["drawing"]["aft_arrangement"].get("forward_house").is_none());
        assert_eq!(r["stern_arrangement"]["style"], json!(style));
    }
}

#[test]
fn the_forward_house_joins_the_gross_tonnage_and_the_steel() {
    let r = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }),
    ));
    let aft = &r["drawing"]["aft_arrangement"];
    let house = f(&aft["forward_house"]["volume_m3"]);
    let deckhouse = f(&aft["deckhouse"]["volume_m3"]);
    // The enclosed volume of the two houses is what the gross tonnage takes as deckhouse volume.
    let counted = f(&r["gross_tonnage"]["deckhouse_volume_m3"]);
    assert!((counted - deckhouse - house).abs() < 1e-6, "{counted} {deckhouse} {house}");
    let items: Vec<&str> = r["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap())
        .collect();
    assert!(items.contains(&"forward house steel"), "{items:?}");
}

#[test]
fn the_aft_house_needs_no_extra_tiers_for_a_sightline_the_wheelhouse_forward_does_not_have() {
    // On the deckhouse style a fixed wheelhouse on the aft house must see over the cargo, which adds
    // tiers; forward, the eye is ahead of the cargo, so the aft house keeps only the tiers the
    // accommodation needs.
    let tiers = |style: &str| {
        let r = results_of(&design(
            110.0,
            11.4,
            6.0,
            json!({ "arrangement_style": style, "bow_rake_length_m": 20.0 }),
        ));
        r["drawing"]["aft_arrangement"]["deckhouse"]["tiers"].as_i64().unwrap()
    };
    assert!(tiers("wheelhouse_forward") <= tiers("deckhouse"));
}
