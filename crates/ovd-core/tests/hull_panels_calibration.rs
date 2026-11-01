//! Issue #722: the panel steel of the tank-barge calibration library, printed beside the steel each
//! case implies (`calibration/tank_lightship.yaml`: k_implied x LOA x B x D, the implied lightship less
//! the outfit). Run with `--nocapture` for the table that `calibration/hull_steel_cases.yaml` records.
//! Label: property
//!
//! The geometry each case does not publish is an Estimate, the same for every case: a double bottom
//! of B/15 and a double side of 0.76 m (the MARPOL Annex I regulation 19.6 minimums for a tanker of
//! under 5,000 t deadweight), a cargo zone of the middle 84 % of the length, one hold per 20 m, two
//! tanks abreast, a closed deck, Cb 0.927 (the box-barge value of `factors.yaml`). These are inland
//! barges, built to river rules lighter than the unrestricted-service rules the model uses. The rule
//! length (ABS 3-1-1/3.1, issue #740) is the box's: 96 % of the length overall, as no case publishes
//! its rakes.

use ovd_core::hull_panels::{
    HullStructureSpec, ScantlingChoices, double_hull_panels, rule_length_m,
};
use ovd_core::vessel_model::StructureBlock;

/// (case id, LOA, B, D, k_implied t/m3), from `calibration/tank_lightship.yaml` and `cases.yaml`.
const CASES: &[(&str, f64, f64, f64, f64)] = &[
    ("meb-rob20-belmax", 118.79, 22.8, 4.0, 0.0733),
    ("meb-rob07-mh4001", 95.23, 16.8, 5.0, 0.109),
    ("kirby-10071-200x35", 60.96, 10.67, 3.66, 0.1112),
    ("kirby-10201-200x35", 60.96, 10.67, 3.66, 0.1354),
    ("kirby-10083-200x35", 60.96, 10.67, 3.66, 0.1238),
    ("kirby-10074-195x35", 59.44, 10.67, 3.66, 0.1281),
    ("cgbm-102-200x35", 60.96, 10.67, 3.81, 0.1104),
    ("mer-13-200x35", 60.96, 10.67, 3.96, 0.1094),
    ("kirby-301-250x50", 76.2, 15.24, 3.66, 0.1114),
    ("kirby-407-263x54", 80.16, 16.46, 4.57, 0.0954),
    ("hfl-406-297x54", 90.53, 16.46, 3.66, 0.1291),
    ("kirby-27773-297x54", 90.65, 16.46, 3.66, 0.114),
    ("kirby-32300b-297x54", 90.53, 16.46, 3.66, 0.08),
    ("ctco-358-297x54", 90.65, 16.46, 3.66, 0.1415),
    ("htco-3064-320x50", 97.54, 15.24, 3.96, 0.1291),
];

fn tank_barge(l: f64, b: f64, d: f64) -> HullStructureSpec {
    HullStructureSpec {
        loa_m: l,
        rule_length_m: rule_length_m(d, |_| l),
        beam_m: b,
        depth_m: d,
        block_coefficient: 0.927,
        cargo_zone_aft_m: 0.08 * l,
        cargo_zone_fwd_m: 0.92 * l,
        double_bottom_m: (b / 15.0).max(0.76),
        wing_width_m: 0.76,
        wing_top_m: d,
        deck_opening_width_m: 0.0,
        wing_deck_opening_width_m: 0.0,
        holds_along: ((0.84 * l / 20.0).round() as usize).max(2),
        tanks_abreast: 2,
        steel_density: 7.85,
        structure: StructureBlock::default(),
    }
}

/// The box hull of a case as a Sections Table, for the plate model (issue #745).
fn box_table(l: f64, b: f64, d: f64) -> ovd_core::sections::SectionsTable {
    ovd_core::hydrostatics::Hull::build(&ovd_core::hydrostatics::HullSpec {
        loa: l,
        beam: b,
        depth: d,
        bow_rake_length_m: 0.0,
        bow_rake_rise_m: d,
        stern: None,
        shape: Default::default(),
        downflooding_deg: None,
        openings: Vec::new(),
    })
    .unwrap()
    .table
}

/// The panel steel of #722 beside the steel the lightship takes since issue #745: the plate model's
/// ordered weight plus the allowance, its shell laid out on the box's Sections Table (end plating
/// included) and its hold bulkheads at the panel model's Estimates.
#[test]
fn the_library_table() {
    println!(
        "| case | L x B x D (m) | implied steel (t) | panel steel (t) | residual (%) | plate steel (t) | residual (%) |"
    );
    println!("|---|---|---:|---:|---:|---:|---:|");
    let mut res = Vec::new();
    let mut res_plate = Vec::new();
    for (id, l, b, d, k) in CASES {
        let s = tank_barge(*l, *b, *d);
        let m = double_hull_panels(&s, &ScantlingChoices::default());
        let table = box_table(*l, *b, *d);
        let (_, plates) = ovd_core::hull_plates::gas_hull_steel(&s, Some((&table, &[])));
        let implied = k * l * b * d;
        let r = (m.total_t / implied - 1.0) * 100.0;
        let rp = (plates.total_t / implied - 1.0) * 100.0;
        res.push(r);
        res_plate.push(rp);
        println!(
            "| {id} | {l} x {b} x {d} | {implied:.0} | {:.0} | {r:+.0} | {:.0} | {rp:+.0} |",
            m.total_t, plates.total_t
        );
        assert!(m.total_t.is_finite() && m.total_t > 0.0, "{id}");
        assert!(plates.total_t.is_finite() && plates.total_t > 0.0, "{id}");
    }
    for (name, r) in [("panel", &mut res), ("plate", &mut res_plate)] {
        r.sort_by(f64::total_cmp);
        println!(
            "{name}: median residual {:+.1} %, range {:+.0} to {:+.0} %",
            r[r.len() / 2],
            r[0],
            r[r.len() - 1]
        );
    }
}

/// The LNG bunker vessel concept of `calibration/hull_steel_cases.yaml` (`lng-bunker-4200-concept`),
/// run as that file records: 95 x 20 x 10 m, LNG, one tank across, two along, tanks at most 11 m OD,
/// fill 0.98. Prints the hull steel and lightship since issue #745 beside the panel steel before it
/// (the panel model on the arrangement's box rule length), and the residual to the published 2,687 t.
#[test]
fn the_bunker_concept() {
    use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
    let mut i = BargeCapacityInputs::new(95.0, 20.0, 10.0);
    i.cargo_type = "LNG_ethane".into();
    i.n_tanks_wide = Some(1);
    i.n_tanks_long = 2;
    i.max_tank_od_m = 11.0;
    i.fill_ratio = 0.98;
    let r = compute(&i).unwrap();
    let b = r.gas_basis.as_ref().unwrap();
    let before = ovd_core::hull_panels::hull_panels(
        &ovd_core::hull_panels::HullStructureSpec::of_gas_arrangement(
            &b.inputs,
            &b.tanks,
            b.inputs.block_coefficient,
            StructureBlock::default(),
            None,
        ),
    )
    .total_t;
    let steel = b.weights.hull_steel_total;
    let factors = 1.0 + b.inputs.piping_factor + b.inputs.outfitting_factor;
    let lightship_before = r.lightship_t - (steel - before) * factors;
    println!(
        "bunker concept: hull steel {steel:.0} t (panel before {before:.0} t); lightship {:.0} t \
         (before {lightship_before:.0} t); residual to 2687 t {:+.1} % (before {:+.1} %)",
        r.lightship_t,
        (r.lightship_t / 2687.0 - 1.0) * 100.0,
        (lightship_before / 2687.0 - 1.0) * 100.0
    );
    assert!(steel > 0.0 && r.lightship_t > 0.0);
}
