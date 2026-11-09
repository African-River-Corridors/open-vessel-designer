//! Seam: the Arrangement Style input through a Design (issue #779, spec 0011 D3, D4, D6, sec. 4.2).
//!
//! Label: property. Each test states a rule: which style an Option takes, that a file saved before
//! keeps the container block, that the deckhouse is in the results of a deckhouse Option only, and
//! that the container block's results are what they were.

use ovd_format::vdesign::{ARRANGEMENT_STYLES, INPUTS};
use ovd_format::{
    DesignSession, Profile, RunOutcome, Source, check, fill_as_new, results_hash, run_option,
};
use serde_json::{Value, json};

/// Issue #925: a legacy fixture has no Service Area, which is now an error. This test is about something else, so
/// it reads the file with a river-only Service Area set, as `ovd fill --service-area river-only` does.
fn read_with_area(path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
    let text = std::fs::read_to_string(path)?;
    Ok(ovd_format::with_service_area(&text, "river-only").unwrap())
}

fn hand(service_area: Option<&str>, extra: Value) -> String {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    if let Some(a) = service_area {
        inputs["service_area"] = json!({ "value": a, "source": "typed" });
    }
    // Issue #1034: a river-only Option needs a number for both river limits (`extra` may give its own).
    if inputs["service_area"]["value"] == "river-only" {
        inputs["draught_limit_m"] = json!({ "value": 12.0, "source": "typed" });
        inputs["air_draft_limit_m"] = json!({ "value": 40.0, "source": "typed" });
    }
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
    .to_string()
}

fn style_of(text: &str, profile: &Profile) -> (Value, Source) {
    let d = fill_as_new(text, profile).design.unwrap();
    let i = d.options[0].input("arrangement_style").unwrap();
    (i.value.clone(), i.source)
}

fn results_of(text: &str) -> Value {
    let d = fill_as_new(text, &Profile::built_in()).design.unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_style_list_is_the_cores() {
    assert_eq!(
        ARRANGEMENT_STYLES,
        ovd_core::aft_arrangement::ARRANGEMENT_STYLES
    );
    let pos = |n: &str| INPUTS.iter().position(|s| s.name == n).unwrap();
    // The default is keyed by the Service Area, which comes first.
    assert!(pos("service_area") < pos("arrangement_style"));
}

#[test]
fn a_new_coastal_option_takes_the_deckhouse_and_a_river_only_one_the_container_block() {
    let p = Profile::built_in();
    let coastal = style_of(&hand(Some("coastal-and-river"), json!({})), &p);
    assert_eq!(coastal.0, json!("deckhouse"));
    assert_eq!(coastal.1, Source::Default);
    assert_eq!(
        style_of(&hand(Some("river-only"), json!({})), &p).0,
        json!("container_block")
    );
    assert_eq!(
        style_of(&hand(None, json!({})), &p).0,
        json!("container_block")
    );
}

#[test]
fn a_typed_style_is_kept() {
    let t = hand(
        Some("coastal-and-river"),
        json!({ "arrangement_style": "container_block" }),
    );
    let got = style_of(&t, &Profile::built_in());
    assert_eq!(got, (json!("container_block"), Source::Typed));
}

#[test]
fn a_file_saved_before_keeps_the_container_block_with_a_note() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = read_with_area(path).unwrap();
    let checked = check(&text, &Profile::built_in());
    let d = checked.design.unwrap();
    let i = d.options[0].input("arrangement_style").unwrap();
    assert_eq!(
        (i.value.clone(), i.source),
        (json!("container_block"), Source::Default)
    );
    assert!(
        checked
            .report
            .warnings
            .iter()
            .any(|w| w.contains("`arrangement_style`") && w.contains("results are unchanged")),
        "{:?}",
        checked.report.warnings
    );
}

#[test]
fn a_pushed_barge_holds_no_arrangement_style() {
    let text = hand(Some("coastal-and-river"), json!({})).replace("self_propelled", "pushed_barge");
    let d = fill_as_new(&text, &Profile::built_in()).design.unwrap();
    assert!(d.options[0].input("arrangement_style").is_none());
}

#[test]
fn a_session_gives_the_deckhouse_when_the_service_area_is_coastal() {
    let mut s = river_session();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s.set_input("Option 1", "service_area", json!("coastal-and-river"))
        .unwrap();
    let v = s.design().options[0]
        .input("arrangement_style")
        .unwrap()
        .value
        .clone();
    assert_eq!(v, json!("deckhouse"));
}

#[test]
fn a_deckhouse_option_has_a_deckhouse_a_fixed_wheelhouse_and_steel_items() {
    let r = results_of(&hand(
        Some("coastal-and-river"),
        json!({ "length_overall_m": 140.0, "beam_m": 29.0, "moulded_depth_m": 9.0 }),
    ));
    let aft = &r["drawing"]["aft_arrangement"];
    let d = &aft["deckhouse"];
    assert!(d["tiers"].as_i64().unwrap() >= 1, "{d}");
    assert_eq!(d["wheelhouse"], json!("fixed"));
    assert_eq!(aft["wheelhouse_lift_m"], json!(0.0));
    assert_eq!(
        aft["container_labels"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|l| l.as_str().unwrap().starts_with("ACCOM"))
            .count(),
        0
    );
    let items: Vec<&str> = r["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap())
        .collect();
    assert!(items.contains(&"deckhouse steel"), "{items:?}");
    assert!(!items.contains(&"accommodation unit") && !items.contains(&"lift column"));
    assert_eq!(
        r["stern_block_mass"]["deckhouse_steel"]["rate_status"],
        json!("unverified estimate")
    );
    // The cost line prices the deckhouse floor, and the container block's lump is not added.
    let area = r["cost_masses"]["stern_items"]["deckhouse_area_m2"]
        .as_f64()
        .unwrap();
    assert!(area > 0.0);
    assert!(
        (area - d["area_m2"].as_f64().unwrap()).abs() < 1e-9,
        "{area}"
    );
    // The lump keeps its wheelhouse and navigation share (an estimate), so they are still priced.
    let split = &r["cost"]["wheelhouse_accommodation_split"];
    assert_eq!(split["status"], json!("estimate"), "{split}");
    assert!(split["wheelhouse_and_navigation_priced"].as_f64().unwrap() > 0.0);
    // The deckhouse volume joins the gross tonnage.
    assert!(r["gross_tonnage"]["deckhouse_volume_m3"].as_f64().unwrap() > 0.0);
    assert!(
        r["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["ok"] == json!(true) || c["severity"] == json!("warning"))
    );
}

#[test]
fn an_air_draft_limit_makes_the_deckhouse_wheelhouse_elevating() {
    let r = results_of(&hand(
        Some("coastal-and-river"),
        json!({ "length_overall_m": 140.0, "beam_m": 29.0, "moulded_depth_m": 9.0, "air_draft_limit_m": 40.0 }),
    ));
    let d = &r["drawing"]["aft_arrangement"]["deckhouse"];
    assert_eq!(d["wheelhouse"], json!("elevating"));
    let items: Vec<&str> = r["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap())
        .collect();
    assert!(items.contains(&"lift column"), "{items:?}");
}

#[test]
fn the_container_block_has_no_deckhouse_key_and_the_style_moves_the_hash() {
    let base = hand(
        Some("coastal-and-river"),
        json!({ "arrangement_style": "container_block" }),
    );
    let block = results_of(&base);
    assert!(
        block["drawing"]["aft_arrangement"]
            .get("deckhouse")
            .is_none()
    );
    assert!(block["stern_block_mass"].get("deckhouse_steel").is_none());
    let house = results_of(&hand(Some("coastal-and-river"), json!({})));
    assert_ne!(results_hash(&block), results_hash(&house));
}

#[test]
fn a_coastal_file_saved_before_keeps_the_container_block_and_its_hashes() {
    // The coastal LNG example, with the `arrangement_style` lines taken out: the file as it was
    // before the input. A coastal file is the one a wrong default would change.
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut file: Value = serde_json::from_str(&read_with_area(path).unwrap()).unwrap();
    let mut removed = 0;
    for o in file["options"].as_array_mut().unwrap() {
        let inputs = o["inputs"].as_object_mut().unwrap();
        assert_eq!(inputs["service_area"]["value"], json!("coastal-and-river"));
        removed += usize::from(inputs.remove("arrangement_style").is_some());
    }
    assert!(removed > 0, "the example holds the input");
    let text = file.to_string();
    let checked = check(&text, &Profile::built_in());
    for o in &checked.design.unwrap().options {
        let i = o.input("arrangement_style").unwrap();
        assert_eq!(
            (i.value.clone(), i.source),
            (json!("container_block"), Source::Default)
        );
    }
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    for (option, status) in &opened.hash_status {
        assert_eq!(status, &ovd_format::HashStatus::Matches, "{option}");
    }
}

// Issue #887: the `stern_arrangement` summary for the Stern & crew view. It is outside the hash, and
// each figure is checked against the inputs typed here and the drawing block, not against itself.
const BIG: fn() -> Value =
    || json!({ "length_overall_m": 140.0, "beam_m": 29.0, "moulded_depth_m": 9.0 });

fn with(extra: Value) -> Value {
    let mut v = BIG();
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    v
}

#[test]
fn the_stern_arrangement_block_is_outside_the_hash() {
    assert!(ovd_format::UNHASHED_KEYS.contains(&"stern_arrangement"));
    let r = results_of(&hand(Some("coastal-and-river"), BIG()));
    let mut without = r.clone();
    without.as_object_mut().unwrap().remove("stern_arrangement");
    assert_eq!(results_hash(&without), results_hash(&r));
}

#[test]
fn a_deckhouse_wheelhouse_is_elevating_only_with_an_air_draft_limit_and_a_lift() {
    let area = Some("coastal-and-river");
    // No limit: fixed, and the reason says no limit is set.
    let r = results_of(&hand(area, BIG()));
    let s = &r["stern_arrangement"];
    assert_eq!(s["style"], "deckhouse");
    assert_eq!(
        s["tiers"],
        r["drawing"]["aft_arrangement"]["deckhouse"]["tiers"]
    );
    assert_eq!(
        (s["wheelhouse"].clone(), s["wheelhouse_basis"].clone()),
        (json!("fixed"), json!("no_air_draft_limit"))
    );
    // A limit and the default lift: elevating, by the limit.
    let r = results_of(&hand(area, with(json!({ "air_draft_limit_m": 40.0 }))));
    let s = &r["stern_arrangement"];
    assert_eq!(
        (s["wheelhouse"].clone(), s["wheelhouse_basis"].clone()),
        (json!("elevating"), json!("air_draft_limit"))
    );
    assert_eq!(
        r["drawing"]["aft_arrangement"]["deckhouse"]["wheelhouse"],
        s["wheelhouse"]
    );
    // A limit and a lift of zero: the limit is set, but nothing can lift, so it is fixed and not "no limit".
    let r = results_of(&hand(
        area,
        with(json!({ "air_draft_limit_m": 40.0, "wheelhouse_max_lift_m": 0.0 })),
    ));
    let s = &r["stern_arrangement"];
    assert_eq!(
        (s["wheelhouse"].clone(), s["wheelhouse_basis"].clone()),
        (json!("fixed"), json!("no_lift"))
    );
}

#[test]
fn a_container_block_wheelhouse_follows_the_type_and_the_lift_inputs() {
    let area = Some("river-only");
    let got = |extra: Value| {
        let r = results_of(&hand(area, with(extra)));
        let s = r["stern_arrangement"].clone();
        assert_eq!(s["style"], "container_block");
        assert!(s["tiers"].is_null());
        assert!(r["drawing"]["aft_arrangement"]["deckhouse"].is_null());
        (s["wheelhouse"].clone(), s["wheelhouse_basis"].clone())
    };
    assert_eq!(
        got(json!({ "wheelhouse_type": "fixed" })),
        (json!("fixed"), json!("fixed_input"))
    );
    assert_eq!(
        got(json!({ "wheelhouse_type": "elevating" })),
        (json!("elevating"), json!("lift_input"))
    );
    // Elevating by type, but a lift of zero: the engine cannot lift it, so the block says fixed.
    assert_eq!(
        got(json!({ "wheelhouse_type": "elevating", "wheelhouse_max_lift_m": 0.0 })),
        (json!("fixed"), json!("no_lift"))
    );
}

// Issue #887: the app's unit test reads `app/scripts/fixtures/deckhouse-run.json`, a real run of the
// deckhouse design of the e2e fixtures. It must be the run the engine gives today.
#[test]
fn the_apps_deckhouse_run_file_is_the_run_the_engine_gives() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text =
        read_with_area(root.join("app/e2e/fixtures/gas_deckhouse_self_propelled.vdesign")).unwrap();
    let d = check(&text, &Profile::built_in()).design.unwrap();
    // `run_design` is what `ovd run` calls; it settles the rooms and the crew, which `run_option` does not.
    let RunOutcome::Results { results, .. } = ovd_format::run_design(&d).remove(0).outcome else {
        panic!("the deckhouse design did not run");
    };
    let stored: Value = serde_json::from_str(
        &read_with_area(root.join("app/scripts/fixtures/deckhouse-run.json")).unwrap(),
    )
    .unwrap();
    for (k, v) in stored.as_object().unwrap() {
        assert_eq!(
            &results[k], v,
            "`{k}` is stale: run python3 app/scripts/fixtures/refresh_deckhouse_run.py <ovd>"
        );
    }
}

/// Issue #1034: a new river-only session needs a number for both river limits before it runs. These are high enough
/// not to bind, so no value of these tests moves.
fn river_session() -> ovd_format::DesignSession {
    let mut s =
        ovd_format::DesignSession::new_design_in(ovd_format::Profile::built_in(), "river-only")
            .unwrap();
    s.set_input("Option 1", "draught_limit_m", serde_json::json!(12.0))
        .unwrap();
    s.set_input("Option 1", "air_draft_limit_m", serde_json::json!(40.0))
        .unwrap();
    s
}
