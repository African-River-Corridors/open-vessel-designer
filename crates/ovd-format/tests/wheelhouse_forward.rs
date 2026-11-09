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
        "draught_limit_m": { "value": 12.0, "source": "typed" },
        "air_draft_limit_m": { "value": 40.0, "source": "typed" },
        // Issue #1034: a typed air-draft limit lets the wheelhouse lift; these tests are about a fixed one.
        "wheelhouse_max_lift_m": { "value": 0.0, "source": "typed" },
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
    assert!(f(&aft["deckhouse"]["x_aft_m"]) < f(&aft["deckhouse"]["x_fwd_m"]));
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
        r["warnings"].as_array().unwrap().iter().any(|w| w
            .as_str()
            .is_some_and(|w| w.contains("takes") && w.contains("of cargo length"))),
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
        let r = results_of(&design(
            110.0,
            11.4,
            6.0,
            json!({ "arrangement_style": style }),
        ));
        assert!(
            r["drawing"]["aft_arrangement"]
                .get("forward_house")
                .is_none()
        );
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
    assert!(
        (counted - deckhouse - house).abs() < 1e-6,
        "{counted} {deckhouse} {house}"
    );
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
        r["drawing"]["aft_arrangement"]["deckhouse"]["tiers"]
            .as_i64()
            .unwrap()
    };
    assert!(tiers("wheelhouse_forward") <= tiers("deckhouse"));
}

#[test]
fn the_forward_eye_is_checked_as_a_blind_distance_and_the_wing_view_is_reported() {
    let r = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }),
    ));
    // The Blind Distance is worked out whatever the sightline method: only the bow is ahead of the eye.
    let s = &r["sightline"];
    assert_eq!(f(&s["lift_needed_m"]), 0.0, "{s}");
    assert_eq!(s["lift_enough"], json!(true));
    assert!(f(&s["blind_distance_light_m"]) < 50.0, "{s}");
    // The wing view is in the stern block and the stern arrangement, with cameras an open item.
    let w = &r["drawing"]["aft_arrangement"]["wing_view"];
    // The wing is at the side of the house, a side deck (0.8 m) inside the hull side: it does not
    // see the full hull side, whatever the cargo does, and the result says by how much.
    assert_eq!(w["sees_full_hull_side"], json!(false), "{w}");
    assert!(
        (f(&w["hull_half_beam_m"]) - f(&w["wing_y_m"]) - 0.8).abs() < 1e-9,
        "{w}"
    );
    assert!((f(&w["short_by_m"]) - 0.8).abs() < 1e-9, "{w}");
    assert_eq!(w["cameras"], json!("open_item"));
    assert_eq!(r["stern_arrangement"]["rear_view"], *w);
    let checks = r["checks"].as_array().unwrap();
    let wing = checks
        .iter()
        .find(|c| c["name"] == json!("wheelhouse wing sees hull side"))
        .unwrap_or_else(|| panic!("the wing check is listed in {checks:?}"));
    assert_eq!(wing["ok"], w["sees_full_hull_side"]);
    assert_eq!(wing["severity"], json!("warning"));
}

#[test]
fn the_other_styles_have_no_wing_view() {
    let r = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "deckhouse" }),
    ));
    assert!(r["drawing"]["aft_arrangement"].get("wing_view").is_none());
    assert!(r["stern_arrangement"].get("rear_view").is_none());
}

#[test]
fn the_vent_mast_is_never_inside_the_forward_house_at_any_rake() {
    // Default rake (9.6 m on a 6 m depth: the house takes cargo length) and a long rake (it does not).
    for extra in [
        json!({ "arrangement_style": "wheelhouse_forward" }),
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }),
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 4.0 }),
    ] {
        let r = results_of(&design(110.0, 11.4, 6.0, extra.clone()));
        let house = f(&r["drawing"]["aft_arrangement"]["forward_house"]["x_aft_m"]);
        let mast = f(&r["results"]["vent_mast_x_m"]);
        assert!(
            mast <= house + 1e-9,
            "mast {mast} m is inside the house, which starts at {house} m ({extra})"
        );
    }
}

#[test]
fn the_forward_eye_is_checked_against_the_limit_of_the_stack_of_its_service_area() {
    let limit = |area: &str, method: &str| {
        let r = results_of(&design(
            110.0,
            11.4,
            6.0,
            json!({
                "arrangement_style": "wheelhouse_forward",
                "bow_rake_length_m": 20.0,
                "service_area": area,
                "sightline_method": method
            }),
        ));
        f(&r["sightline"]["limit_m"])
    };
    // River and estuary: ES-TRIN 7.02(2), 250 m. Coastal: SOLAS V/22.1.1 asks two lengths (220 m of a
    // 110 m ship), the least of the stack's limits. The margin method changes neither: the forward eye
    // is always checked as a distance.
    assert_eq!(limit("river-only", "margin"), 250.0);
    assert_eq!(limit("river-only", "blind_distance"), 250.0);
    assert_eq!(limit("coastal-and-river", "margin"), 220.0);
}

fn hazard(r: &Value) -> &Value {
    &r["stern_arrangement"]["hazard_study"]
}

fn reason_ids(r: &Value) -> Vec<&str> {
    hazard(r)["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect()
}

#[test]
fn a_forward_wheelhouse_on_a_river_needs_a_hazard_study_and_says_why() {
    let r = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }),
    ));
    assert_eq!(hazard(&r)["required"], json!(true));
    let ids = reason_ids(&r);
    for want in ["no_gas_vessel_in_service", "split_crew", "view_aft"] {
        assert!(ids.contains(&want), "{ids:?}");
    }
    // A river-only Service Area does not reach sea: no bow-motion reason, no such warning.
    assert!(!ids.contains(&"bow_motions"), "{ids:?}");
    assert!(
        !r["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().is_some_and(|w| w.contains("bow accelerations")))
    );
    // Every reason has its text.
    assert!(
        hazard(&r)["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["text"].as_str().is_some_and(|t| t.len() > 20))
    );
}

#[test]
fn a_forward_wheelhouse_where_the_sea_is_reached_has_the_bow_motion_warning_and_reason() {
    let r = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({
            "arrangement_style": "wheelhouse_forward",
            "bow_rake_length_m": 20.0,
            "service_area": "coastal-and-river"
        }),
    ));
    assert!(
        reason_ids(&r).contains(&"bow_motions"),
        "{:?}",
        reason_ids(&r)
    );
    let warned = r["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w.as_str().is_some_and(|w| w.contains("bow accelerations")));
    assert!(warned, "{}", r["warnings"]);
    // The style is not refused: the design runs and the flag is set.
    assert_eq!(hazard(&r)["required"], json!(true));
}

#[test]
fn the_other_styles_hold_no_hazard_study() {
    for style in ["container_block", "deckhouse"] {
        let r = results_of(&design(
            110.0,
            11.4,
            6.0,
            json!({ "arrangement_style": style }),
        ));
        assert_eq!(hazard(&r)["required"], json!(false), "{style}");
        assert_eq!(hazard(&r)["reasons"], json!([]), "{style}");
    }
}

#[test]
fn a_file_pinned_to_a_stack_with_no_blind_distance_rule_has_no_forward_check_and_says_so() {
    // The stored stack holds the OVD checks alone: no ES-TRIN or SOLAS Rule gives a Blind Distance limit.
    let ovd = ovd_format::rule_set::Library::built_in()
        .latest("ovd")
        .map(|s| (s.version.to_string(), s.hash.to_string()))
        .unwrap();
    let r = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({
            "arrangement_style": "wheelhouse_forward",
            "bow_rake_length_m": 20.0,
            "rule_sets": [{ "id": "ovd", "version": ovd.0, "hash": ovd.1, "source": "typed" }]
        }),
    ));
    assert!(
        r.get("sightline").is_none() || r["sightline"].is_null(),
        "{}",
        r["sightline"]
    );
    assert!(
        reason_ids(&r).contains(&"no_blind_distance_check"),
        "{:?}",
        reason_ids(&r)
    );
    let why = hazard(&r)["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == json!("no_blind_distance_check"))
        .unwrap()["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(why.starts_with("No Blind Distance check was run"), "{why}");
    // With the default stack the check is there and the reason is not.
    let normal = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }),
    ));
    assert!(!reason_ids(&normal).contains(&"no_blind_distance_check"));
}

fn outcome_of(extra: Value) -> RunOutcome {
    let text = design(110.0, 11.4, 6.0, extra);
    let d = fill_as_new(&text, &Profile::built_in()).design.unwrap();
    run_option(&d.options[0])
}

fn light_trim_by_bow(r: &Value) -> f64 {
    let c = r["stability"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["role"] == json!("ballast_departure"))
        .expect("the ballast departure condition");
    f(&c["draught_fwd_m"]) - f(&c["draught_aft_m"])
}

#[test]
fn moving_the_house_forward_moves_the_mass_and_the_trim_with_it() {
    let fwd = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }),
    ));
    let aft_style = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({ "arrangement_style": "deckhouse", "bow_rake_length_m": 20.0 }),
    ));
    let items = fwd["stern_block_mass"]["items"].as_array().unwrap();
    let item = |label: &str| items.iter().find(|i| i["label"] == json!(label)).unwrap();
    let house = &fwd["drawing"]["aft_arrangement"]["forward_house"];
    // The forward house steel and the wheelhouse steel stand over the forepeak, the machinery aft.
    assert!(f(&item("forward house steel")["lcg_m"]) >= f(&house["x_aft_m"]));
    assert!(f(&item("wheelhouse steel")["lcg_m"]) >= f(&house["x_aft_m"]));
    assert!(f(&item("deckhouse steel")["lcg_m"]) < f(&house["x_aft_m"]));
    // The block's centre of mass about the stern is the sum of its moments over its mass.
    let total: f64 = items.iter().map(|i| f(&i["mass_t"])).sum();
    assert!((total - f(&fwd["stern_block_mass"]["total_t"])).abs() < 1e-6);
    let moment: f64 = items.iter().map(|i| f(&i["mass_t"]) * f(&i["lcg_m"])).sum();
    let aft_moment: f64 = aft_style["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| f(&i["mass_t"]) * f(&i["lcg_m"]))
        .sum();
    let aft_total = f(&aft_style["stern_block_mass"]["total_t"]);
    assert!(
        moment / total > aft_moment / aft_total,
        "the block's centre did not move forward"
    );
    // The trim follows: with the house forward the ship in ballast floats more by the bow.
    assert!(
        light_trim_by_bow(&fwd) > light_trim_by_bow(&aft_style),
        "{} vs {}",
        light_trim_by_bow(&fwd),
        light_trim_by_bow(&aft_style)
    );
}

#[test]
fn the_views_draw_the_forward_house_and_the_vent_mast_distance_to_it() {
    let fwd = ovd_format::ga_views_of(&outcome_of(json!({
        "arrangement_style": "wheelhouse_forward",
        "bow_rake_length_m": 20.0
    })))
    .unwrap();
    let deck = ovd_format::ga_views_of(&outcome_of(json!({
        "arrangement_style": "deckhouse",
        "bow_rake_length_m": 20.0
    })))
    .unwrap();
    for (name, view) in [("elevation", &fwd.side_elevation), ("plan", &fwd.plan)] {
        let text = view.to_string();
        assert!(text.contains("FORWARD HOUSE"), "{name}");
        // The house is one more closed outline on the OBJECT layer than the other style draws.
        let outlines = |v: &Value| v.to_string().matches("\"layer\":\"OBJECT\"").count();
        let other = if name == "plan" {
            &deck.plan
        } else {
            &deck.side_elevation
        };
        assert!(outlines(view) > outlines(other), "{name}");
    }
    // The plan carries a horizontal dimension from the mast to the house.
    let dims = |v: &Value| v.to_string().matches("\"kind\":\"linear\"").count();
    assert!(dims(&fwd.plan) > dims(&deck.plan));
    assert!(!deck.plan.to_string().contains("FORWARD HOUSE"));
}

#[test]
fn the_hazard_words_say_what_is_true() {
    let sea = results_of(&design(
        110.0,
        11.4,
        6.0,
        json!({
            "arrangement_style": "wheelhouse_forward",
            "bow_rake_length_m": 20.0,
            "service_area": "coastal-and-river"
        }),
    ));
    let text_of = |r: &Value, id: &str| {
        hazard(r)["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == json!(id))
            .map(|x| x["text"].as_str().unwrap().to_string())
    };
    // The 1.2 is an estimate from the guidance formula, said so; no phrase reads as if ICLL 39 did not apply.
    let bow = text_of(&sea, "bow_motions").unwrap();
    assert!(
        bow.contains("estimate") && bow.contains("guidance formula"),
        "{bow}"
    );
    assert!(!bow.contains("ICLL"), "{bow}");
    let warning = sea["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|w| w.contains("bow accelerations"))
        .unwrap();
    assert!(
        warning.contains("estimate") && !warning.contains("ICLL"),
        "{warning}"
    );
}
