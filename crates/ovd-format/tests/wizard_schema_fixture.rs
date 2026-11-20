//! The wizard's unit tests (`app/scripts/wizard.test.mjs`) read a short copy of the input schema,
//! `app/scripts/fixtures/wizard-schema.json`: the inputs the wizard uses, each with the fields it reads.
//! This test fails when the copy differs from what the engine gives for the built-in Defaults Profile
//! (same profile, engine version and groups; every input, field and choice in the copy equal to the engine's),
//! so the copy cannot drift (issue #1080, spec 0017). The copy leaves out fields and choices the wizard
//! does not read, such as `basis` or most Vessel Classes; the test does not ask for them.
//!
//! To write the file again after the schema changes:
//! `OVD_WRITE_TEXT=1 cargo test -p ovd-format --test wizard_schema_fixture`. The inputs, fields and choices
//! stay as the file has them; their values come from the engine.
//! Label: property

use ovd_format::Profile;
use ovd_format::session::input_schema;
use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::{Map, Value};
use std::path::Path;

/// The file's form: one space of indent and no final newline, as the copy was first written.
fn render(schema: &Value) -> String {
    let mut out = Vec::new();
    let mut ser = Serializer::with_formatter(&mut out, PrettyFormatter::with_indent(b" "));
    schema.serialize(&mut ser).unwrap();
    String::from_utf8(out).unwrap()
}

/// The key that names an entry of a list: an input has a `name`, a choice a `value`.
fn identity(item: &Value) -> Option<(&'static str, &Value)> {
    ["name", "value"]
        .into_iter()
        .find_map(|k| Some((k, item.get(k)?)))
}

/// The engine's value cut to the shape of the copy: an object keeps the fields the copy has, a list of
/// named entries keeps the entries the copy has (in the copy's order), and anything else is the engine's
/// own. A field or entry the engine no longer has is left out, so the check fails.
fn cut_to(engine: &Value, copy: &Value) -> Value {
    match (engine, copy) {
        (Value::Object(theirs), Value::Object(held)) => Value::Object(
            held.keys()
                .filter_map(|k| Some((k.clone(), cut_to(theirs.get(k)?, &held[k]))))
                .collect::<Map<String, Value>>(),
        ),
        (Value::Array(theirs), Value::Array(held))
            if held.iter().all(|h| identity(h).is_some()) =>
        {
            Value::Array(
                held.iter()
                    .filter_map(|h| {
                        let (key, id) = identity(h)?;
                        let found = theirs.iter().find(|t| t.get(key) == Some(id))?;
                        Some(cut_to(found, h))
                    })
                    .collect(),
            )
        }
        _ => engine.clone(),
    }
}

#[test]
fn the_wizard_schema_fixture_is_the_engines_input_schema() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/scripts/fixtures/wizard-schema.json");
    let engine = input_schema(&Profile::built_in());
    let copy: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let want = cut_to(&engine, &copy);
    if std::env::var_os("OVD_WRITE_TEXT").is_some() {
        std::fs::write(&path, render(&want)).unwrap();
    }
    let have: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(
        have == want,
        "app/scripts/fixtures/wizard-schema.json is not the engine's input schema: run OVD_WRITE_TEXT=1 cargo test -p ovd-format --test wizard_schema_fixture"
    );
}
