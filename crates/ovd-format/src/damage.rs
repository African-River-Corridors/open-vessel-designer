//! The damage data a Rule Set gives the damage case generator (spec 0012): the IGC 2.3.1 extents and the IGC
//! 2.6.1 standard of damage (issue #757), and the Permeability Tables (issue #758), read from the set's tables,
//! so the numbers live in the Rule Set and not in code.

use arc_rules::eval::{MeasureValue, table_value};
use arc_rules::model::RuleSet;
use ovd_core::compartments::SpaceKind;
use ovd_core::damage_cases::{BoxExtents, Permeability, PermeabilityTable};

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

/// The Permeability Table `table_id` of the highest built-in version of set `set_id` that holds it. A run reads
/// the liquid permeabilities this way whatever version of the set the design pins: the older versions hold no
/// table, and the values were in code before.
pub fn built_in_permeability(set_id: &str, table_id: &str) -> Result<PermeabilityTable, String> {
    let lib = crate::rule_set::built_in();
    let mut sets: Vec<&RuleSet> = lib
        .sets()
        .iter()
        .filter(|s| s.id == set_id && s.tables.iter().any(|t| t.id == table_id))
        .collect();
    sets.sort_by(|a, b| {
        if crate::rule_set::is_newer(&a.version, &b.version) {
            core::cmp::Ordering::Greater
        } else if crate::rule_set::is_newer(&b.version, &a.version) {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Equal
        }
    });
    let set = sets
        .last()
        .ok_or_else(|| format!("no built-in `{set_id}` holds table `{table_id}`"))?;
    permeability_table(set, table_id)
}

/// Issue #759 (spec 0012): the IGC damage cases of a gas hull on every loading condition, final stage of
/// flooding by lost buoyancy, with the criteria of IGC 2.7.1.2 and 2.7.2.1.
///
/// The cases, the standard of damage and the permeabilities come from `set` (an `igc-2g` version that holds the
/// tables; an older one gives an error naming the missing table). Every condition with weights is damaged
/// (D10). A breached Type C tank loses its cargo, which leaves the condition (IGC 2.5.3, D8); an intact tank in
/// a flooded hold keeps its buoyancy (D7); a ballast tank floods by its filling (D9). A condition that does not
/// float intact has no cases. The verdicts are named `<condition>: <case>`.
///
/// The second value is the `damage` block of the results (unhashed, D21): every case with its condition, its
/// floating position, its residual stability and the deck-edge test (D11), and for each condition the governing
/// case with its righting lever curve over the 20 degree range (D12). The governing case is the worst: a case that
/// fails beats one that passes, and among cases that fail (or among those that pass) the one with the
/// least maximum righting lever wins; a tie keeps the earlier case.
pub fn gas_damage_verdicts(
    set: &RuleSet,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), String> {
    use ovd_core::damage_cases::{StandardOfDamage, igc_damage_cases};
    use ovd_core::damage_stability::DamageCriteria;
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let data = igc_damage_data(set, loa, beam)?;
    let table = permeability_table(set, "igc-permeability")?;
    let standard = StandardOfDamage::igc_2g(spaces, loa, data.machinery_aft_intact);
    let cases = igc_damage_cases(spaces, loa, beam, &data.extents, &standard, &|k| {
        table.of(k, 0.0).unwrap_or(0.95)
    });
    run_gas_damage(
        &GasRun {
            cases,
            table,
            criteria: DamageCriteria::IGC_2_7,
            stage_criteria: DamageCriteria::IGC_2_7,
            set_key: set.key(),
            method: "lost buoyancy, final stage of flooding (IGC 2.7.1.2, 2.7.2.1)",
            full_load_only: false,
        },
        spaces,
        hull,
        conditions,
        density,
    )
}

/// What a gas damage run is given: the cases, the Permeability Table, the criteria of the final stage and of the
/// intermediate stages, and how the block names its Rule Set and its method.
pub(crate) struct GasRun {
    pub cases: Vec<ovd_core::damage_cases::DamageCase>,
    pub table: PermeabilityTable,
    pub criteria: ovd_core::damage_stability::DamageCriteria,
    pub stage_criteria: ovd_core::damage_stability::DamageCriteria,
    pub set_key: String,
    pub method: &'static str,
    /// Damage the full load departure only (ES-TRIN 28.03(3): the cargo for the maximum draught, evenly spread,
    /// with full supplies), not every IS Code condition.
    pub full_load_only: bool,
}

/// The damage cases of `run` on every loading condition of a gas hull, as [`gas_damage_verdicts`] gives them for
/// IGC (spec 0012, issues #759, #760, #763).
pub(crate) fn run_gas_damage(
    run: &GasRun,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), String> {
    use ovd_core::damage_cases::{deck_edge_immersed, gas_flooding, without_breached_cargo};
    use ovd_core::damage_stability::{
        CutCache, Floats, INTERMEDIATE_FRACTIONS, assess_cached, intermediate_stages_cached,
        righting_lever_at_cached,
    };
    use ovd_core::sections::{Gravity, Waterplane};
    use serde_json::json;
    let num = crate::eval::num;
    let mut rows: Vec<serde_json::Value> = Vec::new();
    let mut governing: Vec<serde_json::Value> = Vec::new();
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let (cases, table) = (&run.cases, &run.table);
    // One cache of hull cuts per worker, kept across the conditions of the run (they share the hull).
    let mut caches: Vec<CutCache> = (0..damage_workers(cases.len()))
        .map(|_| {
            if DAMAGE_CACHE.load(core::sync::atomic::Ordering::Relaxed) {
                CutCache::default()
            } else {
                CutCache::disabled()
            }
        })
        .collect();
    let mut out: Vec<ovd_core::measured::DamageVerdict> = Vec::new();
    for c in conditions {
        if run.full_load_only && c.role != ovd_core::loading_conditions::Role::FullLoadDeparture {
            continue;
        }
        let mass: f64 = c.weights.iter().map(|(_, i)| i.mass_t).sum();
        if mass.is_nan() || mass <= 0.0 {
            continue;
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
        let Ok(float) = hull.float(mass, g, density) else {
            continue;
        };
        let start = (
            float.equilibrium.waterplane.c,
            float.equilibrium.waterplane.trim_deg,
        );
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
        // (index into `out`, the flooding, the damaged mass and centre of gravity) of the governing case, and the
        // case itself
        let mut worst_case: Option<&ovd_core::damage_cases::DamageCase> = None;
        let mut worst: Option<(
            usize,
            Vec<ovd_core::damage_stability::FloodedVolume>,
            f64,
            Gravity,
        )> = None;
        // Every case is independent of the others (issue #921), so they run on worker threads; each worker keeps
        // its own cuts of the hull. The results come back in case order and the choice of the governing case
        // below runs over them in that order, so the output is the serial one, bit for bit.
        let runs = run_ordered(cases, &mut caches, |case, cache| {
            let f = gas_flooding(case, spaces, table, &filling);
            let (m, gd) = without_breached_cargo(mass, g, cargo, spaces, &f.breached);
            let outcome = assess_cached(
                &hull.table,
                &f.flooded,
                m,
                gd,
                density,
                start,
                &run.criteria,
                cache,
            );
            // Issue #760: the intermediate stages, by added mass with the cargo still aboard (D15 to D18).
            let stages = intermediate_stages_cached(
                &hull.table,
                &f.flooded,
                &outcome,
                (mass, g),
                density,
                start,
                &run.stage_criteria,
                cache,
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
            let row = json!({
                "condition": c.condition.name,
                "name": case.name,
                "flooded": case.flooded.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
                "breached_tanks": f.breached,
                "floats": match outcome.floats {
                    Floats::Yes => "yes",
                    Floats::Sinks => "sinks",
                    Floats::NoEquilibrium => "no equilibrium",
                },
                "heel_deg": num(outcome.heel_deg),
                "trim_deg": num(outcome.trim_deg),
                "draught_mid_m": num(outcome.draught_mid_m),
                "range_end_deg": num(outcome.end_deg),
                "max_gz_m": num(outcome.max_gz_m),
                "area_m_rad": num(outcome.area_m_rad),
                "deck_edge_immersed": deck,
                "ok": outcome.ok,
                "stages": stages.iter().zip(INTERMEDIATE_FRACTIONS).map(|(st, fr)| json!({
                    "fraction": num(fr),
                    "floats": match st.floats {
                        Floats::Yes => "yes",
                        Floats::Sinks => "sinks",
                        Floats::NoEquilibrium => "no equilibrium",
                    },
                    "heel_deg": num(st.heel_deg),
                    "max_gz_m": num(st.max_gz_m),
                    "area_m_rad": num(st.area_m_rad),
                    "ok": st.ok,
                })).collect::<Vec<_>>(),
            });
            CaseRun {
                case,
                row,
                verdict: ovd_core::measured::DamageVerdict {
                    name: format!("{}: {}", c.condition.name, case.name),
                    flooded: case.flooded.iter().map(|v| v.name.clone()).collect(),
                    outcome,
                    stages,
                },
                flooded: f.flooded,
                m,
                gd,
            }
        });
        for r in runs {
            let worse = match &worst {
                None => true,
                Some((i, ..)) => {
                    let w = &out[*i];
                    let o: &ovd_core::damage_stability::DamageOutcome = &w.outcome;
                    (o.ok && !r.verdict.outcome.ok)
                        || (o.ok == r.verdict.outcome.ok && r.verdict.outcome.max_gz_m < o.max_gz_m)
                }
            };
            if worse {
                worst_case = Some(r.case);
                worst = Some((out.len(), r.flooded, r.m, r.gd));
            }
            rows.push(r.row);
            out.push(r.verdict);
        }
        if let Some((i, flooded, m, gd)) = worst {
            let o = &out[i].outcome;
            // The righting lever after damage over the 20 degree range of IGC 2.7.2.1, every 5 degrees.
            let side = if o.heel_deg >= 0.0 { 1.0 } else { -1.0 };
            let gz: Vec<serde_json::Value> = if o.floats == Floats::Yes {
                (0..=4)
                    .filter_map(|k| {
                        let heel = o.heel_deg + side * 5.0 * k as f64;
                        righting_lever_at_cached(
                            &hull.table,
                            &flooded,
                            m,
                            gd,
                            density,
                            start,
                            heel,
                            &caches[0],
                        )
                        .map(|v| json!([num(heel), num(side * v)]))
                    })
                    .collect()
            } else {
                Vec::new()
            };
            // What the damage drawing shows (issue #761): each Flooded Volume's shape, and the damaged
            // waterplane.
            let geometry: Vec<serde_json::Value> = flooded
                .iter()
                .map(|v| match &v.shape {
                    ovd_core::compartments::SpaceShape::Prism { section } => json!({
                        "name": v.name,
                        "x_m": [num(v.x_aft_m), num(v.x_fwd_m)],
                        "section_m": section.iter().map(|&(y, z)| json!([num(y), num(z)])).collect::<Vec<_>>(),
                    }),
                    ovd_core::compartments::SpaceShape::Tank {
                        y,
                        z,
                        outer_diameter,
                        ..
                    } => json!({
                        "name": v.name,
                        "x_m": [num(v.x_aft_m), num(v.x_fwd_m)],
                        "tank": { "y_m": num(*y), "z_m": num(*z), "diameter_m": num(*outer_diameter) },
                    }),
                })
                .collect();
            let mut entry = json!({
                "condition": c.condition.name,
                "case": rows[i]["name"],
                "ok": o.ok,
                "floats": rows[i]["floats"],
                "heel_deg": num(o.heel_deg),
                "trim_deg": num(o.trim_deg),
                "draught_mid_m": num(o.draught_mid_m),
                "waterplane": { "heel_deg": num(o.heel_deg), "trim_deg": num(o.trim_deg), "c_m": num(o.waterplane_c_m) },
                "flooded": geometry,
                "gz_after_damage": gz,
            });
            // Issue #928: where the damage is, for the damage drawing.
            if let Some(w) = worst_case {
                entry["x_m"] = json!([num(w.x_m.0), num(w.x_m.1)]);
                if let Some((y0, y1)) = w.y_m {
                    entry["y_m"] = json!([num(y0), num(y1)]);
                }
            }
            governing.push(entry);
        }
    }
    Ok((
        out,
        json!({
            "rule_set": run.set_key,
            "method": run.method,
            "cases": rows,
            "governing": governing,
        }),
    ))
}

/// The box rules a gas hull is run through besides IGC (issue #763).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasBoxRule {
    /// ADN 2025 9.3.1.15 (type G), from the `adn-gas` set.
    AdnGas,
    /// ES-TRIN 2025/1 28.03(4) to (6), a craft over 110 m, from the `es-trin-2025` set.
    EsTrin,
}

/// Issue #763 (spec 0012 D2, D3): the damage cases of ADN 9.3.1.15 or ES-TRIN 28.03 on every loading condition of
/// a gas hull, final stage by lost buoyancy and the intermediate stages by added mass (D15 to D18).
///
/// The Permeability Table is the set's: `adn-gas-permeability` or `es-trin-permeability`. A full cargo tank is 0 %
/// under both texts ("full or empty at the maximum draught", 9.3.1.15.1 (c), 28.03(4)(d)), so no case breaches a
/// tank and every cargo mass stays aboard. The block is shaped as the IGC one is ([`gas_damage_verdicts`]).
pub fn gas_box_damage_verdicts(
    rule: GasBoxRule,
    set: &RuleSet,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), String> {
    use ovd_core::damage_cases::{
        AdnExtents, CaseOptions, adn_damage_cases_with, es_trin_longitudinal_extent_m,
        longitudinal_extent_m, machinery_bulkheads_intact,
    };
    use ovd_core::damage_stability::DamageCriteria;
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
            if k == ovd_core::compartments::SpaceKind::CargoTank {
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
    run_gas_damage(
        &GasRun {
            cases,
            table,
            criteria,
            stage_criteria,
            set_key: set.key(),
            method,
            full_load_only: rule == GasBoxRule::EsTrin,
        },
        spaces,
        hull,
        conditions,
        density,
    )
}

/// How the `damage_by_rule` block names the method of the ES-TRIN 28.03 damage cases on a liquid bulk hull.
pub(crate) const ES_TRIN_METHOD: &str = "lost buoyancy, final stage; intermediate stages by added mass at 25, 50 and 75 % (ES-TRIN 2025/1 28.03(3) to (6)); laden departure";
/// How the `damage_by_rule` block names the method of the NORMAM 5.23.8 damage cases.
pub(crate) const NORMAM_METHOD: &str =
    "lost buoyancy, final stage (NORMAM-202/DPC 5.23.8, 5.23.9); laden departure";

/// What one damage case gives: its row of the `damage` block, its verdict, and what the governing case needs
/// if it is the worst (the flooding, the damaged mass and its centre of gravity).
struct CaseRun<'a> {
    /// The case itself, for the governing case's extent (issue #928).
    case: &'a ovd_core::damage_cases::DamageCase,
    row: serde_json::Value,
    verdict: ovd_core::measured::DamageVerdict,
    flooded: Vec<ovd_core::damage_stability::FloodedVolume>,
    m: f64,
    gd: ovd_core::sections::Gravity,
}

/// How many worker threads the damage cases may use; 0 means one per core.
static DAMAGE_WORKERS: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// Whether the damage cases keep the cuts of the hull (the default), or cut afresh every time.
static DAMAGE_CACHE: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);

/// For tests only (issue #921), hence hidden from the docs: a process-wide knob, so a test that sets it must be
/// the only one in its test binary. Sets the number of worker threads for the damage cases: 1 runs them
/// serially, 0 (the default) uses one per core. The results are the same bits whatever the number. A build for
/// WebAssembly, or without the `parallel` feature, is always serial.
#[doc(hidden)]
pub fn set_damage_workers(n: usize) {
    DAMAGE_WORKERS.store(n, core::sync::atomic::Ordering::Relaxed);
}

/// For tests only (issue #921), hidden from the docs, process-wide like [`set_damage_workers`]: `false` makes
/// the damage cases cut the hull afresh every time, the reference the cache is compared against.
#[doc(hidden)]
pub fn set_damage_cache(on: bool) {
    DAMAGE_CACHE.store(on, core::sync::atomic::Ordering::Relaxed);
}

/// The worker count for `cases` cases: at least 1, at most one per case.
fn damage_workers(cases: usize) -> usize {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        let set = DAMAGE_WORKERS.load(core::sync::atomic::Ordering::Relaxed);
        let n = if set == 0 {
            std::thread::available_parallelism().map_or(1, |n| n.get())
        } else {
            set
        };
        n.min(cases).max(1)
    }
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        let _ = (cases, &DAMAGE_WORKERS);
        1
    }
}

/// Run `f` on every item and return the results in the order of the items. `caches` gives one cut cache per
/// worker: one cache runs the items on this thread; more share the items between scoped threads (std only),
/// each taking the next item not yet taken. `f` is pure in its item, so the order of the results does not
/// depend on which worker ran which item. This is the one place a per-case progress call or a cancel check
/// belongs (issue #923): after `f` returns, or before a worker takes its next item.
fn run_ordered<T, R, F>(
    items: &[T],
    caches: &mut [ovd_core::damage_stability::CutCache],
    f: F,
) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T, &ovd_core::damage_stability::CutCache) -> R + Sync,
{
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    if caches.len() > 1 && items.len() > 1 {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let next = AtomicUsize::new(0);
        let (next, f) = (&next, &f);
        let mut slots: Vec<Option<R>> = items.iter().map(|_| None).collect();
        std::thread::scope(|scope| {
            let workers: Vec<_> = caches
                .iter_mut()
                .map(|cache| {
                    scope.spawn(move || {
                        let mut done: Vec<(usize, R)> = Vec::new();
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(item) = items.get(i) else { break };
                            done.push((i, f(item, cache)));
                        }
                        done
                    })
                })
                .collect();
            for w in workers {
                match w.join() {
                    Ok(done) => {
                        for (i, r) in done {
                            slots[i] = Some(r);
                        }
                    }
                    Err(panic) => std::panic::resume_unwind(panic),
                }
            }
        });
        return slots
            .into_iter()
            .map(|r| r.expect("every case ran once"))
            .collect();
    }
    items.iter().map(|item| f(item, &caches[0])).collect()
}
