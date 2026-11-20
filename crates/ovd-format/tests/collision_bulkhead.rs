//! Seam: the collision bulkhead plane and the forward house behind it (issue #994, the collision
//! bulkhead rework; ES-TRIN 2025/1 Article 3.03).
//!
//! Label: property. Each test states a rule: the plane is reported for every style, it stands 0.04 L aft
//! of the forward perpendicular in the plane of maximum draught unless typed, the `wheelhouse_forward`
//! house stands aft of the plane with its front on it, the cargo length that costs is reported, and the
//! other styles keep their results. The 0.04 L and the house length are read from the results and the
//! sizes module, never worked by hand; the one printed number is a snapshot of today's deckhouse value.

use ovd_core::aft_arrangement::forward_house_sizes as fh;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

/// ES-TRIN 3.03(1)(a): the collision bulkhead at least 0.04 L aft of the forward perpendicular.
const FRACTION_OF_L: f64 = 0.04;
const EPS: f64 = 1e-9;

/// A self-propelled LNG barge of the given size. The river barges carry the two river limits
/// (issue #1034); the coastal one the reference tank size.
fn design(loa: f64, beam: f64, depth: f64, area: &str, style: &str, extra: Value) -> String {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": loa, "source": "typed" },
        "service_area": { "value": area, "source": "typed" },
        "beam_m": { "value": beam, "source": "typed" },
        "moulded_depth_m": { "value": depth, "source": "typed" },
        "arrangement_style": { "value": style, "source": "typed" }
    });
    if area == "river-only" {
        inputs["draught_limit_m"] = json!({ "value": 12.0, "source": "typed" });
        inputs["air_draft_limit_m"] = json!({ "value": 40.0, "source": "typed" });
    } else {
        inputs["max_tank_od_m"] = json!({ "value": 12.0, "source": "typed" });
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

fn results_of(text: &str) -> Value {
    let d = fill_as_new(text, &Profile::built_in()).design.unwrap();
    match run_option(&d.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// The three barges of the design note: river 70 m and 110 m, coastal 140 m (loa, beam, depth, area).
const BARGES: [(f64, f64, f64, &str); 3] = [
    (70.0, 10.0, 4.0, "river-only"),
    (110.0, 11.4, 6.0, "river-only"),
    (140.0, 29.0, 12.0, "coastal-and-river"),
];

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn geo(r: &Value, key: &str) -> f64 {
    f(&r["geometry"][key])
}

fn house(r: &Value) -> &Value {
    &r["drawing"]["aft_arrangement"]["forward_house"]
}

/// The forward perpendicular of the raked bow, from the results: the keel ends `rake` aft of the
/// bow and rises to the deck, so at the maximum draught the hull meets the water `rake x (1 - T/D)`
/// aft of the bow (the rake rise is the moulded depth unless typed).
fn forward_perpendicular(r: &Value, loa: f64, depth: f64) -> f64 {
    let rake = geo(r, "bow_rake_length_m");
    let t_max = f(&r["results"]["draught_max_m"]);
    loa - rake * (1.0 - t_max / depth).max(0.0)
}

#[test]
fn every_style_reports_the_plane_at_0_04_l_aft_of_the_forward_perpendicular() {
    for (loa, beam, depth, area) in BARGES {
        for style in ["deckhouse", "container_block", "wheelhouse_forward"] {
            let r = results_of(&design(loa, beam, depth, area, style, json!({})));
            let fp = geo(&r, "forward_perpendicular_x_m");
            assert!(
                (fp - forward_perpendicular(&r, loa, depth)).abs() < EPS,
                "{loa} {style}: {fp}"
            );
            let from_fp = geo(&r, "collision_bulkhead_from_fp_m");
            assert!(
                (from_fp - FRACTION_OF_L * loa).abs() < EPS,
                "{loa} {style}: {from_fp}"
            );
            let x = geo(&r, "collision_bulkhead_x_m");
            assert!((x - (fp - from_fp)).abs() < EPS, "{loa} {style}: {x}");
            // The plane lies within the hull, aft of the bow and ahead of the cargo.
            assert!(
                x < loa && x > loa - geo(&r, "bow_void_length_m") + EPS,
                "{loa} {style}: {x}"
            );
        }
    }
}

#[test]
fn a_typed_distance_moves_the_plane_and_the_house_follows_it() {
    let (loa, beam, depth, area) = BARGES[1];
    let typed = 6.0;
    let r = results_of(&design(
        loa,
        beam,
        depth,
        area,
        "wheelhouse_forward",
        json!({ "collision_bulkhead_m": typed }),
    ));
    assert_eq!(geo(&r, "collision_bulkhead_from_fp_m"), typed);
    let x = geo(&r, "collision_bulkhead_x_m");
    assert!((x - (geo(&r, "forward_perpendicular_x_m") - typed)).abs() < EPS);
    assert!((f(&house(&r)["x_fwd_m"]) - x).abs() < EPS, "{}", house(&r));
}

#[test]
fn the_forward_house_stands_aft_of_the_plane_with_its_front_on_it() {
    for (loa, beam, depth, area) in BARGES {
        let r = results_of(&design(
            loa,
            beam,
            depth,
            area,
            "wheelhouse_forward",
            json!({}),
        ));
        let h = house(&r);
        let (x_aft, x_fwd) = (f(&h["x_aft_m"]), f(&h["x_fwd_m"]));
        let plane = geo(&r, "collision_bulkhead_x_m");
        // The front is on the plane, or further aft only where the anchor gear needs more deck.
        let front = plane.min(loa - fh::BOW_CLEAR_M);
        assert!(
            (x_fwd - front).abs() < EPS,
            "{loa}: front {x_fwd}, plane {plane}"
        );
        assert!(
            x_fwd <= plane + EPS,
            "{loa}: the house is ahead of the plane"
        );
        assert!((x_fwd - x_aft - fh::LENGTH_M).abs() < EPS, "{loa}: {h}");
        // The cargo zone ends at or aft of the house.
        let cargo_fwd = loa - geo(&r, "bow_void_length_m");
        assert!(
            cargo_fwd <= x_aft + EPS,
            "{loa}: cargo to {cargo_fwd}, house from {x_aft}"
        );
        // The foredeck ahead of the house is the anchor and mooring deck, never less than the clear deck.
        assert!(
            loa - x_fwd >= fh::BOW_CLEAR_M - EPS,
            "{loa}: {}",
            loa - x_fwd
        );
    }
}

#[test]
fn the_cargo_length_the_plane_costs_is_reported_and_equals_the_deckhouse_difference() {
    for (loa, beam, depth, area) in BARGES {
        let d = results_of(&design(loa, beam, depth, area, "deckhouse", json!({})));
        let r = results_of(&design(
            loa,
            beam,
            depth,
            area,
            "wheelhouse_forward",
            json!({}),
        ));
        let lost = f(&house(&r)["cargo_length_lost_m"]);
        assert!(
            lost > 0.0,
            "{loa}: the plane costs cargo length on every barge here"
        );
        let cost = geo(&d, "cargo_zone_length_m") - geo(&r, "cargo_zone_length_m");
        assert!(
            (cost - lost).abs() < EPS,
            "{loa}: cost {cost}, reported {lost}"
        );
        let void = geo(&r, "bow_void_length_m")
            - geo(&r, "bow_rake_length_m")
            - f(&r["drawing"]["separation"]["fwd_separation_m"]);
        assert!(
            (void - lost).abs() < EPS,
            "{loa}: void {void}, reported {lost}"
        );
        // The warning says why: the collision bulkhead, not a short forepeak.
        let said = r["warnings"].as_array().unwrap().iter().any(|w| {
            w.as_str()
                .is_some_and(|w| w.contains("collision bulkhead") && w.contains("of cargo length"))
        });
        assert!(said, "{loa}: {}", r["warnings"]);
    }
}

#[test]
fn a_long_rake_puts_the_house_at_the_plane_not_at_the_stem() {
    let (loa, beam, depth, area) = BARGES[1];
    let r = results_of(&design(
        loa,
        beam,
        depth,
        area,
        "wheelhouse_forward",
        json!({ "bow_rake_length_m": 20.0 }),
    ));
    let plane = geo(&r, "collision_bulkhead_x_m");
    let x_fwd = f(&house(&r)["x_fwd_m"]);
    assert!((x_fwd - plane).abs() < EPS, "{x_fwd} vs {plane}");
    // A long forepeak holds the house entirely: no cargo length lost.
    assert_eq!(f(&house(&r)["cargo_length_lost_m"]), 0.0);
    assert!(
        loa - x_fwd > fh::BOW_CLEAR_M,
        "the deck ahead is more than the clear deck"
    );
}

#[test]
fn the_other_styles_keep_their_cargo_zone() {
    // Label: snapshot. 82.168 m is the deckhouse style's cargo zone on the 110 m barge today.
    let (loa, beam, depth, area) = BARGES[1];
    for style in ["deckhouse", "container_block"] {
        let r = results_of(&design(loa, beam, depth, area, style, json!({})));
        assert!(house(&r).is_null(), "{style}");
        if style == "deckhouse" {
            assert!(
                (geo(&r, "cargo_zone_length_m") - 82.168).abs() < 1e-3,
                "{}",
                geo(&r, "cargo_zone_length_m")
            );
        }
    }
}
