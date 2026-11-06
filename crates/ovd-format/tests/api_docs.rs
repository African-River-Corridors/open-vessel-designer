//! Seam: the Python API page lists every input, every old name and every refused key (issue #937).
//! Label: property. The page is `docs/python-api.md`; the names come from the engine's own tables.

use ovd_format::vdesign::INPUTS;

const PAGE: &str = include_str!("../../../docs/python-api.md");

#[test]
fn the_page_names_every_input() {
    let missing: Vec<&str> = INPUTS
        .iter()
        .map(|s| s.name)
        .filter(|n| !PAGE.contains(&format!("`{n}`")))
        .collect();
    assert!(
        missing.is_empty(),
        "inputs the page does not name: {missing:?}"
    );
}

#[test]
fn the_page_names_the_old_keys_the_refused_keys_and_the_result_blocks() {
    for name in [
        "loa_m",
        "depth_m",
        "freeboard_min_m",
        "eta_d",
        "speed_max_kn",
        "lightship_k_t_per_m3",
        "channel",
        "vessel",
        "self_propelled",
        "atb",
        "cb",
        "midship_coefficient",
        "ruleset",
        "crew_berths",
        "crew_officers",
        "crew_seafarers",
        "crew_gt",
        "crew_reaches_sea",
        "crew_mode_b",
        "damage",
        "rule_trace",
        "cost",
        "calibration",
        "gross_tonnage",
        "crew",
        "rooms",
        "stern_arrangement",
        "spaces",
        "gas_fuel",
        "holding_time",
        "damage_status",
    ] {
        assert!(
            PAGE.contains(&format!("`{name}`")),
            "the page does not name `{name}`"
        );
    }
}
