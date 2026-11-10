//! Parity guard (issue #939): the one step the Python `simulate` dict and the `.vdesign` file do not
//! share is the dict-to-Option step (`option_of`, issue #937). Both paths then call the same run. This
//! file checks that step by the input values it builds, with no run, through the `api` entry's
//! `option_inputs` test seam, and the full results of one Design of each Cargo Containment.
//!
//! Label: property. Each test compares two paths to one answer; no number is pinned.

use ovd_format::vdesign::INPUTS;
use ovd_format::{Profile, RunOutcome, eval, fill_as_new, results_hash, run_option};
use serde_json::{Map, Value, json};
use std::path::Path;

/// Every Option of every `.vdesign` file of `examples/` and of the fixtures: (label, file text, index,
/// stored inputs as `name -> (value, source)`).
struct Case {
    label: String,
    text: String,
    index: usize,
    stored: Vec<(String, Value, String)>,
}

fn vdesign_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.expect("entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            vdesign_files(&path, out);
        } else if path.extension().is_some_and(|x| x == "vdesign") {
            out.push(path);
        }
    }
}

fn cases() -> Vec<Case> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    vdesign_files(&root.join("examples"), &mut files);
    vdesign_files(&root.join("crates/ovd-format/tests/fixtures"), &mut files);
    assert!(files.len() >= 15, "found only {} files", files.len());
    let mut out = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("read");
        let design: Value = serde_json::from_str(&text).expect("JSON");
        for (index, option) in design["options"]
            .as_array()
            .expect("options")
            .iter()
            .enumerate()
        {
            let stored = option["inputs"]
                .as_object()
                .expect("inputs")
                .iter()
                .map(|(k, i)| {
                    (
                        k.clone(),
                        i["value"].clone(),
                        i["source"].as_str().expect("source").to_string(),
                    )
                })
                .collect();
            out.push(Case {
                label: format!(
                    "{} #{index} ({})",
                    path.file_name().expect("name").to_string_lossy(),
                    option["name"].as_str().unwrap_or("")
                ),
                text: text.clone(),
                index,
                stored,
            });
        }
    }
    out
}

/// The `option_inputs` seam: `Ok(inputs)` or `Err(message)`.
fn seam(args: Value) -> Result<Map<String, Value>, String> {
    let case = json!({ "fn": "option_inputs", "args": args });
    match eval("api", &case) {
        Ok(v) if v.get("ok").is_some() => Ok(v["ok"].as_object().expect("object").clone()),
        Ok(v) => Err(format!("{} {}", v["error"]["type"], v["error"]["message"])),
        Err(e) => Err(e.to_string()),
    }
}

fn by_file(case: &Case) -> Map<String, Value> {
    seam(json!({ "design": case.text, "index": case.index }))
        .unwrap_or_else(|e| panic!("{}: fill --as-new: {e}", case.label))
}

fn by_dict(label: &str, dict: &Map<String, Value>) -> Result<Map<String, Value>, String> {
    seam(json!({ "params": dict })).map_err(|e| format!("{label}: {e}"))
}

/// The keys where two Options' input values differ (a key on one side only counts).
/// The dict step's Option, once an input the omission made unused is dropped. A file only warns about
/// an input its Option does not use (omit `propulsor_type` and the stored `thruster_count` no longer
/// applies); the dict step raises, by design (issue #937), so that is not drift.
fn by_dict_without_unused(
    label: &str,
    dict: &Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let mut dict = dict.clone();
    for _ in 0..5 {
        let message = match by_dict(label, &dict) {
            Ok(inputs) => return Ok(inputs),
            Err(m) => m,
        };
        let parts: Vec<&str> = message.split(" is not used").collect();
        let unused: Vec<String> = parts[..parts.len() - 1]
            .iter()
            .filter_map(|head| head.rsplit('`').nth(1))
            .map(|k| k.to_string())
            .collect();
        if unused.is_empty() || unused.iter().any(|k| !dict.contains_key(k)) {
            return Err(message);
        }
        for k in unused {
            dict.remove(&k);
        }
    }
    Err(format!("{label}: inputs still unused after five rounds"))
}

fn differences(got: &Map<String, Value>, want: &Map<String, Value>) -> Vec<String> {
    let mut keys: Vec<&String> = got.keys().chain(want.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter(|k| got.get(*k) != want.get(*k))
        .map(|k| format!("{k}: dict {:?}, file {:?}", got.get(k), want.get(k)))
        .collect()
}

/// The `.vdesign` text of a dict of inputs, every one typed: what the file path reads for that dict.
fn file_of(dict: &Map<String, Value>) -> String {
    let inputs: Map<String, Value> = dict
        .iter()
        .map(|(k, v)| (k.clone(), json!({ "value": v, "source": "typed" })))
        .collect();
    json!({ "format": "vdesign", "format_version": 4, "engine_version": env!("CARGO_PKG_VERSION"),
            "units": "SI", "options": [{ "name": "Option 1", "inputs": inputs }] })
    .to_string()
}

/// The inputs the dict step refuses by design (`midship_coefficient` of a hull from offsets or ShipD,
/// the run's own crew keys), which a stored file may still hold: the message says `is not an input`.
fn by_design_refused(dict: &Map<String, Value>) -> Vec<String> {
    match seam(json!({ "params": dict })) {
        Err(m) => m
            .split("is not an input")
            .filter_map(|head| head.rsplit('`').nth(1))
            .map(str::to_string)
            .collect(),
        Ok(_) => Vec::new(),
    }
}

fn dict_of(case: &Case) -> Map<String, Value> {
    case.stored
        .iter()
        .map(|(k, v, _)| (k.clone(), v.clone()))
        .collect()
}

#[test]
fn the_dict_step_builds_the_files_option_input_by_input() {
    let mut failed = Vec::new();
    for case in cases() {
        let mut dict = dict_of(&case);
        let refused = by_design_refused(&dict);
        for key in &refused {
            dict.remove(key);
        }
        match by_dict(&case.label, &dict) {
            Ok(got) => {
                let mut want = by_file(&case);
                for key in &refused {
                    want.remove(key);
                }
                for d in differences(&got, &want) {
                    failed.push(format!("{}: {d}", case.label));
                }
            }
            Err(e) => failed.push(e),
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn every_profile_defaulted_input_left_out_at_once_is_built_as_the_file_path_builds_it() {
    // The cheap form of the omit check: only the typed inputs (and the Service Area, which has no
    // default, issue #925) are given, so the Profile fills all the rest on both paths. A build with
    // every default left out runs the arrangement search of a gas Design (3 to 16 s each in a debug
    // build), so this is the four Designs of the full-run cases, one of each Cargo Containment.
    let mut failed = Vec::new();
    for case in per_containment() {
        let mut typed = typed_only(&case);
        for key in by_design_refused(&typed) {
            typed.remove(&key);
        }
        match (
            by_dict(&case.label, &typed),
            seam(json!({ "design": file_of(&typed) })),
        ) {
            (Ok(got), Ok(want)) => {
                for d in differences(&got, &want) {
                    failed.push(format!("{}: {d}", case.label));
                }
            }
            (got, want) => failed.push(format!("{}: dict {got:?}, file {want:?}", case.label)),
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn each_profile_defaulted_input_left_out_one_at_a_time_is_built_as_the_file_path_builds_it() {
    // The file path of the same dict: the typed file with that key left out, then `fill --as-new`. A
    // stored file's own default may be older than today's Profile, so the stored value is not the
    // reference; what the Profile fills in is. The Service Area has no default (issue #925).
    let mut failed = Vec::new();
    let mut tried = 0;
    // The four Designs of `per_containment`: every omission builds twice, and a gas Design's builds run
    // the arrangement search.
    for case in per_containment() {
        let mut full = dict_of(&case);
        for key in by_design_refused(&full) {
            full.remove(&key);
        }
        for (key, _, source) in &case.stored {
            if source != "default" || key == "service_area" || !full.contains_key(key) {
                continue;
            }
            let mut left_out = full.clone();
            left_out.remove(key);
            tried += 1;
            let want = seam(json!({ "design": file_of(&left_out) }));
            match (by_dict_without_unused(&case.label, &left_out), want) {
                (Ok(got), Ok(want)) => {
                    for d in differences(&got, &want) {
                        failed.push(format!("{} without `{key}`: {d}", case.label));
                    }
                }
                (got, want) => failed.push(format!(
                    "{} without `{key}`: dict {got:?}, file {want:?}",
                    case.label
                )),
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    assert!(tried > 400, "only {tried} omissions tried");
}

#[test]
fn every_input_of_the_schema_is_accepted_by_the_dict_step() {
    // An input is settable if the dict step never calls it unknown. It may refuse the value (here a
    // placeholder) or the input as unused by this Option, but not its name.
    let mut unknown = Vec::new();
    for spec in INPUTS {
        let dict: Map<String, Value> = [(spec.name.to_string(), json!(1.0))].into_iter().collect();
        if let Err(message) = by_dict(spec.name, &dict)
            && message.contains(&format!("unknown input `{}`", spec.name))
        {
            unknown.push(spec.name);
        }
    }
    assert!(
        unknown.is_empty(),
        "the dict step does not know: {unknown:?}"
    );
}

/// The fixture Design of each Cargo Containment (`box_hold`, `sloped_hopper_hold`, `type_c_tanks`,
/// `integral_tanks`): the Designs of the full-run cases.
const PER_CONTAINMENT: [&str; 4] = [
    "dry_bulk_box_hold",
    "dry_bulk_sloped_hopper",
    "gas_igc_2g_self_propelled",
    "crude_integral_tanks_coastal",
];

fn per_containment() -> Vec<Case> {
    let all = cases();
    PER_CONTAINMENT
        .iter()
        .map(|file| {
            all.iter()
                .position(|c| c.label.starts_with(file) && c.index == 0)
                .unwrap_or_else(|| panic!("no fixture {file}"))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|i| cases().swap_remove(i))
        .collect()
}

fn typed_only(case: &Case) -> Map<String, Value> {
    // The Service Area has no default (issue #925), so a fixture that holds it as a default gives it too.
    case.stored
        .iter()
        .filter(|(k, _, s)| s == "typed" || k == "service_area")
        .map(|(k, v, _)| (k.clone(), v.clone()))
        .collect()
}

/// `simulate(dict, damage=True)` and `ovd fill --as-new` then `ovd run` on one fixture's typed inputs.
fn same_results(file: &str) {
    let case = cases()
        .into_iter()
        .find(|c| c.label.starts_with(file) && c.index == 0)
        .unwrap_or_else(|| panic!("no fixture {file}"));
    assert!(PER_CONTAINMENT.contains(&file));
    let typed = typed_only(&case);
    let typed_file = file_of(&typed);
    let checked = fill_as_new(&typed_file, &Profile::built_in());
    assert!(checked.report.is_ok(), "{:?}", checked.report.errors);
    let RunOutcome::Results { results, hash } =
        run_option(&checked.design.expect("design").options[0])
    else {
        panic!("{file}: no results from the file path")
    };
    let case_json = json!({ "fn": "simulate", "args": { "params": typed, "damage": true } });
    let got = eval("api", &case_json).expect("eval");
    let got = got.get("ok").unwrap_or_else(|| panic!("{file}: {got}"));
    assert_eq!(
        serde_json::to_string(got).expect("json"),
        serde_json::to_string(&results).expect("json"),
        "{file}: results, rule trace and damage blocks"
    );
    assert_eq!(results_hash(got), hash, "{file}: hash");
}

#[test]
fn a_box_hold_design_gives_the_same_results_by_dict_and_by_file() {
    same_results("dry_bulk_box_hold");
}

#[test]
fn a_sloped_hopper_hold_design_gives_the_same_results_by_dict_and_by_file() {
    same_results("dry_bulk_sloped_hopper");
}

#[test]
fn a_type_c_tanks_design_gives_the_same_results_by_dict_and_by_file() {
    same_results("gas_igc_2g_self_propelled");
}

#[test]
fn an_integral_tanks_design_gives_the_same_results_by_dict_and_by_file() {
    same_results("crude_integral_tanks_coastal");
}
