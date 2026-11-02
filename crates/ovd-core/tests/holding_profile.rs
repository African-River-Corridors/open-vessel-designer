//! Seam: the operation profile of a Type C tank (spec 0016 D2, D3, D7, D9; issue #869).
//!
//! Label: property. Each test states a rule of the spec: the default design basis, the leg states,
//! and the passage credit being zero without a gas consumer. No number here is a printed result.

use ovd_core::holding_profile::*;

#[test]
fn the_default_design_basis_is_twenty_days_of_passage_then_ten_idle() {
    let legs = default_design_basis();
    assert_eq!(legs.len(), 2);
    assert_eq!(legs[0].state, LegState::LadenPassage);
    assert_eq!(legs[0].days, 20.0);
    assert_eq!(legs[1].state, LegState::Idle);
    assert_eq!(legs[1].days, 10.0);
    assert_eq!(total_days(&legs), 30.0);
}

#[test]
fn only_a_laden_passage_and_idle_are_leg_states() {
    assert_eq!(LEG_STATES, &["laden_passage", "idle"]);
    for s in ["loading", "discharge", "ballast", ""] {
        assert!(LegState::from_key(s).is_none(), "{s}");
    }
    for s in LEG_STATES {
        assert_eq!(LegState::from_key(s).unwrap().key(), *s);
    }
}

#[test]
fn with_no_gas_consumer_the_passage_credit_is_zero() {
    // A calm-water use of 100 kg/h is on offer, but nothing takes the boil-off.
    assert_eq!(passage_credit_kg_h(false, Some(100.0)), 0.0);
    let leg = Leg {
        state: LegState::LadenPassage,
        days: 20.0,
        passage_speed_kn: Some(8.0),
        gas_use_kg_h: 0.0,
    };
    assert_eq!(leg_credit_kg_h(&leg, 9.7, false, Some(100.0)), 0.0);
    // Which Power Sources are consumers is checked against the format's list in ovd-format.
    assert!(has_gas_consumer("gas_fuelled"));
}

#[test]
fn with_a_consumer_a_typed_speed_credits_the_calm_water_use_and_without_the_speed_none() {
    let mut leg = Leg {
        state: LegState::LadenPassage,
        days: 20.0,
        passage_speed_kn: Some(8.0),
        gas_use_kg_h: 0.0,
    };
    assert_eq!(leg_credit_kg_h(&leg, 9.7, true, Some(100.0)), 100.0);
    assert_eq!(leg_credit_kg_h(&leg, 9.7, true, None), 0.0);
    // No typed speed: the leg takes the design service speed, marked default, and still credits.
    leg.passage_speed_kn = None;
    assert_eq!(leg_credit_kg_h(&leg, 9.7, true, Some(100.0)), 100.0);
    assert_eq!(passage_speed_kn(&leg, 9.7), Some((9.7, true)));
    // A typed speed overrides it.
    leg.passage_speed_kn = Some(8.0);
    assert_eq!(passage_speed_kn(&leg, 9.7), Some((8.0, false)));
    // No usable speed at all: no credit.
    leg.passage_speed_kn = None;
    assert_eq!(leg_credit_kg_h(&leg, 0.0, true, Some(100.0)), 0.0);
    assert_eq!(passage_speed_kn(&default_design_basis()[1], 9.7), None);
}

#[test]
fn an_idle_leg_credits_zero_unless_a_use_is_typed() {
    let mut leg = default_design_basis()[1];
    assert_eq!(leg_credit_kg_h(&leg, 9.7, true, Some(100.0)), 0.0);
    leg.gas_use_kg_h = 5.0;
    assert_eq!(leg_credit_kg_h(&leg, 9.7, false, None), 5.0);
}
