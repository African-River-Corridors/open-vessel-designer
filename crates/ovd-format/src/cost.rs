//! The capital budget of an Option: the core's `barge_cost::compute`, run where the results are
//! assembled, for a gas barge only: the core models a barge with Type C tanks and an LNG cargo system,
//! so any other Cargo has no budget (`cost` is null). The mass inputs come from the Option's own
//! weight breakdown; the tug, which the engine does not size, keeps the core's defaults. The rates are
//! the Option's cost-rate inputs. Nothing here feeds `simulate`: the budget is an output, kept out of the results hash
//! (`UNHASHED_KEYS`), so saved Designs keep their hashes.

use crate::vdesign::TANK_INTERNALS_BASIS;
use crate::vdesign::{
    COST_RATES_BASIS, COST_RATES_BASIS_USER, DesignOption, Source, cost_prices_basis,
};
use ovd_core::barge_cost::{
    BargeCostInputs, BargeCostRates, BargeCostResult, ItemQuantities, ItemRates,
    compute_with_items, item_lines,
};
use ovd_core::liquid_bulk::LiquidBulkResult;
use ovd_core::liquid_bulk::cost::{self as liquid_cost, CostQuantities, LiquidCostRates};
use ovd_core::models::WeightBreakdown;
use serde_json::{Value, json};

/// The 14 numeric rates, as `(input name, how to set it on the core's rates)`. The currency is a
/// text input of its own (issue #429).
type SetRate = fn(&mut BargeCostRates, f64);
pub const RATE_INPUTS: [(&str, SetRate); 14] = [
    ("hull_steel_rate", |r, v| r.hull_steel_rate = v),
    ("tank_shell_rate", |r, v| r.tank_shell_rate = v),
    ("tank_supports_rate", |r, v| r.tank_supports_rate = v),
    ("insulation_rate", |r, v| r.insulation_rate = v),
    ("piping_rate", |r, v| r.piping_rate = v),
    ("outfitting_rate", |r, v| r.outfitting_rate = v),
    ("cargo_system_lump", |r, v| r.cargo_system_lump = v),
    ("paint_anodes_misc", |r, v| r.paint_anodes_misc = v),
    ("battery_rate_per_kwh", |r, v| r.battery_rate_per_kwh = v),
    ("propulsion_rate_per_kw", |r, v| {
        r.propulsion_rate_per_kw = v
    }),
    ("wheelhouse_accom_lump", |r, v| r.wheelhouse_accom_lump = v),
    ("tug_outfitting_lump", |r, v| r.tug_outfitting_lump = v),
    ("yard_margin_fraction", |r, v| r.yard_margin_fraction = v),
    ("delivery_lump", |r, v| r.delivery_lump = v),
];

/// The rates of an Option: its cost-rate inputs and its currency. `None` if an input is missing or
/// not a number, or the currency is not text.
pub fn rates_of(option: &DesignOption) -> Option<BargeCostRates> {
    let mut rates = ovd_core::barge_cost::example_rates().ok()?;
    rates.currency = option.input("currency")?.value.as_str()?.to_string();
    for (name, set) in RATE_INPUTS {
        set(&mut rates, option.input(name)?.value.as_f64()?);
    }
    Some(rates)
}

/// The item rates of a self-propelled hull's own stern (spec 0011 D9, issue #773), as `(input name,
/// unit, how to set it)`. They are Option inputs only: the rates file of the ATB budget keeps its 14
/// keys (its tug has none of these items).
type SetItemRate = fn(&mut ItemRates, f64);
pub const ITEM_RATE_INPUTS: [(&str, SetItemRate); 4] = [
    ("genset_rate_per_kwe", |r, v| r.genset_rate_per_kwe = v),
    ("machinery_steel_rate", |r, v| r.machinery_steel_rate = v),
    ("deckhouse_rate_per_m2", |r, v| r.deckhouse_rate_per_m2 = v),
    ("gas_fuel_system_rate_per_kw", |r, v| {
        r.gas_fuel_system_rate_per_kw = v
    }),
];

/// What each item line prices, in the order of [`item_lines`]: (item, rate input, quantity unit,
/// what the quantity is).
const ITEM_BASES: [(&str, &str, &str, &str); 4] = [
    (
        "gensets",
        "genset_rate_per_kwe",
        "kWe",
        "Installed genset rating, every set and the reserve included: a gas-fuelled stern block's sets, \
         or a diesel-electric plant's N + 1 sets of P / N each. 0 where the stern has no gensets. The \
         propulsion line keeps the installed propulsion power (propulsors, drives, motors, switchboard); \
         the gensets that make the power were in no line before.",
    ),
    (
        "machinery_steel",
        "machinery_steel_rate",
        "t",
        "The engine-room steel and engine seats of the stern block mass (`stern_block_mass.machinery_steel`). \
         0 when the results have none.",
    ),
    (
        "deckhouse",
        "deckhouse_rate_per_m2",
        "m2",
        "Deckhouse deck area. 0: OVD has no deckhouse model yet (issue #779 gives the area); the \
         wheelhouse and accommodation lump stands for today's stern block.",
    ),
    (
        "gas_fuel_system",
        "gas_fuel_system_rate_per_kw",
        "kW",
        "The power the gas fuel system serves. 0: the gas fuel system allowances are not built yet \
         (issue #775 gives the figure).",
    ),
];

/// What an example item rate is (issue #773): no source was read for any of them.
pub const ITEM_RATE_EXAMPLE_BASIS: &str = "example rate: a round placeholder, not a market quote and not from a source. Replace it with your own.";

/// What a default item rate of 0 is: a file saved before the item rates (issue #773).
pub const ITEM_RATE_LEGACY_BASIS: &str = "0: the file was saved before this item rate existed, so the item is not priced, as before. Type a rate to price it.";

/// The item rates of an Option; `None` if one is missing or not a number.
fn item_rates_of(option: &DesignOption) -> Option<ItemRates> {
    let mut rates = ItemRates {
        genset_rate_per_kwe: 0.0,
        machinery_steel_rate: 0.0,
        deckhouse_rate_per_m2: 0.0,
        gas_fuel_system_rate_per_kw: 0.0,
    };
    for (name, set) in ITEM_RATE_INPUTS {
        set(&mut rates, option.input(name)?.value.as_f64()?);
    }
    Some(rates)
}

/// The `stern_items` of `cost_masses` (issue #773): what the item rates price that the stern block
/// gives. The deckhouse area and the gas fuel system power are 0 until their models exist. `v` is
/// returned as it is when it is null or the hull has no stern block (a barge of a tug Formation).
pub fn with_stern_items(
    mut v: Value,
    aft: Option<&ovd_core::aft_arrangement::AftArrangementResult>,
) -> Value {
    let Some(a) = aft else {
        return v;
    };
    if v.is_null() {
        return v;
    }
    v["stern_items"] = json!({
        "genset_kwe": ovd_core::aft_arrangement::installed_genset_kwe(a),
        "deckhouse_area_m2": 0.0,
        "gas_fuel_system_kw": 0.0,
    });
    v
}

/// The item quantities of an Option's results: the stern items of `cost_masses`, and the machinery
/// steel of the stern block mass when the results hold it (else 0).
fn item_quantities_of(results: &Value) -> ItemQuantities {
    let s = &results["cost_masses"]["stern_items"];
    let g = |k: &str| s[k].as_f64().unwrap_or(0.0);
    let steel = &results["stern_block_mass"]["machinery_steel"];
    let t = |k: &str| steel[k].as_f64().unwrap_or(0.0);
    ItemQuantities {
        genset_kwe: g("genset_kwe"),
        machinery_steel_t: t("engine_room_steel_t") + t("seats_t"),
        deckhouse_area_m2: g("deckhouse_area_m2"),
        gas_fuel_system_kw: g("gas_fuel_system_kw"),
    }
}

/// The item lines of a Formation with no tug, and what each prices (rate, quantity, unit, whether the
/// rate is the example's), for the results. With a tug: no lines.
fn items_of(
    option: &DesignOption,
    results: &Value,
    tug: bool,
) -> Option<(Vec<(&'static str, f64)>, Value)> {
    if tug {
        return Some((Vec::new(), Value::Null));
    }
    let rates = item_rates_of(option)?;
    let q = item_quantities_of(results);
    let lines = item_lines(&q, &rates);
    let quantity = [
        q.genset_kwe,
        q.machinery_steel_t,
        q.deckhouse_area_m2,
        q.gas_fuel_system_kw,
    ];
    let rate = [
        rates.genset_rate_per_kwe,
        rates.machinery_steel_rate,
        rates.deckhouse_rate_per_m2,
        rates.gas_fuel_system_rate_per_kw,
    ];
    let trace: Vec<Value> = ITEM_BASES
        .iter()
        .enumerate()
        .map(|(i, (item, input, unit, basis))| {
            let example = option
                .input(input)
                .is_some_and(|x| x.source == Source::Default);
            // A file saved before the item rates reads each as 0 (marked default): not priced.
            let (status, rate_basis) = match (example, rate[i] == 0.0) {
                (true, true) => ("not priced", ITEM_RATE_LEGACY_BASIS),
                (true, false) => ("example rate", ITEM_RATE_EXAMPLE_BASIS),
                (false, _) => ("user rate", COST_RATES_BASIS_USER),
            };
            json!({
                "item": item,
                "quantity": quantity[i],
                "unit": unit,
                "rate": rate[i],
                "rate_input": input,
                "rate_unit": format!("per {unit}"),
                "rate_status": status,
                "rate_basis": rate_basis,
                "quantity_basis": basis,
            })
        })
        .collect();
    Some((lines.to_vec(), Value::Array(trace)))
}

/// The cargo tank rates of a tank barge, as `(input name, how to set it)`: the inputs of the
/// `integral_tanks` containment (issue #334). The yard margin and the other rates are the 14 above.
type SetLiquidRate = fn(&mut LiquidCostRates, f64);
pub const LIQUID_RATE_INPUTS: [(&str, SetLiquidRate); 5] = [
    ("tank_coating_rate", |r, v| r.tank_coating_rate = v),
    ("cargo_pump_lump", |r, v| r.cargo_pump_lump = v),
    ("cargo_piping_lump", |r, v| r.cargo_piping_lump = v),
    ("cargo_heating_rate_per_kw", |r, v| {
        r.heating_rate_per_kw = v
    }),
    ("vents_gauging_lump", |r, v| r.vents_gauging_lump = v),
];

/// The cargo tank rates of an Option; `None` if one is missing or not a number.
fn liquid_rates_of(option: &DesignOption) -> Option<LiquidCostRates> {
    let mut rates = LiquidCostRates {
        tank_coating_rate: 0.0,
        cargo_pump_lump: 0.0,
        cargo_piping_lump: 0.0,
        heating_rate_per_kw: 0.0,
        vents_gauging_lump: 0.0,
    };
    for (name, set) in LIQUID_RATE_INPUTS {
        set(&mut rates, option.input(name)?.value.as_f64()?);
    }
    Some(rates)
}

/// The `cost_masses` of a tank barge: what its cargo tank items are priced on.
pub fn liquid_masses(l: &LiquidBulkResult) -> Value {
    let q = liquid_cost::quantities(l);
    json!({
        "liquid": {
            "hull_steel_t": q.hull_steel_t,
            "coating_area_m2": q.coating_area_m2,
            "cargo_tank_count": q.cargo_tank_count,
            "heating_kw": q.heating_kw,
        }
    })
}

fn liquid_quantities_of(m: &Value) -> Option<CostQuantities> {
    let l = m.get("liquid")?;
    Some(CostQuantities {
        hull_steel_t: l["hull_steel_t"].as_f64()?,
        coating_area_m2: l["coating_area_m2"].as_f64()?,
        cargo_tank_count: u32::try_from(l["cargo_tank_count"].as_u64()?).ok()?,
        heating_kw: l["heating_kw"].as_f64()?,
    })
}

/// The masses and which of them come from the design, as `simulate` reports them.
fn masses_json(m: &BargeCostInputs, from_design: &[&str]) -> Value {
    let all = [
        ("hull_steel_t", m.hull_steel_t),
        ("wing_plate_t", m.wing_plate_t),
        ("weather_shield_t", m.weather_shield_t),
        ("tank_shell_t", m.tank_shell_t),
        ("tank_supports_t", m.tank_supports_t),
        ("insulation_t", m.insulation_t),
        ("piping_machinery_t", m.piping_machinery_t),
        ("outfitting_t", m.outfitting_t),
        ("tug_hull_steel_t", m.tug_hull_steel_t),
        ("battery_kwh", m.battery_kwh),
        ("propulsion_kw", m.propulsion_kw),
    ];
    let core_default: Vec<&str> = all
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| !from_design.contains(n))
        .collect();
    json!({
        "masses": all.iter().map(|(n, v)| ((*n).to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        "from_design": from_design,
        "core_default": core_default,
    })
}

/// The masses of a self-propelled gas Option with its own powering (issue #353): the weight
/// breakdown, and its own battery energy and Installed Power in place of a tug's. The tug's hull
/// steel keeps the core's default.
pub fn self_propelled_gas_masses(
    w: &WeightBreakdown,
    battery_kwh: f64,
    propulsion_kw: f64,
) -> Value {
    let mut v = gas_masses(w);
    v["masses"]["battery_kwh"] = json!(battery_kwh);
    v["masses"]["propulsion_kw"] = json!(propulsion_kw);
    for name in ["battery_kwh", "propulsion_kw"] {
        if let Some(list) = v["from_design"].as_array_mut() {
            list.push(json!(name));
        }
        if let Some(list) = v["core_default"].as_array_mut() {
            list.retain(|n| n != name);
        }
    }
    v
}

/// The masses of a gas barge: its weight breakdown. The tug (hull steel, battery, propulsion) is a
/// separate vessel that the engine does not size, so those three keep the core's defaults.
pub fn gas_masses(w: &WeightBreakdown) -> Value {
    let m = BargeCostInputs {
        hull_steel_t: w.hull_steel_total,
        wing_plate_t: w.wing_plate,
        weather_shield_t: w.weather_shield,
        tank_shell_t: w.tank_shells,
        tank_supports_t: w.tank_internals_supports,
        insulation_t: w.insulation,
        piping_machinery_t: w.piping_machinery,
        outfitting_t: w.outfitting,
        ..BargeCostInputs::default()
    };
    let mut v = masses_json(
        &m,
        &[
            "hull_steel_t",
            "wing_plate_t",
            "weather_shield_t",
            "tank_shell_t",
            "tank_supports_t",
            "insulation_t",
            "piping_machinery_t",
            "outfitting_t",
        ],
    );
    v["steel"] = steel_json(w);
    v
}

/// The two hull steel figures of a gas barge (issue #401): the lightship uses the top-down one (a
/// rate per cubic metre of moulded volume); the budget prices the bottom-up one (hull plating and
/// structure, wing plate, weather shield). `delta_t` is top-down less bottom-up.
fn steel_json(w: &WeightBreakdown) -> Value {
    json!({
        "top_down_t": w.steel_top_down,
        "bottom_up_t": w.steel_bottom_up,
        "delta_t": w.steel_delta,
    })
}

/// Issue #722: the gas hull steel built up from its structural members, set beside the two steel
/// figures as `steel.panel_t`, and the panel table as `hull_panels`. Since issue #745 the table is the
/// plate model rolled up (`ovd_core::hull_plates::rolled_up`), and its total is the hull steel the
/// lightship and the cost take. Concept and preliminary design only; not a structural approval.
pub fn with_hull_panels(mut v: Value, m: Option<&ovd_core::hull_panels::HullPanelModel>) -> Value {
    let Some(m) = m else {
        return v;
    };
    if v.is_null() {
        return v;
    }
    v["steel"]["panel_t"] = json!(m.total_t);
    let members: Vec<Value> = m
        .members
        .iter()
        .map(|p| {
            json!({
                "member": p.member,
                "region": p.region,
                "x_m": [p.x_m.0, p.x_m.1],
                "y_m": [p.y_m.0, p.y_m.1],
                "z_m": [p.z_m.0, p.z_m.1],
                "count": p.count,
                "plate_area_m2": p.plate_area_m2,
                "plate_mm": p.plate_mm,
                "stiffener_direction": p.stiffener_direction.as_str(),
                "stiffener_spacing_mm": p.stiffener_spacing_mm,
                "stiffener": p.stiffener.as_ref().map(|s| json!({
                    "name": s.name,
                    "area_cm2": s.area_cm2(),
                })),
                "stiffener_length_m": p.stiffener_length_m,
                "primary_spacing_m": p.primary_spacing_m,
                "primary": p.primary,
                "plate_t": p.plate_t,
                "stiffeners_t": p.stiffeners_t,
                "total_t": p.total_t,
                "rule": p.rule,
                "rule_set": p.rule_set,
                "edition": p.edition,
                "paragraph": p.paragraph,
                "basis": p.basis,
                "note": p.note,
            })
        })
        .collect();
    let g = &m.hull_girder;
    v["hull_panels"] = json!({
        "scope": "Concept and preliminary design only (ADR 0011). Class minimum scantlings; not a structural approval.",
        "rule_edition": m.rule_edition,
        "members": members,
        "hull_girder": {
            "smr_cm2m": g.smr_cm2m,
            "sm_deck_cm2m": g.sm_deck_cm2m,
            "sm_bottom_cm2m": g.sm_bottom_cm2m,
            "deck_added_mm": g.deck_added_mm,
            "bottom_added_mm": g.bottom_added_mm,
            "neutral_axis_m": g.neutral_axis_m,
        },
        "net_t": m.net_t,
        "allowance_pct": m.allowance_pct,
        "allowance_t": m.allowance_t,
        "allowance_basis": "brackets, collars, welds and small items (an Estimate)",
        "total_t": m.total_t,
    });
    v
}

fn masses_of(m: &Value) -> Option<BargeCostInputs> {
    let g = |k: &str| m["masses"][k].as_f64();
    Some(BargeCostInputs {
        hull_steel_t: g("hull_steel_t")?,
        wing_plate_t: g("wing_plate_t")?,
        weather_shield_t: g("weather_shield_t")?,
        tank_shell_t: g("tank_shell_t")?,
        tank_supports_t: g("tank_supports_t")?,
        insulation_t: g("insulation_t")?,
        piping_machinery_t: g("piping_machinery_t")?,
        outfitting_t: g("outfitting_t")?,
        tug_hull_steel_t: g("tug_hull_steel_t")?,
        battery_kwh: g("battery_kwh")?,
        propulsion_kw: g("propulsion_kw")?,
    })
}

fn money(i: &ovd_core::pyint::PyInt) -> Value {
    serde_json::from_str(i.as_str()).expect("a PyInt is valid JSON integer text")
}

fn rows(list: &[(&str, ovd_core::pyint::PyInt)]) -> Value {
    Value::Array(
        list.iter()
            .map(|(item, amount)| json!({ "item": item, "amount": money(amount) }))
            .collect(),
    )
}

/// Which rates the budget uses (issue #404): `example` while every rate is the Defaults Profile's
/// (the example file's), `user` once any rate or the currency is typed: the example rates are in
/// USD, so they are not the rates of another currency.
fn rates_kind(option: &DesignOption) -> &'static str {
    let typed = RATE_INPUTS
        .iter()
        .map(|(name, _)| name)
        .chain(LIQUID_RATE_INPUTS.iter().map(|(name, _)| name))
        .chain(ITEM_RATE_INPUTS.iter().map(|(name, _)| name))
        .chain(["currency"].iter())
        .any(|name| {
            option
                .input(name)
                .is_some_and(|i| i.source != Source::Default)
        });
    if typed { "user" } else { "example" }
}

fn result_json(
    r: &BargeCostResult,
    masses: &Value,
    tug: bool,
    rates: &str,
    price_base_date: Option<&str>,
) -> Value {
    // With no tug its hull steel is not costed, so it is not listed (issue #403).
    let mut masses = masses.clone();
    if !tug {
        if let Some(m) = masses["masses"].as_object_mut() {
            m.remove("tug_hull_steel_t");
        }
        if let Some(list) = masses["core_default"].as_array_mut() {
            list.retain(|n| n != "tug_hull_steel_t");
        }
    }
    json!({
        "currency": r.rates.currency,
        "rates": rates,
        "price_base_date": price_base_date,
        "basis": format!(
            "{} {}",
            if rates == "user" { COST_RATES_BASIS_USER } else { COST_RATES_BASIS },
            cost_prices_basis(price_base_date)
        ),
        "barge": rows(&r.barge_breakdown),
        "tug": rows(&r.tug_breakdown),
        "barge_subtotal": money(&r.barge_subtotal),
        "tug_subtotal": money(&r.tug_subtotal),
        "yard_margin_fraction": r.rates.yard_margin_fraction,
        "yard_margin": money(&r.yard_margin),
        "delivery": r.delivery,
        "unit_total": money(&r.unit_total),
        "checks": crate::eval::checks_json(&r.checks),
        "masses": masses["masses"],
        "from_design": masses["from_design"],
        "core_default": masses["core_default"],
        "steel": masses["steel"],
        "tank_internals": masses.get("tank_internals").cloned().unwrap_or(Value::Null),
    })
}

/// The `cost` of an Option: the budget, or `null` when the results have no masses (a Cargo that is not
/// gas or liquid bulk, or a gas hull that holds no arrangement) or the figures cannot be rounded. One
/// function for both containments: a tank barge's cargo tank items, a gas barge's Type C items (issue #334).
pub fn cost_of(option: &DesignOption, results: &Value) -> Value {
    let masses = &results["cost_masses"];
    if masses.get("liquid").is_some() {
        return liquid_cost_of(option, results);
    }
    let (Some(mut inputs), Some(rates)) = (masses_of(masses), rates_of(option)) else {
        return Value::Null;
    };
    // Issue #402: part of the tank structure surplus is stiffeners and ring frames in tank steel, priced
    // at the tank shell rate; the rest (saddles) stays at the supports rate.
    let fraction = option
        .input("tank_internals_fraction")
        .and_then(|i| i.value.as_f64())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let surplus = inputs.tank_supports_t;
    let internals = surplus * fraction;
    inputs.tank_shell_t += internals;
    inputs.tank_supports_t = surplus - internals;
    let mut masses = masses.clone();
    masses["masses"]["tank_shell_t"] = json!(inputs.tank_shell_t);
    masses["masses"]["tank_supports_t"] = json!(inputs.tank_supports_t);
    masses["tank_internals"] = json!({
        "fraction": fraction,
        "mass_t": internals,
        "status": "estimate",
        "source": null,
        "basis": TANK_INTERNALS_BASIS,
    });
    let masses = &masses;
    // Only a Formation with a tug costs one (issue #403): a self-propelled vessel has none.
    let tug = has_tug(option);
    let Some((lines, items)) = items_of(option, results, tug) else {
        return Value::Null;
    };
    match compute_with_items(&inputs, &rates, tug, &lines) {
        Ok(r) => {
            let date = option
                .input("price_base_date")
                .and_then(|i| i.value.as_str());
            with_items(
                result_json(&r, masses, tug, rates_kind(option), date),
                items,
            )
        }
        Err(_) => Value::Null,
    }
}

/// The budget of a tank barge. The tug (hull steel, battery, propulsion) keeps the core's defaults, as
/// for a gas barge; a self-propelled hull has its own battery energy and Installed Power.
fn liquid_cost_of(option: &DesignOption, results: &Value) -> Value {
    let masses = &results["cost_masses"];
    let (Some(q), Some(liquid_rates), Some(rates)) = (
        liquid_quantities_of(masses),
        liquid_rates_of(option),
        rates_of(option),
    ) else {
        return Value::Null;
    };
    let tug = has_tug(option);
    let mut inputs = BargeCostInputs::default();
    let mut from_design: Vec<&str> = Vec::new();
    if !tug {
        // The Installed Power the engine sized, and the battery it has (none: no battery).
        let power = &results["powering"];
        if let Some(kw) = power["installed_power_kw"].as_f64() {
            inputs.propulsion_kw = kw;
            inputs.battery_kwh = power["battery_energy_kwh"].as_f64().unwrap_or(0.0);
            from_design.extend(["battery_kwh", "propulsion_kw"]);
        }
    }
    let Some((lines, items)) = items_of(option, results, tug) else {
        return Value::Null;
    };
    match liquid_cost::compute_with_items(&q, &liquid_rates, &inputs, &rates, tug, &lines) {
        Ok(r) => {
            let date = option
                .input("price_base_date")
                .and_then(|i| i.value.as_str());
            let mut m = liquid_masses_json(&q, &inputs, &from_design);
            m["steel"] = Value::Null;
            with_items(result_json(&r, &m, tug, rates_kind(option), date), items)
        }
        Err(_) => Value::Null,
    }
}

/// The quantities and the tug and powering figures of a tank barge, as `result_json` reads them.
fn liquid_masses_json(q: &CostQuantities, m: &BargeCostInputs, from_design: &[&str]) -> Value {
    let all = [
        ("hull_steel_t", q.hull_steel_t),
        ("coating_area_m2", q.coating_area_m2),
        ("cargo_tank_count", f64::from(q.cargo_tank_count)),
        ("heating_kw", q.heating_kw),
        ("tug_hull_steel_t", m.tug_hull_steel_t),
        ("battery_kwh", m.battery_kwh),
        ("propulsion_kw", m.propulsion_kw),
    ];
    let design: Vec<&str> = [
        "hull_steel_t",
        "coating_area_m2",
        "cargo_tank_count",
        "heating_kw",
    ]
    .into_iter()
    .chain(from_design.iter().copied())
    .collect();
    let core_default: Vec<&str> = all
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| !design.contains(n))
        .collect();
    json!({
        "masses": all.iter().map(|(n, v)| ((*n).to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        "from_design": design,
        "core_default": core_default,
    })
}

/// The budget with its `items` (issue #773): what each item line of a self-propelled hull prices, and
/// whether its rate is an example rate. A Formation with a tug has none, and no `items` key.
fn with_items(mut v: Value, items: Value) -> Value {
    if !items.is_null() {
        v["items"] = items;
    }
    v
}

/// Whether the Option's Formation has a tug: every one but `self_propelled`.
fn has_tug(option: &DesignOption) -> bool {
    let kind = option
        .input("formation")
        .and_then(|i| i.value.get("kind"))
        .and_then(Value::as_str);
    kind != Some("self_propelled")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Issue #773: the machinery steel line reads the stern block mass's machinery steel (the
    /// engine-room steel and the seats) when the results hold it, and 0 when they do not.
    /// Label: sourced - hand sum: 15.2560044 + 2.96562 = 18.2216244 t (the two figures are the
    /// machinery steel test's hand sums for two Cat 3512C, used here as plain inputs).
    #[test]
    fn the_machinery_steel_quantity_is_the_room_steel_and_the_seats_or_zero() {
        let mut r = json!({ "cost_masses": { "stern_items": { "genset_kwe": 3150.0 } } });
        let q = item_quantities_of(&r);
        assert_eq!(q.machinery_steel_t, 0.0);
        assert_eq!(q.genset_kwe, 3150.0);
        assert_eq!((q.deckhouse_area_m2, q.gas_fuel_system_kw), (0.0, 0.0));
        r["stern_block_mass"] = json!({
            "machinery_steel": { "engine_room_steel_t": 15.2560044, "seats_t": 2.96562 }
        });
        let q = item_quantities_of(&r);
        assert!((q.machinery_steel_t - 18.2216244).abs() < 1e-9);
    }
}
