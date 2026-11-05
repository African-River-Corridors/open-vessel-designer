---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 937
phase: outline
---

# #937 outline

Starts after #925 and #923 (items 1 and 2) are on `main`. Each phase is one commit or a few, and
ends green. The picks below follow the recommendations in `03-design.md`; a different answer at the
gate changes the phase it names.

## Phase 1. The dict-to-inputs step (pure, no run)

- **Result:** `api::inputs_of(params)` gives the typed inputs of one Option, or every error.
- **Files:** `crates/ovd-format/src/api.rs` (new), `crates/ovd-format/src/lib.rs` (`mod api;`),
  `locales/en/engine.ftl` (the refused-key messages).
- **Does:** the renames, `channel` flattened, `vessel` to `cargo_containment`, the Formation from
  `formation` / `self_propelled` / `atb`, `hull_form` from an old dict (design 3), numbers as text
  (design 4), `None` on a non-nullable input dropped (design 4), the refused keys (design 5), both
  an old name and its glossary name given.
- **Checks:** unit tests in `api.rs`, one per rename, per refused key (its message), per Formation
  flag, per `vessel` value; a test that the rename table is the inverse of `simulate_params_of` for
  every name it renames (read from a built Option, so the two tables cannot drift).

## Phase 2. Build the Option through the reader

- **Result:** `api::option_of(params, profile)` writes `.vdesign` text with every value `typed`,
  calls `fill_from_profile` once, and gives the `DesignOption` or every error.
- **Files:** `api.rs`.
- **Does:** unused-input warnings become errors (design 6); fill errors are joined, one per line.
- **Checks:**
  - a dict without length, beam and Cargo raises and names all three; no 76.5 m anywhere;
  - a dict with only the required inputs builds the same Option (inputs and sources) as
    `fill_as_new` of the same inputs, compared field by field;
  - an unknown key, an unused key and an out-of-range value each raise with the reader's message;
  - a missing Service Area raises #925's message.

## Phase 3. Run and the engine entry

- **Result:** `eval("api", {"fn": "simulate", "args": {"params": ..., "damage": bool}})` returns
  `run`'s results.
- **Files:** `api.rs`, `crates/ovd-format/src/eval.rs` (one arm, one `ENTRY_POINTS` name), and
  `run.rs` only if the maintainer picks design 7 (c).
- **Does:** `run_option_with(&option, &Library::built_in(), &[])` with #923's switch; maps
  `Failed` and `NotAvailable` (design 7).
- **Checks (Rust integration tests in `crates/ovd-format/tests/api_simulate.rs`):**
  - the 140 × 29 m coastal LNG reference design (`examples/gas-140x29-coastal-lng/`) as river-only
    and as coastal-and-river, and `crates/ovd-format/tests/fixtures/crude_integral_tanks_coastal.vdesign`:
    the dict of the file's typed inputs through `api`, damage on, gives the same cargo, outer
    diameter, draught and `rule_trace` as `run_option` on the file;
  - the same comparison on the whole results value and on `results_hash`;
  - the omit check: for every input an ES-TRIN row reads, the dict without it equals the dict with
    the profile default;
  - each run-failure class (design 7) has a test.

## Phase 4. Python wrapper and signature

- **Result:** `ovd.simulate.simulate(p, damage=False)` calls the `api` entry.
- **Files:** `src/ovd/simulate.py`, `src/ovd/_bridge.py` (only if the entry needs a mapping),
  `tests/data/public_api.json` (re-pinned by running the generator), `tests/test_native.py` if it
  lists entries.
- **Checks:** `pytest tests/test_public_api.py tests/test_native.py`; a test that
  `simulate(p, damage=True)` gives `damage_status` `run` and the default gives `not run` on a
  coastal gas dict.

## Phase 5. Re-pins and old callers

- **Result:** the Python suite passes on the new path; every moved pin says why.
- **Files:** `tests/test_simulate.py`, `tests/test_api_small_changes.py`, `tests/api_scenarios.py`,
  `tests/test_api_oracle.py` (`KNOWN_DIFFERENCES`), `tests/test_vdesign_edge_sweep.py`,
  `src/ovd/calibration.py` (design 10 (a): call the old entry by name).
- **Does:** drop `cb` and `ruleset` from the test dicts; re-pin by running the code; one comment per
  pin with its cause; `api_ce96e66.json` untouched (design 11 (a)).
- **Checks:** `pytest tests/`; `python -m ovd.calibration` (or its test) gives the same fitted
  factors as before.

## Phase 6. Docs, fragment, version

- **Result:** a reader can use the API from the docs alone.
- **Files:** `docs/python-api.md` (new), `website/mkdocs.yml` (nav), `README.md` (one link),
  `changes/ticket-937-simulate-one-path.md`, `pyproject.toml` and `Cargo.toml` (`0.4.0`).
- **Checks:** `python3 tools/changelog_collect.py --check`; the docs link check
  (`website/check_links.py`); `bash tools/gates.sh`.

## Out of this ticket

Unknown-key nearest-name hints (#938); the parity guard over every example (#939); the damage
stage with progress (#940); the typed API and `design_grid.sweep` (#941); removing the old entry; the
calibration re-fit (design 10).
