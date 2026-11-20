//! The texts the input schema gives a designer (issue #1098): each input's basis, which the app shows under
//! "About this input", each choice's description, which the wizard shows, and each group note. They are
//! written for a vessel designer, not for a contributor: no code path (`::`), no issue number (`#` and
//! digits), no Rule Set id with a version (`@1.2.3`), no spec number and no source file of the repository.
//! The test reads the schema the engine gives for the built-in Defaults Profile, so a new text is covered.
//! The four required choices (Cargo, Formation, Hull Form, Service Area) each carry a basis, so each has
//! an "About this input" control.
//! Label: property

use ovd_format::Profile;
use ovd_format::session::input_schema;
use serde_json::Value;

/// The keys whose string values are texts for the designer.
const TEXT_KEYS: [&str; 4] = ["basis", "description", "help", "note"];

/// Every designer text in `v`, with the path that leads to it.
fn texts<'a>(v: &'a Value, path: String, out: &mut Vec<(String, &'a str)>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let p = format!("{path}.{k}");
                match x {
                    Value::String(s) if TEXT_KEYS.contains(&k.as_str()) => out.push((p, s)),
                    _ => texts(x, p, out),
                }
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                let key = x
                    .get("name")
                    .or_else(|| x.get("value"))
                    .and_then(Value::as_str)
                    .map_or(i.to_string(), str::to_string);
                texts(x, format!("{path}[{key}]"), out);
            }
        }
        _ => {}
    }
}

/// The first fault a designer should not meet in `s`, if any.
fn fault(s: &str) -> Option<&'static str> {
    let b = s.as_bytes();
    let digit_at = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    if s.contains("::") {
        return Some("a code path (::)");
    }
    for (i, &c) in b.iter().enumerate() {
        if c == b'#' && digit_at(i + 1) {
            return Some("an issue number (#n)");
        }
        // `@1.2.3`: a Rule Set id with its version.
        if c == b'@' && digit_at(i + 1) {
            let rest: String = s[i + 1..]
                .chars()
                .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
                .collect();
            if rest.trim_end_matches('.').matches('.').count() >= 2 {
                return Some("a Rule Set id with a version (@x.y.z)");
            }
        }
    }
    let lower = s.to_lowercase();
    if lower.contains("issue ") && lower.split("issue ").skip(1).any(|t| t.starts_with(|c: char| c.is_ascii_digit())) {
        return Some("an issue number (issue n)");
    }
    if lower.split("spec ").skip(1).any(|t| t.len() >= 4 && t[..4].bytes().all(|c| c.is_ascii_digit())) {
        return Some("a spec number (spec nnnn)");
    }
    for ext in [".rs", ".py", ".yaml", ".yml", ".md"] {
        if s.split(|c: char| c.is_whitespace() || "(),;".contains(c))
            .any(|w| w.ends_with(ext) && w.contains('/'))
        {
            return Some("a source file of the repository");
        }
    }
    None
}

#[test]
fn no_text_for_a_designer_holds_a_code_path_an_issue_number_or_a_rule_set_version() {
    let schema = input_schema(&Profile::built_in());
    let mut all = Vec::new();
    texts(&schema, String::new(), &mut all);
    assert!(all.len() > 100, "the schema gave only {} texts", all.len());
    let bad: Vec<String> = all
        .iter()
        .filter_map(|(p, s)| fault(s).map(|f| format!("{p}: {f}: {s}")))
        .collect();
    assert!(bad.is_empty(), "{} text(s) for programmers:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn the_check_finds_each_fault() {
    assert!(fault("see liquid_bulk::crude_grade").is_some());
    assert!(fault("the tug sizing (#359)").is_some());
    assert!(fault("the stack es-trin-2025@2.1.0").is_some());
    assert!(fault("(issue 1034)").is_some());
    assert!(fault("spec 0008, Hull Forms").is_some());
    assert!(fault("calibration/factors.yaml").is_some());
    assert!(fault("ES-TRIN 2025/1 Article 4.01(1), at least 300 mm; 0.3 m").is_none());
    assert!(fault("UN Treaty Series vol. 1291, No. I-21264").is_none());
}

#[test]
fn the_four_required_choices_each_say_what_they_are() {
    let schema = input_schema(&Profile::built_in());
    for name in ["cargo", "formation", "hull_form", "service_area"] {
        let input = schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap_or_else(|| panic!("{name} is not in the schema"));
        let basis = input["basis"].as_str().unwrap_or("");
        assert!(basis.len() > 20, "{name} has no text for \"About this input\": {basis:?}");
    }
}
