//! Seam: `ovd_core::gas_damage`, the gas damage case loop in the core (issue #935). The serial loop of the
//! core and the threaded loop of `ovd-format` give the same verdicts, and each is named `<condition>: <case>`.
//!
//! Label: property

use ovd_core::gas_arrangement::{
    FitOptions, IsCodeConditions, fit, is_code_cargo, is_code_conditions,
};
use ovd_core::gas_damage::{gas_damage_plan, run_serial};
use ovd_core::hydrostatics::Hull;
use ovd_core::vessel_model::{StructureBlock, VesselModelInputs, assemble};

#[test]
fn gas_damage_runs_in_core() {
    let o = FitOptions {
        is_code: Some(IsCodeConditions {
            consumables_t: 60.0,
            may_sail_without_ballast: false,
            user_ballast: None,
        }),
        ..FitOptions::default()
    };
    let f = fit(110.0, 11.4, "LNG", 5.7, &o).unwrap();
    let r = &f.engine_result;
    let ic = o.is_code.as_ref().unwrap();
    let cargo = is_code_cargo(r, f.payload_t, 110.0, 11.4, 5.7, &o, ic);
    let conditions = is_code_conditions(r, &f.engine_inputs, cargo, &o, ic).unwrap();
    assert!(!conditions.is_empty());
    let hull = Hull::build(r.hull_spec.as_ref().unwrap()).unwrap();
    let model = assemble(&VesselModelInputs {
        capacity: f.engine_inputs.clone(),
        result: r.clone(),
        hull: hull.table.clone(),
        structure: StructureBlock::default(),
    });
    let spaces: Vec<_> = model.spaces.iter().map(|c| c.space.clone()).collect();
    let lib = ovd_format::rule_set::built_in();
    let set = lib.latest("igc-2g").unwrap();
    let plan = gas_damage_plan(set, &spaces, &hull).unwrap();
    let rho = o.water_density_t_per_m3;

    let run = run_serial(&plan, &spaces, &hull, &conditions, rho);
    assert_eq!(run.verdicts.len(), run.cases.len());
    assert_eq!(run.verdicts.len(), plan.case_count(&hull, &conditions, rho));
    for (v, c) in run.verdicts.iter().zip(&run.cases) {
        assert_eq!(v.name, format!("{}: {}", c.condition, c.name));
        assert_eq!(v.stages.len(), 3, "{}", v.name);
    }
    assert!(!run.governing.is_empty());
    // The threaded loop of `ovd-format` is the same loop: the same verdicts, bit for bit.
    let (verdicts, block) =
        ovd_format::damage::gas_damage_verdicts(set, &spaces, &hull, &conditions, rho).unwrap();
    assert_eq!(verdicts, run.verdicts);
    assert_eq!(block["cases"].as_array().unwrap().len(), run.cases.len());
}
