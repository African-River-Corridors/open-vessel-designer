# Python API: `simulate`

OVD is for concept and preliminary design only, never for approval (ADR 0011).

`ovd.simulate.simulate(p, damage=False)` takes a dict of inputs and gives back the results of one Option. It uses the same path as the app and as `ovd fill --as-new` then `ovd run`: the inputs go through the Design reader once, an input left out takes the Defaults Profile value, and the Option is run with the built-in Rule Sets. The same inputs give the same results through the dict, through a `.vdesign` file and through the app.

```python
from ovd.simulate import simulate

result = simulate({
    "cargo": "LNG",
    "hull_form": "raked_box",
    "formation": {"kind": "self_propelled"},
    "length_overall_m": 140.0,
    "beam_m": 29.0,
    "cargo_containment": "type_c_tanks",
    "service_area": "coastal-and-river",
})
result["results"]["cargo_t"]
result["rule_trace"]          # every Rule checked, with its verdict
result["damage_status"]       # "not run": damage is off unless you ask for it
```

## What a dict gives

- **A key you give** is a typed input of the Option.
- **A key you leave out** takes the Defaults Profile value, by Cargo, by Formation and by Service Area. It is marked `default` in the Option, as in a file. Nothing else is invented: there is no starter length, beam or Cargo.
- **A required input you leave out** is an error. The required inputs are `cargo`, `hull_form`, `formation`, `length_overall_m`, `beam_m` and `service_area`. The Service Area has no default: give `river-only` or `coastal-and-river`. A dict in glossary names must give `formation` and `hull_form`, as a file must; a dict that uses an old name need not (see Names).
- **The Option is built once.** The Rule Sets, the Rules of the stack and the Spaces come from the Option, as `ovd run` takes them.

## Names

Use the glossary names of the inputs (the table at the end), as a file does. The names of the old dict still work, so an old script keeps running:

| Old key | Input |
|---|---|
| `loa_m` | `length_overall_m` |
| `depth_m` | `moulded_depth_m` |
| `freeboard_min_m` | `min_freeboard_m` |
| `eta_d` | `propulsive_efficiency` |
| `speed_max_kn` | `max_speed_kn` |
| `lightship_k_t_per_m3` | `lightship_factor_t_per_m3` |
| `channel: {depth_m, ukc_m, bottom_width_m, bank_slope, current_kn}` | `channel_depth_m`, `channel_under_keel_clearance_m`, `channel_bottom_width_m`, `channel_bank_slope`, `channel_current_kn` |
| `vessel: "gas_barge"` | `cargo_containment: "type_c_tanks"` |
| `vessel: "box_barge"` | `cargo_containment: "box_hold"` |
| `vessel: "hopper_barge"` | `cargo_containment: "sloped_hopper_hold"` |
| `vessel: "tank_barge"` | `cargo_containment: "integral_tanks"` |
| `self_propelled: true` | `formation: {"kind": "self_propelled"}` |
| `atb: true` | `formation: {"kind": "atb"}` |
| neither flag (an old dict only) | `formation: {"kind": "pushed_barge"}` |

The nested `channel` object is flattened to the five channel inputs. Give one name for an input, not both: `loa_m` with `length_overall_m` is an error.

A dict that uses an old name (`loa_m`, `vessel`, `self_propelled` or `atb`) and gives no `hull_form` is built with `raked_box`, the hull the old engine made. In the same way, an old dict with neither `formation` nor a flag is a pushed barge, as the old engine took it. A dict in glossary names must give `formation` and `hull_form`: left out, the error is the one a file gets, `missing required input`.

Two readings of the old API are kept: a number written as text (`"80"`, `"8e1"`) is read as Python `float()` reads it, and `None` for an input that has no `null` value of its own means "left out". The four tank inputs that take `null` for "use the rule" (`tank_wall_thickness_mm`, `tank_dome_height_m`, `tank_dome_diameter_m`, `tank_design_pressure_mpa`) keep it. To fix one, give the number.

## Keys that are not inputs

Each is refused, with its reason:

| Key | Why |
|---|---|
| `cb` | A result of the hull, not an input (issue #448). |
| `midship_coefficient` | The hull gives its own midship area. |
| `ruleset` | Replaced by `service_area` and `rule_sets`. |
| `crew_berths`, `crew_officers`, `crew_seafarers`, `crew_gt`, `crew_reaches_sea`, `crew_mode_b` | The run works them out. |

## Errors

- **A refused input** raises `TypeError`, with every message, one per line, in the order of the Design reader: an unknown input, a required input left out (named one by one), an input out of range, a value that is not a number, a Cargo the Cargo Containment cannot carry, and an input that is not used for this Cargo, Containment, Formation or `when` value (the file path only warns about that last one; the API raises, because a script has no screen for a warning).
- **A number written as text that is not a number** raises `ValueError`, as Python `float()` does.
- **A run that fails** returns `{"error": "..."}`. A Python exception raised inside the run keeps its class.

## Damage

The `damage` argument is `False` by default.

`simulate(p)` does not run the damage stability cases, and gives `damage_status: "not run"`. This keeps sweeps fast. `simulate(p, damage=True)` runs them in the same call, as `ovd run` does, and gives `damage_status: "run"`.

## Two stages

Damage stability is the slow part of a run, so it is a second stage. `simulate(p)` is the first stage (`damage_status` is `"not run"`). `simulate_damage(p, progress=None, cancel=None)` runs the damage cases too:

```python
from ovd.simulate import simulate, simulate_damage

def show(done, total):
    print(f"{done} of {total} damage cases")

result = simulate_damage(p, progress=show)
result["damage_status"]      # "run"
```

- **Same answer.** `simulate_damage(p)` equals `simulate(p, damage=True)`, on the whole value, and equals what `ovd run` gives. There is one build path.
- **Stateless.** The call builds the Option again, so the part that `simulate(p)` does runs again (about 45 % of a coastal run). Keep the first stage's result if you have it; do not expect the call to reuse it.
- **`progress(done, total)`** is called as cases finish: first with `done` 0, last with `done == total`. `total` is the same on every call and `done` never falls. A hull with no damage Rule Set gives `damage_status` `"not applicable"` and never calls it.
- **`cancel()`** returning true stops the run. The call then *returns* the results of a run without damage, `damage_status` `"not run"`, equal to `simulate(p)`. No new case starts; cases under way finish. To get those results the core runs the Option again without damage before the call returns (about 45 % of a coastal run), so a cancel does not return at once, and Ctrl-C is not seen during that rerun. To cancel from a `threading.Event`, pass `event.is_set`.
- **Where the callables run.** The core's worker threads never call Python. `progress` and `cancel` run on the thread that called `simulate_damage`, one at a time. Keep them short: the run waits for them. They run only when the count of finished damage cases changes (no timer), so `cancel()` and Ctrl-C are not seen during the first part of the run, which works out no damage, nor inside one long case.
- **Errors and Ctrl-C.** An exception raised in `progress` or `cancel` stops the run and is raised from `simulate_damage`. Ctrl-C raises `KeyboardInterrupt` the same way, if `simulate_damage` was called from the main thread. A plain `simulate(p)` has no callable, so Ctrl-C waits for it to return.
- **The GIL.** The compiled core lets go of the GIL while it calculates, in `simulate`, `simulate_damage` and every other call. Other Python threads run during a call, so a sweep can use threads.

## Result

The result is what `ovd run --json` stores for one Option. These keys come from `run`, beside the keys of the old `simulate` result (`results`, `dims`, `hydrostatics`, `stability`, `checks`, `arrangement` and the rest): `rule_trace`, `cost`, `calibration`, `gross_tonnage`, `crew`, `rooms`, `stern_arrangement`, `spaces`, `gas_fuel`, `holding_time`, `damage_status` and `damage_cases`, and the convoy, lock and pusher blocks for a Formation that has them. A key is there when the run gives it for the Option. The result holds no results hash and no Option inputs: the hash belongs to a saved file.

## Inputs

The inputs are those of `ovd schema`. Run `ovd schema` for the units, the choices and the Defaults Profile value of each. An input applies only for some Cargos, Cargo Containments or Formations; an input that does not apply to the dict is an error.

| Input | Name in words | Kind | Required |
|---|---|---|---|
| `cargo` | Cargo | cargo | yes |
| `hull_form` | Hull Form | choice | yes |
| `formation` | Formation | formation | yes |
| `length_overall_m` | Length overall | number | yes |
| `beam_m` | Beam | number | yes |
| `moulded_depth_m` | Moulded depth | number |  |
| `vessel_scheme` | Class Scheme | vessel_scheme |  |
| `vessel_class` | Vessel Class | vessel_class |  |
| `cargo_containment` | Cargo Containment | choice |  |
| `bow_rake_length_m` | Bow rake length | number |  |
| `bow_rake_rise_m` | Bow rake rise | number |  |
| `bilge_radius_m` | Bilge radius | number |  |
| `sheer_aft_m` | Sheer at the aft end | number |  |
| `sheer_fwd_m` | Sheer at the forward end | number |  |
| `camber_m` | Deck camber | number |  |
| `deadrise_deg` | Deadrise | number |  |
| `keel_half_width_m` | Keel half-width | number |  |
| `chine_half_breadth_m` | Chine half-breadth | number |  |
| `chine_height_m` | Chine height above base | number |  |
| `shipd_parameters` | ShipD parameters | parameter_set |  |
| `hull_offsets` | Offsets table | offsets_table |  |
| `frame_spacing_m` | Frame spacing (web frames and floors) | number |  |
| `longitudinal_spacing_mm` | Longitudinal spacing | number |  |
| `stock_plate_width_m` | Stock plate width | number |  |
| `stock_plate_length_m` | Stock plate length | number |  |
| `thickness_step_mm` | Plate thickness step | number |  |
| `steel_grade` | Hull steel grade | choice |  |
| `structural_wing_deck` | Wing deck in the hull girder | bool |  |
| `min_freeboard_m` | Minimum freeboard (safety clearance) | number |  |
| `bow_rake_length_to_depth_ratio` | Bow rake length over depth | number |  |
| `bow_rake_max_length_m` | Longest bow rake from depth | number |  |
| `gross_tonnage` | Gross tonnage | number |  |
| `pushed_stern_void_m` | Stern void of a pushed barge | number |  |
| `lightship_factor_t_per_m3` | Lightship factor | number |  |
| `lpg_carriage_condition` | LPG carriage condition | choice |  |
| `lpg_loading_limit_basis` | LPG density and loading limit basis | choice |  |
| `lpg_loading_temperature_c` | LPG loading temperature | number |  |
| `fill_ratio` | Fill ratio | number |  |
| `max_tank_od_m` | Maximum tank outer diameter | number |  |
| `tank_design_pressure_mpa` | Tank design pressure (MARVS) | number |  |
| `tank_design_temperature_c` | Tank design temperature | number |  |
| `tank_material` | Tank material | choice |  |
| `tank_head_type` | Tank head type | choice |  |
| `tank_liquid_pressure` | Tank liquid pressure | choice |  |
| `tank_allowable_stress_basis` | Tank allowable stress basis | choice |  |
| `tank_allowable_stress_mpa` | Tank allowable stress (override) | number |  |
| `tank_insulation_type` | Tank insulation type | choice |  |
| `tank_insulation_thickness_mm` | Tank insulation thickness | number |  |
| `tank_insulation_conductivity_w_per_mk` | Tank insulation conductivity | number |  |
| `tank_insulation_density_kg_m3` | Tank insulation density | number |  |
| `holding_profile` | Holding-time profile | operation_profile |  |
| `holding_preset` | Holding-time preset | choice |  |
| `holding_adn_min_use_kg_h` | Minimum guaranteed gas use (ADN) | number |  |
| `holding_igf_domestic_use_kg_h` | Domestic gas load (IGF) | number |  |
| `holding_adn_journey_days` | Journey duration (ADN) | number |  |
| `holding_start_pressure_mpa` | Holding-time start pressure (cargo tanks) | number |  |
| `holding_fuel_service_pressure_mpa` | Normal service pressure (fuel tanks) | number |  |
| `holding_fill_ratio` | Holding-time fill (override) | number |  |
| `tank_structure_factor` | Tank structure factor | number |  |
| `tanks_across` | Tanks across (override) | number |  |
| `tanks_in_line` | Tanks in line | number |  |
| `tank_support_height_m` | Tank support height | number |  |
| `tank_dome_height_m` | Tank dome height (override) | number |  |
| `tank_dome_diameter_m` | Tank dome diameter (override) | number |  |
| `wing_tank_gap_m` | Wing tank gap | number |  |
| `wing_tank_height_ratio` | Wing tank height over tank diameter | number |  |
| `min_side_space_m` | Minimum side space | number |  |
| `weather_shield_thickness_mm` | Weather shield plate thickness | number |  |
| `weather_shield_clearance_m` | Weather shield clearance over the tanks | number |  |
| `tank_forming_allowance_pct` | Tank plate forming allowance | number |  |
| `tank_dome_wall_thickness_mm` | Tank dome wall thickness | number |  |
| `aft_end_space_m` | Aft end-bulkhead space | number |  |
| `fwd_end_space_m` | Forward end-bulkhead space | number |  |
| `opening_setback_basis` | Opening setback measured from | choice |  |
| `tank_wall_thickness_mm` | Tank wall thickness (override) | number |  |
| `optimise_depth` | Optimise depth | bool |  |
| `hold_wing_void_width_m` | Wing void width | number |  |
| `hold_double_bottom_m` | Hold double bottom | number |  |
| `hopper_slope_h_per_v` | Hopper slope (horizontal over vertical) | number |  |
| `hatch_coaming_height_m` | Hatch coaming height | number |  |
| `hold_covering` | Hold covering | choice |  |
| `n_holds` | Number of holds | number |  |
| `bulk_end_bulkhead_m` | End bulkhead allowance | number |  |
| `hold_fill_ratio` | Hold fill ratio | number |  |
| `inner_bottom_plate_mm` | Inner bottom plate thickness | number |  |
| `hopper_plate_mm` | Hopper plate thickness | number |  |
| `hold_bulkhead_plate_mm` | Hold bulkhead plate thickness | number |  |
| `hatch_coaming_plate_mm` | Hatch coaming plate thickness | number |  |
| `double_side_m` | Double side width | number |  |
| `double_bottom_m` | Double bottom height | number |  |
| `liquid_tanks_in_line` | Cargo tanks in line | number |  |
| `centreline_bulkhead` | Centreline bulkhead | bool |  |
| `cofferdam_m` | Cofferdam length | number |  |
| `pump_arrangement` | Pump arrangement | choice |  |
| `pump_room_length_m` | Pump room length | number |  |
| `heating_kw_per_m3` | Cargo heating | number |  |
| `liquid_fill_ratio` | Cargo tank fill ratio | number |  |
| `stern_form` | Stern form | choice |  |
| `stern_run_length_m` | Stern run length | number |  |
| `stern_transom_height_m` | Transom height above base | number |  |
| `tunnel_crown_height_m` | Tunnel crown height above base | number |  |
| `propulsor_type` | Propulsor type | choice |  |
| `propulsor_count` | Propulsor count | number |  |
| `propeller_diameter_m` | Propeller diameter | number |  |
| `propeller_diameter_basis` | Default propeller diameter sized to | choice |  |
| `propulsor_efficiency_factor` | Propulsor efficiency factor | number |  |
| `bollard_pull_factor` | Bollard pull factor | number |  |
| `power_source` | Power source | choice |  |
| `installed_power_kw` | Installed power | number |  |
| `design_speed_kn` | Design speed | number |  |
| `installed_power_margin` | Installed power margin | number |  |
| `bow_thruster` | Bow thruster | choice |  |
| `bow_thruster_power_kw` | Bow thruster power | number |  |
| `battery_energy_kwh` | Battery energy | number |  |
| `trip_length_km` | Trip length | number |  |
| `battery_unit_energy_kwh` | Battery energy per 40 ft unit | number |  |
| `wheelhouse_type` | Wheelhouse type | choice |  |
| `sightline_method` | Sightline method | choice |  |
| `stern_block_mass_included` | Stern block mass in the lightship | bool |  |
| `gas_combustion_unit` | Gas combustion unit | bool |  |
| `tug_included` | Tug included | bool |  |
| `tug_preset` | Tug preset | choice |  |
| `tug_length_m` | Tug length | number |  |
| `tug_beam_m` | Tug beam | number |  |
| `tug_draught_m` | Tug draught | number |  |
| `tug_stern_form` | Tug stern form | choice |  |
| `tug_stern_run_length_m` | Tug stern run length | number |  |
| `tug_stern_transom_height_m` | Tug transom height | number |  |
| `tug_tunnel_crown_height_m` | Tug tunnel crown height | number |  |
| `tug_propulsor_type` | Tug propulsor type | choice |  |
| `tug_propulsor_count` | Tug propulsor count | number |  |
| `tug_propeller_diameter_m` | Tug propeller diameter | number |  |
| `tug_power_source` | Tug power source | choice |  |
| `tug_installed_power_kw` | Tug installed power | number |  |
| `tug_design_speed_kn` | Formation design speed | number |  |
| `tug_installed_power_margin` | Tug installed power margin | number |  |
| `tug_bow_thruster` | Tug bow thruster | choice |  |
| `tug_bow_thruster_power_kw` | Tug bow thruster power | number |  |
| `tug_battery_energy_kwh` | Tug battery energy | number |  |
| `tug_battery_unit_energy_kwh` | Tug battery unit energy | number |  |
| `tug_wheelhouse_type` | Tug wheelhouse type | choice |  |
| `tug_sightline_method` | Tug sightline method | choice |  |
| `formation_resistance_allowance` | Formation resistance allowance | number |  |
| `notch_depth_m` | Notch depth | number |  |
| `notch_width_m` | Notch width | number |  |
| `propulsive_efficiency` | Propulsive efficiency | number |  |
| `resistance_method` | Resistance method | choice |  |
| `max_speed_kn` | Maximum speed | number |  |
| `thruster_count` | Thruster count (most) | number |  |
| `thruster_envelope_m` | Thruster envelope | number |  |
| `thruster_clearance_m` | Thruster clearance | number |  |
| `thruster_room_length_m` | Thruster room length | number |  |
| `battery_unit_count` | Battery units | number |  |
| `accommodation_unit_count` | Accommodation units | number |  |
| `unit_height_m` | Unit height | number |  |
| `unit_side_gap_m` | Unit side gap | number |  |
| `unit_row_gap_m` | Unit row gap | number |  |
| `unit_foundation_m` | Unit foundation height | number |  |
| `wheelhouse_length_m` | Wheelhouse length | number |  |
| `wheelhouse_width_m` | Wheelhouse width | number |  |
| `wheelhouse_height_m` | Wheelhouse height | number |  |
| `wheelhouse_gap_m` | Wheelhouse gap | number |  |
| `wheelhouse_eye_height_m` | Wheelhouse eye height above floor | number |  |
| `sightline_margin_m` | Sightline margin | number |  |
| `wheelhouse_max_lift_m` | Wheelhouse maximum lift | number |  |
| `air_draft_limit_m` | Air-draft limit | number |  |
| `bridge_clearance_m` | Bridge clearance | number |  |
| `draught_limit_m` | Draught limit | number |  |
| `lock_length_m` | Lock length | number |  |
| `lock_width_m` | Lock width | number |  |
| `downflooding_angle_deg` | Angle of down-flooding | number |  |
| `downflooding_openings` | Openings for down-flooding | openings_list |  |
| `channel_depth_m` | Channel depth | number |  |
| `channel_under_keel_clearance_m` | Under-keel clearance | number |  |
| `channel_bottom_width_m` | Channel bottom width | number |  |
| `channel_bank_slope` | Bank slope | number |  |
| `channel_current_kn` | Current | number |  |
| `water_density_t_per_m3` | Water density | number |  |
| `squat_safety_margin_m` | Squat safety margin | number |  |
| `loading_conditions` | Loading conditions | choice |  |
| `may_sail_without_ballast` | May sail without ballast | bool |  |
| `endurance_h` | Endurance (running hours) | number |  |
| `fixed_stores_t` | Fresh water and stores | number |  |
| `ballast_plan` | Ballast plan | choice |  |
| `ballast_departure_double_bottom_fill` | Ballast departure: double bottom fill | number |  |
| `ballast_departure_peak_fill` | Ballast departure: peak tanks fill | number |  |
| `ballast_departure_wing_fill` | Ballast departure: wing tanks fill | number |  |
| `ballast_arrival_double_bottom_fill` | Ballast arrival: double bottom fill | number |  |
| `ballast_arrival_peak_fill` | Ballast arrival: peak tanks fill | number |  |
| `ballast_arrival_wing_fill` | Ballast arrival: wing tanks fill | number |  |
| `hull_steel_rate` | Hull steel rate | number |  |
| `tank_shell_rate` | Tank shell rate | number |  |
| `tank_supports_rate` | Tank supports rate | number |  |
| `insulation_rate` | Insulation rate | number |  |
| `piping_rate` | Piping and machinery rate | number |  |
| `outfitting_rate` | Outfitting rate | number |  |
| `cargo_system_lump` | Cargo system | number |  |
| `paint_anodes_misc` | Paint, anodes and miscellaneous | number |  |
| `battery_rate_per_kwh` | Battery rate | number |  |
| `propulsion_rate_per_kw` | Propulsion rate | number |  |
| `genset_rate_per_kwe` | Genset rate | number |  |
| `machinery_steel_rate` | Machinery steel rate | number |  |
| `deckhouse_rate_per_m2` | Deckhouse rate | number |  |
| `gas_fuel_system_rate_per_kw` | Gas fuel system rate | number |  |
| `wheelhouse_accom_lump` | Wheelhouse and accommodation | number |  |
| `tug_outfitting_lump` | Tug outfitting | number |  |
| `yard_margin_fraction` | Yard margin | number |  |
| `delivery_lump` | Delivery | number |  |
| `tank_coating_rate` | Tank coating rate | number |  |
| `cargo_pump_lump` | Cargo pump | number |  |
| `cargo_piping_lump` | Cargo piping | number |  |
| `cargo_heating_rate_per_kw` | Cargo heating rate | number |  |
| `vents_gauging_lump` | Vents and gauging | number |  |
| `tank_internals_fraction` | Tank internals share of the structure surplus | number |  |
| `currency` | Currency | text |  |
| `price_base_date` | Price base date | text |  |
| `deckhouse_front_opening` | Entrance, air inlet or opening on the deckhouse front | bool |  |
| `deckhouse_openings` | Deckhouse openings | openings_list |  |
| `deckhouse_side_opening_distance_m` | Distance of the first side opening from the deckhouse front | number |  |
| `entrance_distance_m` | Distance of the nearest entrance or openable window from the cargo area | number |  |
| `engine_room_vent_inlet_distance_m` | Distance of the engine room ventilation inlet from the cargo area | number |  |
| `funnel_outlet_distance_m` | Distance of the funnel outlet from the cargo area | number |  |
| `vent_stub_m` | Height of the cargo vent exit above the dome top (river only) | number |  |
| `vent_mast_lowerable` | Cargo vent mast can be lowered for transit | bool |  |
| `service_area` | Service Area | choice | yes |
| `arrangement_style` | Arrangement Style | choice |  |
| `crew` | Crew | number |  |
| `manning_rule_set` | Manning Rule Set | choice |  |
| `navigation_hours_per_day` | Navigation hours per day | number |  |
| `unmanned_machinery_space` | Unmanned machinery space | bool |  |
| `slop_share` | Slop tank share | number |  |
| `rule_sets` | Rule Sets | rule_sets |  |

