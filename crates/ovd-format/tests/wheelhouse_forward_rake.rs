//! Seam: the forward house's clear deck sits over the raked bow (issue #994 follow-up).
//!
//! Label: property. The `wheelhouse_forward` house (8 m) lies within the bow rake plus the forward
//! separation. Its clear deck (3 m) lies on the deck over the bow and needs no length of its own, so
//! a default rake costs no cargo length. A typed rake is never changed. Where the house itself does
//! not fit, it takes cargo length, as before.

use ovd_core::aft_arrangement::forward_house_sizes as fh;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

const LOA: f64 = 110.0;

fn design(extra: Value) -> String {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": LOA, "source": "typed" },
        "service_area": { "value": "river-only", "source": "typed" },
        "draught_limit_m": { "value": 12.0, "source": "typed" },
        "air_draft_limit_m": { "value": 40.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
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

fn results_of(extra: Value) -> Value {
    let d = fill_as_new(&design(extra), &Profile::built_in())
        .design
        .unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn house(r: &Value) -> &Value {
    &r["drawing"]["aft_arrangement"]["forward_house"]
}

fn lost(r: &Value) -> f64 {
    f(&house(r)["cargo_length_lost_m"])
}

fn cargo(r: &Value) -> f64 {
    f(&r["geometry"]["cargo_zone_length_m"])
}

fn rake(r: &Value) -> f64 {
    f(&r["geometry"]["bow_rake_length_m"])
}

fn fwd_sep(r: &Value) -> f64 {
    f(&r["drawing"]["separation"]["fwd_separation_m"])
}

fn says_house_takes_cargo(r: &Value) -> bool {
    r["warnings"].as_array().unwrap().iter().any(|w| {
        w.as_str()
            .is_some_and(|w| w.contains("takes") && w.contains("of cargo length"))
    })
}

#[test]
fn the_default_rake_holds_house_and_clear_deck_and_no_cargo_length_is_lost() {
    let deckhouse = results_of(json!({ "arrangement_style": "deckhouse" }));
    let forward = results_of(json!({ "arrangement_style": "wheelhouse_forward" }));
    // Label: snapshot. 82.168 m is what the deckhouse style gives on this barge; main gave 80.768 m.
    assert!(
        (cargo(&forward) - 82.168).abs() < 1e-3,
        "{}",
        cargo(&forward)
    );
    assert_eq!(cargo(&forward), cargo(&deckhouse));
    assert_eq!(lost(&forward), 0.0);
    assert!(!says_house_takes_cargo(&forward), "{}", forward["warnings"]);
}

#[test]
fn the_house_lies_within_the_rake_and_forward_separation_and_the_clear_deck_ends_at_the_stem() {
    let r = results_of(json!({ "arrangement_style": "wheelhouse_forward" }));
    // The default rake is untouched: depth 6 m x ratio 1.6.
    assert!((rake(&r) - 9.6).abs() < 1e-9, "{}", rake(&r));
    let void = rake(&r) + fwd_sep(&r);
    assert!(fh::LENGTH_M <= void, "{void}");
    let h = house(&r);
    let (x_aft, x_fwd) = (f(&h["x_aft_m"]), f(&h["x_fwd_m"]));
    assert!((x_fwd - x_aft - fh::LENGTH_M).abs() < 1e-9);
    // The house stands no further aft than the end of the cargo zone ...
    assert!(x_aft >= LOA - void - 1e-9, "{x_aft}");
    // ... and the clear deck runs from its forward end to the stem, up to 3 m.
    let clear = LOA - x_fwd;
    assert!((0.0..=fh::BOW_CLEAR_M + 1e-9).contains(&clear), "{clear}");
    assert!((clear - (void - fh::LENGTH_M)).abs() < 1e-9, "{clear}");
}

#[test]
fn a_long_rake_keeps_the_full_clear_deck_ahead_of_the_house() {
    let r =
        results_of(json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": 20.0 }));
    assert_eq!(lost(&r), 0.0);
    assert!((LOA - f(&house(&r)["x_fwd_m"]) - fh::BOW_CLEAR_M).abs() < 1e-9);
}

#[test]
fn a_typed_rake_too_short_for_the_house_itself_takes_cargo_length() {
    let typed = 1.0;
    let r = results_of(
        json!({ "arrangement_style": "wheelhouse_forward", "bow_rake_length_m": typed }),
    );
    let d = results_of(json!({ "arrangement_style": "deckhouse", "bow_rake_length_m": typed }));
    let expect = fh::LENGTH_M - typed - fwd_sep(&r);
    assert!(expect > 0.0, "the set-up must not fit: {expect}");
    assert_eq!(rake(&r), typed);
    assert!((lost(&r) - expect).abs() < 1e-9, "{}", lost(&r));
    assert!((cargo(&d) - cargo(&r) - expect).abs() < 1e-9);
    assert!(says_house_takes_cargo(&r), "{}", r["warnings"]);
}

#[test]
fn the_other_styles_keep_their_results() {
    // The style is the only input that changes the house; the rake and the cargo zone stay as today.
    for style in ["deckhouse", "container_block"] {
        let r = results_of(json!({ "arrangement_style": style }));
        assert!((rake(&r) - 9.6).abs() < 1e-9, "{style}");
        assert!(
            r["drawing"]["aft_arrangement"]["forward_house"].is_null(),
            "{style}"
        );
    }
}
