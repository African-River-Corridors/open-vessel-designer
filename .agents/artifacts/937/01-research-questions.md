---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 937
phase: research-questions
---

# #937 research questions

Ticket: Python `simulate` builds an Option the way `ovd fill --as-new` does and returns `run`'s
results. Read at `a63ca819`. Since `d938baec` only CI and e2e files changed; every pointer under
`crates/`, `src/` and `tests/` below is the same at both.

## A. The build path

1. What does `fill_from_profile` do, and can a new module in the same crate call it as it is?
   `crates/ovd-format/src/vdesign.rs:3974`, `Missing` at `:3936`, `fill_as` at `:3980`.
2. What does `read_option` check and fill, and in what order? `vdesign.rs:4564-4790`
   (unknown input `:4586`, unused input `:4626-4675`, required `:4690-4700`, legacy `:4704-4735`,
   profile default `:4736-4766`).
3. Which inputs are `required` in the schema? `vdesign.rs:2117-2140`.
4. Why must the build not go through `new_design`? `vdesign.rs:3883-3912`.
5. How does a value get checked (null, numbers, choices)? `unmark` `vdesign.rs:4528`,
   `check_value` `vdesign.rs:5171-5215`.
6. Is a Cargo in the wrong Cargo Containment an error or a warning on the file path?
   `vdesign.rs:5623-5650`.

## B. The run path

7. What does `run_option_with` take and return, and how does it fold a domain error and a raised
   exception? `crates/ovd-format/src/run.rs:610-800`, `RunOutcome` `run.rs:32-55`.
8. Which params reach the engine from an Option, under which names? `simulate_params_of`
   `run.rs:94-345`.
9. Which params does `run` add itself (the rooms loop)? `settle_rooms` `run.rs:893-935`.
10. Which result blocks does `run` add beyond the old `simulate` result? `run.rs:700-795`,
    `UNHASHED_KEYS` `run.rs:362-383`.

## C. The old dict API

11. What does the Python `simulate` do today? `src/ovd/simulate.py:41-43`, `src/ovd/_bridge.py:163-186`,
    `crates/ovd-py/src/lib.rs:23-32`.
12. How does the engine dispatch it? `crates/ovd-format/src/eval.rs:53-117`,
    `crates/ovd-format/src/simulate.rs:3625-3643`.
13. What does the old engine assume for a key left out: vessel, Hull Form, Formation, depth, rule
    set, midship coefficient? `simulate.rs:1320-1340`, `:2275-2290`, `:2316-2335`, `:2803-2815`,
    `:420-433`.
14. How does the old engine read numbers given as text? `simulate.rs:68` (`py_float`), `:106`
    (`need`), `:113` (`get_or`).
15. Who calls the Python `simulate` besides users? `git grep simulate -- src tools evaluation`;
    `src/ovd/calibration.py:791-803`.

## D. Defaults and stacks

16. Which profile inputs default by Formation, by table and by Service Area?
    `crates/ovd-format/src/builtin_profile.json` (`default_by_formation`, `default_by_table`,
    `default_by_service_area`), wall thickness at `:866`.
17. What stack does a new Option get per Service Area? `builtin_profile.json:248-260`
    (`rule_sets.default_stacks`).
18. What does the old engine take for the stack? `simulate.rs:1196-1230` (`unstored_rules`).

## E. Tests and pins

19. Which tests call `simulate`, and what do they pin? `tests/test_simulate.py`,
    `tests/test_api_small_changes.py:23-75`, `tests/api_scenarios.py:548-595`,
    `tests/test_vdesign_edge_sweep.py:80-90`, `:228-260`.
20. How does the API oracle treat a moved answer? `tests/test_api_oracle.py:31-100`, `:195-205`,
    `tests/data/api_ce96e66.json:2`.
21. Where is the public signature pinned? `tests/data/public_api.json:2396-2406`.

## F. Dependencies

22. Is #925 (Service Area required) on `origin/main`? If not, what does the build do with a dict
    that gives no Service Area? `vdesign.rs` `INPUTS` `service_area` entry; profile
    `service_area.default`.
23. What must #923 give (the damage switch and `damage_status`)? Issue #923 items 1 and 5.
24. Where do the Python docs live today? `README.md:109-140`, `website/mkdocs.yml:55-66`.
25. What version is the package? `pyproject.toml:3`, `Cargo.toml:8`.
