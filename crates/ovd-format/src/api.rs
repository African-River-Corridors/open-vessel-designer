//! The Python `simulate` as one path (issue #937): a dict of inputs becomes one Option the way
//! `ovd fill --as-new` builds it (the Design reader, once, with the Defaults Profile), and the
//! Option is run the way `ovd run` runs it. Nothing here invents a value: a key the dict leaves
//! out takes the Defaults Profile value, and a required input it leaves out is an error.
//!
//! The dict is the old `simulate` dict (engine names such as `loa_m`, a nested `channel`, `vessel`,
//! `self_propelled`, `atb`) or the same inputs under their glossary names. [`inputs_of`] only
//! renames and re-shapes; it never fills.

use crate::EvalError;
use crate::eval::{ok, raised};
use crate::profile::Profile;
use crate::rule_set::Library;
use crate::vdesign::{ENGINE_VERSION, FORMAT_VERSION, INPUTS, InputKind};
use ovd_core::pystr::{parse_float, repr_str};
use ovd_core::text;
use ovd_core::text::{Arg, Msg, Text};
use serde_json::{Map, Value, json};

/// The engine names of the old `simulate` dict and the input each one is now (issue #937).
const RENAMED: &[(&str, &str)] = &[
    ("loa_m", "length_overall_m"),
    ("depth_m", "moulded_depth_m"),
    ("freeboard_min_m", "min_freeboard_m"),
    ("eta_d", "propulsive_efficiency"),
    ("speed_max_kn", "max_speed_kn"),
    ("lightship_k_t_per_m3", "lightship_factor_t_per_m3"),
];

/// The keys of the old `channel` object and the inputs they are.
const CHANNEL: &[(&str, &str)] = &[
    ("depth_m", "channel_depth_m"),
    ("ukc_m", "channel_under_keel_clearance_m"),
    ("bottom_width_m", "channel_bottom_width_m"),
    ("bank_slope", "channel_bank_slope"),
    ("current_kn", "channel_current_kn"),
];

/// The old `vessel` ids and the Cargo Containment each one is.
const VESSELS: &[(&str, &str)] = &[
    ("gas_barge", "type_c_tanks"),
    ("box_barge", "box_hold"),
    ("hopper_barge", "sloped_hopper_hold"),
    ("tank_barge", "integral_tanks"),
];

/// The run's own settle-loop keys: never an input.
const CREW_KEYS: &[&str] = &[
    "crew_berths",
    "crew_officers",
    "crew_seafarers",
    "crew_gt",
    "crew_reaches_sea",
    "crew_mode_b",
];

/// Why a call did not give a result.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stop {
    /// Input the call refuses: every message, in order. Python: `TypeError`.
    Bad(Vec<String>),
    /// An exception of the old engine that still escapes: class and message.
    Raised(String, String),
}

fn msgs(list: Vec<Text>) -> Stop {
    Stop::Bad(list.into_iter().map(|t| t.en).collect())
}

/// Whether an input takes `null` for a value of its own ("use the rule"). For every other input,
/// `null` in a dict means the input is left out (the old meaning of "absent": the default).
fn takes_null(kind: InputKind) -> bool {
    matches!(
        kind,
        InputKind::Number { nullable: true }
            | InputKind::Text { nullable: true }
            | InputKind::VesselScheme
            | InputKind::VesselClass
            | InputKind::ParameterSet
            | InputKind::OffsetsTable
            | InputKind::OpeningsList
            | InputKind::DeckhouseOpenings
    )
}

fn refused_key(key: &str) -> Option<Text> {
    match key {
        "cb" => Some(Text::fixed(
            "msg-api-cb",
            "`cb` is a result, not an input: the hull gives it (issue #448)",
        )),
        "midship_coefficient" => Some(Text::fixed(
            "msg-api-midship-coefficient",
            "`midship_coefficient` is not an input: the hull gives its own midship area",
        )),
        "ruleset" => Some(Text::fixed(
            "msg-api-ruleset",
            "`ruleset` is replaced by `service_area` and `rule_sets`",
        )),
        k if CREW_KEYS.contains(&k) => Some(text!(
            "msg-api-crew-key",
            "`{key}` is worked out by the run; it is not an input",
            key = Arg::input(k)
        )),
        _ => None,
    }
}

/// The typed inputs of one Option from a `simulate` dict. It renames and re-shapes; it fills
/// nothing (the reader does that, from the Defaults Profile). Every error is collected.
pub(crate) fn inputs_of(params: &Map<String, Value>) -> Result<Map<String, Value>, Stop> {
    let mut errors: Vec<Text> = Vec::new();
    let mut raise: Option<(String, String)> = None;
    let mut out: Map<String, Value> = Map::new();
    let mut old_name = false;
    // The refused keys come first, so the caller gets the reason and not "unknown input".
    for key in params.keys() {
        if let Some(t) = refused_key(key) {
            errors.push(t);
        }
    }
    let put =
        |out: &mut Map<String, Value>, errors: &mut Vec<Text>, old: &str, new: &str, v: &Value| {
            if params.contains_key(new) {
                errors.push(text!(
                    "msg-api-both-names",
                    "give one of `{old}` and `{new}`: they are the same input",
                    old = Arg::input(old),
                    new = Arg::input(new)
                ));
            } else {
                out.insert(new.to_string(), v.clone());
            }
        };
    for (key, value) in params {
        let key = key.as_str();
        if refused_key(key).is_some() {
            continue;
        }
        if let Some((old, new)) = RENAMED.iter().find(|(old, _)| *old == key) {
            if params.contains_key(*new) {
                errors.push(text!(
                    "msg-api-both-names",
                    "give one of `{old}` and `{new}`: they are the same input",
                    old = Arg::input(*old),
                    new = Arg::input(*new)
                ));
            } else {
                if *old == "loa_m" {
                    old_name = true;
                }
                out.insert((*new).to_string(), value.clone());
            }
            continue;
        }
        match key {
            "channel" => match value {
                Value::Object(ch) => {
                    for (k, v) in ch {
                        match CHANNEL.iter().find(|(old, _)| old == k) {
                            Some((_, new)) => put(&mut out, &mut errors, "channel", new, v),
                            None => errors.push(text!(
                                "msg-api-channel-key",
                                "unknown key `{key}` in `channel`; it takes {keys}",
                                key = Arg::input(k),
                                keys = CHANNEL
                                    .iter()
                                    .map(|(old, _)| format!("`{old}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )),
                        }
                    }
                }
                Value::Null => {}
                _ => errors.push(Text::fixed(
                    "msg-api-channel-object",
                    "`channel` must be an object, or left out",
                )),
            },
            "vessel" => {
                old_name = true;
                if value.is_null() {
                    continue;
                }
                match value
                    .as_str()
                    .and_then(|v| VESSELS.iter().find(|(id, _)| *id == v))
                {
                    Some((_, containment)) => put(
                        &mut out,
                        &mut errors,
                        "vessel",
                        "cargo_containment",
                        &json!(containment),
                    ),
                    None => errors.push(text!(
                        "msg-api-vessel",
                        "`vessel` = {value} must be one of: {ids}",
                        value = value.to_string(),
                        ids = VESSELS
                            .iter()
                            .map(|(id, _)| *id)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }
            }
            "self_propelled" | "atb" => {
                old_name = true;
            }
            _ => {
                out.insert(key.to_string(), value.clone());
            }
        }
    }
    // The Formation: the glossary object, or the old flags (neither flag, in an old dict, is a pushed
    // barge, as the old engine took it).
    let flag = |k: &str| params.get(k).and_then(Value::as_bool).unwrap_or(false);
    let (self_propelled, atb) = (flag("self_propelled"), flag("atb"));
    for k in ["self_propelled", "atb"] {
        if let Some(v) = params.get(k)
            && !v.is_null()
            && !v.is_boolean()
        {
            errors.push(text!(
                "msg-bool",
                "`{input}` must be true or false",
                input = Arg::input(k)
            ));
        }
    }
    if self_propelled && atb {
        errors.push(Text::fixed(
            "msg-api-formation-flags",
            "`self_propelled` and `atb` are both true; a vessel is one or the other",
        ));
    }
    if params.get("formation").is_some_and(|f| !f.is_null()) {
        if self_propelled || atb {
            errors.push(Text::fixed(
                "msg-api-formation-both",
                "give the Formation as `formation`, or as `self_propelled` or `atb`, not both",
            ));
        }
    } else if old_name {
        // Only the old dict has a Formation to map: neither flag was a pushed barge there. A dict in
        // glossary names gives `formation`, as a file must; left out, the reader says so.
        let kind = if self_propelled {
            "self_propelled"
        } else if atb {
            "atb"
        } else {
            "pushed_barge"
        };
        out.insert("formation".to_string(), json!({ "kind": kind }));
    }
    // An old dict with no Hull Form was built with a flat bottom and a raked bow: `raked_box`.
    if old_name && out.get("hull_form").is_none_or(Value::is_null) {
        out.insert("hull_form".to_string(), json!("raked_box"));
    }
    // Values: numbers written as text read as Python `float()` reads them; `null` on an input
    // that has no `null` of its own is the input left out.
    let names: Vec<String> = out.keys().cloned().collect();
    for name in names {
        let Some(spec) = INPUTS.iter().find(|s| s.name == name) else {
            continue;
        };
        match out.get(&name) {
            Some(Value::Null) if !takes_null(spec.kind) => {
                out.remove(&name);
            }
            Some(Value::String(s)) if matches!(spec.kind, InputKind::Number { .. }) => {
                match parse_float(s) {
                    Some(x) if x.is_finite() => {
                        out.insert(name, json!(x));
                    }
                    Some(_) => {}
                    None => {
                        raise.get_or_insert((
                            "ValueError".to_string(),
                            format!("could not convert string to float: {}", repr_str(s)),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    if !errors.is_empty() {
        return Err(msgs(errors));
    }
    if let Some((kind, message)) = raise {
        return Err(Stop::Raised(kind, message));
    }
    Ok(out)
}

/// The `.vdesign` text of one Option whose every value is typed.
fn vdesign_text(inputs: &Map<String, Value>) -> String {
    let typed: Map<String, Value> = inputs
        .iter()
        .map(|(k, v)| (k.clone(), json!({ "value": v, "source": "typed" })))
        .collect();
    json!({
        "format": "vdesign",
        "format_version": FORMAT_VERSION,
        "engine_version": ENGINE_VERSION,
        "units": "SI",
        "options": [{ "name": "Option 1", "inputs": typed }],
    })
    .to_string()
}

/// Is this warning, wrapped as an Option's, one of an input the Option does not use?
fn is_unused_warning(msg: &Option<Msg>) -> bool {
    let Some(m) = msg else { return false };
    if m.id.starts_with("msg-input-unused") {
        return true;
    }
    m.args.iter().any(|(_, a)| match a {
        Arg::Msg(inner) => inner.id.starts_with("msg-input-unused"),
        _ => false,
    })
}

/// The Option of a `simulate` dict: the typed inputs through the Design reader once, with the
/// Defaults Profile. Every error (and every input the Option does not use, which the file path
/// only warns about) is raised, one message each.
pub(crate) fn option_of(
    params: &Map<String, Value>,
    profile: &Profile,
) -> Result<crate::vdesign::DesignOption, Stop> {
    let inputs = inputs_of(params)?;
    let checked = crate::vdesign::fill_from_profile(&vdesign_text(&inputs), profile);
    let report = &checked.report;
    let mut errors: Vec<String> = report.errors.clone();
    for (warning, msg) in report.warnings.iter().zip(&report.warning_msgs) {
        if is_unused_warning(msg) {
            errors.push(warning.clone());
        }
    }
    let option = checked.design.and_then(|d| d.options.into_iter().next());
    if let Some(option) = &option {
        // The Service Area has no default (issue #925): one message, from the one place that reads it.
        let named = report
            .error_inputs
            .iter()
            .any(|i| i.iter().any(|n| n == "service_area"));
        if !named && let Err(m) = crate::trace::service_area_of(option) {
            errors.push(m);
        }
    }
    match option {
        Some(o) if errors.is_empty() => Ok(o),
        _ if !errors.is_empty() => Err(Stop::Bad(errors)),
        _ => Err(Stop::Bad(vec!["the Option could not be built".to_string()])),
    }
}

/// The Option of a call's `params`: the one build of `simulate` and `simulate_damage`. The error
/// side is the call's own answer: a refusal, or a raised Python exception with its class.
fn built(args: &Value) -> Result<crate::vdesign::DesignOption, Result<Value, EvalError>> {
    let params = args
        .get("params")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            Err(EvalError::BadInput(
                "`args.params` must be an object".into(),
            ))
        })?;
    match option_of(params, &Profile::built_in()) {
        Ok(o) => Ok(o),
        Err(Stop::Bad(list)) => Err(Err(EvalError::BadInput(list.join("\n")))),
        Err(Stop::Raised(kind, message)) => Err(raised(&kind, &message)),
    }
}

/// Run the built Option as `ovd run` does (the built-in Rule Set Library, no extra sets).
fn run_built(
    option: &crate::vdesign::DesignOption,
    control: &crate::damage::DamageControl,
) -> (crate::RunOutcome, Option<(String, String)>) {
    crate::run::run_option_detail(option, &Library::built_in(), &[], control)
}

/// `simulate(params, damage)`: build the Option, run it as `ovd run` does, and give the results.
fn simulate_design(args: &Value) -> Result<Value, EvalError> {
    let damage = match args.get("damage") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => {
            return Err(EvalError::BadInput(
                "`args.damage` must be true or false".into(),
            ));
        }
    };
    let option = match built(args) {
        Ok(o) => o,
        Err(answer) => return answer,
    };
    let control = crate::damage::DamageControl {
        run: damage,
        progress: None,
    };
    let (outcome, escaped) = run_built(&option, &control);
    answer_of(outcome, escaped)
}

/// `simulate_damage(params)` with a progress hook (issue #940): the damage stage. The hook is the
/// engine's own ([`crate::damage::ProgressHook`]): it is called on this thread only, never on a
/// worker. The answer is what `simulate` with `damage: true` gives, on the whole value (both go
/// through `built` and `run_built`). A hook that breaks cancels the run, and the call then gives the
/// results with `damage_status` `not run`: the value `simulate` gives without damage. A caller that
/// breaks the hook to raise an error, not to ask for those results, says so through `abandoned`: the
/// call then skips that second run and gives `{"error": ...}` (the caller discards it).
pub fn simulate_damage_with(
    args: &Value,
    progress: Option<crate::damage::ProgressHook>,
    abandoned: &dyn Fn() -> bool,
) -> Result<Value, EvalError> {
    let option = match built(args) {
        Ok(o) => o,
        Err(answer) => return answer,
    };
    let control = crate::damage::DamageControl {
        run: true,
        progress,
    };
    let (outcome, escaped) = run_built(&option, &control);
    let cancelled = escaped.as_ref().is_some_and(|(kind, message)| {
        kind == "Cancelled" && message == crate::damage::DAMAGE_CANCELLED
    });
    if cancelled && abandoned() {
        return ok(json!({ "error": crate::damage::DAMAGE_CANCELLED }));
    }
    if cancelled {
        let skip = crate::damage::DamageControl {
            run: false,
            progress: None,
        };
        let (outcome, escaped) = run_built(&option, &skip);
        return answer_of(outcome, escaped);
    }
    answer_of(outcome, escaped)
}

/// What the call gives for a run: the results; `{"error": text}` for a refusal or a result the run
/// cannot give (as the old `simulate` gave a domain message); or the Python exception the engine
/// raised, with its class.
fn answer_of(
    outcome: crate::RunOutcome,
    escaped: Option<(String, String)>,
) -> Result<Value, EvalError> {
    if let Some((kind, message)) = escaped {
        return raised(&kind, &message);
    }
    match outcome {
        crate::RunOutcome::Results { results, .. } => ok(results),
        crate::RunOutcome::NotAvailable(text) | crate::RunOutcome::Failed(text) => {
            ok(json!({ "error": text }))
        }
    }
}

/// Run one `api` function on case-shaped arguments.
pub fn run(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "simulate" => simulate_design(args),
        "simulate_damage" => simulate_damage_with(args, None, &|| false),
        other => Err(EvalError::BadInput(format!(
            "api has no function `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    //! Seam: the dict-to-inputs step and the build of the Option (issue #937). Label: property.
    use super::*;

    fn obj(v: Value) -> Map<String, Value> {
        v.as_object().expect("an object").clone()
    }

    fn bad(r: Result<Map<String, Value>, Stop>) -> Vec<String> {
        match r {
            Err(Stop::Bad(list)) => list,
            other => panic!("expected refused input, got {other:?}"),
        }
    }

    #[test]
    fn each_old_name_becomes_its_glossary_name() {
        for (old, new) in RENAMED {
            let out = inputs_of(&obj(json!({ *old: 1.0 }))).expect("renamed");
            assert_eq!(out.get(*new), Some(&json!(1.0)), "{old}");
            assert!(!out.contains_key(*old), "{old}");
        }
    }

    #[test]
    fn an_old_name_and_its_glossary_name_together_are_an_error() {
        let e = bad(inputs_of(&obj(
            json!({ "loa_m": 80.0, "length_overall_m": 80.0 }),
        )));
        assert_eq!(e.len(), 1, "{e:?}");
        assert!(
            e[0].contains("give one of `loa_m` and `length_overall_m`"),
            "{e:?}"
        );
        let e = bad(inputs_of(&obj(
            json!({ "vessel": "gas_barge", "cargo_containment": "type_c_tanks" }),
        )));
        assert!(
            e.iter().any(|m| m.contains("give one of `vessel`")),
            "{e:?}"
        );
        let e = bad(inputs_of(&obj(
            json!({ "channel": { "depth_m": 3.0 }, "channel_depth_m": 3.0 }),
        )));
        assert!(e.iter().any(|m| m.contains("`channel_depth_m`")), "{e:?}");
    }

    #[test]
    fn the_channel_is_flattened_to_the_five_inputs() {
        let out = inputs_of(&obj(json!({ "channel": {
            "depth_m": 3.0, "ukc_m": 0.5, "bottom_width_m": 40.0, "bank_slope": 3.0, "current_kn": 2.0,
        } })))
        .expect("flattened");
        for (key, new) in CHANNEL {
            assert!(out.contains_key(*new), "{key}");
        }
        assert!(!out.contains_key("channel"));
        let e = bad(inputs_of(&obj(json!({ "channel": { "depth": 3.0 } }))));
        assert!(e[0].contains("unknown key `depth` in `channel`"), "{e:?}");
    }

    #[test]
    fn each_vessel_gives_its_cargo_containment() {
        for (vessel, containment) in VESSELS {
            let out = inputs_of(&obj(json!({ "vessel": vessel }))).expect("mapped");
            assert_eq!(out["cargo_containment"], json!(containment), "{vessel}");
        }
        let e = bad(inputs_of(&obj(json!({ "vessel": "tug" }))));
        assert!(e[0].contains("`vessel` = \"tug\" must be one of"), "{e:?}");
    }

    #[test]
    fn the_formation_comes_from_the_flags_or_the_object() {
        let kind = |p: Value| {
            inputs_of(&obj(p))
                .expect("ok")
                .get("formation")
                .and_then(|f| f["kind"].as_str().map(str::to_string))
        };
        assert_eq!(
            kind(json!({ "self_propelled": true })).as_deref(),
            Some("self_propelled")
        );
        assert_eq!(kind(json!({ "atb": true })).as_deref(), Some("atb"));
        assert_eq!(
            kind(json!({})).as_deref(),
            None,
            "a glossary dict gives its Formation"
        );
        assert_eq!(
            kind(json!({ "vessel": "box_barge" })).as_deref(),
            Some("pushed_barge")
        );
        assert_eq!(
            kind(json!({ "self_propelled": false, "atb": false })).as_deref(),
            Some("pushed_barge")
        );
        let out = inputs_of(&obj(
            json!({ "formation": { "kind": "pushed_convoy", "abreast": 2, "in_line": 2 } }),
        ))
        .expect("ok");
        assert_eq!(out["formation"]["abreast"], json!(2));
        for p in [
            json!({ "self_propelled": true, "atb": true }),
            json!({ "formation": { "kind": "atb" }, "atb": true }),
        ] {
            assert!(!bad(inputs_of(&obj(p))).is_empty());
        }
    }

    #[test]
    fn a_hull_form_is_mapped_only_for_an_old_dict() {
        for old in [
            json!({ "loa_m": 80.0 }),
            json!({ "vessel": "box_barge" }),
            json!({ "self_propelled": true }),
            json!({ "atb": true }),
        ] {
            let out = inputs_of(&obj(old)).expect("ok");
            assert_eq!(out["hull_form"], json!("raked_box"));
        }
        let out = inputs_of(&obj(json!({ "length_overall_m": 80.0 }))).expect("ok");
        assert!(!out.contains_key("hull_form"));
        let out = inputs_of(&obj(json!({ "loa_m": 80.0, "hull_form": "chine" }))).expect("ok");
        assert_eq!(out["hull_form"], json!("chine"));
    }

    #[test]
    fn numbers_as_text_read_as_python_reads_them() {
        for (text, want) in [
            ("80", 80.0),
            (" 80 ", 80.0),
            ("8e1", 80.0),
            ("\u{668}\u{660}", 80.0),
        ] {
            let out = inputs_of(&obj(json!({ "beam_m": text }))).expect("a number");
            assert_eq!(out["beam_m"], json!(want), "{text}");
        }
        match inputs_of(&obj(json!({ "beam_m": "0x10" }))) {
            Err(Stop::Raised(kind, message)) => {
                assert_eq!(kind, "ValueError");
                assert_eq!(message, "could not convert string to float: '0x10'");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn null_on_an_input_with_no_null_of_its_own_is_the_input_left_out() {
        let out = inputs_of(&obj(json!({
            "fill_ratio": null,
            "tank_wall_thickness_mm": null,
            "tank_dome_height_m": null,
            "tank_dome_diameter_m": null,
            "tank_design_pressure_mpa": null,
        })))
        .expect("ok");
        assert!(!out.contains_key("fill_ratio"));
        for key in [
            "tank_wall_thickness_mm",
            "tank_dome_height_m",
            "tank_dome_diameter_m",
            "tank_design_pressure_mpa",
        ] {
            assert_eq!(out.get(key), Some(&Value::Null), "{key} keeps its null");
        }
    }

    #[test]
    fn each_refused_key_gives_its_own_message() {
        for (key, part) in [
            ("cb", "is a result, not an input"),
            ("midship_coefficient", "the hull gives its own midship area"),
            ("ruleset", "is replaced by `service_area` and `rule_sets`"),
            ("crew_berths", "is worked out by the run"),
            ("crew_officers", "is worked out by the run"),
            ("crew_seafarers", "is worked out by the run"),
            ("crew_gt", "is worked out by the run"),
            ("crew_reaches_sea", "is worked out by the run"),
            ("crew_mode_b", "is worked out by the run"),
        ] {
            let e = bad(inputs_of(&obj(json!({ key: 1.0 }))));
            assert_eq!(e.len(), 1, "{key}: {e:?}");
            assert!(e[0].contains(part) && e[0].contains(key), "{key}: {e:?}");
            assert!(!e[0].contains("unknown input"), "{key}: {e:?}");
        }
    }

    #[test]
    fn the_rename_table_is_the_inverse_of_simulate_params_of() {
        // Read from a built Option, so the two tables cannot drift: every key `run` gives the engine
        // becomes the Option input it was read from.
        let text = include_str!("../tests/fixtures/crude_integral_tanks_self_propelled.vdesign");
        let checked = crate::vdesign::fill_as_new(text, &Profile::built_in());
        let option = &checked.design.expect("a design").options[0];
        let params = crate::run::simulate_params_of(option).expect("params");
        let back = inputs_of(&params).expect("inputs");
        for (key, _) in &params {
            let want = match key.as_str() {
                "vessel" => "cargo_containment",
                "self_propelled" | "atb" => "formation",
                "channel" => "channel_depth_m",
                k => RENAMED
                    .iter()
                    .find(|(old, _)| *old == k)
                    .map_or(k, |(_, new)| *new),
            };
            let stored = option
                .input(want)
                .unwrap_or_else(|| panic!("no input {want}"));
            if matches!(key.as_str(), "self_propelled" | "atb") {
                assert_eq!(back[want]["kind"], stored.value["kind"], "{key}");
            } else if key != "channel" {
                assert_eq!(back[want], stored.value, "{key} -> {want}");
            }
        }
    }
    fn gas() -> Value {
        json!({ "cargo": "LNG", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
                "length_overall_m": 140.0, "beam_m": 29.0, "cargo_containment": "type_c_tanks",
                "service_area": "coastal-and-river" })
    }

    fn refused(r: Result<crate::vdesign::DesignOption, Stop>) -> Vec<String> {
        match r {
            Err(Stop::Bad(list)) => list,
            other => panic!("expected refused input, got {other:?}"),
        }
    }

    #[test]
    fn a_dict_with_only_the_required_inputs_builds_what_fill_as_new_builds() {
        let profile = Profile::built_in();
        let typed = obj(gas());
        let built = option_of(&typed, &profile).expect("builds");
        let checked = crate::vdesign::fill_as_new(&vdesign_text(&typed), &profile);
        let reference = checked.design.expect("a design").options.remove(0);
        assert_eq!(built.inputs.len(), reference.inputs.len());
        for ((name, got), (want_name, want)) in built.inputs.iter().zip(&reference.inputs) {
            assert_eq!(name, want_name);
            assert_eq!(got, want, "{name}");
        }
    }

    #[test]
    fn a_dict_without_length_beam_and_cargo_names_all_three() {
        let e = refused(option_of(
            &obj(
                json!({ "hull_form": "raked_box", "formation": { "kind": "pushed_barge" },
                         "cargo_containment": "box_hold", "service_area": "river-only" }),
            ),
            &Profile::built_in(),
        ));
        for name in ["cargo", "length_overall_m", "beam_m"] {
            assert_eq!(
                e.iter()
                    .filter(|m| m.contains(&format!("missing required input `{name}`")))
                    .count(),
                1,
                "{name}: {e:?}"
            );
        }
        assert!(!e.iter().any(|m| m.contains("76.5")), "{e:?}");
    }

    #[test]
    fn an_unknown_an_unused_and_an_out_of_range_input_are_each_refused_with_the_readers_words() {
        let profile = Profile::built_in();
        let with = |k: &str, v: Value| {
            let mut p = obj(gas());
            p.insert(k.to_string(), v);
            refused(option_of(&p, &profile))
        };
        assert!(with("hull_colour", json!("red"))[0].contains("unknown input `hull_colour`"));
        let e = with("hatch_coaming_height_m", json!(1.0));
        assert!(
            e[0].contains(
                "`hatch_coaming_height_m` is not used for a type_c_tanks Cargo Containment"
            ),
            "{e:?}"
        );
        let e = with("beam_m", json!(-3.0));
        assert!(e[0].contains("beam_m"), "{e:?}");
    }

    #[test]
    fn a_missing_service_area_is_the_one_message_of_issue_925() {
        let mut p = obj(gas());
        p.remove("service_area");
        let e = refused(option_of(&p, &Profile::built_in()));
        assert_eq!(e, vec![crate::vdesign::service_area_required().en]);
    }

    #[test]
    fn every_error_is_given_at_once_in_the_readers_order() {
        let mut p = obj(gas());
        p.remove("service_area");
        p.remove("beam_m");
        p.insert("hull_colour".into(), json!(1.0));
        let e = refused(option_of(&p, &Profile::built_in()));
        assert_eq!(e.len(), 3, "{e:?}");
        assert!(e[0].contains("unknown input"), "{e:?}");
        assert!(e[1].contains("beam_m"), "{e:?}");
        assert!(e[2].contains("Service Area"), "{e:?}");
    }
    #[test]
    fn a_run_failure_gives_a_dict_and_a_raised_error_keeps_its_class() {
        // The reader's range checks stop the inputs that made the old engine raise, so no dict reaches
        // these arms today; the mapping is tested on the outcomes themselves, one per class.
        use crate::RunOutcome;
        let got = answer_of(RunOutcome::Failed("no cargo fits".into()), None).expect("ok");
        assert_eq!(got, json!({ "ok": { "error": "no cargo fits" } }));
        let got = answer_of(RunOutcome::NotAvailable("not yet".into()), None).expect("ok");
        assert_eq!(got, json!({ "ok": { "error": "not yet" } }));
        for class in [
            "ValueError",
            "KeyError",
            "TypeError",
            "ZeroDivisionError",
            "OverflowError",
        ] {
            let got = answer_of(
                RunOutcome::Failed(format!("{class}: boom")),
                Some((class.to_string(), "boom".to_string())),
            )
            .expect("ok");
            assert_eq!(
                got,
                json!({ "error": { "type": class, "message": "boom" } }),
                "{class}"
            );
        }
    }
    #[test]
    fn a_glossary_dict_without_a_formation_raises_the_readers_message_and_an_old_dict_is_a_pushed_barge()
     {
        // One path: the file path raises "missing required input `formation`" for the same inputs, so a
        // new caller who forgets it gets that error. Only an old dict (an old name in it) has the
        // pushed barge the old engine assumed (design 3).
        let profile = Profile::built_in();
        let mut glossary = obj(gas());
        glossary.remove("formation");
        let e = refused(option_of(&glossary, &profile));
        assert!(
            e.iter()
                .any(|m| m.contains("missing required input `formation`")),
            "{e:?}"
        );
        let out = inputs_of(&glossary).expect("renames only");
        assert!(!out.contains_key("formation"), "{out:?}");
        let old = obj(
            json!({ "vessel": "box_barge", "cargo": "bauxite", "loa_m": 76.5, "beam_m": 11.4,
                              "service_area": "river-only" }),
        );
        assert_eq!(
            inputs_of(&old).expect("ok")["formation"],
            json!({ "kind": "pushed_barge" })
        );
    }

    #[test]
    fn inputs_of_adds_no_key_but_the_old_dicts_formation_and_hull_form() {
        // Parity guard: `inputs_of` renames and re-shapes. A new default written here would be a second
        // path. The output keys are the input keys, renamed, and `formation` and `hull_form` only for an
        // old dict.
        let names: Vec<&str> = INPUTS
            .iter()
            .map(|s| s.name)
            .filter(|n| !CREW_KEYS.contains(n))
            .collect();
        let glossary: Map<String, Value> =
            names.iter().map(|n| (n.to_string(), json!(1.0))).collect();
        let out = inputs_of(&glossary).expect("glossary names");
        let mut got: Vec<&String> = out.keys().collect();
        let mut want: Vec<&String> = glossary.keys().collect();
        got.sort();
        want.sort();
        assert_eq!(got, want, "a glossary dict gains or loses no key");

        // The same for a minimal glossary dict, as a caller writes it: no key is added.
        let minimal = obj(gas());
        let out = inputs_of(&minimal).expect("glossary names");
        let mut got: Vec<&String> = out.keys().collect();
        let mut want: Vec<&String> = minimal.keys().collect();
        got.sort();
        want.sort();
        assert_eq!(got, want, "a minimal glossary dict gains no key");

        let old = obj(json!({
            "loa_m": 80.0, "depth_m": 4.0, "freeboard_min_m": 0.3, "eta_d": 0.5, "speed_max_kn": 8.0,
            "lightship_k_t_per_m3": 0.1, "vessel": "box_barge", "self_propelled": false,
            "channel": { "depth_m": 3.0, "ukc_m": 0.5, "bottom_width_m": 40.0, "bank_slope": 3.0, "current_kn": 1.0 },
            "cargo": "bauxite", "beam_m": 11.4,
        }));
        let out = inputs_of(&old).expect("old names");
        let mut want: Vec<String> = [
            "length_overall_m",
            "moulded_depth_m",
            "min_freeboard_m",
            "propulsive_efficiency",
            "max_speed_kn",
            "lightship_factor_t_per_m3",
            "cargo_containment",
            "cargo",
            "beam_m",
            "channel_depth_m",
            "channel_under_keel_clearance_m",
            "channel_bottom_width_m",
            "channel_bank_slope",
            "channel_current_kn",
            "formation",
            "hull_form",
        ]
        .map(String::from)
        .to_vec();
        want.sort();
        let mut got: Vec<String> = out.keys().cloned().collect();
        got.sort();
        assert_eq!(got, want);
    }
}
