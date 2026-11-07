//! Seam: the Service Area is a key of a default table (issue #926). A row may name it; a row that
//! leaves it out fits both Service Areas; a profile whose `by` list has the three older keys still
//! loads and fits any Service Area.
//!
//! Label: property. Each test states a relation of the loader and of `table_default`; no number is
//! a printed result.

use ovd_format::Profile;
use serde_json::{Value, json};

const NAME: &str = "tank_design_pressure_mpa";

fn profile_with(edit: impl FnOnce(&mut Value)) -> Result<Profile, String> {
    let mut v: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    edit(&mut v);
    Profile::from_json(&v.to_string())
}

fn table(v: &mut Value) -> &mut Value {
    // The inputs sit at the top level or under `inputs`, whichever the built-in file uses.
    if v.get(NAME).is_some() {
        &mut v[NAME]["default_by_table"]
    } else {
        &mut v["inputs"][NAME]["default_by_table"]
    }
}

fn coastal_row() -> Value {
    json!({
        "when": { "service_area": "coastal-and-river", "cargo_containment": "type_c_tanks" },
        "value": 0.9,
        "status": "estimate",
        "basis": "a test row for the coastal Service Area only"
    })
}

fn context(area: Option<&str>) -> Vec<(&str, &str)> {
    let mut c = vec![
        ("cargo_containment", "type_c_tanks"),
        ("cargo", "LNG"),
        ("lpg_carriage_condition", ""),
    ];
    if let Some(a) = area {
        c.push(("service_area", a));
    }
    c
}

#[test]
fn a_row_that_names_the_service_area_loads_and_fits_only_that_area() {
    let profile = profile_with(|v| {
        table(v)["rows"]
            .as_array_mut()
            .unwrap()
            .insert(0, coastal_row());
    })
    .expect("a row may name the Service Area");
    let value = |area| {
        profile
            .table_default(NAME, &context(area))
            .map(|h| h.value.clone())
    };
    assert_eq!(value(Some("coastal-and-river")), Some(json!(0.9)));
    // River-only, and a context with no Service Area yet, fall through to the existing rows.
    assert_eq!(value(Some("river-only")), Some(json!(0.4)));
    assert_eq!(value(None), Some(json!(0.4)));
}

#[test]
fn a_profile_with_the_three_older_keys_still_loads_and_fits_any_service_area() {
    let profile = profile_with(|v| {
        let t = table(v);
        t["by"] = json!(["cargo_containment", "cargo", "lpg_carriage_condition"]);
        // An older profile has no row that names the Service Area.
        t["rows"]
            .as_array_mut()
            .unwrap()
            .retain(|r| r["when"].get("service_area").is_none());
    })
    .expect("an older three-key `by` list loads");
    for area in ["river-only", "coastal-and-river"] {
        assert_eq!(
            profile
                .table_default(NAME, &context(Some(area)))
                .map(|h| h.value.clone()),
            Some(json!(0.4)),
            "{area}"
        );
    }
}

#[test]
fn a_three_key_profile_may_not_name_the_service_area_in_a_row() {
    let err = profile_with(|v| {
        let t = table(v);
        t["by"] = json!(["cargo_containment", "cargo", "lpg_carriage_condition"]);
        t["rows"].as_array_mut().unwrap().insert(0, coastal_row());
    })
    .expect_err("a row that names the Service Area needs the four-key `by`");
    assert!(err.contains("service_area"), "{err}");
}
