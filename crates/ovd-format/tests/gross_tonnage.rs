//! Seam: the gross tonnage of a run (issue #777, spec 0011 D15). A computed Design shows GT from ITC
//! 1969 with its V and K1, in the results and in the Rule Trace; a typed `gross_tonnage` overrides
//! it; the block is outside the results hash, so no saved Design moves; a file saved before reads
//! none.
//!
//! Label: property. Each test states a rule; the sourced numbers of the formula are in
//! `crates/ovd-core/tests/tonnage_sourced.rs`.

use ovd_format::vdesign::LEGACY_FIXED_OTHER;
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, UNHASHED_KEYS, results_hash};
use serde_json::{Value, json};

fn fixture_text(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn open(name: &str) -> DesignSession {
    let (session, opened) = DesignSession::open(&fixture_text(name), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
    session
}

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref() {
        Some(RunOutcome::Results { results, .. }) => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn number(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

#[test]
fn every_fixture_shows_gt_from_its_v_and_k1() {
    for name in [
        "dry_bulk_box_hold",
        "dry_bulk_sloped_hopper",
        "gas_igc_2g_exact",
        "gas_igc_2g_self_propelled",
        "crude_integral_tanks_self_propelled",
        "towed_barge",
        "pushed_convoy",
    ] {
        let r = results(&open(name));
        let g = &r["gross_tonnage"];
        assert_eq!(g["source"], json!("computed"), "{name}: {g}");
        let v = number(&g["enclosed_volume_m3"]);
        let k1 = number(&g["k1"]);
        let gt = number(&g["gross_tonnage"]);
        assert!(v > 0.0, "{name}");
        assert!((k1 - (0.2 + 0.02 * libm::log10(v))).abs() < 1e-12, "{name}");
        assert!((gt - k1 * v).abs() < 1e-6 * gt, "{name}");
        let parts = number(&g["hull_volume_m3"])
            + number(&g["stern_block_volume_m3"])
            + number(&g["deckhouse_volume_m3"]);
        assert!(
            (parts - v).abs() < 1e-9 * v,
            "{name}: V is the sum of its parts"
        );
        // No deckhouse model yet (issue #779).
        assert_eq!(number(&g["deckhouse_volume_m3"]), 0.0, "{name}");
        // The hull is no bigger than its L x B x D box plus sheer and camber; all fixtures have none.
        let p = &r["geometry"];
        if let (Some(l), Some(b), Some(d)) = (
            p["loa_m"].as_f64(),
            p["beam_m"].as_f64(),
            p["depth_m"].as_f64(),
        ) {
            assert!(
                number(&g["hull_volume_m3"]) <= l * b * d * (1.0 + 1e-9),
                "{name}"
            );
        }
        assert!(
            g["basis"]
                .as_str()
                .is_some_and(|b| b.contains("Regulation 3"))
        );
    }
}

#[test]
fn a_pushed_barge_has_no_stern_block_volume_and_a_self_propelled_hull_has() {
    let pushed = results(&open("towed_barge"));
    assert_eq!(
        number(&pushed["gross_tonnage"]["stern_block_volume_m3"]),
        0.0
    );
    let own = results(&open("gas_igc_2g_self_propelled"));
    assert!(number(&own["gross_tonnage"]["stern_block_volume_m3"]) > 0.0);
}

#[test]
fn the_trace_shows_v_k1_and_gt() {
    let r = results(&open("gas_igc_2g_self_propelled"));
    let measures = r["rule_trace"]["measures"].as_array().expect("a list");
    let value = |id: &str| {
        measures
            .iter()
            .find(|m| m["measure"] == json!(id))
            .map(|m| number(&m["value"]))
            .unwrap_or_else(|| panic!("{id} not in the trace: {measures:?}"))
    };
    let g = &r["gross_tonnage"];
    assert_eq!(
        value("option.enclosed_volume"),
        number(&g["enclosed_volume_m3"])
    );
    assert_eq!(value("option.gross_tonnage_k1"), number(&g["k1"]));
    assert_eq!(value("option.gross_tonnage"), number(&g["gross_tonnage"]));
}

#[test]
fn a_typed_gt_overrides_the_computed_one_and_keeps_it_beside() {
    let mut s = open("gas_igc_2g_self_propelled");
    let computed = number(&results(&s)["gross_tonnage"]["gross_tonnage"]);
    s.set_input("Option 1", "gross_tonnage", json!(1234.0))
        .unwrap();
    let r = results(&s);
    let g = &r["gross_tonnage"];
    assert_eq!(g["source"], json!("typed"));
    assert_eq!(number(&g["gross_tonnage"]), 1234.0);
    assert_eq!(number(&g["typed_gt"]), 1234.0);
    assert_eq!(number(&g["computed_gt"]), computed);
    let measures = r["rule_trace"]["measures"].as_array().unwrap();
    assert!(
        measures
            .iter()
            .any(|m| m["measure"] == json!("option.gross_tonnage") && m["value"] == json!(1234.0))
    );
}

#[test]
fn the_block_is_outside_the_results_hash_and_an_old_file_reads_none() {
    assert!(UNHASHED_KEYS.contains(&"gross_tonnage"));
    let mut r = results(&open("dry_bulk_box_hold"));
    let with = results_hash(&r);
    r.as_object_mut().unwrap().remove("gross_tonnage");
    assert_eq!(results_hash(&r), with);
    let legacy = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "gross_tonnage")
        .expect("a legacy value");
    assert_eq!(legacy.value, "null");
}
