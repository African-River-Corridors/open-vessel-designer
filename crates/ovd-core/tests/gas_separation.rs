//! Seam: the public `ovd_core::gas_separation` API. Expected values are printed by the original
//! Python (`core/gas_separation.py`), not worked out by hand, except the aft end-bulkhead space check,
//! which replaced the type C cofferdam check in issue #280 (Label: sourced - LR inland rules Pt 4 Ch 4
//! 3.5.3 and ADN 2025 9.3.1.11.3(a): 0.20 m, or 0.50 m for plane tank ends). Spec 0011 (D17, D18)
//! changed the rule, not a source: the hull keeps no length for the opening setback, and the hold is
//! segregated by a gastight A-60 bulkhead or a cofferdam in its place (Label: sourced - IGC Code,
//! MSC.370(93), 3.1.2 and 3.2.4.1). The aft separations below are worked from those two rules.

use ovd_core::gas_separation::{
    COFFERDAM_MIN_M, END_BULKHEAD_SPACE_M, END_BULKHEAD_SPACE_PRESSURE_M, GasSeparationInputs,
    OPENING_SETBACK_M, compute, default_end_space,
};

const CLAUSE_WARNING: &str = "Separation clauses are cited from the classification rules for \
inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release.";

#[test]
fn constants_are_the_cited_values() {
    assert_eq!(COFFERDAM_MIN_M, 0.60);
    assert_eq!(OPENING_SETBACK_M, 2.00);
    assert_eq!(END_BULKHEAD_SPACE_M, 0.50);
    assert_eq!(END_BULKHEAD_SPACE_PRESSURE_M, 0.20);
}

#[test]
fn the_defaults_are_the_type_g_rule_for_domed_ends() {
    // Issue #280 (maintainer decision, 2026-10-22): the end space defaults to the type G rule, 0.20 m
    // for domed tank ends and 0.50 m for plane ends, in place of the 0.60 m type C cofferdam the
    // original Python used. The Python constant `COFFERDAM_MIN_M` keeps its value.
    assert_eq!(default_end_space(false), 0.20);
    assert_eq!(default_end_space(true), 0.50);
    let i = GasSeparationInputs::default();
    assert_eq!(i.cofferdam_m, 0.20);
    assert_eq!(i.opening_setback_m, 2.00);
    assert_eq!(i.fwd_end_bulkhead_m, 0.20);
    // Issue #381: the setback is measured from the cargo area.
    assert!(i.setback_from_cargo_area);
    assert!(i.is_pressure_tank);
    assert_eq!(i.side_clearance_m, 0.0);
    assert_eq!(i.double_bottom_m, 0.0);
}

#[test]
fn golden_defaults() {
    let r = compute(&GasSeparationInputs::default()).unwrap();
    // Spec 0011 (D17): the aft separation is the hold end clearance alone; the A-60 bulkhead takes
    // no length and the opening setback is measured on the deckhouse.
    assert_eq!(r.aft_separation_m, 0.2);
    assert_eq!(r.fwd_separation_m, 0.2);
    assert_eq!(r.cofferdam_m, 0.2);
    assert_eq!(r.hold_cofferdam_m, 0.0);
    assert_eq!(r.service_space_m, 0.0);
    assert!((r.total_length_taken_m - 0.4).abs() < 1e-12);
    assert!(r.ok());
    assert_eq!(r.warnings, vec![CLAUSE_WARNING]);
    let names: Vec<&str> = r.checks.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "aft_end_bulkhead_space",
            "hold_machinery_segregation",
            "fwd_end_bulkhead_space"
        ]
    );
    assert_eq!(
        r.checks[0].message,
        "aft end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3)"
    );
    assert_eq!(
        r.checks[1].message,
        "hold segregated from the stern block by a gastight A-60 bulkhead (IGC 3.1.2)"
    );
    assert_eq!(
        r.checks[2].message,
        "forward end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3)"
    );
}

#[test]
fn a_thin_aft_space_fails() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.1,
        ..Default::default()
    })
    .unwrap();
    assert!(!r.ok());
    assert_eq!(r.aft_separation_m, 0.1);
    assert_eq!(r.service_space_m, 0.0);
    assert_eq!(
        r.checks[0].message,
        "aft end-bulkhead space 0.10 m >= 0.20 m (domed tank ends, cl. 3.5.3)"
    );
}

/// Issue #280: the 0.60 m cofferdam of the type C part (3.6.13) is not a limit on a gas Design. The
/// type G space is 0.20 m for domed ends, at and just below the limit, and 0.50 m for plane ends.
#[test]
fn the_aft_end_space_is_the_type_g_limit_not_the_type_c_cofferdam() {
    let check = |cofferdam_m: f64, plane: bool| {
        compute(&GasSeparationInputs {
            cofferdam_m,
            plane_tank_ends: plane,
            ..Default::default()
        })
        .unwrap()
        .checks[0]
            .ok
    };
    assert!(check(0.4, false), "0.40 m is enough for domed ends");
    assert!(check(0.20, false));
    assert!(!check(0.19, false));
    assert!(!check(0.4, true), "plane ends need 0.50 m");
    assert!(check(0.50, true));
    assert!(!check(0.49, true));
}

/// Spec 0011 (D17): the opening setback input no longer sets a length of hull, whatever it holds.
#[test]
fn the_opening_setback_adds_no_length() {
    for opening_setback_m in [0.0, 1.0, 2.0, 5.0] {
        for setback_from_cargo_area in [true, false] {
            let r = compute(&GasSeparationInputs {
                opening_setback_m,
                setback_from_cargo_area,
                ..Default::default()
            })
            .unwrap();
            assert_eq!(r.aft_separation_m, 0.2);
            assert_eq!(r.service_space_m, 0.0);
            assert!(r.ok());
        }
    }
}

#[test]
fn a_wider_end_clearance_is_the_whole_separation() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 3.0,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(r.aft_separation_m, 3.0);
    assert_eq!(r.service_space_m, 0.0);
    assert_eq!(r.total_length_taken_m, 3.2);
}

/// Spec 0011 (D18): no length is charged twice. The aft separation is the hold end clearance, plus a
/// cofferdam only when one is chosen; the cofferdam replaces the A-60 bulkhead, which takes no
/// length, and is not added to a setback or a second cofferdam.
#[test]
fn the_aft_separation_is_the_end_clearance_plus_a_chosen_cofferdam_only() {
    let bulkhead = compute(&GasSeparationInputs::default()).unwrap();
    assert_eq!(bulkhead.aft_separation_m, bulkhead.cofferdam_m);
    let cofferdam = compute(&GasSeparationInputs {
        hold_cofferdam_m: 0.8,
        ..Default::default()
    })
    .unwrap();
    assert!((cofferdam.aft_separation_m - (0.2 + 0.8)).abs() < 1e-12);
    assert_eq!(cofferdam.hold_cofferdam_m, 0.8);
    assert_eq!(cofferdam.service_space_m, 0.0);
    let check = |r: &ovd_core::gas_separation::GasSeparationResult| {
        r.checks
            .iter()
            .find(|c| c.name == "hold_machinery_segregation")
            .unwrap()
            .clone()
    };
    assert!(check(&bulkhead).ok);
    assert!(check(&cofferdam).ok);
    assert_eq!(
        check(&cofferdam).message,
        "hold cofferdam 0.80 m >= 0.60 m in place of the A-60 bulkhead (IGC 3.1.2; width cl. 3.6.13)"
    );
    // A cofferdam too narrow to enter fails; it does not fall back to the bulkhead.
    let thin = compute(&GasSeparationInputs {
        hold_cofferdam_m: 0.3,
        ..Default::default()
    })
    .unwrap();
    assert!(!check(&thin).ok);
    assert!(!thin.ok());
}

/// Issue #215: the inland type-G floors are gone. Clearances given in the inputs add no check, however
/// small they are; the three separation checks are all there is.
#[test]
fn as_designed_clearances_add_no_check() {
    let r = compute(&GasSeparationInputs {
        fwd_end_bulkhead_m: 0.3,
        plane_tank_ends: true,
        side_clearance_m: 0.05,
        double_bottom_m: 0.05,
        ..Default::default()
    })
    .unwrap();
    assert!(!r.ok());
    assert_eq!(r.total_length_taken_m, 0.5);
    let names: Vec<&str> = r.checks.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "aft_end_bulkhead_space",
            "hold_machinery_segregation",
            "fwd_end_bulkhead_space"
        ]
    );
    assert_eq!(
        r.checks[2].message,
        "forward end-bulkhead space 0.30 m >= 0.50 m (plane tank ends, cl. 3.5.3)"
    );
    assert!(r.checks.iter().all(|c| !c.name.contains("type_g")));
}

/// Issue #281: the space is keyed to the tank end shape (3.5.3), not to the tank kind. A tank that
/// is not a pressure tank but has domed ends needs 0.20 m, as a pressure tank does.
#[test]
fn the_end_space_follows_the_tank_ends_not_the_tank_kind() {
    for is_pressure_tank in [true, false] {
        let r = compute(&GasSeparationInputs {
            fwd_end_bulkhead_m: 0.3,
            is_pressure_tank,
            ..Default::default()
        })
        .unwrap();
        assert!(
            r.checks[2].ok,
            "domed ends, pressure tank {is_pressure_tank}"
        );
    }
    let plane = compute(&GasSeparationInputs {
        fwd_end_bulkhead_m: 0.3,
        plane_tank_ends: true,
        ..Default::default()
    })
    .unwrap();
    assert!(!plane.checks[2].ok);
}

/// The checks allow a 1e-9 slack, so a value a hair under the floor still passes.
#[test]
fn the_checks_allow_one_nano_metre_of_slack() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.2 - 1e-10,
        ..Default::default()
    })
    .unwrap();
    assert!(r.checks[0].ok);
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.2 - 1e-8,
        ..Default::default()
    })
    .unwrap();
    assert!(!r.checks[0].ok);
}

#[test]
fn negative_separations_raise_the_python_message() {
    for bad in [
        GasSeparationInputs {
            cofferdam_m: -0.1,
            ..Default::default()
        },
        GasSeparationInputs {
            opening_setback_m: -0.1,
            ..Default::default()
        },
        GasSeparationInputs {
            fwd_end_bulkhead_m: -0.1,
            ..Default::default()
        },
        GasSeparationInputs {
            hold_cofferdam_m: -0.1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            compute(&bad).unwrap_err(),
            "separations must be non-negative"
        );
    }
}

/// Zero is a valid separation: only a negative one raises.
#[test]
fn zero_separations_are_valid_and_fail_their_checks() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.0,
        opening_setback_m: 0.0,
        fwd_end_bulkhead_m: 0.0,
        ..Default::default()
    })
    .unwrap();
    let oks: Vec<bool> = r.checks.iter().map(|c| c.ok).collect();
    // The A-60 bulkhead (no hold cofferdam) still segregates the hold.
    assert_eq!(oks, [false, true, false]);
}
