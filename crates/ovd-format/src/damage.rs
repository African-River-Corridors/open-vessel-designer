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
        rows.push((
            kind,
            if liquid {
                Permeability::ByFilling(value)
            } else {
                Permeability::Fixed(value)
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
/// case with its righting lever curve over the 20 degree range (D12). The governing case is the first that does
/// not meet the criteria, or else the one with the least maximum righting lever.
pub fn gas_damage_verdicts(
    set: &RuleSet,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), String> {
    use ovd_core::damage_cases::{
        StandardOfDamage, deck_edge_immersed, gas_flooding, igc_damage_cases,
        without_breached_cargo,
    };
    use ovd_core::damage_stability::{
        DamageCriteria, Floats, INTERMEDIATE_FRACTIONS, assess, intermediate_stages,
        righting_lever_at,
    };
    use ovd_core::sections::{Gravity, Waterplane};
    use serde_json::json;
    let num = crate::eval::num;
    let mut rows: Vec<serde_json::Value> = Vec::new();
    let mut governing: Vec<serde_json::Value> = Vec::new();
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let data = igc_damage_data(set, loa, beam)?;
    let table = permeability_table(set, "igc-permeability")?;
    let standard = StandardOfDamage::igc_2g(spaces, loa, data.machinery_aft_intact);
    let cases = igc_damage_cases(spaces, loa, beam, &data.extents, &standard, &|k| {
        table.of(k, 0.0).unwrap_or(0.95)
    });
    let mut out: Vec<ovd_core::measured::DamageVerdict> = Vec::new();
    for c in conditions {
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
        // (index into `out`, the flooding, the damaged mass and centre of gravity) of the governing case
        let mut worst: Option<(
            usize,
            Vec<ovd_core::damage_stability::FloodedVolume>,
            f64,
            Gravity,
        )> = None;
        for case in &cases {
            let f = gas_flooding(case, spaces, &table, &filling);
            let (m, gd) = without_breached_cargo(mass, g, cargo, spaces, &f.breached);
            let outcome = assess(
                &hull.table,
                &f.flooded,
                m,
                gd,
                density,
                start,
                &DamageCriteria::IGC_2_7,
            );
            // Issue #760: the intermediate stages, by added mass with the cargo still aboard (D15 to D18).
            let stages = intermediate_stages(
                &hull.table,
                &f.flooded,
                &outcome,
                (mass, g),
                density,
                start,
                &DamageCriteria::IGC_2_7,
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
            let worse = match &worst {
                None => true,
                Some((i, ..)) => {
                    let w = &out[*i];
                    let o: &ovd_core::damage_stability::DamageOutcome = &w.outcome;
                    (o.ok && !outcome.ok) || (o.ok == outcome.ok && outcome.max_gz_m < o.max_gz_m)
                }
            };
            rows.push(json!({
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
            }));
            if worse {
                worst = Some((out.len(), f.flooded.clone(), m, gd));
            }
            out.push(ovd_core::measured::DamageVerdict {
                name: format!("{}: {}", c.condition.name, case.name),
                flooded: case.flooded.iter().map(|v| v.name.clone()).collect(),
                outcome,
                stages,
            });
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
            governing.push(json!({
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
            }));
        }
    }
    Ok((
        out,
        json!({
            "rule_set": set.key(),
            "method": "lost buoyancy, final stage of flooding (IGC 2.7.1.2, 2.7.2.1)",
            "cases": rows,
            "governing": governing,
        }),
    ))
}
