---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 940
phase: design
---

# #940 design: damage as a second stage in Python

Builds on #937 (the `api` entry; PR #949 holds its design) and #923 (the switch, the callback with
stop, `damage_status`). Findings are cited as `R[n]` (`02-research.md`).

## 1. Call shape (settled by the issue)

```python
simulate_damage(params, progress=None, cancel=None) -> dict
```

- Builds the Option as #937 does, runs it with damage on, returns the full results with
  `damage_status`.
- Stateless: the non-damage part runs again (about 45 % of a coastal run, #921).
- `progress(done, total)` is called as cases finish. `cancel()` returning true stops the run.
- A cancelled call **returns** the results with `damage_status: "not run"` (R8). It does not raise.

**Decision for the maintainer:** progress and cancel on `simulate` too.
- (a) Only `simulate_damage` takes them. `simulate(p, damage=True)` stays a plain call.
- (b) `simulate(p, damage=True, progress=..., cancel=...)` as well; `simulate_damage` is that call.
- Recommendation: **(a)**. Two stages, as the app has them; one place for the callback. The
  byte-equal test (section 6) proves the two paths agree.

## 2. Engine function (Rust)

In `crates/ovd-format/src/api.rs`, beside #937's `simulate`:

```rust
pub fn simulate_damage(args: &Value, progress: &dyn DamageProgress) -> Result<Value, EvalError>
```

`DamageProgress` is #923's callback trait (or `&(dyn Fn(usize, usize) -> Control + Sync)`), with
`Control::{Go, Stop}`. It must be `Sync`, because #921 calls it from worker threads (R6).

The JSON seam also gets `eval("api", {"fn": "simulate_damage", ...})` with no callback, so the
parity harness and the entry lists see the name. It equals `simulate` with `damage: true`.

## 3. The binding (`crates/ovd-py`)

A new `#[pyfunction] simulate_damage(py, case_json, progress, cancel) -> PyResult<String>`:

1. Parse the case with the GIL held.
2. `py.detach(|| ovd_format::api::simulate_damage(&case, &callback))` (R2). The closure holds only
   `Py<PyAny>` handles (`Ungil`).
3. The callback, per finished case:
   - bumps an `AtomicUsize` for `done`;
   - takes a `Mutex<()>` so only one thread speaks to Python at a time, and reads `done` inside it,
     so Python sees `done` rise and never fall (R6);
   - `Python::attach`: calls `progress(done, total)` if given; calls `cancel()` if given;
     calls `py.check_signals()`;
   - returns `Stop` if `cancel()` is true, if either callable raised, or if a signal is pending;
     else `Go`.
4. After the run, with the GIL held: if a callable raised or a signal is pending, raise that
   exception; else return the result text.

The last call is always `progress(total, total)` on a run that is not stopped, so a caller that
waits for `done == total` ends.

**Decision for the maintainer:** an exception raised inside `progress` or `cancel`.
- (a) Stop the run and re-raise it from `simulate_damage`.
- (b) Ignore it, warn once, and carry on.
- Recommendation: **(a)**. A broken callback is a bug in the caller's code; hiding it hides the
  bug. Ctrl-C (`KeyboardInterrupt`) takes the same road, so a long run can be stopped from the
  keyboard.

## 4. The GIL in the plain `eval`

Today `eval` holds the GIL for the whole run (R1). `simulate_damage` releases it (section 3).

**Decision for the maintainer:** release the GIL in the existing `eval` too.
- (a) Yes, in #940: wrap the run in `py.detach`. Python threads can then run sweeps in parallel,
  and nothing in the engine touches Python. One line, plus a test that two threads finish in less
  time than one after the other.
- (b) No; leave `eval` as it is, and do it in its own ticket.
- Recommendation: **(a)**. It is the same mechanism and the same review. Ctrl-C during a plain
  `simulate` still waits for the call to return (no callback there); say so in the docs.

## 5. Python side

- `src/ovd/_bridge.py`: `call_with_progress(fn, args, progress, cancel)` encodes the args as `call`
  does and calls `_native.simulate_damage`.
- `src/ovd/simulate.py`: `simulate_damage(params, progress=None, cancel=None)`. It checks that
  `progress` and `cancel` are callable or `None` and raises `TypeError` otherwise, before any work.
- `cancel` is a callable only. A `threading.Event` is passed as `event.is_set`.

## 6. Tests

`tests/test_damage_stages.py` (new; keeps `test_simulate.py` for #937's pins):

1. **Byte-equal:** on the 140 × 29 m coastal LNG reference design as a dict,
   `simulate(p)` then `simulate_damage(p)` equals `simulate(p, damage=True)`. Compare
   `json.dumps(r, sort_keys=True)` of the whole results, damage blocks included; the results hash
   leaves them out and cannot prove it (R11). Also: the result with a `progress` callable equals
   the result with none.
2. **Progress:** record every `(done, total)`; `total` is the same on every call; `done` never
   falls; the last call is `(total, total)`.
3. **Cancel:** `cancel` returns true after the third call; the result has `damage_status`
   `not run`; it equals `simulate(p)`.
4. **Callback raises:** `progress` raises `ValueError`; `simulate_damage` raises that `ValueError`.
5. **Not applicable:** #923's `not applicable` fixture as a dict gives `not applicable`, and
   `progress` is never called.
6. **GIL (if decision 4a):** two threads each run `simulate(p)`; wall time is under the sum of the
   two single runs (a loose bound; logged).

Test 1 runs the coastal example with damage twice; it is slow. Mark it as the project marks slow
Python tests.

## 7. Docs and pins

- `public_api.json` gains `simulate_damage(params, progress=None, cancel=None)`, re-pinned by
  running the generator.
- `docs/python-api.md` (made by #937): a "Two stages" section: why damage is off by default, the
  call, the callback rules (called from the engine's threads but one at a time; keep it short;
  return quickly), cancel, Ctrl-C.
- A `changes/` fragment.

## Dependencies

- #923: the callback form, its `Sync` bound and its stop. Ask the #923 builder to make the
  callback `Sync` and to call it once per finished case.
- #937: the `api` entry and `option_of`. #940 starts after #937 merges.
- #921: not a blocker. It makes the callback multi-threaded, which section 3 handles.
