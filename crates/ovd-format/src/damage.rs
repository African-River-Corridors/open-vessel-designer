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
    let plan = gas_damage_plan(set, spaces, hull)?;
    run_gas_damage(&plan, spaces, hull, conditions, density, &Gauge::none())
        .map_err(|Cancelled| "cancelled".to_string())
}

/// The cases and tables of [`gas_damage_verdicts`], not yet run (issue #923: a run that skips damage still
/// counts them).
pub(crate) fn gas_damage_plan(
    set: &RuleSet,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
) -> Result<GasRun, String> {
    use ovd_core::damage_cases::{StandardOfDamage, igc_damage_cases};
    use ovd_core::damage_stability::DamageCriteria;
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

/// The mass, the centre of gravity and the intact floating position (draught at the origin, trim) of a loading
/// condition that a damage run damages; `None` for a condition it leaves out: not the full load departure when
/// only that is damaged, no mass, or one that does not float intact.
fn afloat(
    full_load_only: bool,
    c: &ovd_core::loading_conditions::LoadedCondition,
    hull: &ovd_core::hydrostatics::Hull,
    density: f64,
) -> Option<(f64, ovd_core::sections::Gravity, (f64, f64))> {
    use ovd_core::sections::Gravity;
    if full_load_only && c.role != ovd_core::loading_conditions::Role::FullLoadDeparture {
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
    Some((mass, g, start))
}

impl GasRun {
    /// How many damage cases [`run_gas_damage`] works out on `conditions`: the cases of the run on each condition
    /// it damages. It works out no case itself (issue #923: the count a run that skips damage still gives).
    pub(crate) fn case_count(
        &self,
        hull: &ovd_core::hydrostatics::Hull,
        conditions: &[ovd_core::loading_conditions::LoadedCondition],
        density: f64,
    ) -> usize {
        conditions
            .iter()
            .filter(|c| afloat(self.full_load_only, c, hull, density).is_some())
            .count()
            * self.cases.len()
    }
}

/// The damage cases of `run` on every loading condition of a gas hull, as [`gas_damage_verdicts`] gives them for
/// IGC (spec 0012, issues #759, #760, #763).
pub(crate) fn run_gas_damage(
    run: &GasRun,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
    gauge: &Gauge,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), Cancelled> {
    use ovd_core::damage_cases::{
        breached_cargo_at_stage, deck_edge_immersed, gas_flooding, without_breached_cargo,
    };
    use ovd_core::damage_stability::{
        Floats, INTERMEDIATE_FRACTIONS, assess, intermediate_stages_releasing, righting_lever_at,
    };
    use ovd_core::sections::{Gravity, Waterplane};
    use serde_json::json;
    let num = crate::eval::num;
    let mut rows: Vec<serde_json::Value> = Vec::new();
    let mut governing: Vec<serde_json::Value> = Vec::new();
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let (cases, table) = (&run.cases, &run.table);
    let workers = damage_workers(cases.len());
    let mut out: Vec<ovd_core::measured::DamageVerdict> = Vec::new();
    for c in conditions {
        let Some((mass, g, start)) = afloat(run.full_load_only, c, hull, density) else {
            continue;
        };
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
        let runs = run_ordered(cases, workers, gauge, |case| {
            let f = gas_flooding(case, spaces, table, &filling);
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
            // Issue #760, #988: the intermediate stages, by added mass; a breached tank has let out the stage's
            // fraction of its cargo and holds sea water to the same fraction (D15 to D18).
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
        })?;
        // `runs` is in case order, so each result pairs with its case (the governing case's extent, issue #928).
        for (case, r) in cases.iter().zip(runs) {
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
                worst_case = Some(case);
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
                        righting_lever_at(&hull.table, &flooded, m, gd, density, start, heel)
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
    let plan = gas_box_damage_plan(rule, set, spaces, hull)?;
    run_gas_damage(&plan, spaces, hull, conditions, density, &Gauge::none())
        .map_err(|Cancelled| "cancelled".to_string())
}

/// The cases and tables of [`gas_box_damage_verdicts`], not yet run (issue #923).
pub(crate) fn gas_box_damage_plan(
    rule: GasBoxRule,
    set: &RuleSet,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
) -> Result<GasRun, String> {
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

/// What the damage stages of a gas hull give (issue #923): the IGC list and block, the ADN and ES-TRIN lists, their
/// `damage_by_rule` blocks, and the number of cases of the stages (counted even when the run skips them).
#[derive(Default)]
pub(crate) struct GasStages {
    pub igc: Vec<ovd_core::measured::DamageVerdict>,
    pub igc_block: serde_json::Value,
    pub adn: Vec<ovd_core::measured::DamageVerdict>,
    pub es_trin: Vec<ovd_core::measured::DamageVerdict>,
    pub by_rule: serde_json::Map<String, serde_json::Value>,
    pub cases: usize,
}

/// The three damage stages of a gas hull on its IS Code conditions: IGC 2.7, ADN 9.3.1.15 and ES-TRIN 28.03 (spec
/// 0012), each when its Rule Set is given. With `control.run` false no case is worked out, but the cases are
/// counted (`cases`). The hook of `control` is called on this thread, from the first call with 0 done, and a
/// Break from it gives `Err(Cancelled)`.
pub(crate) fn gas_damage_stages(
    sets: (Option<&RuleSet>, Option<&RuleSet>, Option<&RuleSet>),
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
    control: &DamageControl,
) -> Result<GasStages, Cancelled> {
    let (igc, adn, es_trin) = sets;
    let mut out = GasStages {
        igc_block: serde_json::Value::Null,
        ..GasStages::default()
    };
    if conditions.is_empty() {
        return Ok(out);
    }
    // A stage whose cases cannot be built (a Rule Set version without the tables) gives nothing, as before.
    let mut plans: Vec<(Option<GasBoxRule>, &RuleSet, GasRun)> = Vec::new();
    if let Some(set) = igc
        && let Ok(plan) = gas_damage_plan(set, spaces, hull)
    {
        plans.push((None, set, plan));
    }
    for (rule, set) in [(GasBoxRule::AdnGas, adn), (GasBoxRule::EsTrin, es_trin)] {
        if let Some(set) = set
            && let Ok(plan) = gas_box_damage_plan(rule, set, spaces, hull)
        {
            plans.push((Some(rule), set, plan));
        }
    }
    out.cases = plans
        .iter()
        .map(|(_, _, p)| p.case_count(hull, conditions, density))
        .sum();
    if !control.run {
        return Ok(out);
    }
    let gauge = Gauge::new(control.progress.clone(), out.cases);
    gauge.report();
    for (rule, set, plan) in &plans {
        let (v, block) = run_gas_damage(plan, spaces, hull, conditions, density, &gauge)?;
        match rule {
            None => (out.igc, out.igc_block) = (v, block),
            Some(r) => {
                match r {
                    GasBoxRule::AdnGas => out.adn = v,
                    GasBoxRule::EsTrin => out.es_trin = v,
                }
                out.by_rule.insert(set.id.clone(), block);
            }
        }
    }
    Ok(out)
}

/// How the `damage_by_rule` block names the method of the ES-TRIN 28.03 damage cases on a liquid bulk hull.
pub(crate) const ES_TRIN_METHOD: &str = "lost buoyancy, final stage; intermediate stages by added mass at 25, 50 and 75 % (ES-TRIN 2025/1 28.03(3) to (6)); laden departure";
/// How the `damage_by_rule` block names the method of the NORMAM 5.23.8 damage cases.
pub(crate) const NORMAM_METHOD: &str =
    "lost buoyancy, final stage (NORMAM-202/DPC 5.23.8, 5.23.9); laden departure";

/// What one damage case gives: its row of the `damage` block, its verdict, and what the governing case needs
/// if it is the worst (the flooding, the damaged mass and its centre of gravity).
struct CaseRun {
    row: serde_json::Value,
    verdict: ovd_core::measured::DamageVerdict,
    flooded: Vec<ovd_core::damage_stability::FloodedVolume>,
    m: f64,
    gd: ovd_core::sections::Gravity,
}

thread_local! {
    static DAMAGE_RUNS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
}

/// How many damage runs the calling thread has started (a run that works cases out, not one that only counts
/// them). A test reads it before and after a call to show that the call ran no damage (issue #924). Per thread,
/// so tests that run side by side do not count each other's runs.
pub fn damage_runs_started() -> usize {
    DAMAGE_RUNS.with(core::cell::Cell::get)
}

/// How many worker threads the damage cases may use; 0 means one per core.
static DAMAGE_WORKERS: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// For tests only (issue #921), hence hidden from the docs: a process-wide knob, so a test that sets it must be
/// the only one in its test binary. Sets the number of worker threads for the damage cases: 1 runs them
/// serially, 0 (the default) uses one per core. The results are the same bits whatever the number. A build for
/// WebAssembly, or without the `parallel` feature, is always serial.
#[doc(hidden)]
pub fn set_damage_workers(n: usize) {
    DAMAGE_WORKERS.store(n, core::sync::atomic::Ordering::Relaxed);
}

/// The worker count for `cases` cases: at least 1, at most one per case.
pub(crate) fn damage_workers(cases: usize) -> usize {
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

/// Issue #923: whether a run works out damage stability. A run that skips it still counts the cases
/// (`damage_cases` in the results), for a time estimate.
#[derive(Clone)]
pub struct DamageControl {
    /// Work out the damage cases. Default: true. False gives `damage_status` `not run`.
    pub run: bool,
    /// Called with the cases done and the total as the cases finish. See [`DamageProgress`].
    pub progress: Option<ProgressHook>,
}

impl Default for DamageControl {
    fn default() -> Self {
        Self {
            run: true,
            progress: None,
        }
    }
}

impl core::fmt::Debug for DamageControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DamageControl")
            .field("run", &self.run)
            .field("progress", &self.progress.as_ref().map(|_| "hook"))
            .finish()
    }
}

/// How far a damage run is: `done` of `total` cases, over every damage Rule Set of the run (issue #923).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageProgress {
    pub done: usize,
    pub total: usize,
}

/// The progress hook. It is called on the thread that called the run, never on a worker thread (a Python
/// callback, and the Ctrl-C of the Python main thread, need that): first with `done` 0, then whenever the count
/// of finished cases changes, at most about every 5 ms, and last when the run ends. Damage runs in one pass
/// only: the passes that size the default propeller or settle the rooms of a deckhouse stern skip it (it feeds
/// neither), so the count never falls. It returns
/// [`ControlFlow::Break`] to cancel: no new case starts, the cases under way finish, and the run stops with
/// [`DAMAGE_CANCELLED`]. Not `Send`: it stays on the calling thread.
pub type ProgressHook = std::rc::Rc<dyn Fn(DamageProgress) -> ControlFlow<()>>;

/// The message of a run that the progress hook cancelled (the `Raised` kind is `Cancelled`).
pub const DAMAGE_CANCELLED: &str = "damage stability was cancelled";

pub use core::ops::ControlFlow;

/// A damage run was cancelled by its progress hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cancelled;

/// Counts the cases done over the whole run, holds the cancel flag, and calls the hook (on the calling thread
/// only: the worker threads touch the two atomics, never the hook).
pub(crate) struct Gauge {
    hook: Option<ProgressHook>,
    total: usize,
    done: core::sync::atomic::AtomicUsize,
    cancelled: core::sync::atomic::AtomicBool,
}

impl Gauge {
    /// No hook, no count: the run works as it did before issue #923.
    pub(crate) fn none() -> Self {
        Self::new(None, 0)
    }

    pub(crate) fn new(hook: Option<ProgressHook>, total: usize) -> Self {
        if total > 0 {
            DAMAGE_RUNS.with(|c| c.set(c.get() + 1));
        }
        Self {
            hook,
            total,
            done: core::sync::atomic::AtomicUsize::new(0),
            cancelled: core::sync::atomic::AtomicBool::new(false),
        }
    }

    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    pub(crate) fn has_hook(&self) -> bool {
        self.hook.is_some()
    }

    pub(crate) fn done(&self) -> usize {
        self.done.load(core::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// [`Gauge::report`] for the thread that waits on workers: if the hook panics, the cancel flag is set before
    /// the panic goes on, so the workers stop at their next case and do not run every case that is left.
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    pub(crate) fn report_guarded(&self) {
        if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.report()))
        {
            self.cancelled
                .store(true, core::sync::atomic::Ordering::Relaxed);
            std::panic::resume_unwind(panic);
        }
    }

    /// Call the hook with the count now; a Break sets the cancel flag. Calling thread only. After a cancel the
    /// hook is not called again.
    pub(crate) fn report(&self) {
        if self.is_cancelled() {
            return;
        }
        if let Some(h) = &self.hook
            && h(DamageProgress {
                done: self.done(),
                total: self.total,
            })
            .is_break()
        {
            self.cancelled
                .store(true, core::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Run `f` on every item and return the results in the order of the items. With `workers` above 1 the items
/// are shared between scoped threads (std only), each taking the next item not yet taken. `f` is pure in its
/// item, so the order of the results does not depend on which worker ran which item. This is the one place a
/// per-case progress call or a cancel check belongs (issue #923): after `f` returns, or before a worker takes
/// its next item.
pub(crate) fn run_ordered<T, R, F>(
    items: &[T],
    workers: usize,
    gauge: &Gauge,
    f: F,
) -> Result<Vec<R>, Cancelled>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    use core::sync::atomic::Ordering;
    if gauge.is_cancelled() {
        return Err(Cancelled);
    }
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    if workers > 1 && items.len() > 1 {
        use std::sync::atomic::AtomicUsize;
        let next = AtomicUsize::new(0);
        // The workers see the two atomics and nothing else of the gauge: the hook stays on this thread.
        let (next, f, done, cancelled) = (&next, &f, &gauge.done, &gauge.cancelled);
        let mut slots: Vec<Option<R>> = items.iter().map(|_| None).collect();
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(move || {
                        let mut out: Vec<(usize, R)> = Vec::new();
                        while !cancelled.load(Ordering::Relaxed) {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(item) = items.get(i) else { break };
                            out.push((i, f(item)));
                            done.fetch_add(1, Ordering::Relaxed);
                        }
                        out
                    })
                })
                .collect();
            // This thread polls the counter while the workers run, and calls the hook (issue #923).
            if gauge.has_hook() {
                let mut last = gauge.done();
                while !handles.iter().all(|h| h.is_finished()) {
                    if gauge.done() != last {
                        last = gauge.done();
                        gauge.report_guarded();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                if gauge.done() != last {
                    gauge.report_guarded();
                }
            }
            for h in handles {
                match h.join() {
                    Ok(out) => {
                        for (i, r) in out {
                            slots[i] = Some(r);
                        }
                    }
                    Err(panic) => std::panic::resume_unwind(panic),
                }
            }
        });
        if gauge.is_cancelled() {
            return Err(Cancelled);
        }
        return Ok(slots
            .into_iter()
            .map(|r| r.expect("every case ran once"))
            .collect());
    }
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    let _ = workers;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(f(item));
        gauge.done.fetch_add(1, Ordering::Relaxed);
        gauge.report();
        if gauge.is_cancelled() {
            return Err(Cancelled);
        }
    }
    Ok(out)
}

#[cfg(all(test, feature = "parallel", not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Issue #923: a hook that panics stops the workers at their next case; they do not run every case left.
    #[test]
    fn a_panicking_hook_stops_the_workers() {
        let ran = AtomicUsize::new(0);
        let items: Vec<usize> = (0..400).collect();
        let hook: ProgressHook = std::rc::Rc::new(|p| {
            if p.done >= 2 {
                panic!("hook failed");
            }
            ControlFlow::Continue(())
        });
        let gauge = Gauge::new(Some(hook), items.len());
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_ordered(&items, 4, &gauge, |_| {
                ran.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_millis(5));
            })
        }));
        assert!(caught.is_err(), "the panic did not reach the caller");
        assert!(gauge.is_cancelled());
        let n = ran.load(Ordering::Relaxed);
        assert!(n < 100, "{n} of 400 cases ran after the hook panicked");
    }

    /// Issue #961: a cancel stops the workers at their next case. The result is `Cancelled` either way (the flag is
    /// read after the scope), so the count of cases started is what shows the early stop.
    #[test]
    fn a_cancel_stops_the_workers_early() {
        let started = AtomicUsize::new(0);
        let items: Vec<usize> = (0..200).collect();
        let at_break = std::rc::Rc::new(std::cell::Cell::new(None));
        let seen = at_break.clone();
        let hook: ProgressHook = std::rc::Rc::new(move |p| {
            if p.done >= 1 {
                seen.set(Some(seen.get().unwrap_or(p.done)));
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        });
        let gauge = Gauge::new(Some(hook), items.len());
        let out = run_ordered(&items, 4, &gauge, |_| {
            started.fetch_add(1, Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_millis(2));
        });
        assert!(out.is_err());
        let (n, done) = (started.load(Ordering::Relaxed), at_break.get().unwrap());
        // Cases under way finish (one per worker), and the poll is up to 5 ms apart: about 10 more starts.
        assert!(
            n <= done + 4 + 20,
            "{n} cases started; {done} were done at the break"
        );
        assert!(n < 50, "{n} of 200 cases started after the cancel");
    }
}
