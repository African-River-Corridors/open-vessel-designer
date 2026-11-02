//! The liquid bulk (crude oil) path of `simulate`: the tank layout read from the params, and the
//! `liquid` block of the results (issue #331, spec 0006). The capacity and the stability chain is the
//! one of `ovd_core::barge_capacity::compute`, with the liquid layout (`ovd_core::liquid_bulk`) as its
//! volume branch. Nothing here runs for a gas or a dry bulk Cargo, so their results keep every key.

use crate::eval::num;
use ovd_core::barge_capacity::BargeCapacityResult;
use ovd_core::compartments::{Space, SpaceKind};
use ovd_core::liquid_bulk::adn_vessel_type::{self, AdnFlags};
use ovd_core::liquid_bulk::crude_grade;
use ovd_core::liquid_bulk::{LiquidBulkResult, LiquidLayout, PumpArrangement};
use serde_json::{Map, Value, json};

fn number(p: &Map<String, Value>, key: &str) -> Result<f64, String> {
    p.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("`params.{key}` must be a number"))
}

/// The cargo tank layout of the params: the inputs of the `integral_tanks` containment. Every key is
/// required: the Defaults Profile gives each one a default, so a Design always has them.
pub(crate) fn layout_of(p: &Map<String, Value>) -> Result<LiquidLayout, String> {
    let tanks = number(p, "liquid_tanks_in_line")?;
    if tanks.fract() != 0.0 || !(1.0..=1e6).contains(&tanks) {
        return Err("`params.liquid_tanks_in_line` must be a whole number of at least 1".into());
    }
    let pump_arrangement = match p.get("pump_arrangement").and_then(Value::as_str) {
        Some("deepwell") => PumpArrangement::Deepwell,
        Some("pump_room") => PumpArrangement::PumpRoom {
            length_m: number(p, "pump_room_length_m")?,
        },
        _ => return Err("`params.pump_arrangement` must be `deepwell` or `pump_room`".into()),
    };
    let centreline_bulkhead = p
        .get("centreline_bulkhead")
        .and_then(Value::as_bool)
        .ok_or("`params.centreline_bulkhead` must be true or false")?;
    Ok(LiquidLayout {
        double_side_m: number(p, "double_side_m")?,
        double_bottom_m: number(p, "double_bottom_m")?,
        tanks_in_line: tanks as u32,
        centreline_bulkhead,
        cofferdam_m: number(p, "cofferdam_m")?,
        pump_arrangement,
        slop_share: number(p, "slop_share")?,
        fill_ratio: number(p, "liquid_fill_ratio")?,
        heating_kw_per_m3: number(p, "heating_kw_per_m3")?,
        lightship_k_t_per_m3: number(p, "lightship_k_t_per_m3")?,
    })
}

fn space_json(s: &Space) -> Value {
    let full = s.fill(1.0);
    json!({
        "name": s.name,
        "kind": s.kind.key(),
        "side": s.side.key(),
        "x_aft_m": num(s.x_aft),
        "x_fwd_m": num(s.x_fwd),
        "volume_m3": num(s.volume_m3()),
        "lcg_m": num(full.lcg_m),
        "tcg_m": num(full.tcg_m),
        "kcg_m": num(full.vcg_m),
    })
}

/// The ADN Tank Vessel Type of the Cargo, from its Crude Grade, with the branch that decided it.
fn adn_type_json(cargo_key: &str) -> Value {
    match crude_grade::grade_of(cargo_key) {
        Some(g) => {
            let r = adn_vessel_type::derive(&AdnFlags::of(g));
            json!({
                "type": r.vessel_type.key(),
                "walls_distinct_from_hull": r.walls_distinct_from_hull,
                "basis": r.branch.text(),
                "grade": g.name,
            })
        }
        None => Value::Null,
    }
}

/// The `liquid` block of the results: the Spaces, the volumes, the free surfaces, the lightship
/// items and the ADN Tank Vessel Type. The masses and volumes are what the cost lines read.
pub(crate) fn liquid_json(l: &LiquidBulkResult, cargo_key: &str) -> Value {
    json!({
        "tanks_in_line": l.tanks_in_line,
        "cargo_tank_count": l.of_kind(SpaceKind::CargoTank).count(),
        "cargo_gross_m3": num(l.cargo_gross_m3),
        "slop_gross_m3": num(l.slop_gross_m3),
        "cargo_volume_m3": num(l.cargo_volume_m3),
        "cargo_volume_bound_t": num(l.cargo_mass_t),
        "slop_share": num(l.slop_share_actual),
        "cargo_tank_length_min_m": num(l.cargo_tank_length_min_m),
        "cargo_tank_length_max_m": num(l.cargo_tank_length_max_m),
        "cargo_tank_breadth_max_m": num(l.cargo_tank_breadth_max_m),
        "adn_vessel_type": adn_type_json(cargo_key),
        "spaces": l.spaces.iter().map(space_json).collect::<Vec<_>>(),
        "free_surface": l.free_surface.iter().map(|f| json!({
            "name": f.name,
            "length_m": num(f.length_m),
            "breadth_m": num(f.breadth_m),
            "inertia_m4": num(f.inertia_m4),
        })).collect::<Vec<_>>(),
        "lightship_items": l.lightship_items.iter().map(|(name, i)| json!({
            "name": name,
            "mass_t": num(i.mass_t),
            "kcg_m": num(i.kcg_m),
            "lcg_m": num(i.lcg_m),
        })).collect::<Vec<_>>(),
    })
}

/// A Space for the drawing: the Space as the results give it, with its cross-section corners
/// (`section_m`, (y, z) in metres), which the general arrangement draws.
fn drawing_space_json(s: &Space) -> Value {
    let mut v = space_json(s);
    v["section_m"] = Value::Array(
        s.section()
            .iter()
            .map(|(y, z)| json!([num(*y), num(*z)]))
            .collect(),
    );
    v
}

/// The ballast tanks of a dry bulk hull for the drawing, as `drawing.ballast_spaces` (issue #426).
/// Kept out of the results hash with the rest of `drawing`.
pub(crate) fn ballast_drawing_value(tanks: &[Space]) -> (&'static str, Value) {
    (
        "ballast_spaces",
        Value::Array(tanks.iter().map(drawing_space_json).collect()),
    )
}

/// The `drawing` values of a liquid Option: the layout the general arrangement will draw. Kept out
/// of the results hash with the rest of `drawing`.
pub(crate) fn drawing_values(r: &BargeCapacityResult) -> Vec<(&'static str, Value)> {
    let Some(l) = r.liquid.as_ref() else {
        return Vec::new();
    };
    vec![(
        "liquid_tanks",
        json!({
            "spaces": l.spaces.iter().map(drawing_space_json).collect::<Vec<_>>(),
        }),
    )]
}

/// The laden departure condition of a liquid bulk hull, as the damage runs take it: its mass and centre of
/// gravity (the damage changes neither), the intact floating position and the cargo.
pub(crate) struct Laden {
    pub mass: f64,
    pub g: ovd_core::sections::Gravity,
    /// The intact (height of the waterplane, trim in degrees) the solves start from.
    pub start: (f64, f64),
    /// The cargo's mass (t) and height of its centre of gravity (m).
    pub cargo: (f64, f64),
}

/// The laden departure condition of `conditions`; `None` without one, without weights, or when it has no
/// floating position.
pub(crate) fn laden_departure(
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
) -> Option<Laden> {
    use ovd_core::loading_conditions::Role;
    use ovd_core::sections::Gravity;
    let laden = conditions
        .iter()
        .find(|c| c.role == Role::FullLoadDeparture)?;
    // The mass and its centre of gravity: the weights of the condition. The damage does not change either.
    let mass: f64 = laden.weights.iter().map(|(_, i)| i.mass_t).sum();
    if mass.is_nan() || mass <= 0.0 {
        return None;
    }
    let g = Gravity {
        lcg_m: laden
            .weights
            .iter()
            .map(|(_, i)| i.mass_t * i.lcg_m)
            .sum::<f64>()
            / mass,
        tcg_m: 0.0,
        kg_m: laden
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
    let cargo = laden
        .weights
        .iter()
        .find(|(n, _)| n == "Cargo")
        .map_or((0.0, 0.0), |(_, i)| (i.mass_t, i.kcg_m));
    Some(Laden {
        mass,
        g,
        start,
        cargo,
    })
}

/// Issue #341: the assumed damages of the hull and what each does to the laden departure condition.
///
/// The first list is by the ADN extents of the vessel type of the Cargo (9.3.2.15.1 for type C, 9.3.3.15.1 for
/// type N); it is empty when the Cargo has no ADN type. The second is by the extents of MARPOL Annex I
/// regulation 28.2, worked out only when `marpol` is true (the Rule Set is in the stack). Both are empty when the
/// departure condition has no floating position. A gas Cargo has neither list (IGC 2.3 damage is not modelled).
pub(crate) fn damage_verdicts(
    hull: &ovd_core::hydrostatics::Hull,
    liquid: &LiquidBulkResult,
    cargo_key: &str,
    density: f64,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    marpol: bool,
) -> (
    Vec<ovd_core::measured::DamageVerdict>,
    Vec<ovd_core::measured::DamageVerdict>,
) {
    use ovd_core::damage_cases::{AdnExtents, DamageCase, adn_damage_cases, marpol_damage_cases};
    use ovd_core::damage_stability::{DamageCriteria, assess};
    let adn = crate::trace::adn_vessel_type_of(cargo_key);
    if adn.is_none() && !marpol {
        return (Vec::new(), Vec::new());
    }
    let none = || (Vec::new(), Vec::new());
    let Some(Laden {
        mass,
        g,
        start,
        cargo,
    }) = laden_departure(hull, conditions, density)
    else {
        return none();
    };
    // MARPOL Annex I 28.4.1 (spec 0012 D20, issue #764): "any outflow of liquids from damaged compartments"
    // is accounted for, so a breached cargo tank's oil leaves the condition. The cargo tanks are filled alike;
    // the slop tanks are empty in the loading conditions, so they lose nothing. ADN keeps the mass (its cargo
    // tanks are not breached: their permeability is 0).
    let verdicts = |cases: Vec<DamageCase>, criteria: &DamageCriteria, outflow: bool| {
        cases
            .into_iter()
            .map(|case| {
                let (m, gd) = if outflow {
                    let breached: Vec<String> = case
                        .flooded
                        .iter()
                        .filter(|f| {
                            liquid
                                .spaces
                                .iter()
                                .any(|s| s.name == f.name && s.kind == SpaceKind::CargoTank)
                        })
                        .map(|f| f.name.clone())
                        .collect();
                    ovd_core::damage_cases::without_breached_cargo(
                        mass,
                        g,
                        cargo,
                        &liquid.spaces,
                        &breached,
                    )
                } else {
                    (mass, g)
                };
                ovd_core::measured::DamageVerdict {
                    flooded: case.flooded.iter().map(|f| f.name.clone()).collect(),
                    outcome: assess(&hull.table, &case.flooded, m, gd, density, start, criteria),
                    name: case.name,
                    stages: Vec::new(),
                }
            })
            .collect::<Vec<_>>()
    };
    // ADN 9.3.2.15.1 (c) and 9.3.3.15.1 (c): the permeabilities are the `adn-tank` set's table (spec 0012 D9).
    let Ok(adn_permeability) = crate::damage::built_in_permeability("adn-tank", "adn-permeability")
    else {
        return none();
    };
    let by_adn = adn.map_or_else(Vec::new, |t| {
        let extents = if t == adn_vessel_type::AdnVesselType::C {
            AdnExtents::TYPE_C
        } else {
            AdnExtents::TYPE_N
        };
        verdicts(
            adn_damage_cases(
                &liquid.spaces,
                hull.spec.loa,
                hull.spec.beam,
                &extents,
                &|kind| adn_permeability.of(kind, 0.0).unwrap_or(0.0),
            ),
            &DamageCriteria::ADN_2025,
            false,
        )
    });
    let by_marpol = if marpol {
        verdicts(
            marpol_damage_cases(&liquid.spaces, hull.spec.loa, hull.spec.beam),
            &DamageCriteria::MARPOL_I_28,
            true,
        )
    } else {
        Vec::new()
    };
    (by_adn, by_marpol)
}

/// Which rule's damage a liquid bulk hull is run through (issue #763).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiquidRule {
    /// ES-TRIN 2025/1 28.03(4) to (6), a craft over 110 m: final and intermediate stages.
    EsTrin,
    /// NORMAM-202/DPC 5.23.8 and 5.23.9: the final stage.
    Normam,
}

/// Issue #763: the assumed damages of ES-TRIN 28.03(4) or NORMAM 5.23.8 on the laden departure condition of a
/// liquid bulk hull, and what each does. `table` is the rule's Permeability Table; the cargo and slop tanks are
/// full at the laden departure (0 or 95 % "according to their function", ES-TRIN 28.03(4)(d), NORMAM 5.23.8 (f)),
/// so they flood nothing and keep their oil; every other space is empty (its table value). Empty when the
/// departure condition has no floating position.
pub(crate) fn liquid_rule_damage(
    rule: LiquidRule,
    hull: &ovd_core::hydrostatics::Hull,
    liquid: &LiquidBulkResult,
    density: f64,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    table: &ovd_core::damage_cases::PermeabilityTable,
) -> Vec<ovd_core::measured::DamageVerdict> {
    use ovd_core::damage_cases::{
        AdnExtents, CaseOptions, adn_damage_cases_with, deck_edge_immersed,
        es_trin_longitudinal_extent_m, machinery_bulkheads_intact, normam_damage_cases,
    };
    use ovd_core::damage_stability::{DamageCriteria, assess, intermediate_stages};
    use ovd_core::sections::Waterplane;
    let Some(Laden { mass, g, start, .. }) = laden_departure(hull, conditions, density) else {
        return Vec::new();
    };
    let (loa, beam) = (hull.spec.loa, hull.spec.beam);
    let by_function = |kind: SpaceKind| {
        let full = matches!(kind, SpaceKind::CargoTank | SpaceKind::Slop);
        table.of(kind, if full { 1.0 } else { 0.0 }).unwrap_or(0.0)
    };
    let cases = match rule {
        LiquidRule::EsTrin => adn_damage_cases_with(
            &liquid.spaces,
            loa,
            beam,
            &AdnExtents::ES_TRIN_28_03,
            &CaseOptions {
                long_m: es_trin_longitudinal_extent_m(loa),
                cut: true,
                intact_bulkheads_m: &machinery_bulkheads_intact(&liquid.spaces, loa),
            },
            &by_function,
        ),
        LiquidRule::Normam => normam_damage_cases(&liquid.spaces, loa, beam, &by_function),
    };
    let (criteria, stage_criteria) = match rule {
        LiquidRule::EsTrin => (
            DamageCriteria::ES_TRIN_28_03_FINAL,
            DamageCriteria::ES_TRIN_28_03_INTERMEDIATE,
        ),
        LiquidRule::Normam => (DamageCriteria::NORMAM_5_23_9, DamageCriteria::NORMAM_5_23_9),
    };
    cases
        .into_iter()
        .map(|case| {
            let mut outcome = assess(
                &hull.table,
                &case.flooded,
                mass,
                g,
                density,
                start,
                &criteria,
            );
            // NORMAM 5.23.9 (c): a heel above 25 degrees is accepted up to 30 only when the deck is not immersed.
            if rule == LiquidRule::Normam
                && outcome.ok
                && outcome.heel_deg.abs() > 25.0
                && deck_edge_immersed(
                    &Waterplane {
                        heel_deg: outcome.heel_deg,
                        trim_deg: outcome.trim_deg,
                        c: outcome.waterplane_c_m,
                    },
                    (0.0, loa),
                    0.5 * beam,
                    hull.spec.depth,
                )
            {
                outcome.ok = false;
            }
            let stages = if rule == LiquidRule::EsTrin {
                intermediate_stages(
                    &hull.table,
                    &case.flooded,
                    &outcome,
                    (mass, g),
                    density,
                    start,
                    &stage_criteria,
                )
            } else {
                Vec::new()
            };
            ovd_core::measured::DamageVerdict {
                flooded: case.flooded.iter().map(|f| f.name.clone()).collect(),
                outcome,
                name: case.name,
                stages,
            }
        })
        .collect()
}

/// The `damage` block of the results: what each assumed damage does (unhashed, with the rest of the
/// Rule Trace inputs).
pub(crate) fn damage_json(verdicts: &[ovd_core::measured::DamageVerdict]) -> Value {
    Value::Array(
        verdicts
            .iter()
            .map(|d| {
                let o = &d.outcome;
                json!({
                    "name": d.name,
                    "flooded": d.flooded,
                    "floats": match o.floats {
                        ovd_core::damage_stability::Floats::Yes => "yes",
                        ovd_core::damage_stability::Floats::Sinks => "sinks",
                        ovd_core::damage_stability::Floats::NoEquilibrium => "no equilibrium",
                    },
                    "heel_deg": num(o.heel_deg),
                    "trim_deg": num(o.trim_deg),
                    "draught_mid_m": num(o.draught_mid_m),
                    "range_end_deg": num(o.end_deg),
                    "max_gz_m": num(o.max_gz_m),
                    "area_m_rad": num(o.area_m_rad),
                    "ok": o.ok,
                })
            })
            .collect(),
    )
}

/// Issue #763: the block of one rule's damage cases on a liquid bulk hull (unhashed): the Rule Set, the method and
/// each case as [`damage_json`] gives it, with its intermediate stages where it has them.
pub(crate) fn rule_damage_json(
    set_key: &str,
    method: &str,
    verdicts: &[ovd_core::measured::DamageVerdict],
) -> Value {
    let Value::Array(mut cases) = damage_json(verdicts) else {
        return Value::Null;
    };
    for (case, d) in cases.iter_mut().zip(verdicts) {
        if d.stages.is_empty() {
            continue;
        }
        case["stages"] = Value::Array(
            d.stages
                .iter()
                .zip(ovd_core::damage_stability::INTERMEDIATE_FRACTIONS)
                .map(|(st, fraction)| {
                    json!({
                        "fraction": num(fraction),
                        "floats": match st.floats {
                            ovd_core::damage_stability::Floats::Yes => "yes",
                            ovd_core::damage_stability::Floats::Sinks => "sinks",
                            ovd_core::damage_stability::Floats::NoEquilibrium => "no equilibrium",
                        },
                        "heel_deg": num(st.heel_deg),
                        "max_gz_m": num(st.max_gz_m),
                        "area_m_rad": num(st.area_m_rad),
                        "ok": st.ok,
                    })
                })
                .collect(),
        );
    }
    json!({ "rule_set": set_key, "method": method, "cases": cases })
}

/// The `oil_outflow` block of the results: OM of MARPOL Annex I regulation 23 and what it is made of (unhashed,
/// with the rest of the Rule Trace inputs).
pub(crate) fn oil_outflow_json(o: &ovd_core::liquid_bulk::oil_outflow::OilOutflow) -> Value {
    json!({
        "cargo_volume_m3": num(o.c_m3),
        "nominal_density_kg_per_m3": num(o.rho_n),
        "mean_side_outflow_m3": num(o.oms_m3),
        "mean_bottom_outflow_tide_0_m3": num(o.omb_0_m3),
        "mean_bottom_outflow_tide_minus_2_5_m3": num(o.omb_25_m3),
        "mean_bottom_outflow_m3": num(o.omb_m3),
        "mean_outflow_parameter": num(o.om),
        "limit": num(o.limit),
        "ratio": num(o.ratio()),
        "tanks": o.tanks.iter().map(|t| json!({
            "name": t.name,
            "oil_m3": num(t.oil_m3),
            "side_probability_port": num(t.ps_port),
            "side_probability_starboard": num(t.ps_starboard),
            "bottom_probability_port": num(t.pb_port),
            "bottom_probability_starboard": num(t.pb_starboard),
            "bottom_outflow_tide_0_m3": num(t.ob_0_m3),
            "bottom_outflow_tide_minus_2_5_m3": num(t.ob_25_m3),
            "capture_factor": num(t.cdb),
        })).collect::<Vec<_>>(),
    })
}
