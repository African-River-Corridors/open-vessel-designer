//! The `.vdesign` format, the Defaults Profile and validation, running a Design, and the `eval`
//! dispatch over every entry point of the calculation core (the Python package's seam).

mod api;
pub mod class_input;
mod convoy;
mod cost;
pub mod crew;
pub mod damage;
pub mod deckhouse_openings;
mod eval;
pub mod export;
mod formation;
pub mod ga;
mod gas_arrangement;
mod gas_fuel;
pub mod gas_stability;
pub mod holding_profile;
mod holding_time;
mod liquid;
mod objects;
pub mod offsets;
pub mod openings;
pub mod profile;
pub mod rule_set;
mod run;
pub mod session;
pub mod shipd;
mod simulate;
pub mod steel;
mod text_ids;
pub mod trace;
pub mod vdesign;

pub use api::simulate_damage_with;
pub use eval::{ENTRY_POINTS, EvalError, eval, parse_case};
pub use ga::{
    CONVOY_NO_VIEWS, GaViews, NO_ARRANGEMENT_PLAN, NO_DAMAGE_DRAWING, NO_TUG_GA,
    arrangement_plan_for, arrangement_plan_named, damage_drawings_by_rule,
    damage_drawings_by_rule_named, damage_drawings_for, damage_drawings_named, ga_views,
    ga_views_for, ga_views_named, ga_views_of, option_name_fits, tug_ga_for, tug_ga_named,
};
pub use profile::Profile;
pub use run::{
    CONVOY_NOT_AVAILABLE, HashStatus, RunOption, RunOutcome, UNHASHED_KEYS, VesselRun, Waterline,
    canonical_json, hash_warning, mesh_hash, option_hull_table, option_meshes,
    option_model_waterlines, option_vessel_model, option_vessel_run, option_vessel_run_with_damage,
    results_hash, run_design, run_design_with, run_design_with_damage, run_option, run_option_with,
    run_option_with_damage, run_option_with_vessel, simulate_params_of, stamp, summary_lines,
};
pub use session::{
    DesignSession, Evaluation, Message, OpenReport, OptionEvaluation, Scope, Severity, input_schema,
};
pub use text_ids::msg_json;
pub use vdesign::{
    Checked, Design, DesignOption, Report, ResultsStage, Source, StoredInput, check, fill,
    fill_as_new, new_design, service_area_required, with_river_limits, with_service_area,
};
