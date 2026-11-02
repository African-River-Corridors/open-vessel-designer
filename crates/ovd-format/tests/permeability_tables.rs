//! Seam: `ovd_format::damage::permeability_table`, the Permeability Tables of spec 0012 D9 (issue #758), read from
//! the Rule Sets' data.
//!
//! Label: sourced - ADN 2025 9.3.2.15.1 (c) (printed p. 554) and 9.3.3.15.1 (c) (p. 594): 95 % in general, 0 % or
//! 95 % for tanks by whether they are full or empty at the maximum draught. IGC Code (MSC.370(93) as amended)
//! 2.5.2, annex p. 29: machinery 0.85, voids and hold spaces 0.95, consumable and other liquids 0 to 0.95.
//! ES-TRIN 2025/1 28.03(4)(d), page 200: 95 %, engine and service rooms 85 %, cargo holds 70 %, tanks 0 or 95 %.

use ovd_core::compartments::SpaceKind;
use ovd_format::damage::{built_in_permeability, permeability_table};
use ovd_format::rule_set::built_in;

/// The ADN liquid set keeps 0/95: a full cargo or slop tank 0, every other kind 0.95, whatever the filling. These
/// are the values the liquid damage path used in code before the table.
#[test]
fn the_adn_liquid_table_keeps_0_and_95() {
    let t = built_in_permeability("adn-tank", "adn-permeability").expect("adn-tank table");
    for kind in SpaceKind::ALL {
        let want = if matches!(kind, SpaceKind::CargoTank | SpaceKind::Slop) {
            0.0
        } else {
            0.95
        };
        for filling in [0.0, 0.5, 1.0] {
            assert_eq!(t.of(kind, filling), Some(want), "{}", kind.key());
        }
    }
}

/// IGC 2.5.2: machinery 0.85; voids and holds 0.95; a liquid space 0.95 empty, 0 full, in step between.
#[test]
fn the_igc_table_is_2_5_2() {
    let set = built_in().latest("igc-2g").expect("igc-2g");
    let t = permeability_table(set, "igc-permeability").expect("igc table");
    assert_eq!(t.of(SpaceKind::Machinery, 0.0), Some(0.85));
    assert_eq!(t.of(SpaceKind::CargoHold, 0.0), Some(0.95));
    assert_eq!(t.of(SpaceKind::Void, 0.0), Some(0.95));
    assert_eq!(t.of(SpaceKind::WingBallast, 0.0), Some(0.95));
    assert_eq!(t.of(SpaceKind::WingBallast, 1.0), Some(0.0));
    assert_eq!(t.of(SpaceKind::CargoTank, 0.5), Some(0.95 * 0.5));
}

/// ES-TRIN 28.03(4)(d): engine and service rooms 85 %, cargo holds 70 %, tanks 0 or 95 % as full or empty.
#[test]
fn the_es_trin_table_is_28_03_4_d() {
    let set = built_in().latest("es-trin-2025").expect("es-trin-2025");
    let t = permeability_table(set, "es-trin-permeability").expect("es-trin table");
    assert_eq!(t.of(SpaceKind::Machinery, 0.0), Some(0.85));
    assert_eq!(t.of(SpaceKind::CargoHold, 0.0), Some(0.70));
    assert_eq!(t.of(SpaceKind::DoubleBottomBallast, 0.0), Some(0.95));
    assert_eq!(t.of(SpaceKind::DoubleBottomBallast, 1.0), Some(0.0));
}

/// The tables are data: each names every kind, and a set version without the table gives an error that names it.
#[test]
fn the_tables_are_rule_set_data() {
    let lib = built_in();
    for (set, table) in [
        ("igc-2g", "igc-permeability"),
        ("adn-tank", "adn-permeability"),
        ("es-trin-2025", "es-trin-permeability"),
    ] {
        let t = permeability_table(lib.latest(set).unwrap(), table).expect(table);
        assert_eq!(t.rows.len(), SpaceKind::ALL.len(), "{table}");
    }
    let old = lib.find("adn-tank", "1.0.0").expect("adn-tank 1.0.0");
    let err = permeability_table(old, "adn-permeability").unwrap_err();
    assert!(err.contains("adn-permeability"), "{err}");
}
