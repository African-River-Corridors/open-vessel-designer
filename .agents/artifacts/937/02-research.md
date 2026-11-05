---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 937
phase: research
---

# #937 research findings

Read at `a63ca819` with git and grep. Nothing was built or run. Numbers in brackets are the
question numbers in `01-research-questions.md`.

## A. The build path

**[1] `fill_from_profile`.** `vdesign.rs:3974-3978`. It reads `.vdesign` text with
`Missing::FillProfile` and names the profile when it filled anything. It does not stamp a hash
(`fill_as`, `:3980-3989`, does: it calls `run::stamp`, which runs the Design once more). It is
`pub(crate)`, so a new `api.rs` in `ovd-format` can call it without a change to `vdesign.rs`. The
app calls it on every edit (`session.rs:1217`).

**[2] `read_option`.** `vdesign.rs:4564-4790`, in this order:
- `class_input::prefill` writes the length, beam and Formation of a named class
  (`vdesign.rs:4576-4581`, `class_input.rs:70`).
- An unknown key is an **error** `msg-unknown-input` (`:4582-4594`).
- For each schema input, in schema order: a given value that does not apply (wrong Cargo
  Containment, Formation, Cargo or `when`) is a **warning** `msg-input-unused-*`, and it is dropped
  (`:4626-4675`). A given value that applies goes through `check_value` (`:4676-4689`).
- A required input left out is an **error** `msg-input-required`, one per input, by name
  (`:4690-4700`).
- Under `FillProfile` the legacy branch is skipped (`:4704-4735`); the profile default is written,
  marked `default`, with a warning `msg-input-filled` (`:4736-4766`).

So the reader already names every missing required input, refuses unknown keys and fills by
Formation, table and Service Area. It does **not** refuse an unused input: it warns.

**[3] Required inputs.** `vdesign.rs:2117-2140`: `cargo`, `hull_form`, `formation`,
`length_overall_m`, `beam_m`. `moulded_depth_m` is not required: left out, it follows the
depth-to-beam rule (`:2141-2149`). `service_area` is `required: false` on `main`
(`vdesign.rs:3658`); its profile default is `river-only` (`builtin_profile.json:243-246`).

**[4] Why not `new_design`.** `vdesign.rs:3883-3912` types bauxite, `raked_box`, a pushed barge,
L 76.5, B 11.4, D 3.5, then calls `stamp`. A dict without a length would run at 76.5 m, and a typed
D would stop the depth rule. The ticket's warning holds.

**[5] Value checks.** `unmark` (`vdesign.rs:4528-4560`) reads `{value, source}` or a bare value as
typed. `check_value` (`:5171-5215`): a number must be a JSON number; null is allowed only where the
kind is `NUM_OR_NONE` (`:2111`). `fill_ratio` is `NUM` (not nullable). Text such as `"80"` is not a
number here. A `hull_form` that is valid but not built (`box`) is refused (`:5207-5215`,
`BUILT_HULL_FORMS` `:287`).

**[6] Cargo in the wrong Containment.** An **error** `msg-containment-cargo` on the file path
(`vdesign.rs:5623-5650`). The old `simulate` returns it as a result `{"error": "LNG cannot be carried
in a box-hold dry bulk barge."}` (`simulate.rs:1331-1338`; pinned in `tests/data/api_ce96e66.json`,
scenario `simulate.wrong_vessel_for_cargo`).

## B. The run path

**[7] `run_option_with`.** `run.rs:610-800`. It maps the Option to params
(`simulate_params_of`), picks the stack (`trace::stack_for`), runs `settle_rooms`, then adds the
convoy totals, lock checks, ATB drawing, pusher block, gas fuel, holding time, cost, rule trace,
calibration, gross tonnage, crew, rooms, stern arrangement and Spaces. Outcomes:
- a result with `"error"` becomes `RunOutcome::Failed(message)` (`run.rs:631-634`);
- a raised Python-class error becomes `Failed("{kind}: {message}")` (`run.rs:798`);
- a refusal becomes `Failed(message)` (`run.rs:799`).
So once through `run_option_with`, a domain message and a raised `ValueError` are both plain
text. `RunOutcome::NotAvailable` exists (`run.rs:36`) for a convoy that cannot be computed.

**[8] Param names.** `simulate_params_of` (`run.rs:94-345`) is the map from glossary names to
engine names. The renames: `length_overall_m`→`loa_m`, `moulded_depth_m`→`depth_m`,
`min_freeboard_m`→`freeboard_min_m`, `propulsive_efficiency`→`eta_d`,
`max_speed_kn`→`speed_max_kn`, `lightship_factor_t_per_m3`→`lightship_k_t_per_m3`, and
`channel_depth_m`, `channel_under_keel_clearance_m`, `channel_bottom_width_m`,
`channel_bank_slope`, `channel_current_kn` into a nested `channel` object (`run.rs:320-337`).
`cargo_containment` becomes `vessel` (`run.rs:298-305`). The Formation becomes `self_propelled` and
`atb` (`run.rs:314-318`). The four tri-state tank keys pass a stored null on (`run.rs:284-296`).
Every other name passes as it is.

**[9] Params `run` adds.** `settle_rooms` (`run.rs:893-935`) writes `crew_berths`,
`crew_officers`, `crew_seafarers`, `crew_gt`, `crew_reaches_sea` and `crew_mode_b` into the params
between passes on a deckhouse stern. They are not inputs.

**[10] New blocks.** `run` adds `rule_trace` (`run.rs:733-738`), `cost` (`:741`), `calibration`
(`:751`), and later `gross_tonnage`, `crew`, `rooms`, `stern_arrangement`, `spaces`, the convoy,
lock and pusher blocks, `gas_fuel` and `holding_time`. All but `spaces` are in `UNHASHED_KEYS`
(`run.rs:362-383`).

## C. The old dict API

**[11] Python today.** `simulate.py:41-43` sends the dict as `{"params": p}` to the `simulate` entry
and decodes the answer. `_bridge.call` raises the class the core names (`_bridge.py:151-186`). The
binding holds the GIL for the whole call; it does not release it (`ovd-py/src/lib.rs:23-32`).

**[12] Dispatch.** `eval` matches the entry name (`eval.rs:85-117`); `"simulate"` goes to
`simulate::run` (`eval.rs:103`, `simulate.rs:3625-3643`). `EvalError::BadInput` becomes a Python
`TypeError` (`ovd-py/src/lib.rs:29`). `ENTRY_POINTS` is a fixed list (`eval.rs:53-76`).

**[13] What the old engine assumes for a key left out.**

| Key left out | Old engine | Pointer |
|---|---|---|
| `vessel` | `box_barge` | `simulate.rs:1321-1325` |
| `hull_form` | flat bottom (`hull_shape_of`) with the raked bow of `bow_rake_of`: the `raked_box` shape | `simulate.rs:2316-2335`, `:2275-2290` |
| Formation | `self_propelled` false, `atb` false: a pushed barge | `simulate.rs:1342`, `:2349` |
| `loa_m`, `beam_m`, `depth_m` | required (`need`) | `simulate.rs:1339` |
| Service Area | not river-only; no Rule Sets | `simulate.rs:1209-1228` |
| `ruleset` | `IGC_2G`; any other value gives `{"error": "Unknown ruleset ..."}` | `simulate.rs:2803-2815` |
| `midship_coefficient` | the default; unused where a hull gives its midship area | `simulate.rs:420-433` |
| `cb` | not read | no `"cb"` read in `simulate.rs` |

**[14] Numbers as text.** `py_float` (`simulate.rs:68`) reads `"80"`, `" 80 "`, `"8e1"` and Arabic
digits as Python's `float()` does, and raises `ValueError` for `"0x10"`. Pinned by
`tests/test_api_small_changes.py:32-70`. The file path refuses text (finding [5]).

**[15] Other callers.** `src/ovd/calibration.py:791-803` fits the installed-power factor by calling
`simulate` with `"cb": None`, no `hull_form`, no Service Area. `tests/test_vdesign_edge_sweep.py:80-90`
maps Option inputs to old names. No other module under `src/ovd`, `tools` or `evaluation` calls it.

## D. Defaults and stacks

**[16] Profile rules.** By Formation (4): `may_sail_without_ballast`, `tug_stern_form`,
`tug_propulsor_type`, `formation_resistance_allowance`. By table (11), keyed on
`cargo_containment`, `cargo`, `lpg_carriage_condition`: `lpg_loading_temperature_c`, `fill_ratio`,
`tank_design_pressure_mpa`, `tank_design_temperature_c`, `tank_head_type`, `tank_material`,
`tank_insulation_type`, `tank_insulation_thickness_mm`, `tank_structure_factor`,
`tank_dome_height_m`, `tank_dome_diameter_m`. By Service Area (4): `slop_share`,
`arrangement_style`, `manning_rule_set`, `navigation_hours_per_day`. `cargo_containment` defaults
by cargo phase (dry bulk `box_hold`, gas `type_c_tanks`, liquid bulk `integral_tanks`).
`tank_wall_thickness_mm` defaults to null (`builtin_profile.json:866-868`).

**[17] Stacks.** `builtin_profile.json:248-260`: river-only gas has six sets, `es-trin-2025`
included; coastal-and-river gas has `igc-2g`, ICLL, SOLAS V/22, SOLAS II-2, MLC and the ABS sets.

**[18] Old stack.** `unstored_rules` (`simulate.rs:1209-1228`) gives Rule Sets only to a gas hull
whose Service Area is explicitly river-only, and leaves out `es-trin-2025`.

## E. Tests and pins

**[19] Callers in tests.**
- `tests/test_simulate.py` (11 calls) pins cargo on the CN-II gas dict, which passes `cb` and
  `ruleset`.
- `tests/test_api_small_changes.py` pins the `ruleset` error text (`:23-29`), numbers as text
  (`:32-62`) and the `ValueError` text (`:65-71`).
- `tests/api_scenarios.py:548-595` has 7 `simulate.*` scenarios; each passes `cb` or `ruleset`.
- `tests/test_vdesign_edge_sweep.py:228-260` runs every valid Option through `simulate`. On the
  new path each of those calls is a full `run` (`fill_from_profile`, the rule stack, the rooms
  loop, cost and trace; R7, R9, R10), not the old bare engine call, so the sweep costs more per
  Option. Damage off by default (#923) keeps the largest share out.

**[20] The oracle.** `api_ce96e66.json` says it was printed by the frozen Python at `ce96e66` and
is not to be edited by hand (`:2`). A moved scenario is listed in `KNOWN_DIFFERENCES`
(`test_api_oracle.py:31-100`) and skipped with its reason (`:201-202`). Six of the seven
`simulate.*` scenarios are listed already; `simulate.wrong_vessel_for_cargo` is still compared.

**[21] Signature.** `public_api.json:2396-2406`: `simulate(p)`, one positional-or-keyword param.

## F. Dependencies

**[22] #925.** Not merged at `a63ca819`, and no PR yet. The planned change makes `service_area` required (`true`), adds
`service_area_required()` and a check in the old `simulate`. Without #925, `read_option` fills a
missing Service Area with the profile's `river-only` and says nothing an API caller sees.

**[23] #923.** No PR and no branch yet. #937 needs its item 1 (the engine switch and
`damage_status`). Damage runs inside `simulate_once` today, after the tank fit (gas
`simulate.rs:3250-3410`; liquid `:1659-1720`), so skipping it should not move the sizing; #923's
byte-equal test is what proves it.

**[24] Docs.** There is no Python API page. The README has one worked example (`README.md:109-140`).
The docs site nav has no API entry (`website/mkdocs.yml:55-66`).

**[25] Version.** `0.3.0` in `pyproject.toml:3` and `Cargo.toml:8`. A merge to `main` that changes
the version publishes a numbered pre-release (`.github/workflows/app-build.yml:6-9`).
`ENGINE_VERSION` is `env!("CARGO_PKG_VERSION")` (`vdesign.rs:47`), and it is stamped into every
saved file. So a bump in this PR is also a release and a file-format stamp change.

**[26] Calibration path.** `calibration.py:791-803` fits the installed-power factor through the old
`simulate`. The fitted factor is then used by every Option through `run`'s path. Once `run` and the
old `simulate` give different powering for the same hull (#932), the factor was fitted on a path no
design uses.
