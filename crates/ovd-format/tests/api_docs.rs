//! Seam: the Python API page lists every input, every old name and every refused key (issue #937).
//! Label: property. The page is `docs/python-api.md`; the names come from the engine's own tables.

use ovd_format::vdesign::INPUTS;
use ovd_format::{Profile, input_schema};

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

#[test]
fn the_inputs_table_is_the_schema_row_by_row() {
    // Name, label, kind and required, as `ovd schema` gives them, in the schema's order.
    let schema = input_schema(&Profile::built_in());
    let want: Vec<String> = schema["inputs"]
        .as_array()
        .expect("inputs")
        .iter()
        .map(|e| {
            format!(
                "| `{}` | {} | {} | {} |",
                e["name"].as_str().expect("name"),
                e["label"].as_str().expect("label"),
                e["kind"].as_str().expect("kind"),
                if e["required"].as_bool().expect("required") {
                    "yes"
                } else {
                    ""
                },
            )
        })
        .collect();
    let have: Vec<String> = PAGE
        .lines()
        .skip_while(|l| !l.starts_with("| Input |"))
        .skip(2)
        .take_while(|l| l.starts_with('|'))
        .map(|l| l.trim_end().to_string())
        .collect();
    let want: Vec<String> = want.iter().map(|l| l.trim_end().to_string()).collect();
    assert_eq!(
        have, want,
        "regenerate the table of docs/python-api.md from `ovd schema`"
    );
}
