# Slow tests: left out of the PR gate, run nightly

The `test` job of every PR runs the fast gate (`tools/gates.sh`) with the `pr` profile. The tests
below are left out of it and run in the `nightly` workflow (`.github/workflows/nightly.yml`), every
day and on any PR that changes this list, the nextest profiles or the gate script. It is split as
`ci.yml` is (issue #1046): Rust in twelve partitions, pytest in four shards, so no job nears its 30
minute limit. No test is deleted. Run every test locally with
`GATES_NEXTEST_PROFILE=nightly bash tools/gates.sh`.

The `test` check is split (issue #561). `ci.yml` runs the gate in four parallel jobs: `gate-lint`, `gate-rust` (twelve
partitions of the Rust tests), `gate-impact` and `gate-python`. The job named `test` waits for all of them and
passes only if each passed. `tools/gates.sh` takes `GATES_STEPS` (a list of step groups) and `GATES_PARTITION`
(for example `2/12`) to run one part; with neither set it runs everything, as before.

A partition runs the slow Rust tests that `.config/rust-partitions.txt` places in it, then its hash share of
every other test, both at once (issue #819). The file places each test of 15 s or more by its measured time, so no job
draws several slow tests by chance. Rewrite it from a CI run's JUnit artifacts (`nextest-junit-*`) when the
slow tests change: `python3 tools/rust_partitions.py 12 <junit files> > .config/rust-partitions.txt`. A stale
line costs balance only: a test the file does not name still runs in its hash partition. A job whose
placed lines all name no test fails (nextest `--no-tests=fail`), so the file is rewritten then.

A test moves here only by its measured time, and never if it proves a gate: the evaluation harness
false-pass check, the fixture and results hashes, WebAssembly parity, the release guard and the
embargo check, or the design-impact check.

`tests/test_gates_slow_list.py` checks that this table, the `pr` filter in `.config/nextest.toml` and
the tests marked `@pytest.mark.slow` name the same tests.

Times: measured on 2026-10-21 on a developer machine (Apple silicon, 1,185 Rust tests in 326 s of
test time; pytest 285 s in one process). A CI runner is about as fast for pytest and up to two times
slower for Rust.

| Test | Measured | Why it can wait for the nightly run |
|---|---|---|
| `ovd-core::gas_arrangement_feasible no_arrangement_labelled_feasible_has_a_failing_check` | 29.0 s | A property sweep of the gas arrangement search over many hulls. The search's own unit and fixture tests stay in the PR gate, and so do the fixture hashes, which move if a feasibility verdict moves. |
| `ovd-format::ga a_sweep_of_hull_sizes_always_draws_a_valid_sheet` | 26.0 s | A drawing property sweep over hull sizes. Drawings are not in the results hash; the drawing cross-check job still runs on every PR. |
| `ovd-core::gas_arrangement property_the_smallest_viable_length_is_a_length_that_passes` | 11.5 s | A property sweep of the smallest viable length. The fit's unit tests and the fixture hashes stay in the PR gate. |
| `ovd-format::aft_block_inputs each_input_moves_the_aft_block_in_the_drawing_object` | 11.1 s | Moves each of the aft block inputs and checks the drawing object. Drawing data only, not in the results hash. |
| `ovd-format::light_angle_of_max_gz a_light_failure_with_another_criterion_failing_stays_an_error` | 217 s on the CI runner (the JUnit of run 39742410265, with other tests running beside it) | Runs the arrangement search on a 60 x 20 m hull where no candidate passes, so it tries every one. It checks that a light-condition warning is not given when another criterion fails too. The rule itself, and the same rule on feasible hulls, stay in the PR gate (`light_angle_of_max_gz` has 3 other tests); the fixture hashes still move if a verdict moves. |
| `ovd-format::damage_refine_bounds the_search_carries_at_least_the_best_fixed_across_pick_that_passes_damage` | 143 s on a developer machine (release profile; the other test of the file, the repro, is about 15 s and stays in the PR gate) | Runs the arrangement search with damage on four hulls, once searched and once for each fixed number across. The repro of issue #1031, and the fixture hashes, which move if a pick moves, stay in the PR gate. |
| `ovd-format::api_parity each_profile_defaulted_input_left_out_one_at_a_time_is_built_as_the_file_path_builds_it` | 157 s on a developer machine (test profile) | Leaves out each Profile-defaulted input of four Designs (one of each Cargo Containment) one at a time, 444 omissions, and builds the Option twice for each: by the dict step and by `ovd fill --as-new`. A build with a default left out can run the arrangement search of a gas Design. The same check with every default left out at once (`every_profile_defaulted_input_left_out_at_once_is_built_as_the_file_path_builds_it`, 4 s), the per-input comparison of every stored Option and the four full-run cases stay in the PR gate (issue #939). |
| `tests/test_parity.py::test_each_profile_defaulted_input_left_out_one_at_a_time_is_built_as_the_file_path_builds_it` | 213 s on a developer machine | The same one-at-a-time omission check, through the Python API. It also retries a build once an omission makes another input unused. The other tests of `tests/test_parity.py` (all at once, every stored Option, every schema input, the `ovd run` equality) stay in the PR gate. |
| `tests/test_evaluation_runner.py::test_the_mutants_file_is_what_the_generator_writes` | 165.9 s | Regenerates `evaluation/mutants.yaml` from the Python oracle and the base cases, and checks the file is current. The file changes only when `evaluation/` changes (3 commits so far), so the PR `test` job still runs this check whenever a PR changes `evaluation/`, and on every run that is not a PR. The false-pass check itself (`evaluation/run.py --check` and `test_the_real_corpus_has_no_false_pass_and_its_results_are_current`) stays in every PR. |
| `tests/test_damage_stages.py::test_the_two_stages_equal_one_call_with_damage_on_the_whole_value` | 21.4 s | Runs the coastal gas example's damage cases three times, through the Python API, and compares the whole results value. The same equality is checked in Rust (`api_damage the_json_seam_stage_equals_simulate_with_damage_on_the_whole_value`), which stays in the PR gate. |
| `tests/test_damage_stages.py::test_a_cancel_returns_the_results_of_a_run_without_damage` | 14.0 s | Cancels a coastal damage run after three cases, and runs the first stage to compare. The Rust test `api_damage a_hook_that_breaks_gives_the_results_with_damage_not_run` stays in the PR gate. |
| `tests/test_damage_stages.py::test_an_exception_in_a_callable_stops_the_run_and_is_raised` | 14.9 s | Two coastal damage runs that each stop on the second callable call (the stop skips the second run a cancel gives). The progress and Ctrl-C tests of the same file stay in the PR gate. |
