---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 940
phase: research
---

# #940 research findings

Read at `a63ca819`. Nothing built or run. #923 has no PR; #937's design is in PR #949; #921 has no PR yet.

## A. The Python boundary

**[1] The binding.** Two functions: `version()` and `eval(entry_point, case_json) -> str`
(`ovd-py/src/lib.rs:20-39`). `eval` runs the whole case while it holds the GIL; there is no
`detach`. So today a long `simulate` blocks every other Python thread and cannot be stopped with
Ctrl-C until it returns.

**[2] PyO3.** Locked at 0.29.3 (`Cargo.lock:2570-2571`). In that version the calls are
`Python::attach` (take the GIL; `marker.rs:414`) and `py.detach(f)` (release it; `marker.rs:562`;
the closure must be `Ungil`, so it may hold `Py<PyAny>` handles but no borrowed `Bound`), and
`py.check_signals()` (`marker.rs:773`). The old names `with_gil` and `allow_threads` are not used.

**[3] No callable through JSON.** `eval` takes a parsed JSON value (`eval.rs:78-117`); `_bridge.raw`
sends JSON text (`_bridge.py:163-166`). A callback cannot ride the seam. It needs its own binding
function and an engine function that takes a Rust callback.

**[4] Errors.** `EvalError::BadInput` → `TypeError`, any other `EvalError` → `RuntimeError`
(`lib.rs:27-31`). A core result `{"error": {"type", "message"}}` is raised as that class by
`_bridge._raise` (`_bridge.py:151-160`).

## B. The engine side

**[5] #923.** Promises an engine callback (cases done and total) and a way for it to stop the run;
`damage_status` `run` / `not run` / `not applicable`. The form of the callback is not fixed yet.
#940 needs it to be callable from any thread, so it must be `Sync`.

**[6] Threads.** With #921 as planned, cases of one loading condition run on worker threads
(`std::thread::scope`), per worker, and come back in case order. WebAssembly stays serial. So the
callback can be called from several threads at once, and `done` counted by each worker can reach
Python out of order unless the binding orders it.

**[6a] Signals.** Python runs signal handlers only in the main thread, and `check_signals` only
reports a pending signal there. Called from a #921 worker thread it never sees Ctrl-C. So every
Python call (`progress`, `cancel`, `check_signals`) must happen on the thread that called
`simulate_damage`, not on the engine's workers.

**[7] #937.** Adds the `api` entry with `fn: "simulate"`, which builds the Option through
`fill_from_profile` and runs it with `run_option_with`; `simulate(p, damage=False)` in Python.
#940 reuses the same build (`option_of`) and run.

**[8] Cancel.** The issue: `cancel()` returning true stops the run; `damage_status` stays `not run`.
It does not say whether the call raises or returns. Criterion 2 reads the status after a cancel, so
the call returns the results.

## C. Tests and pins

**[9] Public API.** `public_api.json` pins every public name and its parameters; a new name in
`ovd.simulate` is a new entry there, re-pinned by running the generator.

**[10] Not applicable.** A dry-bulk river-only Option takes `is2008-a`, `es-trin-2025`, `ovd`
(`builtin_profile.json:252`). In `simulate.rs` the ES-TRIN 28.03 damage is read on the gas path for
a craft over 110 m (`simulate.rs:3283-3291`). Whether a short dry-bulk hull gives `not applicable`
depends on how #923 defines the status; reuse #923's own `not applicable` fixture so the two
tickets agree.

**[11] Byte-equal.** `canonical_json` gives sorted keys and no spaces (`run.rs:351`).
`results_hash` leaves out the damage blocks (`run.rs:362-383`), so it cannot prove damage parity
(#921 notes the same). The test must compare the whole results value, damage blocks included.
