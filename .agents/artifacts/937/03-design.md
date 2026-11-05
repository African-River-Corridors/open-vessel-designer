---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 937
phase: design
---

# #937 design: one path for the Python `simulate`

The goal, in the maintainer's words: "the ui and the api should just be different ways to give the
same inputs and receive similar outputs". The issue settles most of the shape. This file settles
the rest and marks each choice the maintainer must make. Findings are cited as `R[n]` (questions in
`02-research.md`).

## The shape (settled by the issue)

```
dict ──translate──▶ typed inputs ──.vdesign text──▶ fill_from_profile (once)
     ──errors?──▶ raise every message
     ──run_option_with(built-in library, no extra sets)──▶ run's results ──▶ dict
```

- New `crates/ovd-format/src/api.rs` with `simulate_design(args)`; one `pub mod`/`mod` line in
  `lib.rs`; one arm and one `ENTRY_POINTS` name in `eval.rs`.
- No change to `simulate.rs`, `run.rs`, `vdesign.rs`, `session.rs` (R1: `fill_from_profile` is
  `pub(crate)`).
- No `new_design` and no `stamp` (R4, R1).

## 1. Entry name

Options: (a) a new function on the `simulate` entry (`eval("simulate", {"fn": "design"})`);
(b) a new entry `api` with `fn: "simulate"`.
**Pick (b)**, entry `"api"`, function `"simulate"`. It keeps the old entry untouched, so the old
oracle scenarios keep their meaning, and #940 adds `"simulate_damage"` beside it.

## 2. The dict-to-inputs step

The step is a pure function `inputs_of(params) -> Result<Map, Vec<String>>` in `api.rs`. It
renames, it never fills. Every value it writes is marked `typed`.

| Dict key | Option input | Rule |
|---|---|---|
| glossary name | same name | passed as it is |
| `loa_m`, `depth_m`, `freeboard_min_m`, `eta_d`, `speed_max_kn`, `lightship_k_t_per_m3` | the glossary name (R8) | renamed |
| `channel: {depth_m, ukc_m, bottom_width_m, bank_slope, current_kn}` | the five `channel_*` inputs (R8) | flattened |
| `vessel` | `cargo_containment` | `gas_barge`→`type_c_tanks`, `box_barge`→`box_hold`, `hopper_barge`→`sloped_hopper_hold`, `tank_barge`→`integral_tanks` |
| `formation` (glossary object) | `formation` | passed as it is |
| `self_propelled: true` / `atb: true` / neither | `formation` `{kind: self_propelled}` / `{kind: atb}` / `{kind: pushed_barge}` | as the issue settles |
| an old name and its glossary name both given | — | error: "give one of `loa_m` and `length_overall_m`" |

`vessel` left out: no `cargo_containment` is written, and the profile picks it by cargo phase
(R16). For a dry-bulk Cargo this equals the old `box_barge`. For a gas or liquid Cargo the old
engine returned "cannot be carried"; the new path builds the right Containment. **Pick:** follow
the profile; it is the file path's answer.

## 3. Required inputs the old dict never gave

Required: `cargo`, `hull_form`, `formation`, `length_overall_m`, `beam_m` (R3), and, after #925,
`service_area`.

- **Cargo, length, beam:** the old dict gave them. Left out, they raise, named one by one.
- **Formation:** mapped from `self_propelled` / `atb` / neither (settled by the issue).
- **Service Area:** raises, with #925's one message. No mapping: the old meaning ("no Rule Sets")
  is the drift #925 removes.
- **Hull Form:** the old engine built a flat-bottomed hull with the raked bow of `bow_rake_of`
  when no `hull_form` was given (R13). That is the `raked_box` form.

**Decision for the maintainer:** a dict with no `hull_form`.
- (a) Map it to `raked_box`, as the old engine built. Same logic as "neither flag means a pushed
  barge", which the issue already accepts. Every old dict keeps running.
- (b) Raise `msg-input-required` for `hull_form`. Strict; every old caller must add one key.
- Recommendation: **(a), but only when the dict also uses an old name** (`loa_m`, `vessel`,
  `self_propelled` or `atb`). A dict in glossary names is a new caller; it must give `hull_form`,
  as a file must. This keeps old scripts running and makes no new default.

## 4. Values the file path reads differently

**Decision for the maintainer:** numbers written as text (`"80"`, `"8e1"`, Arabic digits).
The old engine reads them as Python `float()` does and the 0.1.0 oracle pins it (R14). The file
path refuses them (R5).
- (a) Keep the old reading in the dict-to-inputs step only: convert with `simulate.rs`'s
  `py_float` rules, then type the number. Text that is not a number raises the same `ValueError`.
- (b) Refuse, as the file does.
- Recommendation: **(a)**. It is how a value is encoded, not which value is used, so it adds no
  drift in results. `tests/test_api_small_changes.py:32-71` keeps passing unchanged.

**Decision for the maintainer:** `None` for an input that is not nullable (for example
`fill_ratio=None`, which `api_scenarios.py` and `test_simulate.py` send). In the old API `None`
meant "the default". The file path refuses null on a `NUM` input.
- (a) Treat `None` as "left out" for a non-nullable input. The four tri-state tank keys keep null
  ("use the rule"), as the issue settles.
- (b) Refuse.
- Recommendation: **(a)**. The old meaning and the new meaning of "absent" are the same thing here:
  the default.

## 5. Params with no Option input

Settled by the issue: `cb`, `midship_coefficient`, `ruleset`, and the six `crew_*` keys are
refused, each with its own message. The step checks these names before the unknown-key check, so
the caller gets the reason, not "unknown input".

| Key | Message (English) |
|---|---|
| `cb` | "`cb` is a result, not an input: the hull gives it (issue #448)" |
| `midship_coefficient` | "`midship_coefficient` is not an input: the hull gives its own midship area" |
| `ruleset` | "`ruleset` is replaced by `service_area` and `rule_sets`" |
| `crew_*` | "`{key}` is worked out by the run; it is not an input" |

Each message gets a Message Id (ADR 0015) in `locales/en/engine.ftl`.

`calibration.py` sends `"cb": None` (R15). See section 10.

## 6. Unused inputs

The file path **warns** and drops an input that does not apply (R2). The issue says the API
refuses one ("a refused input (unknown, not used for this Cargo, not a number) raises").

**Decision for the maintainer:** an input that does not apply to this Cargo, Containment,
Formation or `when` value.
- (a) Raise, as the issue says. `api.rs` reads the report's `warning_msgs` and turns each
  `msg-input-unused-*` into an error.
- (b) Warn, as the file and the app do, and return the warnings in the result's `warnings`.
- Recommendation: **(a)**. A script has no screen that shows a warning, and a silently ignored key
  is the drift #932 found. Note the cost: a caller who sends one dict to many hull types must trim
  it per type. `tests/test_vdesign_edge_sweep.py:80-90` sends a `channel` to every hull and is
  rewritten to send only what applies.

## 7. Errors

- **Fill errors** (unknown, missing, unused, out of range, not a number, a Cargo the Containment
  cannot carry): one `TypeError` (`EvalError::BadInput`), with every message, one per line, in the
  report's order. This is the issue's settled class.
- **The Cargo in the wrong Containment** (`msg-containment-cargo`, R6): today a result
  `{"error": ...}`; on the file path a fill error.

**Decision for the maintainer:** the Cargo in the wrong Containment.
- (a) Raise, as every other fill error. `simulate.wrong_vessel_for_cargo` joins
  `KNOWN_DIFFERENCES`.
- (b) Return `{"error": <the fill message>}` for this one message, so callers that test for
  `"error"` keep working.
- Recommendation: **(a)**. One rule for every input error. The old answer was an accident of the
  dict path.

- **Run failures.** `run_option_with` turns both a domain message and a raised error into
  `Failed(text)`; a raised one reads `"ValueError: ..."` (R7, `run.rs:798`).

**Decision for the maintainer:** how `api.rs` gives a run failure back.
- (a) `{"error": text}` for every `Failed`. A raised `ValueError` inside the engine stops being a
  Python exception. A caller that catches a raise from inside the run gets a dict instead. No
  test on `main` pins such a raise through `simulate`, so the break is silent for users.
- (b) Re-raise when the text starts with a known Python class (`ValueError: `, `KeyError: `,
  `TypeError: `, `ZeroDivisionError: `, `OverflowError: `); else `{"error": text}`.
- (c) Allow one small `pub(crate)` helper in `run.rs` that returns the inner `Outcome`, so
  `api.rs` keeps the class without parsing text. This breaks the issue's "run.rs not changed" line.
- Recommendation: **(c)** if the maintainer allows the one helper; else **(b)**, with a test per
  class.

`RunOutcome::NotAvailable(text)` gives `{"error": text}`.

## 8. Result shape

- The result is `run`'s `results` value, as `ovd run --json` stores it for one Option. Every key
  the old `simulate` returned stays; `rule_trace`, `cost`, `calibration`, `gross_tonnage`, `crew`,
  `rooms`, `stern_arrangement`, `spaces`, `gas_fuel`, `holding_time` and the convoy, lock and
  pusher blocks are added where the run gives them (R10).
- `damage_status` comes from #923.
- No `results_hash` key and no Option inputs block. The hash is a file concept; #939 checks parity
  of the built Option inside Rust, so the API does not need to return it.

## 9. Damage flag

`simulate(p, damage=False)`. Default off, as the issue settles. The flag is passed to the #923
engine switch. `damage=True` runs it in the same call. `public_api.json` gains the keyword
parameter `damage` with default `False`, re-pinned by running the code (the pin is taken from
`inspect.signature`).

## 10. Calibration

`src/ovd/calibration.py:791-803` fits the installed-power factor through `simulate`, with `cb: None`,
no Hull Form and no Service Area (R15). Under the new path it would raise (`cb`) and, once fixed,
its fitted factor would move, and that factor is a calibrated value the profile cites.

**Decision for the maintainer:** calibration in #937.
- (a) Calibration calls the old engine entry by name (`_bridge.call("simulate", "simulate", ...)`)
  until a follow-up moves it to the new path with its own re-fit and design-impact table.
- (b) Move it now: add `hull_form`, `service_area`, drop `cb`, re-fit, and report the moved
  factors in this PR.
- Recommendation: **(a)**. The re-fit is a calibration change with its own evidence; it should not
  ride in an API ticket. File the follow-up when #937 merges.

## 11. Oracle and re-pins

- `tests/test_simulate.py` and `tests/test_api_small_changes.py`: re-pin by running the code. Each
  re-pin comment names the cause (profile defaults, the run's stack, rooms, consumables or loading
  conditions; #932 rows 3, 5 to 13). Dicts that send `cb` or `ruleset` drop them.
- `tests/api_scenarios.py`: the `simulate.*` scenarios drop `cb` and `ruleset` and add
  `hull_form` where they use glossary names (see section 3).

**Decision for the maintainer:** `tests/data/api_ce96e66.json`. Its header says it was printed by
the frozen Python at `ce96e66` and must not be edited by hand (R20). The issue lists it to re-pin.
- (a) Leave the frozen file. Add `simulate.wrong_vessel_for_cargo` to `KNOWN_DIFFERENCES` with
  the reason. The new answers are pinned in `tests/test_simulate.py`.
- (b) Re-print it.
- Recommendation: **(a)**. The file is evidence of what 0.1.0 did; re-printing it from new code
  would erase that.

## 12. Old entry

Kept, as the maintainer decided on #932: the `"simulate"` engine entry and `simulate.rs` stay
until a follow-up removes them. The Python package exposes no alias to it. Calibration
(section 10) and `cargo_options()` still call it.

## 13. Docs and version

- **Docs:** a new page `docs/python-api.md`, linked from the site nav (`website/mkdocs.yml`) and
  from the README. It lists the inputs (generated from `INPUTS`, as the schema already gives them),
  the old names and their glossary names, the refused keys, the `damage` flag and the new result
  blocks.
- **Version:** `0.3.0` → `0.4.0` (a 0.x minor bump, as the issue settles). A `changes/` fragment with
  the moved results.

## Dependencies

- **#925 (not merged at `a63ca819`).** What depends on it: the "Service Area raises" rule in
  section 3, and the parity tests for a missing Service Area. Without #925 the reader fills
  `river-only` silently (R22). So #937 must merge after #925. If #925 is late, `api.rs` must not
  add its own check: that would be a second reading of the rule. Wait for it.
- **#923.** The `damage` flag and `damage_status` (section 9). #937 cannot pass its first
  acceptance test ("damage on") without it.

## What this changes for a caller

| Caller did | Before | After |
|---|---|---|
| left a key out | the engine before the input existed | the Defaults Profile value |
| gave no Service Area | no Rule Sets | an error (#925) |
| gave `cb` or `ruleset` | ignored / checked | an error with the reason |
| gave a key that does not apply | ignored | an error (if section 6 (a)) |
| a gas hull, coastal | no Rule Sets | the coastal stack and its rows |
| — | — | `rule_trace`, `cost`, `crew`, `rooms` and more in the result |
