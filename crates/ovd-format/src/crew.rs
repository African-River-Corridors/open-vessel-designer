//! The crew model of spec 0011 section 4.1 (issue #778) on the manning Rule Sets: it reads the tables of
//! `liberia-man-001`, `uk-msn-1868` and `ccnr-rpn-19` from the library, runs the method of
//! [`ovd_core::crew`], and gives the `crew` results block.
//!
//! Concept and preliminary design only: never a Safe Manning Document (ADR 0011). The block is a new
//! output outside the results hash: nothing reads the crew until the deckhouse rooms are sized (issue
//! #780).

use crate::eval::num;
use crate::rule_set::{Library, is_newer};
use crate::trace::service_area_of;
use crate::vdesign::{DesignOption, Source};
use arc_rules::eval::{MeasureValue, table_value};
use arc_rules::model::{RowWhen, RuleSet, Table};
use ovd_core::crew::{
    self as cr, ColumnChoice, InlandCrew, LIBERIA_ROLES, LiberiaColumn, RPN_ROLES, Role, RpnCell,
};
use serde_json::{Value, json};

/// What the model reads of a computed hull.
#[derive(Debug, Clone, PartialEq)]
pub struct CrewFacts {
    /// The gross tonnage used (computed, ITC 1969, or typed) and where it came from.
    pub gt: f64,
    pub gt_source: String,
    /// The installed propulsion power (kW).
    pub propulsion_kw: f64,
    /// The total genset rating (kWe) of a diesel- or gas-electric hull; 0 where there are none (D14).
    pub genset_kw: f64,
    /// The length of the hull (m), for the RPN length group.
    pub length_m: f64,
    /// The Cargo is a gas: the cargo engineer is an owner option (step 7).
    pub gas_cargo: bool,
}

/// The newest built-in or loaded set of this id.
fn latest<'a>(library: &'a Library, id: &str) -> Option<&'a RuleSet> {
    library.sets().iter().filter(|s| s.id == id).fold(
        None,
        |best: Option<&RuleSet>, s| match best {
            Some(b) if !is_newer(&s.version, &b.version) => Some(b),
            _ => Some(s),
        },
    )
}

fn table<'a>(set: &'a RuleSet, id: &str) -> Result<&'a Table, String> {
    set.table(id)
        .ok_or_else(|| format!("{} holds no table `{id}`", set.key()))
}

/// A numeric table read at a number.
pub fn at(set: &RuleSet, id: &str, v: f64) -> Result<f64, String> {
    table_value(table(set, id)?, &MeasureValue::Number(v))
        .map_err(|e| format!("{}: `{id}`: {e}", set.key()))
}

/// A text-keyed table as `(key, value)` rows, in the order of the table.
fn keyed(set: &RuleSet, id: &str) -> Result<Vec<(String, f64)>, String> {
    let t = table(set, id)?;
    let mut out = Vec::new();
    for row in &t.rows {
        let RowWhen::OneOf(keys) = &row.when else {
            return Err(format!("{}: `{id}` is not a keyed table", set.key()));
        };
        let value = row
            .coefficients
            .iter()
            .find(|(n, _)| n == "a")
            .map(|(_, v)| *v)
            .ok_or_else(|| format!("{}: `{id}` has a row with no value", set.key()))?;
        out.extend(keys.iter().map(|k| (k.clone(), value)));
    }
    Ok(out)
}

fn whole(v: f64) -> u32 {
    if v.is_finite() && v > 0.0 {
        (v + 0.5) as u32
    } else {
        0
    }
}

/// The columns of the Liberia table (12.2) from the set.
pub fn liberia_columns(set: &RuleSet) -> Result<Vec<LiberiaColumn>, String> {
    let gt = keyed(set, "liberia-col-gt-band")?;
    let kw = keyed(set, "liberia-col-kw-band")?;
    let ums = keyed(set, "liberia-col-ums")?;
    let cw = keyed(set, "liberia-col-coastwise")?;
    let roles: Vec<Vec<(String, f64)>> = LIBERIA_ROLES
        .iter()
        .map(|r| keyed(set, &format!("liberia-{r}")))
        .collect::<Result<_, _>>()?;
    let find = |rows: &[(String, f64)], id: &str| {
        rows.iter()
            .find(|(k, _)| k == id)
            .map(|(_, v)| *v)
            .ok_or_else(|| format!("{}: no value for column `{id}`", set.key()))
    };
    gt.iter()
        .map(|(id, g)| {
            let mut counts = [0u32; 10];
            for (slot, rows) in counts.iter_mut().zip(&roles) {
                *slot = whole(find(rows, id)?);
            }
            Ok(LiberiaColumn {
                id: id.clone(),
                gt_band: whole(*g),
                kw_band: whole(find(&kw, id)?),
                ums: find(&ums, id)? > 0.5,
                coastwise: find(&cw, id)? > 0.5,
                counts,
            })
        })
        .collect()
}

/// The cells of the RPN table of article `art` (`19.02` or `19.03`) from the set.
pub fn rpn_cells(set: &RuleSet, art: &str) -> Result<Vec<RpnCell>, String> {
    let keys = keyed(set, &format!("ccnr-rpn-{art}-boatmaster"))?;
    let mut by_role: Vec<Option<Vec<(String, f64)>>> = Vec::new();
    for (_, name) in RPN_ROLES {
        // The 19.02 table has no engineer.
        by_role.push(keyed(set, &format!("ccnr-rpn-{art}-{name}")).ok());
    }
    Ok(keys
        .iter()
        .map(|(key, _)| {
            let mut counts = [0u32; 6];
            for (slot, rows) in counts.iter_mut().zip(&by_role) {
                if let Some(rows) = rows
                    && let Some((_, v)) = rows.iter().find(|(k, _)| k == key)
                {
                    *slot = whole(*v);
                }
            }
            RpnCell {
                key: key.clone(),
                counts,
            }
        })
        .collect())
}

/// The minimum crew of a rigid convoy or other rigid assembly of RPN 19.03, group 1 to 5 (issue #785 wires it to the tug Formation), for the
/// daily navigation hours. A pusher with its pushed barges is one of these groups; the group comes
/// from the assembly's length and breadth and the number of pushed barges, which the model does not
/// hold yet, so a run does not call this (the tug Formation, wave 3 of spec 0011).
pub fn rigid_convoy_crew(
    library: &Library,
    group: u32,
    navigation_hours: f64,
) -> Result<InlandCrew, String> {
    let set = latest(library, "ccnr-rpn-19").ok_or("no ccnr-rpn-19 Rule Set")?;
    let mode = cr::rpn_mode(whole(at(
        set,
        "ccnr-rpn-18.01-operating-mode",
        navigation_hours,
    )?));
    let cells = rpn_cells(set, "19.03")?;
    cr::inland_crew(&cells, group, mode)
        .ok_or_else(|| format!("RPN 19.03 has no group {group} in mode {mode}"))
}

fn roles_json(roles: &[Role]) -> Value {
    Value::Array(
        roles
            .iter()
            .map(|r| json!({ "role": r.role, "count": r.count }))
            .collect(),
    )
}

fn set_json(set: &RuleSet) -> Value {
    json!({
        "id": set.id,
        "version": set.version,
        "hash": set.hash,
        "status": "draft",
        "basis": "a draft Rule Set: no second reader has verified it",
    })
}

/// The UK guidance for the cross-check: officers of Annex A and Annex B, trading area unlimited.
/// "Near coastal" is the UK's own meaning (150 nm from a UK haven), which the Service Area cannot
/// show, so the guidance for unlimited trade is taken.
fn uk_officers(
    set: &RuleSet,
    gt: f64,
    kw: f64,
    notes: &mut Vec<String>,
) -> Result<(u32, u32), String> {
    let deck = whole(at(set, "uk-annex-a-unlimited", gt)?);
    // Annex B starts at 350 kW; below it the ship is off the table and the 350 kW row is taken.
    if kw < 350.0 {
        notes.push(format!(
            "{} kW is under the 350 kW start of MSN 1868 Annex B: the row for 350 to 750 kW is used",
            num_text(kw)
        ));
    }
    let eng = whole(at(set, "uk-annex-b-unlimited", kw.max(350.0))?);
    Ok((deck, eng))
}

fn num_text(v: f64) -> String {
    format!("{v:.0}")
}

/// The `crew` results block of a self-propelled hull, or `None` when the hull has no power to man.
///
/// `Err` is a message the run shows as a note: a Rule Set that lacks a table or a value the tables do
/// not give.
pub fn crew_json(
    option: &DesignOption,
    library: &Library,
    facts: &CrewFacts,
) -> Result<Value, String> {
    let area = service_area_of(option);
    let reaches_sea = area == "coastal-and-river";
    let input_num = |name: &str| option.input(name).and_then(|i| i.value.as_f64());
    let default_set = if reaches_sea {
        "liberia-man-001"
    } else {
        "ccnr-rpn-19"
    };
    // A value marked default is the Service Area's default, whatever a file saved before filled in
    // (its fixed legacy value is for a river-only hull): only a typed value overrides it.
    let typed_input = |name: &str| option.input(name).filter(|i| i.source != Source::Default);
    let chosen = typed_input("manning_rule_set")
        .and_then(|i| i.value.as_str())
        .unwrap_or(default_set);
    if !cr::MANNING_RULE_SETS.contains(&chosen) {
        return Err(format!("`{chosen}` is not a manning Rule Set"));
    }
    let hours = typed_input("navigation_hours_per_day")
        .and_then(|i| i.value.as_f64())
        .unwrap_or(if reaches_sea { 24.0 } else { 14.0 });
    let ums = option
        .input("unmanned_machinery_space")
        .and_then(|i| i.value.as_bool())
        .unwrap_or(true);
    let typed = input_num("crew").filter(|c| *c > 0.0).map(whole);
    let mut notes: Vec<String> = Vec::new();
    // D14: the power band of a diesel- or gas-electric hull takes the total genset kW, the higher
    // band; the greater of that and the propulsion power is taken.
    let power_kw = facts.propulsion_kw.max(facts.genset_kw);
    let power_basis = if facts.genset_kw > facts.propulsion_kw {
        "the total genset rating (the higher band, spec 0011 D14)"
    } else {
        "the installed propulsion power"
    };

    let mut block = json!({
        "inputs": {
            "gross_tonnage": num(facts.gt),
            "gross_tonnage_source": facts.gt_source,
            "navigation_hours_per_day": num(hours),
            "unmanned_machinery_space": ums,
            "propulsion_kw": num(facts.propulsion_kw),
            "genset_kw": num(facts.genset_kw),
            "power_kw": num(power_kw),
            "power_basis": power_basis,
            "length_m": num(facts.length_m),
            "gas_cargo": facts.gas_cargo,
            "service_area": area,
        },
    });
    let minimum;
    let margin;
    if chosen == "ccnr-rpn-19" {
        let set = latest(library, "ccnr-rpn-19").ok_or("no ccnr-rpn-19 Rule Set")?;
        let group = whole(at(set, "ccnr-rpn-19.02-group", facts.length_m)?);
        let mode_index = at(set, "ccnr-rpn-18.01-operating-mode", hours);
        let mode = match mode_index {
            Ok(v) => cr::rpn_mode(whole(v)),
            Err(_) => {
                notes.push(format!(
                    "{} navigation hours in 24 is over the 24 hours of operating mode B (RPN 18.01(1)); mode B is used",
                    num_text(hours)
                ));
                "B"
            }
        };
        let cells = rpn_cells(set, "19.02")?;
        let crew = cr::inland_crew(&cells, group, mode)
            .ok_or_else(|| format!("RPN 19.02 has no group {group} in mode {mode}"))?;
        notes.push(format!(
            "RPN 19.02 table, group {group}, operating mode {mode}: the equipment standard is not an input, so the greatest crew of S1 and S2 and of every alternative the table offers is taken ({} persons; the least is {}). A craft below S1 takes more (RPN 19.05), not assumed",
            crew.minimum, crew.least
        ));
        notes.push(
            "RPN 19.02 prescribes no engineer, no cook and no MLC accommodation (MLC does not reach inland-only ships); a mode B crew sleeps one to a cabin when it rests underway (RPN 18.02(4))"
                .to_string(),
        );
        minimum = crew.minimum;
        margin = cr::owner_margin(false, facts.gas_cargo);
        block["regime"] = json!("inland");
        block["rule_set"] = set_json(set);
        block["rpn"] = json!({
            "article": "19.02(1)",
            "group": group,
            "operating_mode": mode,
            "cell": crew.key,
            "least_cell_total": crew.least,
        });
        block["roles"] = roles_json(
            &crew
                .roles
                .iter()
                .filter(|r| r.count > 0)
                .cloned()
                .collect::<Vec<_>>(),
        );
        block["cook"] = json!(0);
    } else {
        let lib_set = latest(library, "liberia-man-001").ok_or("no liberia-man-001 Rule Set")?;
        let uk_set = latest(library, "uk-msn-1868").ok_or("no uk-msn-1868 Rule Set")?;
        let columns = liberia_columns(lib_set)?;
        let gt_band = whole(at(lib_set, "liberia-gt-band", facts.gt)?);
        let kw_band = whole(at(lib_set, "liberia-kw-band", power_kw)?);
        if power_kw < 750.0 {
            notes.push(format!(
                "{} kW is under the 750 kW start of the MAN-001 table: the 750 to 3,000 kW column is used",
                num_text(power_kw)
            ));
        }
        let ColumnChoice {
            index,
            exact,
            other_neighbour,
        } = cr::select_liberia_column(&columns, gt_band, kw_band, ums)
            .ok_or("MAN-001 table has no column for this hull")?;
        let column = &columns[index];
        let crew = cr::seagoing_crew(column, facts.gt, hours, reaches_sea);
        let mut column_json = json!({
            "column": column.id,
            "exact": exact,
            "gt_band": column.gt_band,
            "kw_band": column.kw_band,
            "unattended_machinery": column.ums,
            "table_total": column.total(),
        });
        if !exact {
            let alt = other_neighbour.map(|i| &columns[i]);
            let text = format!(
                "MAN-001 12.2 has no column for GT band {gt_band}, kW band {kw_band}, {} machinery: column {} (the higher neighbour, spec 0011 4.1 step 6) is used{}",
                if ums { "unattended" } else { "attended" },
                column.id,
                alt.map_or(String::new(), |a| format!(
                    "; the other neighbour, column {}, has {} persons before the cook",
                    a.id,
                    a.total()
                ))
            );
            column_json["hole"] = json!(text);
            if let Some(a) = alt {
                column_json["other_neighbour_column"] = json!(a.id);
                column_json["other_neighbour_total"] = json!(a.total());
            }
            notes.push(text);
        }
        if !reaches_sea {
            notes.push(
                "A seagoing scale on a river-only hull: MLC 2006 does not reach it, so no cook is added"
                    .to_string(),
            );
        }
        let (uk_deck, uk_eng) = uk_officers(uk_set, facts.gt, power_kw, &mut notes)?;
        let liberia_officers = crew.deck_officers + crew.engineers;
        let check = cr::cross_check(uk_deck + uk_eng, liberia_officers);
        if check.flagged {
            notes.push(format!(
                "The UK guidance and the Liberia scale differ by {} officers ({} against {}): over one person, so a flag may ask for another number",
                check.difference.abs(), check.uk_officers, check.liberia_officers
            ));
        }
        let mut roles = crew.roles.clone();
        let mut cook = crew.cook;
        let mut min = crew.minimum;
        if chosen == "uk-msn-1868" {
            // The UK guidance gives officers only: its officers, with the ratings of MAN-001.
            let deck = uk_deck.max(crew.watches.deck_officers);
            let before_cook = deck + uk_eng + crew.ratings;
            cook = u32::from(reaches_sea && before_cook >= cr::COOK_AT_MANNING);
            min = before_cook + cook;
            roles = vec![
                Role {
                    role: "deck-officers".to_string(),
                    count: deck,
                },
                Role {
                    role: "engineer-officers".to_string(),
                    count: uk_eng,
                },
                Role {
                    role: "ab-deck-and-ratings-deck".to_string(),
                    count: crew.roles[3].count + crew.roles[4].count,
                },
                Role {
                    role: "ab-engine-and-ratings-engine".to_string(),
                    count: crew.roles[8].count + crew.roles[9].count,
                },
            ];
            notes.push(
                "MSN 1868 gives officers only (ratings are left to the factors in its section 2): the officers are the UK guidance, the ratings and the cook those of the MAN-001 column"
                    .to_string(),
            );
        }
        if !ums {
            notes.push(
                "Not unattended: the non-UMS columns of MAN-001 are used; the UK table assumes unattended machinery, so its engineers are an understatement"
                    .to_string(),
            );
        }
        if crew.added_by_watches > 0 {
            notes.push(format!(
                "The watches need {} more deck officer(s) than the MAN-001 column: added as officers of the watch",
                crew.added_by_watches
            ));
        }
        minimum = min;
        margin = cr::owner_margin(true, facts.gas_cargo);
        let selected_set = if chosen == "uk-msn-1868" {
            uk_set
        } else {
            lib_set
        };
        block["regime"] = json!("seagoing");
        block["rule_set"] = set_json(selected_set);
        block["manning_scale"] = column_json;
        block["watches"] = json!({
            "watchkeepers": crew.watches.watchkeepers,
            "master_watch_free": crew.watches.master_watch_free,
            "deck_officers_needed": crew.watches.deck_officers,
            "basis": "more than 12 navigation hours in 24: three watchkeepers (STCW A-VIII/1 rest, an estimate); the master keeps no regular watch above 3,000 GT (MAN-001 3.2.1)",
        });
        block["roles"] = roles_json(
            &roles
                .iter()
                .filter(|r| r.count > 0)
                .cloned()
                .collect::<Vec<_>>(),
        );
        // The officers' berths: the deck officers and engineers of the selected set, and the owner
        // margin persons who perform officers' duties (the electro-technical officer, the cargo
        // engineer). The rooms size these to MLC A3.1.9(k) and (m) (issue #892). With a typed crew
        // the margin is not itemised, so none is added.
        let manning_officers = if chosen == "uk-msn-1868" {
            uk_deck.max(crew.watches.deck_officers) + uk_eng
        } else {
            crew.deck_officers + crew.engineers
        };
        let margin_officers: u32 = if typed.is_some() {
            0
        } else {
            margin
                .iter()
                .filter(|r| r.role == "electro-technical-officer" || r.role == "cargo-engineer")
                .map(|r| r.count)
                .sum()
        };
        block["officers"] = json!(manning_officers + margin_officers);
        block["deck_officers"] = json!(crew.deck_officers);
        block["engineers"] = json!(crew.engineers);
        block["ratings"] = json!(crew.ratings);
        block["cook"] = json!(cook);
        block["cross_check"] = json!({
            "rule_set": set_json(uk_set),
            "uk_officers": check.uk_officers,
            "uk_deck_officers": uk_deck,
            "uk_engineer_officers": uk_eng,
            "liberia_officers": check.liberia_officers,
            "difference": check.difference,
            "flagged": check.flagged,
            "trading_area": "unlimited",
            "basis": "officers only (MSN 1868 gives no ratings); trading area unlimited, the cautious reading, as the UK's near-coastal limit (150 nm from a UK haven) is a national meaning the Service Area cannot show; a difference over one person is flagged (spec 0011 4.1 step 10)",
        });
    }
    let b = cr::berths(minimum, &margin, typed);
    let below = typed.is_some_and(|t| t < minimum);
    if below {
        notes.push(format!(
            "The typed crew of {} is under the manning minimum of {minimum}: a flag would ask for the minimum",
            typed.unwrap_or(0)
        ));
    }
    block["source"] = json!(if typed.is_some() { "typed" } else { "computed" });
    block["typed_crew"] = typed.map_or(Value::Null, |t| json!(t));
    block["below_minimum"] = json!(below);
    block["minimum"] = json!(minimum);
    block["owner_margin"] = roles_json(&margin);
    block["planning_low"] = json!(b.planning.0);
    block["planning_high"] = json!(b.planning.1);
    block["crew"] = json!(b.crew);
    block["spare_berths"] = json!(cr::SPARE_BERTHS);
    block["berths"] = json!(b.berths);
    block["notes"] = json!(notes);
    block["status"] = json!("estimate");
    block["basis"] = json!(
        "Spec 0011 section 4.1: the minimum crew from a manning Rule Set (draft, unverified), the owner margin apart from it, and 2 spare berths (D16, an estimate: a pilot and one supernumerary). Concept and preliminary design only: never a Safe Manning Document. The deckhouse rooms read the berths (issue #780)."
    );
    Ok(block)
}
