//! Seam: the Vessel Model, the meshes and the exports follow the stage of the results (issue #1041). A run with
//! damage screens the arrangement (issue #935), so the model and `ovd export` show the screened pick; a run that
//! skips damage shows the unscreened one, as the app's first evaluation does.
//!
//! Label: snapshot - the 140 x 29 x 12 m coastal LNG reference design of `damage_tanks_along.rs`: unscreened 2 x 2
//! (13.28 m), screened 2 x 3 (13.38 m). Provisional until #1009 lands, as there.

use ovd_format::damage::DamageControl;
use ovd_format::export::{bytes_hash, option_export, option_export_with_damage};
use ovd_format::{
    DesignOption, Profile, RunOutcome, VesselRun, fill_as_new, option_vessel_run,
    option_vessel_run_with_damage, run_option_with_damage,
};
use serde_json::{Value, json};

fn reference() -> DesignOption {
    let inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "moulded_depth_m": { "value": 12.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "loading_conditions": { "value": "is_code_2008", "source": "typed" }
    });
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    filled.design.unwrap().options.remove(0)
}

fn damage(run: bool) -> DamageControl {
    DamageControl {
        run,
        progress: None,
    }
}

fn results(option: &DesignOption, run: bool) -> Value {
    match run_option_with_damage(option, ovd_format::rule_set::built_in(), &[], &damage(run)) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// The tanks of a model: (number, outer diameter in m).
fn tanks(run: &VesselRun) -> (usize, f64) {
    let layout = run.model.tank_layout.as_ref().expect("a Tank Layout");
    (layout.tanks.len(), layout.outer_diameter_m)
}

/// Each stage's model holds that stage's tanks: the screened run's 2 x 3, the unscreened run's 2 x 2.
#[test]
fn the_model_holds_the_tanks_of_its_stage() {
    let option = reference();
    let mut found = Vec::new();
    for run in [true, false] {
        let r = results(&option, run);
        let want_n = (r["arrangement"]["n_wide"].as_i64().unwrap()
            * r["arrangement"]["n_long"].as_i64().unwrap()) as usize;
        let want_od = r["arrangement"]["tank_od_m"].as_f64().unwrap();
        let (n, od) = tanks(&option_vessel_run_with_damage(&option, &damage(run)).unwrap());
        assert_eq!(n, want_n, "damage run: {run}");
        assert!(
            (od - want_od).abs() < 1e-9,
            "damage run {run}: {od} against {want_od}"
        );
        found.push((n, od));
    }
    // The two stages differ on this design, so the test can tell them apart.
    assert_eq!((found[0].0, found[1].0), (6, 4));
    // `option_vessel_run` is the unscreened stage, the app's first evaluation.
    assert_eq!(tanks(&option_vessel_run(&option).unwrap()), found[1]);
}

/// `ovd export` runs damage, as `ovd run` and `ovd draw` do: its glTF is the screened model's. (The OBJ is the
/// hull alone, which on this design's typed depth is the same at both stages.)
#[test]
fn the_export_is_the_screened_model() {
    let option = reference();
    let screened = option_export("gltf", &option, None).unwrap();
    let unscreened = option_export_with_damage("gltf", &option, None, &damage(false)).unwrap();
    assert_ne!(bytes_hash(&screened), bytes_hash(&unscreened));
    // The glTF names its tank nodes: three rows along.
    let has = |name: &str| screened.windows(name.len()).any(|w| w == name.as_bytes());
    assert!(
        has("CT3P") && has("CT3S") && !has("CT4P"),
        "three tanks along"
    );
}
