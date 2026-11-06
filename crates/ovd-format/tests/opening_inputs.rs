//! The opening object inputs in OVD's vocabulary for the Rule loader (issue #913), and the new
//! Measure in `ovd rules --measures`.
//!
//! Label: property. No fixed number: each name is an object input with the values the engine
//! answers, and a Rule file may name them in an Applicability.

use arc_rules::domain::{Domain, InputScope};
use ovd_format::rule_set::Ovd;

#[test]
fn the_opening_inputs_are_object_inputs_with_their_values() {
    for (name, values) in ovd_core::deckhouse_openings::OPENING_INPUTS {
        let def = Ovd.input(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(def.scope, InputScope::Object, "{name}");
        assert_eq!(&def.values, values, "{name}");
    }
    assert!(Ovd.input("opening_colour").is_none());
}

#[test]
fn the_measure_is_registered_on_the_opening_object() {
    let m = Ovd.measure("opening.clear_of_cargo_face").unwrap();
    assert_eq!(m.object, "opening");
}
