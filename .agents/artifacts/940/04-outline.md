---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 940
phase: outline
---

# #940 outline

Starts after #937 and #923 are on `main`. The picks follow `03-design.md`.

## Phase 1. Engine function

- **Result:** `api::simulate_damage(args, progress)` runs the Option with damage and calls the
  callback per case; `Stop` gives `damage_status` `not run`.
- **Files:** `crates/ovd-format/src/api.rs`, `crates/ovd-format/src/eval.rs` (the JSON-seam name).
- **Checks:** Rust tests in `crates/ovd-format/tests/api_damage.rs`: the callback sees `done`
  reach `total` on the coastal example; `Stop` after 3 calls gives `not run`; the JSON-seam call
  equals `simulate` with `damage: true` on the whole value.

## Phase 2. Binding

- **Result:** `_native.simulate_damage(case_json, progress, cancel)` releases the GIL, runs the
  engine on a scoped thread whose callback only counts, and keeps every Python call (`progress`,
  `cancel`, `check_signals`) on the calling thread. It stops on cancel, an exception or a signal,
  and re-raises after.
- **Files:** `crates/ovd-py/src/lib.rs` (and `eval` wrapped in `py.detach` if decision 4a).
- **Checks:** `cargo clippy -p ovd-py`; the Python tests of phase 3 (the binding has no Rust tests
  of its own, as today).

## Phase 3. Python API and tests

- **Result:** `ovd.simulate.simulate_damage` works as `03-design.md` section 1 says.
- **Files:** `src/ovd/_bridge.py`, `src/ovd/simulate.py`, `tests/test_damage_stages.py` (new),
  `tests/data/public_api.json` (re-pinned by running the generator).
- **Checks:** the seven tests of design section 6 (test 7: Ctrl-C through
  `_thread.interrupt_main()`); `pytest tests/test_public_api.py`.

## Phase 4. Docs and gates

- **Files:** `docs/python-api.md` ("Two stages"), `changes/ticket-940-damage-stage.md`.
- **Checks:** `python3 tools/changelog_collect.py --check`; `bash tools/gates.sh`.

## Out of this ticket

A handle that keeps the first stage between calls; progress on `simulate` (decision 1); a
`threading.Event` type for `cancel`.
