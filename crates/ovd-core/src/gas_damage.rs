//! The IGC, ADN and ES-TRIN damage cases of a gas hull (spec 0012, issues #757 to #763, #935): the damage data a
//! Rule Set gives the case generator, the Permeability Tables, and the loop over the cases of a loading
//! condition, as typed values. `ovd-format` renders the results block from them and runs the cases on worker
//! threads; the arrangement search calls the loop here, so it needs no callback.
//!
//! Reading the tables takes a Rule Set from `arc-rules` (the kernel without its loader), so the module is
//! `no_std` like the rest of the core.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use arc_rules::eval::{MeasureValue, table_value};
use arc_rules::model::RuleSet;

use crate::compartments::{Space, SpaceKind};
use crate::damage_cases::{
    BoxExtents, DamageCase, Permeability, PermeabilityTable, breached_cargo_at_stage,
    deck_edge_immersed, gas_flooding, without_breached_cargo,
};
use crate::damage_stability::{
    DamageCriteria, DamageOutcome, Floats, FloodedVolume, assess, intermediate_stages_releasing,
    righting_lever_at,
};
use crate::hydrostatics::Hull;
use crate::loading_conditions::{LoadedCondition, Role};
use crate::measured::DamageVerdict;
use crate::sections::{Gravity, Waterplane};

/// The IGC damage data of a hull of length `loa_m` and beam `beam_m`: the extents of IGC 2.3.1, and whether
/// the bulkheads of a machinery space aft stay intact (IGC 2.6.1.3, a type 2G ship of 150 m or less).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IgcDamageData {
    pub extents: BoxExtents,
    pub machinery_aft_intact: bool,
}

/// The table ids of a set that holds IGC damage data.
pub const IGC_DAMAGE_TABLES: &[&str] = &[
    "igc-damage-side-long",
    "igc-damage-side-inboard",
    "igc-damage-bottom-long",
    "igc-damage-bottom-width-forward",
    "igc-damage-bottom-width",
    "igc-damage-bottom-height",
    "igc-damage-forward-zone",
    "igc-damage-machinery-aft-intact",
];

/// Read the IGC damage data from a Rule Set's tables. The two longitudinal extents are keyed on `L^(2/3)`;
/// the widths and the height on `B`; the forward zone and the standard of damage on `L`. An error names a
/// table the set does not hold.
pub fn igc_damage_data(set: &RuleSet, loa_m: f64, beam_m: f64) -> Result<IgcDamageData, String> {
    let at = |id: &str, v: f64| -> Result<f64, String> {
        let table = set
            .tables
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("{} holds no table `{id}`", set.key()))?;
        table_value(table, &MeasureValue::Number(v))
    };
    let cbrt = libm::cbrt(loa_m);
    let l23 = cbrt * cbrt;
    Ok(IgcDamageData {
        extents: BoxExtents {
            side_long_m: at("igc-damage-side-long", l23)?,
            side_inboard_m: at("igc-damage-side-inboard", beam_m)?,
            bottom_fwd_long_m: at("igc-damage-bottom-long", l23)?,
            bottom_fwd_width_m: at("igc-damage-bottom-width-forward", beam_m)?,
            bottom_aft_long_m: at("igc-damage-bottom-long", l23)?,
            bottom_aft_width_m: at("igc-damage-bottom-width", beam_m)?,
            bottom_height_m: at("igc-damage-bottom-height", beam_m)?,
            forward_zone: at("igc-damage-forward-zone", loa_m)?,
        },
        machinery_aft_intact: at("igc-damage-machinery-aft-intact", loa_m)? != 0.0,
    })
}

/// Read a Permeability Table from a Rule Set (spec 0012 D9). Table `table_id` gives the permeability of each
/// `SpaceKind` by its key (a liquid space: the value when it is empty). The kinds that table `<table_id>-by-filling`
/// names (if the set has it) take their value times the empty fraction of the condition's filling. An error
/// names a missing table or a kind the table does not give.
pub fn permeability_table(set: &RuleSet, table_id: &str) -> Result<PermeabilityTable, String> {
    let find = |id: &str| set.tables.iter().find(|t| t.id == id);
    let table =
        find(table_id).ok_or_else(|| format!("{} holds no table `{table_id}`", set.key()))?;
    let by_filling = find(&format!("{table_id}-by-filling"));
    // Issue #763: the kinds that table `<table_id>-gross-floor` names take the greater of their value times the net
    // volume and the floor times the gross volume.
    let gross_floor = find(&format!("{table_id}-gross-floor"));
    let mut rows = Vec::new();
    for kind in SpaceKind::ALL {
        let key = MeasureValue::Text(kind.key().to_string());
        let value = table_value(table, &key).map_err(|e| {
            format!(
                "{}: `{table_id}` gives no value for `{}`: {e}",
                set.key(),
                kind.key()
            )
        })?;
        let liquid = by_filling.is_some_and(|t| table_value(t, &key).is_ok_and(|v| v != 0.0));
        let floor = gross_floor.and_then(|t| table_value(t, &key).ok());
        rows.push((
            kind,
            match floor {
                Some(floor) => Permeability::WithGrossFloor { net: value, floor },
                None if liquid => Permeability::ByFilling(value),
                None => Permeability::Fixed(value),
            },
        ));
    }
    Ok(PermeabilityTable { rows })
}

/// What a gas damage run is given: the cases, the Permeability Table, the criteria of the final stage and of the
/// intermediate stages, and how the block names its Rule Set and its method.
#[derive(Debug, Clone)]
pub struct GasRun {
    pub cases: Vec<DamageCase>,
    pub table: PermeabilityTable,
    pub criteria: DamageCriteria,
    pub stage_criteria: DamageCriteria,
    pub set_key: String,
    pub method: &'static str,
    /// Damage the full load departure only (ES-TRIN 28.03(3): the cargo for the maximum draught, evenly spread,
    /// with full supplies), not every IS Code condition.
    pub full_load_only: bool,
}

/// The box rules a gas hull is run through besides IGC (issue #763).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasBoxRule {
    /// ADN 2025 9.3.1.15 (type G), from the `adn-gas` set.
    AdnGas,
    /// ES-TRIN 2025/1 28.03(4) to (6), a craft over 110 m, from the `es-trin-2025` set.
    EsTrin,
}

/// The cases and tables of the IGC damage run (issue #759), not yet run (issue #923: a run that skips damage
/// still counts them). The cases, the standard of damage and the permeabilities come from `set` (an `igc-2g`
/// version that holds the tables; an older one gives an error naming the missing table).
pub fn gas_damage_plan(set: &RuleSet, spaces: &[Space], hull: &Hull) -> Result<GasRun, String> {
    use crate::damage_cases::{StandardOfDamage, igc_damage_cases};
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let data = igc_damage_data(set, loa, beam)?;
    let table = permeability_table(set, "igc-permeability")?;
    let standard = StandardOfDamage::igc_2g(spaces, loa, data.machinery_aft_intact);
    let cases = igc_damage_cases(spaces, loa, beam, &data.extents, &standard, &|k| {
        table.of(k, 0.0).unwrap_or(0.95)
    });
    Ok(GasRun {
        cases,
        table,
        criteria: DamageCriteria::IGC_2_7,
        stage_criteria: DamageCriteria::IGC_2_7,
        set_key: set.key(),
        method: "lost buoyancy, final stage (IGC 2.7.1.2, 2.7.2.1); intermediate stages by added mass at 25, 50 and 75 % (IGC 2.7.1.3): a breached tank holds sea water to the stage's fraction of its flood and keeps its cargo for the rest (IGC 2.5.3)",
        full_load_only: false,
    })
}

/// The cases and tables of the ADN 9.3.1.15 or ES-TRIN 28.03 damage run (issue #763), not yet run. The
/// Permeability Table is the set's: `adn-gas-permeability` or `es-trin-permeability`. A full cargo tank is 0 %
/// under both texts ("full or empty at the maximum draught", 9.3.1.15.1 (c), 28.03(4)(d)), so no case breaches a
/// tank and every cargo mass stays aboard.
pub fn gas_box_damage_plan(
    rule: GasBoxRule,
    set: &RuleSet,
    spaces: &[Space],
    hull: &Hull,
) -> Result<GasRun, String> {
    use crate::damage_cases::{
        AdnExtents, CaseOptions, adn_damage_cases_with, es_trin_longitudinal_extent_m,
        longitudinal_extent_m, machinery_bulkheads_intact,
    };
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let table = permeability_table(
        set,
        match rule {
            GasBoxRule::AdnGas => "adn-gas-permeability",
            GasBoxRule::EsTrin => "es-trin-permeability",
        },
    )?;
    let (extents, long_m) = match rule {
        GasBoxRule::AdnGas => (AdnExtents::TYPE_G, longitudinal_extent_m(loa)),
        GasBoxRule::EsTrin => (
            AdnExtents::ES_TRIN_28_03,
            es_trin_longitudinal_extent_m(loa),
        ),
    };
    let intact = machinery_bulkheads_intact(spaces, loa);
    let cases = adn_damage_cases_with(
        spaces,
        loa,
        beam,
        &extents,
        &CaseOptions {
            long_m,
            // a gas hull is cut at its hold bulkheads already (spec 0012 D6)
            cut: false,
            intact_bulkheads_m: &intact,
        },
        &|k| {
            if k == SpaceKind::CargoTank {
                0.0
            } else {
                table.of(k, 0.0).unwrap_or(0.95)
            }
        },
    );
    let (criteria, stage_criteria, method) = match rule {
        GasBoxRule::AdnGas => (
            DamageCriteria::ADN_9_3_1_FINAL,
            DamageCriteria::ADN_9_3_1_INTERMEDIATE,
            "lost buoyancy, final stage; intermediate stages by added mass at 25, 50 and 75 % (ADN 2025 9.3.1.15.1, 9.3.1.15.2)",
        ),
        GasBoxRule::EsTrin => (
            DamageCriteria::ES_TRIN_28_03_FINAL,
            DamageCriteria::ES_TRIN_28_03_INTERMEDIATE,
            "lost buoyancy, final stage; intermediate stages by added mass at 25, 50 and 75 % (ES-TRIN 2025/1 28.03(3) to (6)); full load departure",
        ),
    };
    Ok(GasRun {
        cases,
        table,
        criteria,
        stage_criteria,
        set_key: set.key(),
        method,
        full_load_only: rule == GasBoxRule::EsTrin,
    })
}

/// The mass, the centre of gravity and the intact floating position (draught at the origin, trim) of a loading
/// condition that a damage run damages.
#[derive(Debug, Clone, Copy)]
pub struct Afloat {
    pub mass_t: f64,
    pub g: Gravity,
    pub start: (f64, f64),
}

/// The intact state of a condition that `run` damages; `None` for a condition it leaves out: not the full load
/// departure when only that is damaged, no mass, or one that does not float intact.
pub fn afloat(run: &GasRun, c: &LoadedCondition, hull: &Hull, density: f64) -> Option<Afloat> {
    if run.full_load_only && c.role != Role::FullLoadDeparture {
        return None;
    }
    let mass: f64 = c.weights.iter().map(|(_, i)| i.mass_t).sum();
    if mass.is_nan() || mass <= 0.0 {
        return None;
    }
    let tcg_moment: f64 = c.ballast.iter().map(|t| t.mass_t * t.tcg_m).sum();
    let g = Gravity {
        lcg_m: c
            .weights
            .iter()
            .map(|(_, i)| i.mass_t * i.lcg_m)
            .sum::<f64>()
            / mass,
        tcg_m: tcg_moment / mass,
        kg_m: c
            .weights
            .iter()
            .map(|(_, i)| i.mass_t * i.kcg_m)
            .sum::<f64>()
            / mass,
    };
    let float = hull.float(mass, g, density).ok()?;
    let start = (
        float.equilibrium.waterplane.c,
        float.equilibrium.waterplane.trim_deg,
    );
    Some(Afloat {
        mass_t: mass,
        g,
        start,
    })
}

impl GasRun {
    /// How many damage cases a run works out on `conditions`: the cases of the run on each condition it damages.
    /// It works out no case itself (issue #923: the count a run that skips damage still gives).
    pub fn case_count(&self, hull: &Hull, conditions: &[LoadedCondition], density: f64) -> usize {
        conditions
            .iter()
            .filter(|c| afloat(self, c, hull, density).is_some())
            .count()
            * self.cases.len()
    }
}

/// One damage case on one condition: what the results block lists. The outcome and the stages are in the
/// verdict at the same index of [`GasDamageRun::verdicts`].
#[derive(Debug, Clone, PartialEq)]
pub struct CaseRecord {
    pub condition: String,
    pub name: String,
    pub flooded: Vec<String>,
    /// The names of the breached Type C tanks.
    pub breached_tanks: Vec<String>,
    pub deck_edge_immersed: bool,
}

/// What one damage case gives, and what the governing case needs if it is the worst (the flooding, the damaged
/// mass and its centre of gravity).
#[derive(Debug, Clone)]
pub struct CaseRun {
    pub record: CaseRecord,
    pub verdict: DamageVerdict,
    flooded: Vec<FloodedVolume>,
    m: f64,
    gd: Gravity,
}

/// The governing case of one condition: the worst, where a case that fails beats one that passes, and among
/// cases that fail (or among those that pass) the one with the least maximum righting lever wins; a tie keeps
/// the earlier case.
#[derive(Debug, Clone, PartialEq)]
pub struct Governing {
    pub condition: String,
    /// The index of the case in [`GasDamageRun::cases`] and [`GasDamageRun::verdicts`].
    pub case_index: usize,
    /// The flooded volumes of the case, for the damage drawing (issue #761).
    pub flooded: Vec<FloodedVolume>,
    /// The righting lever after damage over the 20 degree range of IGC 2.7.2.1, every 5 degrees: (heel, GZ).
    pub gz_after_damage: Vec<(f64, f64)>,
    /// Where the damage is, for the damage drawing (issue #928).
    pub x_m: (f64, f64),
    pub y_m: Option<(f64, f64)>,
}

/// A damage run of a gas hull: every case on every condition, in condition then case order, with the governing
/// case of each condition.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GasDamageRun {
    pub verdicts: Vec<DamageVerdict>,
    pub cases: Vec<CaseRecord>,
    pub governing: Vec<Governing>,
}

/// Work out one case on a condition. Every case is independent of the others (issue #921), so a caller may run
/// them on threads and hand the results, in case order, to [`GasDamageRun::push_condition`].
pub fn run_case(
    run: &GasRun,
    spaces: &[Space],
    hull: &Hull,
    c: &LoadedCondition,
    state: &Afloat,
    density: f64,
    case: &DamageCase,
) -> CaseRun {
    let Afloat {
        mass_t: mass,
        g,
        start,
    } = *state;
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let cargo = c
        .weights
        .iter()
        .find(|(n, _)| n == "Cargo")
        .map_or((0.0, 0.0), |(_, i)| (i.mass_t, i.kcg_m));
    let filling = |name: &str| {
        c.ballast
            .iter()
            .find(|t| t.name == name)
            .map_or(0.0, |t| t.fill)
    };
    let f = gas_flooding(case, spaces, &run.table, &filling);
    let (m, gd) = without_breached_cargo(mass, g, cargo, spaces, &f.breached);
    let outcome = assess(
        &hull.table,
        &f.flooded,
        m,
        gd,
        density,
        start,
        &run.criteria,
    );
    // Issue #760, #988: the intermediate stages, by added mass; a breached tank has let out the stage's fraction
    // of its cargo and holds sea water to the same fraction (D15 to D18).
    let stages = intermediate_stages_releasing(
        &hull.table,
        &f.flooded,
        &outcome,
        &|fraction| breached_cargo_at_stage(fraction, mass, g, cargo, spaces, &f.breached),
        density,
        start,
        &run.stage_criteria,
    );
    let deck = outcome.floats == Floats::Yes
        && deck_edge_immersed(
            &Waterplane {
                heel_deg: outcome.heel_deg,
                trim_deg: outcome.trim_deg,
                c: outcome.waterplane_c_m,
            },
            (0.0, loa),
            0.5 * beam,
            hull.spec.depth,
        );
    let flooded_names: Vec<String> = case.flooded.iter().map(|v| v.name.clone()).collect();
    CaseRun {
        record: CaseRecord {
            condition: c.condition.name.clone(),
            name: case.name.clone(),
            flooded: flooded_names.clone(),
            breached_tanks: f.breached,
            deck_edge_immersed: deck,
        },
        verdict: DamageVerdict {
            name: format!("{}: {}", c.condition.name, case.name),
            flooded: flooded_names,
            outcome,
            stages,
        },
        flooded: f.flooded,
        m,
        gd,
    }
}

impl GasDamageRun {
    /// Add one condition: its cases' results `runs` in case order (`cases` is `run.cases`), and the governing case
    /// among them.
    pub fn push_condition(
        &mut self,
        run: &GasRun,
        hull: &Hull,
        c: &LoadedCondition,
        state: &Afloat,
        density: f64,
        runs: Vec<CaseRun>,
    ) {
        let start = state.start;
        // (index of the verdict, the flooding, the damaged mass and centre of gravity) of the governing case, and
        // the case itself.
        let mut worst_case: Option<&DamageCase> = None;
        let mut worst: Option<(usize, Vec<FloodedVolume>, f64, Gravity)> = None;
        // `runs` is in case order, so each result pairs with its case (the governing case's extent, issue #928).
        for (case, r) in run.cases.iter().zip(runs) {
            let worse = match &worst {
                None => true,
                Some((i, ..)) => {
                    let o: &DamageOutcome = &self.verdicts[*i].outcome;
                    (o.ok && !r.verdict.outcome.ok)
                        || (o.ok == r.verdict.outcome.ok && r.verdict.outcome.max_gz_m < o.max_gz_m)
                }
            };
            if worse {
                worst_case = Some(case);
                worst = Some((self.verdicts.len(), r.flooded, r.m, r.gd));
            }
            self.cases.push(r.record);
            self.verdicts.push(r.verdict);
        }
        if let Some((i, flooded, m, gd)) = worst {
            let o = &self.verdicts[i].outcome;
            let side = if o.heel_deg >= 0.0 { 1.0 } else { -1.0 };
            let gz: Vec<(f64, f64)> = if o.floats == Floats::Yes {
                (0..=4)
                    .filter_map(|k| {
                        let heel = o.heel_deg + side * 5.0 * k as f64;
                        righting_lever_at(&hull.table, &flooded, m, gd, density, start, heel)
                            .map(|v| (heel, side * v))
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let w = worst_case.expect("a worst case has its case");
            self.governing.push(Governing {
                condition: c.condition.name.clone(),
                case_index: i,
                flooded,
                gz_after_damage: gz,
                x_m: w.x_m,
                y_m: w.y_m,
            });
        }
    }
}

/// The damage cases of `run` on every loading condition of a gas hull, serially (the order and the bits of the
/// threaded run in `ovd-format`). A condition that does not float intact has no cases.
pub fn run_serial(
    run: &GasRun,
    spaces: &[Space],
    hull: &Hull,
    conditions: &[LoadedCondition],
    density: f64,
) -> GasDamageRun {
    let mut out = GasDamageRun::default();
    for c in conditions {
        let Some(state) = afloat(run, c, hull, density) else {
            continue;
        };
        let runs: Vec<CaseRun> = run
            .cases
            .iter()
            .map(|case| run_case(run, spaces, hull, c, &state, density, case))
            .collect();
        out.push_condition(run, hull, c, &state, density, runs);
    }
    out
}

/// What the damage verdicts of a candidate depend on besides its hull and spaces (issue #1038): the lightship and the
/// cargo of its loading conditions. A later depth pass moves the installed power, so the lightship and the fuel, a
/// little; verdicts kept from an earlier pass are reused only while this basis has moved by less than
/// [`KEPT_VERDICT_TOLERANCE`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeptBasis {
    pub lightship_t: f64,
    pub cargo_t: f64,
}

/// The relative change of the lightship, and of the cargo, within which the verdicts of an earlier depth pass are
/// reused (issue #1038).
pub const KEPT_VERDICT_TOLERANCE: f64 = 0.0001;

impl KeptBasis {
    /// Whether verdicts worked out on `self` still stand for `now`.
    pub fn stands_for(&self, now: &KeptBasis) -> bool {
        let near = |a: f64, b: f64| (a - b).abs() <= KEPT_VERDICT_TOLERANCE * a.abs().max(b.abs());
        near(self.lightship_t, now.lightship_t) && near(self.cargo_t, now.cargo_t)
    }
}

/// Verdicts an earlier depth pass worked out for a candidate (issue #1038), and whether the lightship and the cargo
/// they were worked out on still stand for the candidate now.
#[derive(Debug, Clone)]
pub struct Kept {
    pub verdicts: Vec<DamageVerdict>,
    pub stands: bool,
}

impl Kept {
    /// Whether the candidate failed damage on the kept verdicts (a final or an intermediate stage).
    pub fn failed(&self) -> bool {
        self.verdicts
            .iter()
            .any(|v| !v.outcome.ok || v.stages.iter().any(|s| !s.ok))
    }
}

/// The message of a run that the progress hook cancelled (the `Raised` kind is `Cancelled`).
pub const DAMAGE_CANCELLED: &str = "damage stability was cancelled";

/// A damage run was cancelled (by the progress hook of `ovd-format`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

/// How the arrangement search works out the damage of a candidate (issue #935). The search is in this crate,
/// which has no threads and no clock, so the caller supplies the runner: `ovd-format` gives one that runs the
/// cases on worker threads, reports progress and honours a cancel; [`SerialRunner`] is the plain loop.
pub trait DamageRunner {
    /// Whether the search screens candidates by damage at all (a run told to skip damage says no).
    fn enabled(&self) -> bool {
        true
    }

    /// The most damage runs the search expects to make, told before the first (an upper bound from the candidates
    /// that pass the cheap checks and the refinement, within the cap). A runner that reports progress plans its
    /// total from it, so the fraction done never falls while runs start.
    fn plan(&self, _runs: usize) {}

    /// The damage runs this runner has already worked out for the whole run (the searches of its depth passes
    /// share one budget), so the cap of the search counts them.
    fn runs_used(&self) -> usize {
        0
    }

    /// The verdicts a search of this run already worked out for a candidate with this key (an earlier depth pass
    /// built the same candidate), if any: reusing them costs no damage run, and a later pass that runs out of
    /// budget keeps what an earlier one found.
    fn recall(&self, _key: &str, _basis: &KeptBasis) -> Option<Kept> {
        None
    }

    /// Keep the verdicts of a candidate for [`DamageRunner::recall`].
    fn remember(&self, _key: &str, _basis: &KeptBasis, _verdicts: &[DamageVerdict]) {}

    /// The cases of `run` on every condition of `conditions`. `Err(Cancelled)` stops the search.
    fn run(
        &self,
        run: &GasRun,
        spaces: &[Space],
        hull: &Hull,
        conditions: &[LoadedCondition],
        density: f64,
    ) -> Result<GasDamageRun, Cancelled>;
}

/// The serial runner: no threads, no progress, no cancel.
#[derive(Debug, Clone, Copy, Default)]
pub struct SerialRunner;

impl DamageRunner for SerialRunner {
    fn run(
        &self,
        run: &GasRun,
        spaces: &[Space],
        hull: &Hull,
        conditions: &[LoadedCondition],
        density: f64,
    ) -> Result<GasDamageRun, Cancelled> {
        Ok(run_serial(run, spaces, hull, conditions, density))
    }
}
