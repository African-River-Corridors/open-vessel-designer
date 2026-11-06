//! Seam: one insulation thickness (issue #919). The thickness of the tank design is the one every
//! path reads. A capacity result that carries no thickness is drawn with the design's, not with a
//! stand-in of 350 mm.
//!
//! Label: property

use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::ga_adapter::to_ga;

fn gas(thickness_mm: f64) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LNG".into();
    i.tank_wall_thickness = Some(25.0);
    i.tank_design.insulation_thickness_mm = thickness_mm;
    i
}

#[test]
fn a_result_with_no_thickness_is_drawn_with_the_design_thickness() {
    let i = gas(123.0);
    let mut r = compute(&i).unwrap();
    assert_eq!(r.insulation_thickness_mm, Some(123.0));
    let od = r.tank_outer_diameter_m.unwrap();
    r.insulation_thickness_mm = None;
    r.tank_inner_diameter_m = None;
    let (gi, gr) = to_ga(&i, &r, None, true).unwrap();
    assert_eq!(gi.inputs.insulation_thickness, 123.0);
    // The cargo space is the outer diameter less the insulation and the steel, both sides.
    let inner = gr.tanks.unwrap().inner_diameter;
    assert!(
        (inner - (od - 2.0 * 0.123 - 2.0 * 0.025)).abs() < 1e-12,
        "{inner}"
    );
}

#[test]
fn the_drawn_thickness_follows_the_design_not_a_constant() {
    for t in [0.0, 123.0, 400.0] {
        let i = gas(t);
        let mut r = compute(&i).unwrap();
        r.insulation_thickness_mm = None;
        let (gi, _) = to_ga(&i, &r, None, true).unwrap();
        assert_eq!(gi.inputs.insulation_thickness, t);
    }
}
