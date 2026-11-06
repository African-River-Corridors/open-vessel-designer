---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 940
phase: research-questions
---

# #940 research questions

Ticket: Damage as a second stage in the Python API, with progress. Read at `a63ca819`.

## A. The Python boundary

1. What does the binding expose, and does it hold the GIL during a run?
   `crates/ovd-py/src/lib.rs:20-39`.
2. Which PyO3 version is locked, and what are its calls to release and take the GIL and to check
   for Ctrl-C? `Cargo.lock:2570-2571`; PyO3 0.29.3 `src/marker.rs:414` (`attach`), `:562`
   (`detach`, `F: Ungil`), `:773` (`check_signals`).
3. Can a callable travel through the JSON seam? `crates/ovd-format/src/eval.rs:78-117`
   (`parse_case`, `eval`), `src/ovd/_bridge.py:163-186`.
4. How are errors turned into Python exceptions? `crates/ovd-py/src/lib.rs:27-31`,
   `src/ovd/_bridge.py:151-160`.

## B. The engine side

5. What does #923 give: the switch, `damage_status`, a callback (done, total) and a stop?
   Issue #923 items 1 and 5, and its comment.
6. From which threads will the callback be called once #921 lands? Issue #921 item 1 (no PR
   yet); `crates/ovd-format/src/damage.rs:134-190`, where the case loop runs.
6a. On which thread does Python handle signals? Python `signal` module docs ("Python signal
   handlers are always executed in the main Python thread"); PyO3 `check_signals`
   (`marker.rs:773`).
7. What does #937 build that #940 reuses? `.agents/artifacts/937/03-design.md` (PR #949):
   the `api` entry, `simulate(p, damage=False)`.
8. What does a cancelled run return? Issue #940 item 2 and acceptance criterion 2.

## C. Tests and pins

9. Where is the public API pinned, and how is a new name added? `tests/data/public_api.json`,
   `tests/test_public_api.py:1-70`.
10. Which hull has no damage Rule Set, for the `not applicable` test? `builtin_profile.json:248-260`
    (`default_stacks`), `crates/ovd-format/src/simulate.rs:3273-3295` (the ADN and ES-TRIN damage
    filters).
11. How is the byte-equal check written today for results? `crates/ovd-format/src/run.rs:351`, `:389-398`
    (`results_hash`, `canonical_json`).
