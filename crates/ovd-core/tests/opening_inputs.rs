//! The object inputs of an opening and the Measure `opening.clear_of_cargo_face` (issue #913): what
//! a Rule's Applicability reads to pick the openings it governs.
//!
//! Label: property. No fixed number: the input of each opening is its own kind, space and face, the
//! new Measure is the negation of `opening.faces_cargo_area`, and the zone length is the `clamp`
//! shape of the Rule files at b 0.04, lo 3, hi 5.

use ovd_core::deckhouse_openings::*;
use ovd_core::measured::Measured;
use ovd_core::rule_trace::{MeasureValue, ObjectType};
use ovd_core::rules::MeasureRef;

fn frame(l_icll: f64) -> Frame {
    Frame {
        x_aft_m: 0.0,
        x_fwd_m: 20.0,
        width_m: 12.0,
        base_m: 6.0,
        tier_height_m: 2.6,
        tiers: 2,
        wheelhouse: Some((1.0, 5.0, 3.0, 11.2)),
        l_icll_m: l_icll,
    }
}

fn typed(face: Face, x: f64) -> Opening {
    Opening {
        id: "door".into(),
        kind: Kind::Entrance,
        space: Space::Accommodation,
        face,
        x_m: x,
        y_m: 6.0,
        z_m: 7.0,
        openable: true,
        gas_tight: false,
        tier: 1,
    }
}

fn measured(l: f64, typed_list: Option<&[Opening]>) -> Measured {
    Measured {
        openings: Some(place(&frame(l), typed_list)),
        ..Default::default()
    }
}

fn flag(m: &Measured, id: &str, index: usize) -> bool {
    let r = MeasureRef {
        id: id.to_string(),
        args: Vec::new(),
    };
    match m.value(&r, index).unwrap() {
        MeasureValue::Bool(b) => b,
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_opening_answers_its_own_kind_space_face_and_flags() {
    let m = measured(140.0, None);
    let p = m.openings.as_ref().unwrap();
    assert!(!p.openings.is_empty());
    for (i, o) in p.openings.iter().enumerate() {
        let ask = |name: &str| m.object_input(ObjectType::Opening.key(), i, name);
        assert_eq!(ask("opening_kind").as_deref(), Some(o.opening.kind.key()));
        assert_eq!(ask("opening_space").as_deref(), Some(o.opening.space.key()));
        assert_eq!(ask("opening_face").as_deref(), Some(o.opening.face.key()));
        let yn = |b: bool| Some(if b { "yes" } else { "no" }.to_string());
        assert_eq!(ask("opening_openable"), yn(o.opening.openable));
        assert_eq!(ask("opening_gas_tight"), yn(o.opening.gas_tight));
        // Each answer is one of the values the vocabulary lists.
        for (name, values) in OPENING_INPUTS {
            assert!(values.contains(&ask(name).unwrap().as_str()), "{name}");
        }
    }
    // The wheelhouse doors of the default layout are in the wheelhouse and gas-tight.
    let door = p
        .openings
        .iter()
        .position(|o| o.opening.id == "wheelhouse door, port")
        .unwrap();
    assert_eq!(
        m.object_input("opening", door, "opening_space").as_deref(),
        Some("wheelhouse")
    );
    assert_eq!(
        m.object_input("opening", door, "opening_gas_tight")
            .as_deref(),
        Some("yes")
    );
}

#[test]
fn an_unknown_input_an_unknown_opening_or_no_openings_give_no_answer() {
    let m = measured(140.0, None);
    assert_eq!(m.object_input("opening", 0, "opening_colour"), None);
    assert_eq!(m.object_input("opening", 10_000, "opening_kind"), None);
    assert_eq!(
        Measured::default().object_input("opening", 0, "opening_kind"),
        None
    );
    assert_eq!(opening_input_values("opening_colour"), None);
}

#[test]
fn clear_of_cargo_face_is_the_negation_of_faces_cargo_area_for_every_opening() {
    // The default layout, and a typed list with one opening on each face.
    let list = [
        typed(Face::Front, 20.0),
        typed(Face::Side, 10.0),
        typed(Face::Aft, 0.0),
    ];
    for m in [measured(140.0, None), measured(70.0, Some(&list))] {
        let n = m.openings.as_ref().unwrap().openings.len();
        assert!(n >= 3);
        for i in 0..n {
            assert_ne!(
                flag(&m, "opening.clear_of_cargo_face", i),
                flag(&m, "opening.faces_cargo_area", i),
                "opening {i}"
            );
        }
    }
    let m = measured(70.0, Some(&list));
    assert!(!flag(&m, "opening.clear_of_cargo_face", 0));
    assert!(flag(&m, "opening.clear_of_cargo_face", 1));
    assert!(flag(&m, "opening.clear_of_cargo_face", 2));
}

#[test]
fn the_zone_length_is_the_clamp_shape_of_the_rule_files() {
    // `clamp`: a + x * b / c, then at most hi, then at least lo (`rule_trace.rs`), with the
    // coefficients of the IGC 3.2.4.1 rows: a 0, b 0.04, c 1, lo 3, hi 5.
    let clamp = |x: f64| {
        let v = 0.0 + x * 0.04 / 1.0;
        let v = if v < 5.0 { v } else { 5.0 };
        if v > 3.0 { v } else { 3.0 }
    };
    for k in 0..=2000 {
        let l = f64::from(k) * 0.1;
        assert_eq!(zone_length_m(l), clamp(l), "L = {l}");
    }
}
