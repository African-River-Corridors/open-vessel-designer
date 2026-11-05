//! Seam: the Service Area is a required input with no default (issue #925). A Design that lacks it is the
//! same error at every entry point: the file check, `fill`, a session open (the app), a run, and the
//! `simulate` entry (the Python API). `fill`'s fix (`with_service_area`) makes the same file run.
//!
//! Label: property. The message is compared as text between entry points; no number is pinned.

use ovd_format::{
    DesignSession, Profile, RunOutcome, check, fill, run_option, service_area_required,
    with_service_area,
};
use serde_json::{Value, json};

/// One bare Option of the given kind, with no Service Area.
fn file(cargo: &str, containment: &str) -> String {
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": cargo, "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 110.0, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 6.0, "source": "typed" },
            "cargo_containment": { "value": containment, "source": "typed" },
        } }]
    })
    .to_string()
}

const VESSELS: &[(&str, &str)] = &[
    ("LNG", "type_c_tanks"),
    ("bauxite", "box_hold"),
    ("bauxite", "sloped_hopper_hold"),
    ("crude_oil", "integral_tanks"),
];

fn message() -> String {
    service_area_required().to_string()
}

#[test]
fn the_message_names_the_two_values_and_the_fix() {
    let m = message();
    assert!(
        m.contains("`river-only`") && m.contains("`coastal-and-river`"),
        "{m}"
    );
    assert!(m.contains("ovd fill --service-area"), "{m}");
}

#[test]
fn the_file_check_and_fill_give_the_message_for_every_vessel_type() {
    for (cargo, containment) in VESSELS {
        let text = file(cargo, containment);
        for (entry, errors) in [
            ("check", check(&text, &Profile::built_in()).report.errors),
            ("fill", fill(&text, &Profile::built_in()).report.errors),
        ] {
            assert!(
                errors.iter().any(|e| e.contains(&message())),
                "{entry} {containment}: {errors:?}"
            );
        }
    }
}

#[test]
fn opening_a_file_in_the_app_session_reports_the_message() {
    // The app opens through the session: the file opens, the error is in its report, and the Option
    // has no Service Area to show until one is chosen.
    for (cargo, containment) in VESSELS {
        let (session, open) = DesignSession::open(&file(cargo, containment), Profile::built_in())
            .expect("a file with errors still opens");
        assert!(
            open.report.errors.iter().any(|e| e.contains(&message())),
            "{containment}: {:?}",
            open.report.errors
        );
        assert!(session.design().options[0].input("service_area").is_none());
    }
}

#[test]
fn a_run_of_an_option_with_no_service_area_gives_the_message() {
    // A new design in the session has none until it is chosen; evaluating it says so.
    let session = DesignSession::new_design(Profile::built_in()).unwrap();
    assert!(session.design().options[0].input("service_area").is_none());
    match run_option(&session.design().options[0]) {
        RunOutcome::Failed(m) => assert!(m.contains(&message()), "{m}"),
        _ => panic!("a run with no Service Area must be an error"),
    }
}

#[test]
fn simulate_gives_the_same_message_for_every_vessel_type() {
    for vessel in ["gas_barge", "box_barge", "hopper_barge", "tank_barge"] {
        let args = json!({ "params": {
            "vessel": vessel, "cargo": "bauxite", "loa_m": 80.0, "beam_m": 12.0, "depth_m": 4.0,
        } });
        let out = ovd_format::eval("simulate", &json!({ "fn": "simulate", "args": args }));
        let text = format!("{out:?}");
        assert!(text.contains(&message()), "{vessel}: {text}");
    }
}

#[test]
fn fill_with_a_service_area_then_run_succeeds() {
    for (cargo, containment) in VESSELS {
        let text = file(cargo, containment);
        let set = with_service_area(&text, "river-only").unwrap();
        let filled = fill(&set, &Profile::built_in());
        assert!(
            filled.report.errors.is_empty(),
            "{containment}: {:?}",
            filled.report.errors
        );
        let design = filled.design.unwrap();
        let service_area = design.options[0].input("service_area").unwrap();
        assert_eq!(service_area.value, json!("river-only"));
        assert_eq!(service_area.source, ovd_format::Source::Typed);
        assert!(
            matches!(run_option(&design.options[0]), RunOutcome::Results { .. }),
            "{containment}"
        );
    }
}

#[test]
fn with_service_area_keeps_one_an_option_has_and_refuses_a_wrong_value() {
    let text = file("LNG", "type_c_tanks");
    let first = with_service_area(&text, "coastal-and-river").unwrap();
    let second = with_service_area(&first, "river-only").unwrap();
    let design: Value = serde_json::from_str(&second).unwrap();
    assert_eq!(
        design["options"][0]["inputs"]["service_area"]["value"],
        json!("coastal-and-river")
    );
    let err = with_service_area(&text, "ocean-only").unwrap_err();
    assert!(
        err.contains("river-only") && err.contains("coastal-and-river"),
        "{err}"
    );
}
