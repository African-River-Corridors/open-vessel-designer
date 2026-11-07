//! IGC 3.2.4.1 and 3.2.4.2 in `igc-2g` 4.0.0 (issue #913): the rows read the placed openings, one row
//! for each opening, and the worst opening governs. A hull with no placed openings gets rows that
//! are `not evaluated`, never `pass`.
//!
//! Label: sourced - IGC Code, MSC.370(93), 3.2.4.1 (entrances, air inlets and openings of
//! accommodation, service and machinery spaces and control stations do not face the cargo area; on a
//! side, at least 4 % of L, not less than 3 m and not more than 5 m, from the end of the deckhouse
//! that faces the cargo area) and 3.2.4.2 (windows facing the cargo area or in that zone are fixed;
//! wheelhouse doors may stand there only if the wheelhouse can be made gas-tight rapidly). The
//! maintainer reads 3.2.4.2 strictly: no wheelhouse door may face the cargo area, gas-tight or not
//! (issue #913, pull request #951).

use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

const FRONT_DOOR: &str = "igc/3.2.4.1/no-opening-faces-cargo";
const SIDE_ENTRANCE: &str = "igc/3.2.4.1/side-opening-setback-placed";
const AIR_INLET: &str = "igc/3.2.4.1/air-inlet-setback-placed";
const WINDOWS: &str = "igc/3.2.4.2/windows-fixed";
const WHEELHOUSE: &str = "igc/3.2.4.2/wheelhouse-gas-tight";
const NEW_ROWS: [&str; 5] = [FRONT_DOOR, SIDE_ENTRANCE, AIR_INLET, WINDOWS, WHEELHOUSE];
const OLD_ROWS: [&str; 4] = [
    "igc/3.2.4.1/front-openings",
    "igc/3.2.4.1/side-opening-distance",
    "igc/3.2.4.1/opening-setback",
    "igc/3.2.4.1/air-inlet-setback",
];

/// The first Option of the worked 140 x 29 m coastal LNG reference design, in the deckhouse style
/// (the example itself is a container-block stern), with `extra` inputs typed on it.
fn reference_text(extra: Value) -> String {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut option = file["options"][0].clone();
    // The example stores the stack it was run with: drop it, so the Option takes the default stack
    // (each set at its highest built-in version).
    option["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("rule_sets");
    option["inputs"]["arrangement_style"] = json!({ "value": "deckhouse", "source": "typed" });
    for (k, v) in extra.as_object().unwrap() {
        option["inputs"][k] = json!({ "value": v, "source": "typed" });
    }
    file["options"] = json!([option]);
    file.to_string()
}

fn run_text(text: String) -> Value {
    let filled = fill_as_new(&text, &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn reference(extra: Value) -> Value {
    run_text(reference_text(extra))
}

fn rows<'a>(results: &'a Value, id: &str) -> Vec<&'a Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id && r["outcome"] != "not applicable")
        .collect()
}

fn outcomes(results: &Value, id: &str) -> Vec<String> {
    rows(results, id)
        .iter()
        .map(|r| r["outcome"].as_str().unwrap().to_string())
        .collect()
}

/// x of the deckhouse front, m forward of the stern: an aft-end opening of the default layout is
/// `distance_from_front_m` behind it.
fn x_front(default_run: &Value) -> f64 {
    let list = default_run["stern_arrangement"]["openings"]
        .as_array()
        .unwrap();
    let o = list.iter().find(|o| o["face"] == "aft").unwrap();
    o["x_m"].as_f64().unwrap() + o["distance_from_front_m"].as_f64().unwrap()
}

fn typed_row(name: &str, kind: &str, space: &str, face: &str, x: f64, extra: Value) -> Value {
    let mut row = json!({
        "kind": kind, "space": space, "face": face,
        "x_m": x, "y_m": 0.0, "z_m": 7.0, "name": name
    });
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

#[test]
fn the_default_layout_of_the_reference_design_meets_the_rows_and_the_typed_input_rows_do_not_apply()
{
    let r = reference(json!({}));
    assert!(
        r["rule_trace"]["rule_sets"]
            .to_string()
            .contains("igc-2g@4.0.0"),
        "{}",
        r["rule_trace"]["rule_sets"]
    );
    // The four typed-input rows are kept for a container-block stern only: on a deckhouse they are `not applicable`.
    for id in OLD_ROWS {
        assert!(rows(&r, id).is_empty(), "{id}");
    }
    for id in NEW_ROWS {
        assert!(outcomes(&r, id).iter().all(|o| o == "pass"), "{id}");
    }
    // Every entrance and inlet of the layout is on the aft end or in the wheelhouse: a row each for
    // the cargo-face rule. The layout has no side entrance, so no setback row (the wheelhouse side
    // doors and windows are gas-tight, so that rule has no object either).
    assert!(outcomes(&r, FRONT_DOOR).len() >= 6);
    assert!(outcomes(&r, SIDE_ENTRANCE).is_empty());
    assert!(outcomes(&r, WHEELHOUSE).is_empty());
}

#[test]
fn each_side_entrance_has_its_own_row_and_the_reference_design_passes_at_the_zone_edge() {
    let probe = reference(json!({}));
    let xf = x_front(&probe);
    // L of the reference hull is over 125 m, so the zone is 5.0 m: 5.00 m passes, 6 m passes.
    let doors = [
        typed_row(
            "door at 5 m",
            "entrance",
            "accommodation",
            "side",
            xf - 5.0,
            json!({}),
        ),
        typed_row(
            "door at 6 m",
            "entrance",
            "machinery",
            "side",
            xf - 6.0,
            json!({}),
        ),
        typed_row(
            "door aft",
            "entrance",
            "accommodation",
            "aft",
            0.0,
            json!({}),
        ),
    ];
    let r = reference(json!({ "deckhouse_openings": doors }));
    assert_eq!(
        r["stern_arrangement"]["opening_zone_m"].as_f64().unwrap(),
        5.0
    );
    assert_eq!(outcomes(&r, SIDE_ENTRANCE), ["pass", "pass"]);
    assert_eq!(outcomes(&r, FRONT_DOOR), ["pass", "pass", "pass"]);
}

#[test]
fn a_side_entrance_inside_the_zone_fails_and_the_failing_row_names_the_door() {
    let xf = x_front(&reference(json!({})));
    let doors = [
        typed_row(
            "far door",
            "entrance",
            "accommodation",
            "side",
            xf - 8.0,
            json!({}),
        ),
        typed_row(
            "near door",
            "entrance",
            "accommodation",
            "side",
            xf - 1.0,
            json!({}),
        ),
    ];
    let r = reference(json!({ "deckhouse_openings": doors }));
    let side = rows(&r, SIDE_ENTRANCE);
    assert_eq!(side.len(), 2);
    let near = side.iter().find(|x| x["object"] == "near door").unwrap();
    let far = side.iter().find(|x| x["object"] == "far door").unwrap();
    assert_eq!(near["outcome"], "fail");
    assert_eq!(far["outcome"], "pass");
    // Only the worst opening of the Rule is the governing row.
    assert_eq!(near["governing"], true);
    assert_eq!(far["governing"], false);
    assert!(near["margin"].as_f64().unwrap() < 0.0 && far["margin"].as_f64().unwrap() > 0.0);
}

#[test]
fn an_accommodation_door_on_the_front_fails_and_the_failing_row_names_it() {
    let doors = [
        typed_row(
            "aft door",
            "entrance",
            "accommodation",
            "aft",
            0.0,
            json!({}),
        ),
        typed_row(
            "front door",
            "entrance",
            "accommodation",
            "front",
            0.0,
            json!({}),
        ),
    ];
    let r = reference(json!({ "deckhouse_openings": doors }));
    let list = rows(&r, FRONT_DOOR);
    assert_eq!(list.len(), 2);
    let aft = list.iter().find(|x| x["object"] == "aft door").unwrap();
    let front = list.iter().find(|x| x["object"] == "front door").unwrap();
    assert_eq!(aft["outcome"], "pass");
    assert_eq!(front["outcome"], "fail");
}

#[test]
fn a_wheelhouse_door_on_the_front_fails_even_when_gas_tight() {
    // The maintainer's strict reading of IGC 3.2.4.2.
    let door = typed_row(
        "wheelhouse door",
        "entrance",
        "wheelhouse",
        "front",
        0.0,
        json!({"gas_tight": true}),
    );
    let r = reference(json!({ "deckhouse_openings": [door] }));
    assert_eq!(outcomes(&r, FRONT_DOOR), ["fail"]);
}

#[test]
fn a_gas_tight_wheelhouse_side_door_passes_and_one_not_gas_tight_fails_in_the_zone() {
    let xf = x_front(&reference(json!({})));
    let tight = typed_row(
        "tight",
        "entrance",
        "wheelhouse",
        "side",
        xf - 1.0,
        json!({"gas_tight": true}),
    );
    let loose = typed_row(
        "loose",
        "entrance",
        "wheelhouse",
        "side",
        xf - 1.0,
        json!({"gas_tight": false}),
    );
    let r = reference(json!({ "deckhouse_openings": [tight] }));
    assert!(outcomes(&r, WHEELHOUSE).is_empty());
    assert_eq!(outcomes(&r, FRONT_DOOR), ["pass"]);
    let r = reference(json!({ "deckhouse_openings": [loose] }));
    assert_eq!(outcomes(&r, WHEELHOUSE), ["fail"]);
}

#[test]
fn a_front_opening_non_gas_tight_wheelhouse_window_fails() {
    // IGC 3.2.4.2: a wheelhouse window may open within the limits only if the wheelhouse can be made
    // gas-tight; on the front it is at distance 0, so it fails when not designed so, and passes when it is.
    let window = |tight: bool| {
        typed_row(
            "wheelhouse window",
            "window",
            "wheelhouse",
            "front",
            0.0,
            json!({"openable": true, "gas_tight": tight}),
        )
    };
    let r = reference(json!({ "deckhouse_openings": [window(false)] }));
    assert_eq!(outcomes(&r, WHEELHOUSE), ["fail"]);
    let r = reference(json!({ "deckhouse_openings": [window(true)] }));
    assert!(outcomes(&r, WHEELHOUSE).is_empty());
    // A fixed one is not covered.
    let fixed = typed_row(
        "fixed",
        "window",
        "wheelhouse",
        "front",
        0.0,
        json!({"openable": false}),
    );
    let r = reference(json!({ "deckhouse_openings": [fixed] }));
    assert!(outcomes(&r, WHEELHOUSE).is_empty());
}

#[test]
fn only_the_worst_opening_of_a_rule_is_governing_and_no_other_rule_changes() {
    let xf = x_front(&reference(json!({})));
    let doors = [
        typed_row(
            "a",
            "entrance",
            "accommodation",
            "side",
            xf - 9.0,
            json!({}),
        ),
        typed_row(
            "b",
            "entrance",
            "accommodation",
            "side",
            xf - 6.0,
            json!({}),
        ),
        typed_row(
            "c",
            "entrance",
            "accommodation",
            "side",
            xf - 7.0,
            json!({}),
        ),
    ];
    let r = reference(json!({ "deckhouse_openings": doors }));
    let side = rows(&r, SIDE_ENTRANCE);
    let governing: Vec<&str> = side
        .iter()
        .filter(|x| x["governing"] == true)
        .map(|x| x["object"].as_str().unwrap())
        .collect();
    assert_eq!(governing, ["b"], "the smallest margin governs");
    // Rows of other objects (the tanks, say) keep the kernel's marking: one governing row each.
    let tank_rows: Vec<&Value> = r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["object_type"] != "opening" && x["governing"] == true)
        .collect();
    assert!(!tank_rows.is_empty());
}

#[test]
fn a_container_block_option_has_the_new_rows_not_evaluated_and_is_checked_on_its_typed_inputs() {
    // The typed distances at the 5 m the rule never exceeds, and no opening on the front.
    let ok = json!({
        "arrangement_style": "container_block", "entrance_distance_m": 5.0,
        "engine_room_vent_inlet_distance_m": 5.0, "deckhouse_side_opening_distance_m": 5.0,
        "deckhouse_front_opening": false
    });
    let r = run_text(reference_text(ok));
    assert!(r["stern_arrangement"].get("openings").is_none());
    for id in NEW_ROWS {
        assert!(outcomes(&r, id).iter().all(|x| x != "pass"), "{id}");
    }
    for id in [FRONT_DOOR, SIDE_ENTRANCE] {
        assert_eq!(outcomes(&r, id), ["not evaluated"], "{id}");
    }
    // The four typed-input rows of 3.3.0 still check what the Option types (design choice B, #913).
    for id in OLD_ROWS {
        assert_eq!(outcomes(&r, id), ["pass"], "{id}");
    }
    // 2.5 m of entrance distance met the old inland 2.00 m but not IGC 3.2.4.1 (at least 3 m; 5 m at L = 140 m).
    let r = run_text(reference_text(
        json!({ "arrangement_style": "container_block", "entrance_distance_m": 2.5 }),
    ));
    assert_eq!(outcomes(&r, "igc/3.2.4.1/opening-setback"), ["fail"]);
}
